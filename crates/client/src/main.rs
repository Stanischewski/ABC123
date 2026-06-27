//! Client-Skelett (Phase 0).
//!
//! Noch keine Grafik — das ist Absicht. Dieses Skelett zeigt, dass der
//! geteilte `core`-Kern auch client-seitig läuft: Es baut das Demo-Heimatsystem
//! auf und propagiert die Körperpositionen über mehrere Zeitpunkte **selbst**
//! aus den Bahnelementen (DESIGN.md §5.6) — exakt wie es später die
//! schematische Systemansicht (egui, Phase 1) und die Bevy-Renderung (Phase 2)
//! tun werden.

use gamecore::Resource;

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
}
