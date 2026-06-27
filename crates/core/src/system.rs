//! Sonnensystem als Hierarchie von Himmelskörpern.
//!
//! Monde umkreisen Planeten, Planeten die Sonne (DESIGN.md §5.5). Die absolute
//! Weltposition eines Körpers ist die Summe der relativen Kepler-Positionen
//! entlang der Kette bis zur Wurzel (Stern, ohne Bahn).

use serde::{Deserialize, Serialize};

use crate::math::Vec2;
use crate::orbit::OrbitalElements;

/// Stabile Kennung eines Körpers innerhalb eines Systems.
pub type BodyId = u32;

/// Geländeprofil bzw. Körpertyp — bestimmt, welche Rohstoffe förderbar sind
/// (DESIGN.md §3.1, §4.1). In Phase 0 bewusst grob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyKind {
    Star,
    Rocky,
    Gas,
    Icy,
    Moon,
    Asteroid,
}

/// Ein einzelner Himmelskörper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub id: BodyId,
    pub name: String,
    pub kind: BodyKind,
    /// Übergeordneter Körper, den dieser umkreist; `None` für den Stern.
    pub parent: Option<BodyId>,
    /// Bahnelemente relativ zum `parent`; `None` für den (ruhenden) Stern.
    pub orbit: Option<OrbitalElements>,
}

impl Body {
    /// Konstruiert den zentralen Stern (ruhend im Ursprung).
    pub fn star(id: BodyId, name: impl Into<String>) -> Self {
        Body {
            id,
            name: name.into(),
            kind: BodyKind::Star,
            parent: None,
            orbit: None,
        }
    }

    /// Konstruiert einen umkreisenden Körper.
    pub fn orbiting(
        id: BodyId,
        name: impl Into<String>,
        kind: BodyKind,
        parent: BodyId,
        orbit: OrbitalElements,
    ) -> Self {
        Body {
            id,
            name: name.into(),
            kind,
            parent: Some(parent),
            orbit: Some(orbit),
        }
    }
}

/// Ein Sonnensystem: eine flache Liste von Körpern mit Eltern-Verweisen.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct System {
    pub name: String,
    pub bodies: Vec<Body>,
}

impl System {
    pub fn new(name: impl Into<String>) -> Self {
        System {
            name: name.into(),
            bodies: Vec::new(),
        }
    }

    /// Fügt einen Körper hinzu und gibt seine Id zurück.
    pub fn add(&mut self, body: Body) -> BodyId {
        let id = body.id;
        self.bodies.push(body);
        id
    }

    /// Sucht einen Körper per Id.
    pub fn body(&self, id: BodyId) -> Option<&Body> {
        self.bodies.iter().find(|b| b.id == id)
    }

    /// Absolute Weltposition eines Körpers zum Zeitpunkt `t` (Sekunden).
    ///
    /// Summiert die relativen Kepler-Positionen entlang der Eltern-Kette.
    /// Gibt `None`, wenn die Id unbekannt ist; Zyklen werden über eine
    /// Tiefenbegrenzung abgefangen.
    pub fn position_of(&self, id: BodyId, t: f64) -> Option<Vec2> {
        let mut pos = Vec2::ZERO;
        let mut current = Some(id);
        let mut guard = 0usize;
        while let Some(cid) = current {
            let body = self.body(cid)?;
            if let Some(orbit) = &body.orbit {
                pos = pos + orbit.relative_position_at(t);
            }
            current = body.parent;
            guard += 1;
            if guard > self.bodies.len() + 1 {
                // Defensive: zyklische Eltern-Kette.
                return None;
            }
        }
        Some(pos)
    }

    /// Abstand zwischen zwei Körpern zum Zeitpunkt `t` (km).
    pub fn distance_between(&self, a: BodyId, b: BodyId, t: f64) -> Option<f64> {
        Some(self.position_of(a, t)?.distance(self.position_of(b, t)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orbit::OrbitalElements;

    /// Stern, ein Planet auf Kreisbahn, ein Mond um den Planeten.
    fn toy_system() -> System {
        let mut sys = System::new("Testsystem");
        sys.add(Body::star(0, "Sonne"));
        sys.add(Body::orbiting(
            1,
            "Heimat",
            BodyKind::Rocky,
            0,
            OrbitalElements::circular_solar(1.0e8, 0.0),
        ));
        // Mond auf enger Bahn um den Planeten (μ des Planeten, klein).
        let moon_orbit = OrbitalElements {
            semi_major_axis: 4.0e5,
            eccentricity: 0.0,
            arg_periapsis: 0.0,
            mean_anomaly_at_epoch: 0.0,
            epoch: 0.0,
            mu: 5.0e3,
        };
        sys.add(Body::orbiting(2, "Mond", BodyKind::Moon, 1, moon_orbit));
        sys
    }

    #[test]
    fn star_sits_at_origin() {
        let sys = toy_system();
        assert_eq!(sys.position_of(0, 0.0), Some(Vec2::ZERO));
        assert_eq!(sys.position_of(0, 9.9e9), Some(Vec2::ZERO));
    }

    #[test]
    fn moon_position_is_planet_plus_relative() {
        let sys = toy_system();
        let t = 1.234e6;
        let planet = sys.position_of(1, t).unwrap();
        let moon = sys.position_of(2, t).unwrap();
        // Der Mond ist stets ~4e5 km vom Planeten entfernt.
        assert!((planet.distance(moon) - 4.0e5).abs() < 1.0);
    }

    #[test]
    fn unknown_body_returns_none() {
        let sys = toy_system();
        assert_eq!(sys.position_of(99, 0.0), None);
        assert_eq!(sys.distance_between(1, 99, 0.0), None);
    }

    #[test]
    fn system_round_trips_through_json() {
        let sys = toy_system();
        let json = serde_json::to_string(&sys).unwrap();
        let back: System = serde_json::from_str(&json).unwrap();
        assert_eq!(sys, back);
    }
}
