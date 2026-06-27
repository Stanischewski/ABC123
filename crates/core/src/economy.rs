//! Ökonomie: Lager, Energiebudget, Produktionsrate und Logistik-Kapazität.
//!
//! Leitgedanke **allokieren statt routen** (Begleitdokument §1): Materialfluss
//! ist abstrakt und global, Energie ein flaches Budget mit Priorität, Logistik
//! eine räumliche Kapazitätsschicht — mechanisch dasselbe wie das Energiebudget,
//! nur distanz-gewichtet (Begleitdokument §2, §8).
//!
//! Lager **integrieren über die Zeit** und werden **beim Abruf** berechnet
//! (kein Sekunden-Tick, DESIGN.md §5.4 / Begleitdokument §5).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resource::Resource;

/// Untergrenze, ab der ein Bruchteil als „voll" gilt (Gleitkomma-Toleranz).
const FULL_EPSILON: f64 = 1e-9;

/// Ein globales Lager: eine fungible Menge je Stoff, ortlos (Begleitdokument §2).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stockpile {
    amounts: HashMap<Resource, f64>,
}

impl Stockpile {
    pub fn new() -> Self {
        Stockpile::default()
    }

    /// Aktueller Bestand eines Stoffs (0, falls nie gesetzt).
    pub fn get(&self, r: Resource) -> f64 {
        self.amounts.get(&r).copied().unwrap_or(0.0)
    }

    /// Setzt den Bestand absolut.
    pub fn set(&mut self, r: Resource, amount: f64) {
        self.amounts.insert(r, amount.max(0.0));
    }

    /// Fügt hinzu (negativ = Abbau); klemmt bei 0.
    pub fn add(&mut self, r: Resource, delta: f64) {
        let next = (self.get(r) + delta).max(0.0);
        self.amounts.insert(r, next);
    }

    /// Versucht, `amount` zu entnehmen. Gibt `true` bei Erfolg, sonst bleibt der
    /// Bestand unverändert.
    pub fn try_consume(&mut self, r: Resource, amount: f64) -> bool {
        if self.get(r) + FULL_EPSILON >= amount {
            self.add(r, -amount);
            true
        } else {
            false
        }
    }
}

/// Logistik-Effizienz: weicher Abfall `min(1, Angebot/Bedarf)`, nie auf eine
/// Klippe (Begleitdokument §8). `floor` modelliert den lokalen Restbetrieb
/// („ein Produzent geht nie ganz auf null").
///
/// Identisch nutzbar für das Energiebudget (Angebot vs. Bedarf, flach).
pub fn logistics_efficiency(supply: f64, demand: f64, floor: f64) -> f64 {
    if demand <= FULL_EPSILON {
        return 1.0;
    }
    let raw = (supply / demand).clamp(0.0, 1.0);
    raw.max(floor).min(1.0)
}

/// Ein Energie-Verbraucher mit Priorität (höher = wird zuerst bedient).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnergyDemand {
    pub priority: i32,
    pub amount: f64,
}

/// Flaches Energiebudget: Angebot gegen priorisierten Bedarf
/// (DESIGN.md §4.1, Begleitdokument §2).
///
/// Verteilt das Angebot strikt nach Priorität: hohe Prioritäten werden voll
/// bedient, bevor niedrigere etwas bekommen; bei Gleichstand wird anteilig
/// gedrosselt. Gibt je Verbraucher den **Verfügbarkeitsbruch** `[0,1]` zurück
/// — passend zum Produktionsfaktor `Energie-verfügbar`.
pub fn allocate_energy(supply: f64, demands: &[EnergyDemand]) -> Vec<f64> {
    let mut result = vec![0.0; demands.len()];

    // Indizes nach Priorität absteigend gruppieren.
    let mut order: Vec<usize> = (0..demands.len()).collect();
    order.sort_by(|&a, &b| demands[b].priority.cmp(&demands[a].priority));

    let mut remaining = supply.max(0.0);
    let mut i = 0;
    while i < order.len() {
        // Alle Verbraucher gleicher Priorität bilden eine Schicht.
        let prio = demands[order[i]].priority;
        let mut layer = Vec::new();
        while i < order.len() && demands[order[i]].priority == prio {
            layer.push(order[i]);
            i += 1;
        }

        let layer_demand: f64 = layer.iter().map(|&idx| demands[idx].amount).sum();
        if layer_demand <= FULL_EPSILON {
            continue;
        }

        // Anteil, den diese Schicht insgesamt bekommt.
        let frac = (remaining / layer_demand).clamp(0.0, 1.0);
        for &idx in &layer {
            result[idx] = frac;
        }
        remaining -= layer_demand * frac;
        if remaining <= FULL_EPSILON {
            break;
        }
    }

    result
}

/// Ein Produzent (Förderer oder Raffinerie) auf der Bau-Ebene.
///
/// Die Rate folgt `Basis × Adjazenz × Energie-verfügbar × Input-verfügbar`
/// (Begleitdokument §5). `adjacency` bündelt die Platzierungs-Boni (≥ 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Producer {
    pub output: Resource,
    /// Basisrate in Output-Einheiten pro Sekunde bei voller Versorgung.
    pub base_rate: f64,
    /// Multiplikator aus Adjazenz/Gelände (1.0 = neutral).
    pub adjacency: f64,
}

impl Producer {
    /// Effektive Produktionsrate (Output-Einheiten/Sekunde) gegeben die
    /// Verfügbarkeit von Energie und Eingängen (jeweils `[0,1]`).
    pub fn rate(&self, energy_available: f64, input_available: f64) -> f64 {
        self.base_rate
            * self.adjacency
            * energy_available.clamp(0.0, 1.0)
            * input_available.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stockpile_clamps_at_zero_and_consumes() {
        let mut s = Stockpile::new();
        s.add(Resource::Metals, 10.0);
        assert!(s.try_consume(Resource::Metals, 4.0));
        assert!((s.get(Resource::Metals) - 6.0).abs() < 1e-9);
        // Zu viel: schlägt fehl, Bestand unverändert.
        assert!(!s.try_consume(Resource::Metals, 100.0));
        assert!((s.get(Resource::Metals) - 6.0).abs() < 1e-9);
        // Negatives Add klemmt bei 0.
        s.add(Resource::Metals, -1000.0);
        assert_eq!(s.get(Resource::Metals), 0.0);
    }

    #[test]
    fn logistics_efficiency_soft_falloff() {
        assert_eq!(logistics_efficiency(100.0, 50.0, 0.0), 1.0); // Überangebot
        assert!((logistics_efficiency(30.0, 60.0, 0.0) - 0.5).abs() < 1e-9);
        assert_eq!(logistics_efficiency(0.0, 60.0, 0.0), 0.0);
        // Boden hält den Restbetrieb.
        assert!((logistics_efficiency(0.0, 60.0, 0.2) - 0.2).abs() < 1e-9);
        // Kein Bedarf → volle Effizienz.
        assert_eq!(logistics_efficiency(0.0, 0.0, 0.0), 1.0);
    }

    #[test]
    fn energy_is_allocated_by_priority() {
        // Angebot 10, Bedarf: hohe Prio 6, niedrige Prio 8.
        let demands = [
            EnergyDemand { priority: 10, amount: 6.0 },
            EnergyDemand { priority: 1, amount: 8.0 },
        ];
        let alloc = allocate_energy(10.0, &demands);
        // Hohe Prio voll, Rest (4) auf die niedrige (4/8 = 0.5).
        assert!((alloc[0] - 1.0).abs() < 1e-9);
        assert!((alloc[1] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn equal_priority_throttles_proportionally() {
        let demands = [
            EnergyDemand { priority: 5, amount: 4.0 },
            EnergyDemand { priority: 5, amount: 4.0 },
        ];
        // Nur die Hälfte des Bedarfs verfügbar → beide auf 0.5.
        let alloc = allocate_energy(4.0, &demands);
        assert!((alloc[0] - 0.5).abs() < 1e-9);
        assert!((alloc[1] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn surplus_energy_serves_everyone_fully() {
        let demands = [
            EnergyDemand { priority: 1, amount: 3.0 },
            EnergyDemand { priority: 2, amount: 3.0 },
        ];
        let alloc = allocate_energy(100.0, &demands);
        assert!(alloc.iter().all(|&f| (f - 1.0).abs() < 1e-9));
    }

    #[test]
    fn producer_rate_multiplies_factors() {
        let p = Producer {
            output: Resource::Metals,
            base_rate: 10.0,
            adjacency: 1.5,
        };
        // Volle Versorgung.
        assert!((p.rate(1.0, 1.0) - 15.0).abs() < 1e-9);
        // Halbe Energie halbiert die Rate.
        assert!((p.rate(0.5, 1.0) - 7.5).abs() < 1e-9);
        // Fehlender Input stoppt die Förderung.
        assert_eq!(p.rate(1.0, 0.0), 0.0);
    }
}
