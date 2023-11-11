use chrono::Local;
use clap::Parser;
use patch_tuesday::CVRFDocument;
use reqwest::header;

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

    for vulnerability in cvrf_document
        .vulnerability
        .into_iter()
        .filter(|vuln| vuln.is_product_affected(&full_product_name.product_id))
    {
        let title = vulnerability.title.value.unwrap_or_default();
        let cve = vulnerability.cve;
        let description = vulnerability
            .notes
            .iter()
            .find(|note| note.title == "Description")
            .and_then(|note| note.value.clone())
            .unwrap_or_default();
        let acknowledgments: String = vulnerability
            .acknowledgments
            .iter()
            .flat_map(|ack| ack.name.iter())
            .map(|field| field.value.clone().unwrap_or_default())
            .collect();
        println!("Title: {}", title);
        println!("CVE: {}", cve);
        println!("Description: {}", description);
        println!("Acknowledgments: {}", acknowledgments);
        println!("{}", "-".repeat(8));
    }

    Ok(())
}
