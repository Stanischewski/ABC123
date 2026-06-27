//! Client-Skelett (Phase 0).
//!
//! Noch keine Grafik — das ist Absicht. Dieses Skelett zeigt, dass der
//! geteilte `core`-Kern auch client-seitig läuft: Es baut das Demo-Heimatsystem
//! auf und propagiert die Körperpositionen über mehrere Zeitpunkte **selbst**
//! aus den Bahnelementen (DESIGN.md §5.6) — exakt wie es später die
//! schematische Systemansicht (egui, Phase 1) und die Bevy-Renderung (Phase 2)
//! tun werden.

use gamecore::{Building, BuildingKind, Resource, Stockpile};

fn main() {
    let system = gamecore::demo_home_system();

    println!("== {} ==", system.name);
    println!("Körper:");
    for body in &system.bodies {
        let orbit = match &body.orbit {
            Some(o) => format!(
                "a = {:.3e} km, e = {:.3}, T = {:.1} Tage",
                o.semi_major_axis,
                o.eccentricity,
                o.period() / 86_400.0
            ),
            None => "— (ruht im Ursprung)".to_string(),
        };
        println!("  [{}] {:<8} {:?}  {}", body.id, body.name, body.kind, orbit);
    }

    // Positionen über ein paar Tage propagieren — rein client-seitig.
    println!("\nPropagation (Heimat & Mond, relativ zum Stern):");
    let day = 86_400.0;
    for d in 0..=4 {
        let t = d as f64 * 30.0 * day; // alle 30 Sim-Tage
        let planet = system.position_of(1, t).unwrap();
        let moon = system.position_of(2, t).unwrap();
        println!(
            "  t = {:>3} Tage: Heimat ({:+.3e}, {:+.3e})  Mond-Abstand {:.0} km",
            (t / day) as i64,
            planet.x,
            planet.y,
            planet.distance(moon)
        );
    }

    // Kurzer Blick auf das Ressourcenmodell (3 + 2 + 1).
    println!("\nRessourcenmodell (3 roh + 2 veredelt + 1 Gate-Gut):");
    for r in Resource::ALL {
        match r.recipe() {
            Some(recipe) => {
                let inputs: Vec<String> = recipe
                    .inputs
                    .iter()
                    .map(|(res, qty)| format!("{qty}× {res:?}"))
                    .collect();
                println!(
                    "  {:?} ({:?}) ← {} (+ {} Energie)",
                    r,
                    r.tier(),
                    inputs.join(" + "),
                    recipe.energy_cost
                );
            }
            None => println!("  {:?} ({:?}) ← Förderung aus Gelände", r, r.tier()),
        }
    }

    // --- Bau-Ebene (Phase 1): eine kleine Starterbasis aufbauen und simulieren.
    println!("\nBau-Ebene — Starterbasis auf dem Heimatplaneten:");
    let mut planet = gamecore::demo_home_planet();
    // Mine auf Gestein, daneben Lager (Adjazenz) und ein Solarkollektor.
    let layout = [
        (1, 1, BuildingKind::MetalMine),
        (2, 1, BuildingKind::Depot),
        (3, 1, BuildingKind::SolarCollector),
        (6, 1, BuildingKind::CrystalExtractor),
        (4, 1, BuildingKind::Smelter),
    ];
    for (x, y, kind) in layout {
        match planet.place(x, y, Building::new(kind)) {
            Ok(()) => println!("  platziert: {kind:?} @ ({x},{y})"),
            Err(e) => println!("  ABGELEHNT: {kind:?} @ ({x},{y}) — {e:?}"),
        }
    }

    // Lager als Startbestand, damit der Smelter sofort etwas zu veredeln hat.
    let mut stock = Stockpile::new();
    stock.set(Resource::Metals, 200.0);

    // Den Planeten über einen Sim-Tag fortschreiben (Heimat steht auf ~1 AE).
    let radius = system.position_of(1, 0.0).unwrap().length();
    let report = gamecore::resolve_step(&planet, &mut stock, radius, 86_400.0);

    println!(
        "\nNach 1 Sim-Tag (Energie {:.1}/{:.1}/s, {}):",
        report.energy_supply,
        report.energy_demand,
        if report.energy_satisfied() {
            "gedeckt"
        } else {
            "KNAPP"
        }
    );
    for r in Resource::ALL {
        let amount = stock.get(r);
        if amount.abs() > 1e-6 {
            println!("  {:?}: {:.0}", r, amount);
        }
    }
}
