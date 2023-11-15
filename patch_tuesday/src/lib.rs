pub mod cvrf;
use std::fmt;

pub enum Severity {
    Critical,
    Important,
    High,
    Medium,
    Low,
}

pub enum Impact {
    RemoteCodeExecution,
    EscalationOfPrivilege,
    DenialOfService,
    SecurityFeatureBypass,
    InformationDisclosure,
    Spoofing,
}
pub enum Product {} // todo!
pub struct Vulnerability {
    pub title: String,
    pub cve: String,
    pub severity: String,
    pub impact: String,
    pub description: Option<String>,
    pub acknowledgments: Option<String>,
    pub public: bool,
    pub exploited: bool,
}

impl From<&cvrf::Vulnerability> for Vulnerability {
    fn from(item: &cvrf::Vulnerability) -> Self {
        let title = item.title.value.clone().unwrap_or_default();
        let cve = item.cve.clone();
        let severity = item
            .threats
            .iter()
            .find(|threat| threat.type_ == 3)
            .and_then(|note| note.description.clone().unwrap().value)
            .unwrap_or_default();
        let impact = item
            .threats
            .iter()
            .find(|threat| threat.type_ == 0)
            .and_then(|note| note.description.clone().unwrap().value)
            .unwrap_or_default();
        let description = item
            .notes
            .iter()
            .find(|note| note.title == "Description")
            .and_then(|note| note.value.clone());
        let acknowledgments: Option<String> = item
            .acknowledgments
            .iter()
            .flat_map(|ack| ack.name.iter())
            .map(|field| field.value.clone())
            .collect();
        let vuln_exploitability = item
            .threats
            .iter()
            .find(|threat| threat.type_ == 1)
            .and_then(|note| note.description.clone().unwrap().value)
            .unwrap_or_default();
        let exploitability_fields: Vec<&str> = vuln_exploitability.split(';').collect();
        // println!("{:#?}", exploitability_fields); todo! some only have "DOS:N/A"
        let public = exploitability_fields.get(0).unwrap_or(&"").contains("Yes");
        let exploited = exploitability_fields.get(1).unwrap_or(&"").contains("Yes");
        Vulnerability {
            title,
            cve,
            severity,
            impact,
            description,
            acknowledgments,
            public,
            exploited,
        }
    }
}

impl fmt::Display for Vulnerability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.title)?;
        write!(f, "{}\n", self.cve)?;
        write!(f, "Severity: {}\n", self.severity)?;
        write!(f, "Impact: {}\n", self.impact)?;
        if let Some(description) = &self.description {
            write!(f, "Description: {description}\n")?;
        }
        write!(f, "Publicly Disclosed: {}\n", self.public)?;
        write!(f, "Exploited: {}\n", self.exploited)?;
        if let Some(acknowledgments) = &self.acknowledgments {
            write!(f, "Acknowledgments: {acknowledgments}\n")?;
        }
        write!(f, "{}", "-".repeat(8))
    }
}
