//! Kleine, abhängigkeitsfreie 2D-Mathematik.
//!
//! Die Systemansicht ist planar (DESIGN.md §5.5), daher genügt ein schlanker
//! `Vec2` auf `f64`. Server-intern wird durchgehend mit `f64` gerechnet — der
//! Determinismus muss nur server-seitig gelten (DESIGN.md §5.3).

use serde::{Deserialize, Serialize};

/// Ein 2D-Vektor in Weltkoordinaten (Einheiten: Kilometer).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    /// Länge des Vektors.
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Quadrierte Länge — billiger, wenn nur Vergleiche nötig sind.
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Abstand zwischen zwei Punkten.
    pub fn distance(self, other: Vec2) -> f64 {
        (self - other).length()
    }

    /// Dreht den Vektor um den Winkel `radians` gegen den Uhrzeigersinn.
    pub fn rotated(self, radians: f64) -> Vec2 {
        let (s, c) = radians.sin_cos();
        Vec2::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

/// Normiert einen Winkel auf das Intervall `[0, 2π)`.
pub fn wrap_two_pi(angle: f64) -> f64 {
    use std::f64::consts::TAU;
    let a = angle % TAU;
    if a < 0.0 {
        a + TAU
    } else {
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{PI, TAU};

    #[test]
    fn length_and_distance() {
        let v = Vec2::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 1e-12);
        assert!((Vec2::ZERO.distance(v) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn rotation_by_quarter_turn() {
        let v = Vec2::new(1.0, 0.0).rotated(PI / 2.0);
        assert!(v.x.abs() < 1e-12);
        assert!((v.y - 1.0).abs() < 1e-12);
    }

    #[test]
    fn wrap_normalises_into_range() {
        assert!((wrap_two_pi(-0.5) - (TAU - 0.5)).abs() < 1e-12);
        assert!(wrap_two_pi(TAU + 1.0).abs() - 1.0 < 1e-12);
        assert!((0.0..TAU).contains(&wrap_two_pi(100.0)));
    }
}
