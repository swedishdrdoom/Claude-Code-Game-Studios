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
        // Matrix from Enemy Data GDD:
        //              Light  Medium Heavy  Fort   Hero   Unarmored
        // Normal       100%   150%   100%   70%    100%   100%
        // Piercing     200%   75%    100%   35%    50%    150%
        // Siege        100%   50%    100%   150%   50%    150%
        // Magic        125%   75%    200%   35%    50%    100%
        // Chaos        100%   100%   100%   100%   100%   100%
        Self {
            multipliers: [
                // Normal
                [1.00, 1.50, 1.00, 0.70, 1.00, 1.00],
                // Piercing
                [2.00, 0.75, 1.00, 0.35, 0.50, 1.50],
                // Siege
                [1.00, 0.50, 1.00, 1.50, 0.50, 1.50],
                // Magic
                [1.25, 0.75, 2.00, 0.35, 0.50, 1.00],
                // Chaos
                [1.00, 1.00, 1.00, 1.00, 1.00, 1.00],
            ],
        }
    }
}

impl DamageMatrix {
    /// Look up the damage multiplier for a damage type vs armor type.
    pub fn get_multiplier(&self, damage_type: DamageType, armor_type: ArmorType) -> f32 {
        let dt = match damage_type {
            DamageType::Normal => 0,
            DamageType::Piercing => 1,
            DamageType::Siege => 2,
            DamageType::Magic => 3,
            DamageType::Chaos => 4,
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
        assert_eq!(matrix.get_multiplier(DamageType::Piercing, ArmorType::Light), 2.00);
        assert_eq!(matrix.get_multiplier(DamageType::Magic, ArmorType::Heavy), 2.00);
        assert_eq!(matrix.get_multiplier(DamageType::Siege, ArmorType::Fortified), 1.50);
        assert_eq!(matrix.get_multiplier(DamageType::Chaos, ArmorType::Hero), 1.00);
        assert_eq!(matrix.get_multiplier(DamageType::Normal, ArmorType::Fortified), 0.70);
        assert_eq!(matrix.get_multiplier(DamageType::Piercing, ArmorType::Fortified), 0.35);
    }
}
