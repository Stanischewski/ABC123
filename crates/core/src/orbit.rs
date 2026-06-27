//! Kepler-Orbitmechanik („on rails").
//!
//! Himmelskörper laufen auf festen Kepler-Bahnen: je Körper ein Satz
//! Bahnelemente, Position = analytische Funktion der Zeit (DESIGN.md §5.5).
//! Kein n-Body, kein Dauer-Tick — die Bahn driftet nie und ist für jeden
//! Zeitpunkt (auch in der Zukunft) gratis berechenbar. Genau das speist die
//! Konjunktions-Vorschau der Logistik (Begleitdokument §8).
//!
//! Identische Propagation auf Server und Client (DESIGN.md §5.6): der Server
//! streamt keine Körperpositionen, der Client rechnet sie selbst aus den
//! Bahnelementen.

use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

use crate::math::{wrap_two_pi, Vec2};

/// Gravitationsparameter μ = G·M des Zentralkörpers, in km³/s².
///
/// Wir parametrisieren Bahnen über μ statt über Massen — μ ist das, was die
/// Bewegung tatsächlich bestimmt, und für die Sonne deutlich genauer bekannt
/// als G und M einzeln.
pub type Mu = f64;

/// μ der Sonne, km³/s² (Standardwert; reale Heliozentrik).
pub const MU_SUN: Mu = 1.327_124_400_18e11;

/// Maximale Newton-Raphson-Iterationen beim Lösen der Kepler-Gleichung.
const KEPLER_MAX_ITER: usize = 64;
/// Abbruchschwelle (Radiant) für das Kepler-Solve.
const KEPLER_TOLERANCE: f64 = 1e-12;

/// Klassische Bahnelemente für eine planare (2D) Ellipsenbahn.
///
/// Reduziert auf die in 2D relevanten Größen — Inklination und Knotenlänge
/// entfallen, weil die Systemansicht planar ist (DESIGN.md §5.5).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OrbitalElements {
    /// Große Halbachse `a` (km). Bestimmt mit μ die Umlaufzeit.
    pub semi_major_axis: f64,
    /// Exzentrizität `e` (0 = Kreis, <1 = Ellipse).
    pub eccentricity: f64,
    /// Argument der Periapsis `ω` (Radiant): Drehung der Bahn in der Ebene.
    pub arg_periapsis: f64,
    /// Mittlere Anomalie `M₀` zur Epoche `t₀` (Radiant).
    pub mean_anomaly_at_epoch: f64,
    /// Epochenzeit `t₀` (Sekunden seit Welt-Beginn), auf die `M₀` bezogen ist.
    pub epoch: f64,
    /// Gravitationsparameter des umkreisten Zentralkörpers (km³/s²).
    pub mu: Mu,
}

impl OrbitalElements {
    /// Bequemer Konstruktor für eine Kreisbahn (e = 0) um die Sonne.
    pub fn circular_solar(radius: f64, phase: f64) -> Self {
        OrbitalElements {
            semi_major_axis: radius,
            eccentricity: 0.0,
            arg_periapsis: 0.0,
            mean_anomaly_at_epoch: phase,
            epoch: 0.0,
            mu: MU_SUN,
        }
    }

    /// Mittlere Bewegung `n = sqrt(μ / a³)` (Radiant/Sekunde).
    pub fn mean_motion(&self) -> f64 {
        (self.mu / self.semi_major_axis.powi(3)).sqrt()
    }

    /// Umlaufzeit `T = 2π / n` (Sekunden).
    pub fn period(&self) -> f64 {
        TAU / self.mean_motion()
    }

    /// Mittlere Anomalie `M(t)`, normiert auf `[0, 2π)`.
    pub fn mean_anomaly_at(&self, t: f64) -> f64 {
        wrap_two_pi(self.mean_anomaly_at_epoch + self.mean_motion() * (t - self.epoch))
    }

    /// Löst die Kepler-Gleichung `M = E - e·sin E` per Newton-Raphson und gibt
    /// die exzentrische Anomalie `E` zurück.
    pub fn eccentric_anomaly_at(&self, t: f64) -> f64 {
        let m = self.mean_anomaly_at(t);
        let e = self.eccentricity;

        // Startwert: bei hoher Exzentrizität ist M ein schlechter Schätzer.
        let mut ea = if e < 0.8 { m } else { std::f64::consts::PI };
        for _ in 0..KEPLER_MAX_ITER {
            let f = ea - e * ea.sin() - m;
            let f_prime = 1.0 - e * ea.cos();
            let delta = f / f_prime;
            ea -= delta;
            if delta.abs() < KEPLER_TOLERANCE {
                break;
            }
        }
        ea
    }

    /// Wahre Anomalie `ν` (Radiant) zum Zeitpunkt `t`.
    pub fn true_anomaly_at(&self, t: f64) -> f64 {
        let ea = self.eccentric_anomaly_at(t);
        let e = self.eccentricity;
        // Numerisch stabile Halbwinkel-Form.
        let half = ((1.0 + e) / (1.0 - e)).sqrt() * (ea / 2.0).tan();
        wrap_two_pi(2.0 * half.atan())
    }

    /// Position relativ zum Zentralkörper (km) zum Zeitpunkt `t`.
    ///
    /// Liefert den Vektor in der Bahnebene, bereits um `ω` gedreht. Die
    /// absolute Weltposition ergibt sich, indem die Position des Zentralkörpers
    /// addiert wird (siehe [`crate::system::System::position_of`]).
    pub fn relative_position_at(&self, t: f64) -> Vec2 {
        let ea = self.eccentric_anomaly_at(t);
        let a = self.semi_major_axis;
        let e = self.eccentricity;

        // Position in der Periapsis-Bezugsebene.
        let px = a * (ea.cos() - e);
        let py = a * (1.0 - e * e).sqrt() * ea.sin();

        Vec2::new(px, py).rotated(self.arg_periapsis)
    }

    /// Aktueller Bahnradius `r = a·(1 - e·cos E)` (km) zum Zeitpunkt `t`.
    pub fn radius_at(&self, t: f64) -> f64 {
        let ea = self.eccentric_anomaly_at(t);
        self.semi_major_axis * (1.0 - self.eccentricity * ea.cos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    /// Eine einfache Erd-ähnliche Kreisbahn: a = 1 AE, e = 0.
    fn earthlike() -> OrbitalElements {
        OrbitalElements::circular_solar(1.495_978_707e8, 0.0)
    }

    #[test]
    fn circular_radius_is_constant() {
        let o = earthlike();
        for &t in &[0.0, 1.0e6, 5.0e6, 2.0e7] {
            assert!((o.radius_at(t) - o.semi_major_axis).abs() < 1e-3);
        }
    }

    #[test]
    fn earthlike_period_is_about_one_year() {
        let year_seconds = 365.25 * 86_400.0;
        let p = earthlike().period();
        // Innerhalb eines Prozents eines Jahres.
        assert!((p - year_seconds).abs() / year_seconds < 0.01, "period was {p}");
    }

    #[test]
    fn kepler_solver_satisfies_equation() {
        // Deutlich exzentrische Bahn, um den Solver zu fordern.
        let o = OrbitalElements {
            semi_major_axis: 2.0e8,
            eccentricity: 0.7,
            arg_periapsis: 0.4,
            mean_anomaly_at_epoch: 0.0,
            epoch: 0.0,
            mu: MU_SUN,
        };
        for i in 0..16 {
            let t = i as f64 * 1.0e6;
            let m = o.mean_anomaly_at(t);
            let ea = o.eccentric_anomaly_at(t);
            let residual = ea - o.eccentricity * ea.sin() - m;
            assert!(residual.abs() < 1e-9, "residual {residual} at t={t}");
        }
    }

    #[test]
    fn quarter_period_advances_circular_orbit_90_degrees() {
        let o = OrbitalElements::circular_solar(1.0e8, 0.0);
        let p0 = o.relative_position_at(0.0);
        let p_quarter = o.relative_position_at(o.period() / 4.0);
        // Start auf +x-Achse, nach Vierteldrehung auf +y-Achse.
        assert!((p0.x - 1.0e8).abs() < 1.0);
        assert!(p0.y.abs() < 1.0);
        assert!(p_quarter.x.abs() < 10.0);
        assert!((p_quarter.y - 1.0e8).abs() < 10.0);
    }

    #[test]
    fn position_is_periodic() {
        let o = earthlike();
        let p0 = o.relative_position_at(123.0);
        let p1 = o.relative_position_at(123.0 + o.period());
        assert!(p0.distance(p1) < 1.0, "orbit not periodic: {p0:?} vs {p1:?}");
    }

    #[test]
    fn true_anomaly_zero_at_periapsis() {
        // Bei M₀ = 0 steht der Körper in der Periapsis → ν = 0.
        let o = OrbitalElements {
            semi_major_axis: 1.0e8,
            eccentricity: 0.3,
            arg_periapsis: PI / 3.0,
            mean_anomaly_at_epoch: 0.0,
            epoch: 0.0,
            mu: MU_SUN,
        };
        assert!(o.true_anomaly_at(0.0).abs() < 1e-9);
        // In der Periapsis ist der Radius minimal: r = a(1 - e).
        assert!((o.radius_at(0.0) - o.semi_major_axis * (1.0 - o.eccentricity)).abs() < 1.0);
    }
}
