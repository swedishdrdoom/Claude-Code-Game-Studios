use rand::Rng;

use crate::components::combat::DamageType;
use crate::components::enemy::ArmorType;
use bevy::prelude::*;

/// The damage type vs armor type multiplier matrix.
/// Loaded as a Resource, used by Damage Calculation.
#[derive(Resource, Debug)]
pub struct DamageMatrix {
    /// Multipliers indexed by [DamageType][ArmorType].
    /// Values are percentages: 100 = 1.0x, 200 = 2.0x, etc.
    multipliers: [[f32; 6]; 5],
}

impl Default for DamageMatrix {
    fn default() -> Self {
        // All types deal 100% base damage; +100% to their strong matchup.
        // Chaos deals +100% to ALL armor types.
        //              Light  Medium Heavy  Fort   Hero   Unarmored
        // Normal       100%   200%   100%   100%   100%   100%
        // Piercing     200%   100%   100%   100%   100%   100%
        // Siege        100%   100%   100%   200%   100%   100%
        // Magic        100%   100%   200%   100%   100%   100%
        // Chaos        200%   200%   200%   200%   200%   200%
        Self {
            multipliers: [
                // Normal    — strong vs Medium
                [1.00, 2.00, 1.00, 1.00, 1.00, 1.00],
                // Piercing  — strong vs Light
                [2.00, 1.00, 1.00, 1.00, 1.00, 1.00],
                // Siege     — strong vs Fortified
                [1.00, 1.00, 1.00, 2.00, 1.00, 1.00],
                // Magic     — strong vs Heavy
                [1.00, 1.00, 2.00, 1.00, 1.00, 1.00],
                // Chaos     — strong vs all
                [2.00, 2.00, 2.00, 2.00, 2.00, 2.00],
            ],
        }
    }
}

impl DamageMatrix {
    /// Look up the damage multiplier for a damage type vs armor type.
    /// Chaos is special: random between 0.5x and 2.0x per hit.
    pub fn get_multiplier(&self, damage_type: DamageType, armor_type: ArmorType) -> f32 {
        if matches!(damage_type, DamageType::Chaos) {
            return rand::rng().random_range(0.5..=2.0);
        }
        let dt = match damage_type {
            DamageType::Normal => 0,
            DamageType::Piercing => 1,
            DamageType::Siege => 2,
            DamageType::Magic => 3,
            DamageType::Chaos => unreachable!(),
        };
        let at = match armor_type {
            ArmorType::Light => 0,
            ArmorType::Medium => 1,
            ArmorType::Heavy => 2,
            ArmorType::Fortified => 3,
            ArmorType::Hero => 4,
            ArmorType::Unarmored => 5,
        };
        self.multipliers[dt][at]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_matrix_values() {
        let matrix = DamageMatrix::default();
        // Strong matchups = 2.0x
        assert_eq!(matrix.get_multiplier(DamageType::Piercing, ArmorType::Light), 2.00);
        assert_eq!(matrix.get_multiplier(DamageType::Normal, ArmorType::Medium), 2.00);
        assert_eq!(matrix.get_multiplier(DamageType::Magic, ArmorType::Heavy), 2.00);
        assert_eq!(matrix.get_multiplier(DamageType::Siege, ArmorType::Fortified), 2.00);
        // Chaos = random 0.5–2.0x
        for _ in 0..100 {
            let m = matrix.get_multiplier(DamageType::Chaos, ArmorType::Hero);
            assert!(m >= 0.5 && m <= 2.0, "Chaos multiplier out of range: {}", m);
        }
        // Non-strong matchups = 1.0x
        assert_eq!(matrix.get_multiplier(DamageType::Normal, ArmorType::Fortified), 1.00);
        assert_eq!(matrix.get_multiplier(DamageType::Piercing, ArmorType::Fortified), 1.00);
    }
}
