//! Client — Bau-Ebene (Phase 1), egui/eframe.
//!
//! Die interaktive Oberfläche des ökonomischen Herzens (DESIGN.md §3.1, §5.2):
//! ein Planeten-Raster, auf das Gebäude gesetzt werden, mit Lager-, Energie- und
//! Adjazenz-Rückmeldung und einer Simulation, die sich schrittweise oder
//! automatisch fortschreiben lässt. Die gesamte Spiellogik liegt im geteilten
//! `core` (als `gamecore`); diese Crate stellt nur dar und nimmt Eingaben.
//!
//! Nativ lauffähig; dieselbe App lässt sich später nach Wasm (trunk) bauen.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use gamecore::{Building, BuildingKind, Grid, Resource, StepReport, Stockpile, Terrain};

/// Gebäude in der Palette, in Bau-Reihenfolge.
const PALETTE: [BuildingKind; 9] = [
    BuildingKind::MetalMine,
    BuildingKind::CrystalExtractor,
    BuildingKind::GasCollector,
    BuildingKind::Smelter,
    BuildingKind::ElectronicsFab,
    BuildingKind::CompositeFab,
    BuildingKind::SolarCollector,
    BuildingKind::FusionReactor,
    BuildingKind::Depot,
];

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ABC123 — Bau-Ebene (Phase 1)",
        options,
        Box::new(|_cc| Ok(Box::new(BuildApp::new()))),
    )
}

/// Was nach dem Rendern (außerhalb der Render-Closures) auf den Zustand
/// angewendet wird — vermeidet Borrow-Konflikte mit den egui-Closures.
enum Action {
    Step(f64),
    Toggle(u32, u32),
}

struct BuildApp {
    grid: Grid,
    stock: Stockpile,
    sim_time: f64,
    orbit_radius_km: f64,
    selected: BuildingKind,
    auto: bool,
    last_report: Option<StepReport>,
    log: Vec<String>,
}

impl BuildApp {
    fn new() -> Self {
        // Heimatplanet und sein aktueller Bahnradius (für den Solarertrag).
        let system = gamecore::demo_home_system();
        let orbit_radius_km = system.position_of(1, 0.0).map(|p| p.length()).unwrap_or(1.0);

        // Großzügiger Startbestand, damit man sofort bauen kann.
        let mut stock = Stockpile::new();
        stock.set(Resource::Metals, 1500.0);
        stock.set(Resource::Alloys, 200.0);
        stock.set(Resource::Electronics, 100.0);
        stock.set(Resource::Gases, 200.0);

        BuildApp {
            grid: gamecore::demo_home_planet(),
            stock,
            sim_time: 0.0,
            orbit_radius_km,
            selected: BuildingKind::MetalMine,
            auto: false,
            last_report: None,
            log: vec!["Willkommen. Wähle ein Gebäude und klicke eine Kachel.".into()],
        }
    }

    /// Schreibt die Simulation um `dt` Sekunden fort.
    fn step(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        self.sim_time += dt;
        self.last_report = Some(gamecore::resolve_step(
            &self.grid,
            &mut self.stock,
            self.orbit_radius_km,
            dt,
        ));
    }

    /// Platziert das gewählte Gebäude bzw. reißt ein vorhandenes ab.
    fn toggle(&mut self, x: u32, y: u32) {
        let Some(tile) = self.grid.tile(x, y).copied() else {
            return;
        };

        // Besetzte Kachel → Abriss mit voller Kostenerstattung.
        if let Some(existing) = tile.building {
            self.grid.remove(x, y);
            refund(&mut self.stock, existing.kind.spec().build_cost);
            self.log
                .push(format!("Abgerissen: {} @ ({x},{y})", name(existing.kind)));
            return;
        }

        // Leere Kachel → bauen, falls Gelände passt und bezahlbar.
        let kind = self.selected;
        if !kind.can_build_on(tile.terrain) {
            self.log
                .push(format!("{}: falsches Gelände @ ({x},{y})", name(kind)));
            return;
        }
        let cost = kind.spec().build_cost;
        if !can_afford(&self.stock, cost) {
            self.log.push(format!("{}: zu teuer", name(kind)));
            return;
        }
        pay(&mut self.stock, cost);
        let _ = self.grid.place(x, y, Building::new(kind));
        self.log.push(format!("Gebaut: {} @ ({x},{y})", name(kind)));
    }
}

impl eframe::App for BuildApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut actions: Vec<Action> = Vec::new();
        let mut selected = self.selected;
        let mut auto = self.auto;

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Bau-Ebene");
                ui.separator();
                ui.label(format!("Sim-Zeit: {:.2} Tage", self.sim_time / 86_400.0));
                ui.separator();
                if ui.button("+1 Stunde").clicked() {
                    actions.push(Action::Step(3_600.0));
                }
                if ui.button("+1 Tag").clicked() {
                    actions.push(Action::Step(86_400.0));
                }
                ui.checkbox(&mut auto, "Auto (1 s ≈ 1 h)");
            });
        });

        egui::SidePanel::left("side")
            .resizable(false)
            .min_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Bauten");
                for kind in PALETTE {
                    let label = format!("{}  ({})", name(kind), cost_string(kind.spec().build_cost));
                    ui.selectable_value(&mut selected, kind, label);
                }

                ui.separator();
                ui.heading("Lager");
                egui::Grid::new("stock").striped(true).show(ui, |ui| {
                    for r in Resource::ALL {
                        ui.label(format!("{r:?}"));
                        ui.label(format!("{:.0}", self.stock.get(r)));
                        ui.end_row();
                    }
                });

                ui.separator();
                if let Some(rep) = self.last_report {
                    let status = if rep.energy_satisfied() { "gedeckt" } else { "KNAPP" };
                    ui.label(format!(
                        "Energie: {:.1} / {:.1} ({status})",
                        rep.energy_supply, rep.energy_demand
                    ));
                } else {
                    ui.label("Energie: — (noch kein Schritt)");
                }

                ui.separator();
                ui.heading("Log");
                for line in self.log.iter().rev().take(8) {
                    ui.small(line);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Linksklick: bauen · Klick auf Gebäude: abreißen");
            ui.add_space(6.0);
            let cell = egui::vec2(56.0, 56.0);
            for y in 0..self.grid.height {
                ui.horizontal(|ui| {
                    for x in 0..self.grid.width {
                        let tile = self.grid.tile(x, y).copied().unwrap();
                        let text = tile.building.map(|b| short(b.kind)).unwrap_or("");
                        let btn = egui::Button::new(egui::RichText::new(text).strong())
                            .fill(terrain_color(tile.terrain))
                            .min_size(cell);
                        let resp = ui
                            .add_sized(cell, btn)
                            .on_hover_text(tile_tooltip(&tile));
                        if resp.clicked() {
                            actions.push(Action::Toggle(x, y));
                        }
                    }
                });
            }
        });

        // Eingaben anwenden (nach dem Rendern).
        self.selected = selected;
        self.auto = auto;
        for a in actions {
            match a {
                Action::Step(dt) => self.step(dt),
                Action::Toggle(x, y) => self.toggle(x, y),
            }
        }

        // Auto-Tick: reale Zeit × Faktor.
        if self.auto {
            let dt = ctx.input(|i| i.stable_dt) as f64 * 3_600.0;
            self.step(dt);
            ctx.request_repaint();
        }
    }
}

// --- Reine Darstellungs-Helfer (UI-Anliegen, gehören nicht in `core`) --------

fn name(kind: BuildingKind) -> &'static str {
    use BuildingKind::*;
    match kind {
        MetalMine => "Mine",
        CrystalExtractor => "Kristall-Förderer",
        GasCollector => "Gas-Kollektor",
        Smelter => "Hütte",
        ElectronicsFab => "Elektronik-Fab",
        CompositeFab => "Komposit-Fab",
        SolarCollector => "Solar",
        FusionReactor => "Fusion",
        Depot => "Lager",
    }
}

fn short(kind: BuildingKind) -> &'static str {
    use BuildingKind::*;
    match kind {
        MetalMine => "Mi",
        CrystalExtractor => "Kr",
        GasCollector => "Ga",
        Smelter => "Hü",
        ElectronicsFab => "El",
        CompositeFab => "Ko",
        SolarCollector => "So",
        FusionReactor => "Fu",
        Depot => "La",
    }
}

fn terrain_color(terrain: Terrain) -> egui::Color32 {
    match terrain {
        Terrain::Rock => egui::Color32::from_rgb(96, 96, 102),
        Terrain::Crystal => egui::Color32::from_rgb(72, 132, 196),
        Terrain::GasField => egui::Color32::from_rgb(188, 128, 56),
        Terrain::Ice => egui::Color32::from_rgb(150, 190, 220),
        Terrain::Barren => egui::Color32::from_rgb(48, 48, 54),
    }
}

fn terrain_name(terrain: Terrain) -> &'static str {
    match terrain {
        Terrain::Rock => "Gestein",
        Terrain::Crystal => "Kristall",
        Terrain::GasField => "Gasvorkommen",
        Terrain::Ice => "Eis",
        Terrain::Barren => "karg",
    }
}

fn tile_tooltip(tile: &gamecore::Tile) -> String {
    match tile.building {
        Some(b) => format!("{} auf {}", name(b.kind), terrain_name(tile.terrain)),
        None => terrain_name(tile.terrain).to_string(),
    }
}

fn cost_string(cost: &[(Resource, f64)]) -> String {
    cost.iter()
        .map(|(r, q)| format!("{:.0} {:?}", q, r))
        .collect::<Vec<_>>()
        .join(", ")
}

fn can_afford(stock: &Stockpile, cost: &[(Resource, f64)]) -> bool {
    cost.iter().all(|(r, q)| stock.get(*r) >= *q)
}

fn pay(stock: &mut Stockpile, cost: &[(Resource, f64)]) {
    for (r, q) in cost {
        stock.add(*r, -*q);
    }
}

fn refund(stock: &mut Stockpile, cost: &[(Resource, f64)]) {
    for (r, q) in cost {
        stock.add(*r, *q);
    }
}
