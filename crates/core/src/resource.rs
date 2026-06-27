//! Das Ressourcenmodell: 3 roh + 2 veredelt + 1 Gate-Gut.
//!
//! Flach genug, um nie nach Factorio zu riechen, dicht genug für echte
//! Tradeoffs schon auf einem Planeten (DESIGN.md §4.1, Begleitdokument §3).
//! Der Baum ist `3 → 2 → 1`, keine Rekursion, keine Bänder:
//!
//! ```text
//! Metalle ──┬───────────────→ Legierungen ──┐
//!           │                                ├──→ Komposit
//! Silikate ─┴─→ Elektronik ──────────────────┘
//! Gase ─────→ (Energie/Treibstoff, anderswo modelliert)
//! ```
//!
//! Die **Konvergenz bei Metalle** (Elektronik *und* Legierungen ziehen daran)
//! ist die zentrale Frühspiel-Spannung — der geteilte Engpass.

use serde::{Deserialize, Serialize};

/// Alle handelbaren Stoffe. `Energie` ist bewusst *kein* lagerbarer Stoff,
/// sondern ein flaches Budget (siehe [`crate::economy`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    // Roh (an Gelände gebunden)
    Metals,
    Silicates,
    Gases,
    // Veredelt (energiekostend)
    Alloys,
    Electronics,
    // Gate-Gut (Tier 2, spät)
    Composite,
}

impl Resource {
    /// Alle Ressourcen in kanonischer Reihenfolge — praktisch für Lager-Arrays
    /// und UI.
    pub const ALL: [Resource; 6] = [
        Resource::Metals,
        Resource::Silicates,
        Resource::Gases,
        Resource::Alloys,
        Resource::Electronics,
        Resource::Composite,
    ];

    /// Verarbeitungsstufe.
    pub fn tier(self) -> Tier {
        match self {
            Resource::Metals | Resource::Silicates | Resource::Gases => Tier::Raw,
            Resource::Alloys | Resource::Electronics => Tier::Refined,
            Resource::Composite => Tier::Gate,
        }
    }

    /// Das Rezept, das diese Ressource erzeugt (leer für Rohstoffe).
    ///
    /// Mengen sind Input-Einheiten je Einheit Output; die Zahlen sind
    /// Platzhalter-Balance für Phase 0 (Tuning offen, DESIGN.md §7).
    pub fn recipe(self) -> Option<Recipe> {
        match self {
            // Roh: keine Eingänge, nur Förderung aus Gelände.
            Resource::Metals | Resource::Silicates | Resource::Gases => None,

            // Legierungen ← Metalle (+ Energie)
            Resource::Alloys => Some(Recipe {
                output: Resource::Alloys,
                inputs: &[(Resource::Metals, 2.0)],
                energy_cost: 1.0,
            }),

            // Elektronik ← Silikate + Metalle (+ Energie) — zieht *auch* an Metalle.
            Resource::Electronics => Some(Recipe {
                output: Resource::Electronics,
                inputs: &[(Resource::Silicates, 1.0), (Resource::Metals, 1.0)],
                energy_cost: 2.0,
            }),

            // Komposit ← Legierungen + Elektronik (+ viel Energie)
            Resource::Composite => Some(Recipe {
                output: Resource::Composite,
                inputs: &[(Resource::Alloys, 1.0), (Resource::Electronics, 1.0)],
                energy_cost: 5.0,
            }),
        }
    }
}

/// Verarbeitungsstufe einer Ressource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Raw,
    Refined,
    Gate,
}

/// Ein Produktionsrezept: feste Eingangsmengen + Energiekosten je Output-Einheit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Recipe {
    pub output: Resource,
    /// `(Ressource, Menge je Output-Einheit)`.
    pub inputs: &'static [(Resource, f64)],
    /// Energie je Output-Einheit (flaches Budget, siehe [`crate::economy`]).
    pub energy_cost: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_are_assigned_correctly() {
        assert_eq!(Resource::Metals.tier(), Tier::Raw);
        assert_eq!(Resource::Alloys.tier(), Tier::Refined);
        assert_eq!(Resource::Composite.tier(), Tier::Gate);
    }

    #[test]
    fn raw_resources_have_no_recipe() {
        for r in [Resource::Metals, Resource::Silicates, Resource::Gases] {
            assert!(r.recipe().is_none(), "{r:?} should be raw");
        }
    }

    #[test]
    fn metals_are_the_shared_bottleneck() {
        // Sowohl Legierungen als auch Elektronik ziehen an Metalle.
        let alloys = Resource::Alloys.recipe().unwrap();
        let electronics = Resource::Electronics.recipe().unwrap();
        assert!(alloys.inputs.iter().any(|(r, _)| *r == Resource::Metals));
        assert!(electronics.inputs.iter().any(|(r, _)| *r == Resource::Metals));
    }

    #[test]
    fn composite_is_a_gate_good_from_refined_inputs() {
        let c = Resource::Composite.recipe().unwrap();
        assert_eq!(c.output, Resource::Composite);
        for (r, _) in c.inputs {
            assert_eq!(r.tier(), Tier::Refined, "{r:?} should be refined");
        }
        // Komposit ist am energiehungrigsten.
        assert!(c.energy_cost > Resource::Alloys.recipe().unwrap().energy_cost);
    }
}
