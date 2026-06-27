//! # `core` — geteilter Simulationskern
//!
//! Spielregeln, Kepler-Mathematik und Typen, die **Server und Client teilen**
//! (DESIGN.md §5.6). Diese Crate ist server-autoritativ gedacht, aber rein und
//! deterministisch: identische Eingaben → identische Ausgaben, sodass der
//! Client Körperpositionen selbst propagieren kann, ohne dass der Server sie
//! streamt.
//!
//! Phase 0 (Fundament) setzt hier das **Simulationsmodell** um (DESIGN.md §6):
//!
//! - [`orbit`] / [`system`] — Kepler-Bahnen „on rails", planar (2D).
//! - [`resource`] — das Ressourcenmodell 3 roh + 2 veredelt + 1 Gate-Gut.
//! - [`economy`] — Lager, Energiebudget, Produktionsrate, Logistik-Kapazität.
//! - [`time`] — die autoritative Weltuhr.
//!
//! Spätere Phasen bauen Flotten, Gefecht und die Galaxie-Ebene darauf auf.

pub mod economy;
pub mod math;
pub mod orbit;
pub mod resource;
pub mod system;
pub mod time;

pub use economy::{allocate_energy, logistics_efficiency, EnergyDemand, Producer, Stockpile};
pub use math::Vec2;
pub use orbit::{OrbitalElements, MU_SUN};
pub use resource::{Recipe, Resource, Tier};
pub use system::{Body, BodyId, BodyKind, System};
pub use time::SimClock;

/// Baut ein kleines Demo-Heimatsystem auf: Stern, ein Gesteinsplanet auf
/// Kreisbahn und ein Mond. Dient Server und Client in Phase 0 als gemeinsamer,
/// sichtbarer Anfangszustand (das „Skelett läuft").
pub fn demo_home_system() -> System {
    let mut sys = System::new("Heimatsystem");
    sys.add(Body::star(0, "Sonne"));
    sys.add(Body::orbiting(
        1,
        "Heimat",
        BodyKind::Rocky,
        0,
        OrbitalElements::circular_solar(1.495_978_707e8, 0.0),
    ));
    sys.add(Body::orbiting(
        2,
        "Mond",
        BodyKind::Moon,
        1,
        OrbitalElements {
            semi_major_axis: 3.84e5,
            eccentricity: 0.0549,
            arg_periapsis: 0.0,
            mean_anomaly_at_epoch: 0.0,
            epoch: 0.0,
            // μ des umkreisten Körpers = der erd-ähnliche Planet „Heimat"
            // (nicht der Mond selbst): ≈ 3.986e5 km³/s² → Umlauf ~27 Tage.
            mu: 3.986_004_418e5,
        },
    ));
    sys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_system_is_well_formed() {
        let sys = demo_home_system();
        assert_eq!(sys.bodies.len(), 3);
        // Stern ruht im Ursprung.
        assert_eq!(sys.position_of(0, 0.0), Some(Vec2::ZERO));
        // Planet ist etwa 1 AE vom Stern entfernt.
        let d = sys.distance_between(0, 1, 0.0).unwrap();
        assert!((d - 1.495_978_707e8).abs() < 1.0);
    }
}
