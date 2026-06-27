//! Produktionsauflösung: integriert die Bau-Ebene über die Zeit.
//!
//! Passt zum ereignisbasierten Sim-Modell (DESIGN.md §5.4): Lager werden **bei
//! Bedarf** für ein Zeitintervall `dt` fortgeschrieben, nicht jede Sekunde
//! getickt. Die Rate folgt `Basis × Adjazenz × Energie × Input`
//! (Begleitdokument §5); bei Knappheit drosselt nach Priorität.
//!
//! Ein Schritt läuft in fester, zyklenfreier Reihenfolge:
//! 1. **Energie-Angebot** — Solar (skaliert mit `1/r²` zum Bahnradius) plus
//!    Fusion (verbrennt vorhandenes Gas).
//! 2. **Energie-Verteilung** — Angebot gegen priorisierten Bedarf
//!    ([`crate::economy::allocate_energy`]).
//! 3. **Produktion in Stufen-Reihenfolge** — erst roh, dann veredelt, dann
//!    Gate-Gut; jede Stufe sieht die Lager der vorigen.

use std::collections::HashMap;

use crate::economy::{allocate_energy, EnergyDemand, Stockpile};
use crate::planet::Grid;
use crate::resource::{Resource, Tier};

/// Referenz-Bahnradius für Solarertrag (1 AE in km): bei diesem Radius liefert
/// ein Kollektor seine Nennleistung.
pub const SOLAR_REFERENCE_RADIUS_KM: f64 = 1.495_978_707e8;

/// Zusammenfassung eines aufgelösten Schritts — nützlich für UI und Tests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepReport {
    /// Energie-Angebot pro Sekunde (Solar + tatsächlich gefeuerte Fusion).
    pub energy_supply: f64,
    /// Energie-Bedarf pro Sekunde bei voller Last aller Verbraucher.
    pub energy_demand: f64,
    /// Verstrichene Sim-Zeit dieses Schritts (Sekunden).
    pub dt: f64,
}

impl StepReport {
    /// Ob das Energiebudget die volle Last trägt.
    pub fn energy_satisfied(&self) -> bool {
        self.energy_supply + 1e-9 >= self.energy_demand
    }
}

/// Schreibt `stock` um `dt` Sekunden fort, gegeben das Raster und den aktuellen
/// Bahnradius des Körpers (für den Solarertrag).
pub fn resolve_step(grid: &Grid, stock: &mut Stockpile, orbit_radius_km: f64, dt: f64) -> StepReport {
    let dt = dt.max(0.0);

    // --- 1. Energie-Angebot ---------------------------------------------------
    let solar_factor = if orbit_radius_km > 0.0 {
        (SOLAR_REFERENCE_RADIUS_KM / orbit_radius_km).powi(2)
    } else {
        0.0
    };

    let mut energy_supply = 0.0;
    for (_, _, b) in grid.buildings() {
        if !b.is_operational() {
            continue;
        }
        let spec = b.kind.spec();
        if spec.solar {
            energy_supply += spec.energy_output * solar_factor;
        } else if spec.fuel_rate > 0.0 && spec.energy_output > 0.0 {
            // Fusion: durch vorhandenes Gas begrenzt (verbrennt Startbestand).
            let needed = spec.fuel_rate * dt;
            let frac = if needed > 0.0 {
                (stock.get(crate::resource::Resource::Gases) / needed).clamp(0.0, 1.0)
            } else {
                1.0
            };
            stock.add(crate::resource::Resource::Gases, -spec.fuel_rate * dt * frac);
            energy_supply += spec.energy_output * frac;
        } else if spec.energy_output > 0.0 {
            energy_supply += spec.energy_output;
        }
    }

    // --- 2. Energie-Verteilung nach Priorität --------------------------------
    // Verbraucher = alle Produzenten mit Energiebedarf. Wir merken uns ihre
    // Rasterposition, um nach der Zuteilung die Rate zu berechnen.
    let mut consumers: Vec<(u32, u32, EnergyDemand)> = Vec::new();
    for (x, y, b) in grid.buildings() {
        if !b.is_operational() {
            continue;
        }
        let spec = b.kind.spec();
        if spec.output.is_some() && spec.energy_demand > 0.0 {
            consumers.push((
                x,
                y,
                EnergyDemand {
                    priority: b.priority,
                    amount: spec.energy_demand,
                },
            ));
        }
    }
    let energy_demand: f64 = consumers.iter().map(|(_, _, d)| d.amount).sum();
    let demands: Vec<EnergyDemand> = consumers.iter().map(|(_, _, d)| *d).collect();
    let energy_avail = allocate_energy(energy_supply, &demands);

    // Schneller Zugriff: Energie-Verfügbarkeit je Rasterposition.
    let avail_at = |x: u32, y: u32| -> f64 {
        consumers
            .iter()
            .position(|(cx, cy, _)| *cx == x && *cy == y)
            .map(|i| energy_avail[i])
            .unwrap_or(1.0)
    };

    // --- 3. Produktion in Stufen-Reihenfolge ---------------------------------
    // Volle Lager üben Gegendruck aus: Produktion staut sich an der Kapazität,
    // statt überzulaufen — und Eingänge bleiben erhalten (keine Verschwendung).
    // Forschung ist ausgenommen (eine Währung, kein Lagergut).
    //
    // Die Decke ist **verbrauchsbewusst**: ein Produzent darf so viel nachfüllen,
    // wie diesen Schritt auch verbraucht wird. Sonst würde ein Stoff, der zugleich
    // gefördert *und* verbraucht wird (Metalle!), bei voller Decke jeden Schritt
    // um den Verbrauch oszillieren — die Förderung würde am Schrittanfang
    // blockiert, bevor der Verbrauch später Platz schafft. Das ergäbe ein
    // flackerndes „1 raus, 1 rein". Mit dem Verbrauch in der Decke bleibt der
    // Bestand stabil an der Kapazität (Netto-Rate ~0).
    let capacity = grid.storage_capacity();

    let mut consumption: HashMap<Resource, f64> = HashMap::new();
    for (x, y, b) in grid.buildings() {
        if !b.is_operational() {
            continue;
        }
        let spec = b.kind.spec();
        let Some(output) = spec.output else { continue };
        let Some(recipe) = output.recipe() else { continue };
        let desired = spec.base_rate * grid.adjacency_multiplier(x, y) * avail_at(x, y) * dt;
        for (res, qty) in recipe.inputs {
            *consumption.entry(*res).or_insert(0.0) += desired * qty;
        }
    }

    // Bestand vor der Produktion — die Sicherheits-Klemme darf bereits
    // über der Decke liegende Vorräte (z. B. nach Lager-Abriss) nicht abräumen,
    // nur frische Produktion deckeln.
    let before: Vec<(Resource, f64)> = Resource::ALL.iter().map(|r| (*r, stock.get(*r))).collect();

    for tier in [Tier::Raw, Tier::Refined, Tier::Gate, Tier::Research] {
        for (x, y, b) in grid.buildings() {
            if !b.is_operational() {
                continue;
            }
            let spec = b.kind.spec();
            let Some(output) = spec.output else { continue };
            if output.tier() != tier {
                continue;
            }

            let rate = spec.base_rate * grid.adjacency_multiplier(x, y) * avail_at(x, y);
            let desired = rate * dt;
            if desired <= 0.0 {
                continue;
            }

            let headroom = if output.tier() == Tier::Research {
                f64::INFINITY
            } else {
                (capacity - stock.get(output)).max(0.0)
                    + consumption.get(&output).copied().unwrap_or(0.0)
            };

            match output.recipe() {
                // Roh: keine Eingänge, Förderung staut an der (verbrauchsbewussten) Decke.
                None => stock.add(output, desired.min(headroom)),
                // Veredelt/Gate: durch Eingangsbestand *und* Kapazität gedrosselt.
                Some(recipe) => {
                    let mut frac = 1.0_f64;
                    for (res, qty) in recipe.inputs {
                        let need = desired * qty;
                        if need > 0.0 {
                            frac = frac.min(stock.get(*res) / need);
                        }
                    }
                    let actual = (desired * frac.clamp(0.0, 1.0)).min(headroom);
                    for (res, qty) in recipe.inputs {
                        stock.add(*res, -actual * qty);
                    }
                    stock.add(output, actual);
                }
            }
        }
    }

    // Sicherheits-Klemme: war der Verbrauch input-/energie-begrenzt, kann ein
    // Stoff knapp über die Decke geraten — der Überlauf verfällt. Bereits zuvor
    // über der Decke liegende Vorräte bleiben aber unangetastet.
    for (r, start) in before {
        if r.tier() == Tier::Research {
            continue;
        }
        let ceiling = capacity.max(start);
        if stock.get(r) > ceiling {
            stock.set(r, ceiling);
        }
    }

    StepReport {
        energy_supply,
        energy_demand,
        dt,
    }
}

/// Ein voller Simulationsschritt: erst **Produktion** ([`resolve_step`]), dann
/// **Baufortschritt** ([`Grid::advance_construction`]). Baustellen ziehen ihr
/// Material aus demselben globalen Pool — die in diesem Schritt geförderten
/// Stoffe stehen dem Bau bereits zur Verfügung.
pub fn advance(grid: &mut Grid, stock: &mut Stockpile, orbit_radius_km: f64, dt: f64) -> StepReport {
    let report = resolve_step(&*grid, stock, orbit_radius_km, dt);
    grid.advance_construction(stock, dt);
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::{Building, BuildingKind, Grid, Terrain};
    use crate::resource::Resource;

    const AU: f64 = SOLAR_REFERENCE_RADIUS_KM;

    /// Mine + Solarkollektor auf einem kleinen Raster.
    fn mine_with_power() -> Grid {
        let mut g = Grid::new(3, 1, Terrain::Barren);
        g.set_terrain(0, 0, Terrain::Rock);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        g
    }

    #[test]
    fn mine_with_power_produces_metals() {
        let g = mine_with_power();
        let mut stock = Stockpile::new();
        let report = resolve_step(&g, &mut stock, AU, 100.0);
        // Mine: base 1.0/s × 100s = 100 Metalle (volle Energie).
        assert!((stock.get(Resource::Metals) - 100.0).abs() < 1e-6);
        assert!(report.energy_satisfied());
    }

    #[test]
    fn mine_without_power_produces_nothing() {
        let mut g = Grid::new(1, 1, Terrain::Rock);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        let mut stock = Stockpile::new();
        let report = resolve_step(&g, &mut stock, AU, 100.0);
        assert_eq!(stock.get(Resource::Metals), 0.0);
        assert!(!report.energy_satisfied());
    }

    #[test]
    fn adjacency_increases_output() {
        // Ohne Nachbarn: Mine(0) … Solar(1) → genau 100 Metalle.
        let plain = mine_with_power();
        let mut s_plain = Stockpile::new();
        resolve_step(&plain, &mut s_plain, AU, 100.0);
        assert!((s_plain.get(Resource::Metals) - 100.0).abs() < 1e-6);

        // Mit Lager direkt neben der Mine: Mine(0) - Depot(1) - Solar(2) → +10 %.
        let mut boosted = Grid::new(3, 1, Terrain::Barren);
        boosted.set_terrain(0, 0, Terrain::Rock);
        boosted.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        boosted.place(1, 0, Building::new(BuildingKind::Depot)).unwrap();
        boosted
            .place(2, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        let mut s_boost = Stockpile::new();
        resolve_step(&boosted, &mut s_boost, AU, 100.0);
        assert!((s_boost.get(Resource::Metals) - 110.0).abs() < 1e-6);
    }

    #[test]
    fn smelter_consumes_metals_and_makes_alloys() {
        let mut g = Grid::new(2, 1, Terrain::Barren);
        g.place(0, 0, Building::new(BuildingKind::Smelter)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        let mut stock = Stockpile::new();
        stock.set(Resource::Metals, 1000.0);
        resolve_step(&g, &mut stock, AU, 100.0);
        // Smelter: 0.5/s × 100s = 50 Legierungen, verbraucht 2× = 100 Metalle.
        assert!((stock.get(Resource::Alloys) - 50.0).abs() < 1e-6);
        assert!((stock.get(Resource::Metals) - 900.0).abs() < 1e-6);
    }

    #[test]
    fn refinery_throttled_by_missing_input() {
        let mut g = Grid::new(2, 1, Terrain::Barren);
        g.place(0, 0, Building::new(BuildingKind::Smelter)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        let mut stock = Stockpile::new();
        // Nur 20 Metalle: reicht für 10 Legierungen (2 Metalle je Stück).
        stock.set(Resource::Metals, 20.0);
        resolve_step(&g, &mut stock, AU, 100.0);
        assert!((stock.get(Resource::Alloys) - 10.0).abs() < 1e-6);
        assert!(stock.get(Resource::Metals) < 1e-6);
    }

    #[test]
    fn energy_scarcity_throttles_by_priority() {
        // Zwei Minen, aber Energie nur für eine; höhere Priorität gewinnt.
        let mut g = Grid::new(3, 1, Terrain::Rock);
        // Solar liefert 10 Energie; jede Mine will 2 — eigentlich reicht das für
        // beide. Um Knappheit zu erzwingen, nehmen wir energiehungrige Fabs.
        g.set_terrain(2, 0, Terrain::Barren);
        g.place(0, 0, Building::with_priority(BuildingKind::Smelter, 10))
            .unwrap();
        g.place(1, 0, Building::with_priority(BuildingKind::Smelter, 1))
            .unwrap();
        g.place(2, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        // Solar = 10 Energie; zwei Smelter à 3 = 6 Bedarf → eigentlich gedeckt.
        // Reduziere Angebot über großen Bahnradius (Solar fällt mit 1/r²).
        let mut stock = Stockpile::new();
        stock.set(Resource::Metals, 100_000.0);
        // r so wählen, dass Solar ≈ 4 liefert: factor = 4/10 → r = AU/sqrt(0.4).
        let r = AU / (0.4_f64).sqrt();
        resolve_step(&g, &mut stock, r, 100.0);
        // Hohe Prio (3 Energie) voll bedient → 50 Legierungen.
        // Niedrige bekommt Rest (1 Energie / 3 nötig = 1/3) → ~16.7.
        let alloys = stock.get(Resource::Alloys);
        // Voll wäre 100; Knappheit drückt es darunter.
        assert!(alloys < 100.0 && alloys > 60.0, "alloys = {alloys}");
    }

    #[test]
    fn solar_scales_with_orbit_radius() {
        let g = mine_with_power();
        // Doppelter Radius → Viertel der Solarleistung.
        let mut near = Stockpile::new();
        let r_near = resolve_step(&g, &mut near, AU, 1.0);
        let mut far = Stockpile::new();
        let r_far = resolve_step(&g, &mut far, AU * 2.0, 1.0);
        assert!((r_near.energy_supply - 10.0).abs() < 1e-6);
        assert!((r_far.energy_supply - 2.5).abs() < 1e-6);
    }

    #[test]
    fn set_priority_redirects_scarce_energy() {
        // Knappe Energie, zwei *verschiedene* Verbraucher: Hütte (Legierungen)
        // und Elektronik-Fab. Wer Priorität bekommt, läuft; der andere hungert.
        let build = |smelter_first: bool| {
            let mut g = Grid::new(3, 1, Terrain::Barren);
            g.place(0, 0, Building::new(BuildingKind::Smelter)).unwrap();
            g.place(1, 0, Building::new(BuildingKind::ElectronicsFab)).unwrap();
            g.place(2, 0, Building::new(BuildingKind::SolarCollector)).unwrap();
            if smelter_first {
                g.set_priority(0, 0, 10);
            } else {
                g.set_priority(1, 0, 10);
            }
            g
        };
        // Solar bei großem Radius → ~4 Energie; Bedarf 3 (Hütte) + 4 (Fab) = 7.
        let r = AU / (0.4_f64).sqrt();
        let mut stocked = || {
            let mut s = Stockpile::new();
            s.set(Resource::Metals, 100_000.0);
            s.set(Resource::Silicates, 100_000.0);
            s
        };

        // Hütte zuerst: sie läuft voll (50 Legierungen), die Fab hungert.
        let mut sa = stocked();
        resolve_step(&build(true), &mut sa, r, 100.0);
        // Fab zuerst: sie läuft voll, die Hütte bekommt kaum/nichts.
        let mut sb = stocked();
        resolve_step(&build(false), &mut sb, r, 100.0);

        assert!(
            sa.get(Resource::Alloys) > sb.get(Resource::Alloys) + 10.0,
            "Hütte mit Priorität sollte deutlich mehr Legierungen liefern: {} vs {}",
            sa.get(Resource::Alloys),
            sb.get(Resource::Alloys)
        );
        assert!(
            sb.get(Resource::Electronics) > sa.get(Resource::Electronics) + 5.0,
            "Fab mit Priorität sollte deutlich mehr Elektronik liefern: {} vs {}",
            sb.get(Resource::Electronics),
            sa.get(Resource::Electronics)
        );
    }

    #[test]
    fn advance_produces_then_builds_from_same_pool() {
        // Betriebsbereite Mine + Solar fördern Metalle (Zentrale hebt die
        // Lager-Decke); eine Depot-Baustelle wird im selben Schritt aus dem
        // frisch geförderten Material fertig.
        let mut g = Grid::new(4, 1, Terrain::Rock);
        g.set_terrain(2, 0, Terrain::Barren);
        g.set_terrain(3, 0, Terrain::Barren);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        g.place(2, 0, Building::construction_site(BuildingKind::Depot))
            .unwrap();
        g.place(3, 0, Building::new(BuildingKind::Headquarters))
            .unwrap();

        let mut stock = Stockpile::new();
        // Eine Stunde: Förderung füllt das Lager (Zentrale → Kapazität 600),
        // Depot (3600 s, 30 Metalle) wird fertig.
        advance(&mut g, &mut stock, AU, 3_600.0);
        assert!(stock.get(Resource::Metals) > 400.0);
        assert!(g.tile(2, 0).unwrap().building.unwrap().is_operational());
    }

    #[test]
    fn production_stops_at_storage_capacity() {
        // Mine + Solar, kein Lager → Decke = STORAGE_BASE (100).
        let mut g = Grid::new(2, 1, Terrain::Rock);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        let cap = g.storage_capacity();
        let mut stock = Stockpile::new();
        // Lange Zeit: Förderung staut sich an der Decke.
        resolve_step(&g, &mut stock, AU, 1_000_000.0);
        assert!((stock.get(Resource::Metals) - cap).abs() < 1e-6);
    }

    #[test]
    fn full_storage_does_not_oscillate() {
        // Metalle werden zugleich gefördert (Mine) und verbraucht (Hütte), genau
        // im Gleichgewicht. Bei voller Decke muss der Bestand stabil bleiben —
        // nicht jeden Schritt „1 raus, 1 rein" flackern.
        let mut g = Grid::new(5, 1, Terrain::Barren);
        g.set_terrain(0, 0, Terrain::Rock);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        // (1,0) bleibt leer → Mine und Hütte sind nicht benachbart (keine Adjazenz).
        g.place(2, 0, Building::new(BuildingKind::Smelter)).unwrap();
        g.place(3, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        g.place(4, 0, Building::new(BuildingKind::Depot)).unwrap();
        let cap = g.storage_capacity();
        let mut stock = Stockpile::new();
        stock.set(Resource::Metals, cap);

        resolve_step(&g, &mut stock, AU, 100.0);
        let m1 = stock.get(Resource::Metals);
        resolve_step(&g, &mut stock, AU, 100.0);
        let m2 = stock.get(Resource::Metals);
        assert!((m1 - cap).abs() < 1e-6, "Metalle nicht an der Decke: {m1}");
        assert!((m1 - m2).abs() < 1e-6, "Metalle oszillieren: {m1} vs {m2}");
    }

    #[test]
    fn full_storage_preserves_refinery_inputs() {
        // Hütte + Solar, Legierungen schon am Deckel → kein Output, und die
        // Metalle bleiben unangetastet (kein Input-Verbrauch ins Leere).
        let mut g = Grid::new(2, 1, Terrain::Barren);
        g.place(0, 0, Building::new(BuildingKind::Smelter)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        let cap = g.storage_capacity();
        let mut stock = Stockpile::new();
        stock.set(Resource::Alloys, cap);
        stock.set(Resource::Metals, 1_000.0);
        resolve_step(&g, &mut stock, AU, 100.0);
        assert!((stock.get(Resource::Alloys) - cap).abs() < 1e-6);
        assert!((stock.get(Resource::Metals) - 1_000.0).abs() < 1e-6);
    }

    #[test]
    fn disabled_building_is_inert() {
        // Mine + Solar, aber der Solarkollektor ist ausgeschaltet → keine
        // Energie → die Mine fördert nichts.
        let mut g = Grid::new(2, 1, Terrain::Rock);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        assert!(g.set_enabled(1, 0, false));
        let mut stock = Stockpile::new();
        let report = resolve_step(&g, &mut stock, AU, 100.0);
        assert_eq!(report.energy_supply, 0.0);
        assert_eq!(stock.get(Resource::Metals), 0.0);

        // Wieder eingeschaltet → Förderung läuft.
        let mut g2 = g.clone();
        assert!(g2.set_enabled(1, 0, true));
        let mut stock2 = Stockpile::new();
        resolve_step(&g2, &mut stock2, AU, 100.0);
        assert!((stock2.get(Resource::Metals) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn disabled_depot_grants_no_adjacency() {
        let mut g = Grid::new(2, 1, Terrain::Rock);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::Depot)).unwrap();
        assert!((g.adjacency_multiplier(0, 0) - 1.1).abs() < 1e-9);
        g.set_enabled(1, 0, false);
        assert!((g.adjacency_multiplier(0, 0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn headquarters_produces_metals_without_energy() {
        // Keine Energiequelle — die Zentrale fördert trotzdem (Anti-Softlock).
        let mut g = Grid::new(1, 1, Terrain::Barren);
        g.place(0, 0, Building::new(BuildingKind::Headquarters))
            .unwrap();
        let mut stock = Stockpile::new();
        resolve_step(&g, &mut stock, AU, 100.0);
        // base_rate 0.25/s × 100 s = 25 Metalle.
        assert!((stock.get(Resource::Metals) - 25.0).abs() < 1e-6);
    }

    #[test]
    fn research_lab_turns_electronics_into_research() {
        let mut g = Grid::new(2, 1, Terrain::Barren);
        g.place(0, 0, Building::new(BuildingKind::ResearchLab)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::SolarCollector))
            .unwrap();
        let mut stock = Stockpile::new();
        stock.set(Resource::Electronics, 1000.0);
        resolve_step(&g, &mut stock, AU, 100.0);
        // 0.3/s × 100 s = 30 Forschung, verbraucht 30 Elektronik.
        assert!((stock.get(Resource::Research) - 30.0).abs() < 1e-6);
        assert!((stock.get(Resource::Electronics) - 970.0).abs() < 1e-6);
    }

    #[test]
    fn fusion_burns_gas_for_energy() {
        let mut g = Grid::new(1, 1, Terrain::Barren);
        g.place(0, 0, Building::new(BuildingKind::FusionReactor))
            .unwrap();
        let mut stock = Stockpile::new();
        stock.set(Resource::Gases, 50.0);
        let report = resolve_step(&g, &mut stock, AU, 100.0);
        // Fusion will 1 Gas/s × 100s = 100, hat aber nur 50 → 50 % Leistung.
        assert!((report.energy_supply - 7.5).abs() < 1e-6);
        assert!(stock.get(Resource::Gases) < 1e-6);
    }
}
