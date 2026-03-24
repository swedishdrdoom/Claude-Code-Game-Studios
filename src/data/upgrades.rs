use serde::Deserialize;

/// Raw upgrade definition from upgrades.json.
#[derive(Debug, Clone, Deserialize)]
pub struct UpgradeDefinition {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Rarity")]
    pub rarity: String,
    #[serde(rename = "Type")]
    pub upgrade_type: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Image")]
    pub image: String,
}
