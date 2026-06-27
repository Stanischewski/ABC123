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
//! Phase 1 (Bau-Ebene) baut darauf auf:
//!
//! - [`planet`] — Raster, Gelände, Gebäude, Adjazenz-Boni.
//! - [`production`] — Zeit-Integration der Produktion (energie- und
//!   input-gedrosselt, Solar an den Bahnradius gekoppelt).
//!
//! Spätere Phasen bauen Flotten, Gefecht und die Galaxie-Ebene darauf auf.

pub mod economy;
pub mod math;
pub mod orbit;
pub mod planet;
pub mod production;
pub mod resource;
pub mod system;
pub mod time;

pub use economy::{allocate_energy, logistics_efficiency, EnergyDemand, Producer, Stockpile};
pub use math::Vec2;
pub use orbit::{OrbitalElements, MU_SUN};
pub use planet::{Building, BuildingKind, BuildingSpec, Grid, PlaceError, Terrain, Tile};
pub use production::{advance, resolve_step, StepReport};
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

/// Baut ein kleines Demo-Heimatraster: gemischtes Gelände mit ein paar
/// Ressourcenfeldern. Dient als sichtbarer Startzustand der Bau-Ebene; das
/// Layout (was wohin) bleibt dem Spieler überlassen.
pub fn demo_home_planet() -> Grid {
    let mut g = Grid::new(8, 6, Terrain::Barren);
    // Ein paar Gelände-Vorkommen einstreuen (DESIGN.md §3.1: Profile).
    for (x, y) in [(1, 1), (2, 1), (1, 2), (5, 4)] {
        g.set_terrain(x, y, Terrain::Rock);
    }
    for (x, y) in [(6, 1), (6, 2)] {
        g.set_terrain(x, y, Terrain::Crystal);
    }
    for (x, y) in [(3, 4), (4, 4)] {
        g.set_terrain(x, y, Terrain::GasField);
    }
    g.set_terrain(0, 5, Terrain::Ice);
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_planet_has_mixed_terrain_and_room() {
        let p = demo_home_planet();
        assert_eq!(p.width * p.height, 48);
        // Mindestens je ein Förder-Gelände vorhanden.
        assert_eq!(p.tile(1, 1).unwrap().terrain, Terrain::Rock);
        assert_eq!(p.tile(6, 1).unwrap().terrain, Terrain::Crystal);
        assert_eq!(p.tile(3, 4).unwrap().terrain, Terrain::GasField);
    }

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
