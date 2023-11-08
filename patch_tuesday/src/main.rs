use chrono::Local;
use clap::Parser;
use patch_tuesday::CVRFDocument;
use reqwest::{header, StatusCode};

/// Simple program that parses info from Microsoft's Security Updates
#[derive(Parser)]
struct Args {
    /// Date to get information from
    #[arg(short, long,default_value_t=Local::now().format("%Y-%b").to_string())]
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

    if response.status() == StatusCode::NOT_FOUND {
        println!("No Security Update found for {}", args.date);
        return Ok(());
    }
    let cvrf_document = response.json::<CVRFDocument>().await?;

    let desired_product = cvrf_document
        .product_tree
        .full_product_name
        .iter()
        .find(|product| product.value == args.product)
        .expect("Product not found");

    for vulnerability in cvrf_document.vulnerability {
        let is_product = vulnerability.product_statuses.iter().any(|status| {
            status
                .product_id
                .clone()
                .unwrap_or_default()
                .contains(&desired_product.product_id)
        });
        if !is_product {
            continue;
        }
        let title = vulnerability.title.value.unwrap_or_default();
        let cve = vulnerability.cve;
        let description = vulnerability
            .notes
            .iter()
            .find(|note| note.title == "Description")
            .and_then(|note| note.value.clone())
            .unwrap_or_default();

        println!("Title: {}", title);
        println!("CVE: {}", cve);
        println!("Description: {}", description);
    }

    Ok(())
}
