use serde::Deserialize;

/// Raw weapon definition from weapons.json.
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponDefinition {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Rarity")]
    pub rarity: String,
    #[serde(rename = "Weapon")]
    pub weapon_type: String,
    #[serde(rename = "AttackType")]
    pub attack_type: String,
    #[serde(rename = "Damage")]
    pub damage: u32,
    #[serde(rename = "DPS")]
    pub dps: u32,
    #[serde(rename = "AttackCooldown")]
    pub attack_cooldown: f32,
    #[serde(rename = "Range")]
    pub range: u32,
    #[serde(rename = "Ability")]
    pub ability: String,
    #[serde(rename = "Image")]
    pub image: String,
}
