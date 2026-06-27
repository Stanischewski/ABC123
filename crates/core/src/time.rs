//! Simulationszeit.
//!
//! Die Welt rechnet in Sekunden seit einem festen Welt-Beginn. Himmelskörper
//! sind analytisch (Kepler), Ökonomie und Bauschlangen werden **bei Bedarf**
//! aus dem Zeitstempel berechnet, statt jede Sekunde zu ticken (DESIGN.md §5.4).
//! Diese Uhr ist damit der gemeinsame Bezugspunkt, kein Treiber einer Schleife.

use serde::{Deserialize, Serialize};

/// Autoritative Weltzeit in Sekunden seit Welt-Beginn (`t = 0`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SimClock {
    seconds: f64,
}

impl Default for SimClock {
    fn default() -> Self {
        SimClock { seconds: 0.0 }
    }
}

impl SimClock {
    pub fn new(seconds: f64) -> Self {
        SimClock { seconds }
    }

    /// Aktuelle Weltzeit in Sekunden.
    pub fn now(&self) -> f64 {
        self.seconds
    }

    /// Schreitet um `dt` Sekunden voran (klemmt negatives `dt` auf 0 — die
    /// Weltzeit läuft nie rückwärts).
    pub fn advance(&mut self, dt: f64) {
        self.seconds += dt.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_advances_monotonically() {
        let mut c = SimClock::default();
        assert_eq!(c.now(), 0.0);
        c.advance(5.0);
        c.advance(2.5);
        assert!((c.now() - 7.5).abs() < 1e-12);
        // Rückwärts wird ignoriert.
        c.advance(-100.0);
        assert!((c.now() - 7.5).abs() < 1e-12);
    }
}
