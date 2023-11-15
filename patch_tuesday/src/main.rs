use chrono::Local;
use clap::Parser;
use patch_tuesday::cvrf::CVRFDocument;
use patch_tuesday::Vulnerability;
use reqwest::header;

/// Simple program that parses info from Microsoft's Security Updates
#[derive(Parser)]
struct Args {
    /// Date to get information from
    #[arg(short, long, default_value_t=Local::now().format("%Y-%b").to_string())]
    date: String,

    /// Product to get information from
    #[arg(
        short,
        long,
        default_value = "Windows 11 Version 22H2 for x64-based Systems"
    )]
    product: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

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

    let full_product_name = cvrf_document
        .product_tree
        .full_product_name
        .iter()
        .find(|product| product.value == args.product)
        .expect("Product not found");

    let vulns: Vec<Vulnerability> = cvrf_document
        .vulnerability
        .iter()
        .map(|cvrf_vulnerability| Vulnerability::from(cvrf_vulnerability))
        .collect();

    vulns
        .into_iter()
        .filter(|vuln| vuln.exploited)
        .for_each(|vuln| println!("{vuln}"));

    // for cvrf_vulnerability in cvrf_document
    //     .vulnerability
    //     .into_iter()
    //     .filter(|vuln| vuln.is_product_affected(&full_product_name.product_id))
    // {
    //     let vuln = Vulnerability::from(&cvrf_vulnerability);
    //     println!("{}", vuln.title);
    //     println!("{}", vuln.cve);
    //     println!("Severity: {}", vuln.severity);
    //     println!("Impact: {}", vuln.impact);
    //     if let Some(description) = vuln.description {
    //         println!("Description: {description}");
    //     }
    //     println!("Publicly Disclosed: {}", vuln.public);
    //     println!("Exploited: {}", vuln.exploited);
    //     if let Some(acknowledgments) = vuln.acknowledgments {
    //         println!("Acknowledgments: {acknowledgments}");
    //     }
    //     println!("{}", "-".repeat(8));
    // }

    Ok(())
}
