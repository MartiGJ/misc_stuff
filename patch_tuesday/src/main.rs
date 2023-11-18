use std::sync::{Arc, Mutex};

use chrono::Local;
use clap::Parser;
use futures::{stream, StreamExt};
use patch_tuesday::cvrf::CVRFDocument;
use patch_tuesday::{Product, Severity, Vulnerability};
use reqwest::header;
use tokio;

/// Simple program that parses info from Microsoft's Security Updates
#[derive(Parser)]
struct Args {
    /// Date from which to obtain information
    #[arg(short, long, default_value_t=Local::now().format("%Y-%b").to_string())]
    date: String,

    /// Product from which to obtain information
    #[arg(
        value_enum,
        short,
        long,
        default_value_t = Product::Win11_22H2_x64 //"Windows 11 Version 22H2 for x64-based Systems"
    )]
    product: Product,

    /// Year from which to obtain information
    #[arg(long)]
    year: Option<u64>,

    /// Filter by given severity
    #[arg(long)]
    severity: Option<Severity>,

    /// Filter by given text contained in title
    #[arg(long)]
    title: Option<String>,

    /// Filter by given text contained in acknowledgements
    #[arg(long)]
    acknowledgement: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut vulns: Vec<Vulnerability>;

    if let Some(year) = args.year {
        let vulns_year = Arc::new(Mutex::new(Vec::<Vulnerability>::new()));
        let months: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let urls: Vec<String> = months
            .iter()
            .map(|month| {
                format!(
                    "https://api.msrc.microsoft.com/cvrf/v2.0/cvrf/{}-{}",
                    year, month
                )
            })
            .collect();

        let client = reqwest::Client::new();

        let cvrf_documents = stream::iter(urls)
            .map(|url| {
                let client = client.clone();
                tokio::spawn(async move {
                    let resp = client
                        .get(&url)
                        .header(header::ACCEPT, "application/json")
                        .send()
                        .await
                        .map_err(|err| err.to_string())?;
                    if !resp.status().is_success() {
                        return Err(format!("No Security Update found for {}", url));
                    }
                    resp.json::<CVRFDocument>()
                        .await
                        .map_err(|err| err.to_string())
                })
            })
            .buffer_unordered(6);

        cvrf_documents
            .for_each(|cvrf_document| async {
                match cvrf_document {
                    Ok(Ok(cvrf_document)) => {
                        let vulns_month = cvrf_document
                            .vulnerability
                            .iter()
                            .map(|cvrf_vulnerability| Vulnerability::from(cvrf_vulnerability));
                        vulns_year.lock().unwrap().extend(vulns_month);
                    }
                    Ok(Err(e)) => eprintln!("Got a reqwest::Error: {}", e),
                    Err(e) => eprintln!("Got a tokio::JoinError: {}", e),
                }
            })
            .await;

        vulns = vulns_year.lock().unwrap().to_vec();
    } else {
        let response = reqwest::Client::new()
            .get(format!(
                "https://api.msrc.microsoft.com/cvrf/v2.0/cvrf/{}",
                args.date
            ))
            .header(header::ACCEPT, "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            println!("No Security Update found for {}", args.date);
            return Ok(());
        }
        let cvrf_document = response.json::<CVRFDocument>().await?;

        vulns = cvrf_document
            .vulnerability
            .iter()
            .map(|cvrf_vulnerability| Vulnerability::from(cvrf_vulnerability))
            .collect();
    }

    if let Some(severity) = args.severity {
        vulns.retain(|vuln| vuln.severity == severity);
    }

    if let Some(title) = args.title {
        vulns.retain(|vuln| {
            vuln.title
                .clone()
                .to_lowercase()
                .contains(&title.to_lowercase())
        });
    }

    if let Some(acknowledgement) = args.acknowledgement {
        vulns.retain(|vuln| {
            vuln.acknowledgements
                .clone()
                .is_some_and(|acknowledgements| {
                    acknowledgements
                        .to_lowercase()
                        .contains(&acknowledgement.to_lowercase())
                })
        });
    }

    if args.product != Product::All {
        vulns.retain(|vuln| {
            vuln.affected_products
                .contains(&(args.product as u64).to_string())
        });
    }

    vulns.iter().for_each(|vuln| println!("{vuln}"));

    Ok(())
}
