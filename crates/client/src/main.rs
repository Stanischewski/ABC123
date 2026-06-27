//! Client — Bau-Ebene & schematische System-Ansicht (Phase 1), egui/eframe.
//!
//! Zwei Ansichten, getragen vom geteilten `core` (als `gamecore`):
//!
//! - **Bau-Ebene** (DESIGN.md §3.1) — ein Planeten-Raster, auf das Gebäude
//!   gesetzt werden, mit Lager-, Energie- und Adjazenz-Rückmeldung.
//! - **System** (DESIGN.md §3.2, Begleitdokument §13) — die *schematische*
//!   Systemansicht: Körper auf ihren echten Kepler-Bahnen als statische Marker
//!   (kein Bevy, keine Schiffe). Heimatbasis und Dashboard.
//!
//! Die gesamte Spiellogik liegt im Kern; diese Crate stellt nur dar und nimmt
//! Eingaben entgegen. Nativ lauffähig; Wasm-Target (trunk) später.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use gamecore::{Building, BuildingKind, BodyKind, Grid, Resource, StepReport, Stockpile, System, Terrain};

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
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ABC123 — Phase 1",
        options,
        Box::new(|_cc| Ok(Box::new(BuildApp::new()))),
    )
}

/// Aktive Ansicht.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Build,
    System,
}

/// Was nach dem Rendern (außerhalb der egui-Closures) angewendet wird —
/// vermeidet Borrow-Konflikte.
enum Action {
    Step(f64),
    Toggle(u32, u32),
}

struct BuildApp {
    view: View,
    grid: Grid,
    system: System,
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
        let system = gamecore::demo_home_system();
        let orbit_radius_km = system.position_of(1, 0.0).map(|p| p.length()).unwrap_or(1.0);

        let mut stock = Stockpile::new();
        stock.set(Resource::Metals, 1500.0);
        stock.set(Resource::Alloys, 200.0);
        stock.set(Resource::Electronics, 100.0);
        stock.set(Resource::Gases, 200.0);

        BuildApp {
            view: View::Build,
            grid: gamecore::demo_home_planet(),
            system,
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

        if let Some(existing) = tile.building {
            self.grid.remove(x, y);
            refund(&mut self.stock, existing.kind.spec().build_cost);
            self.log
                .push(format!("Abgerissen: {} @ ({x},{y})", name(existing.kind)));
            return;
        }

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
        let mut view = self.view;

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut view, View::Build, "Bau-Ebene");
                ui.selectable_value(&mut view, View::System, "System");
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

        match view {
            View::Build => {
                self.show_build_side(ctx, &mut selected);
                self.show_build_grid(ctx, &mut actions);
            }
            View::System => {
                self.show_system_side(ctx);
                self.show_system_view(ctx);
            }
        }

        // Eingaben anwenden.
        self.selected = selected;
        self.auto = auto;
        self.view = view;
        for a in actions {
            match a {
                Action::Step(dt) => self.step(dt),
                Action::Toggle(x, y) => self.toggle(x, y),
            }
        }

        if self.auto {
            let dt = ctx.input(|i| i.stable_dt) as f64 * 3_600.0;
            self.step(dt);
            ctx.request_repaint();
        }
    }
}

impl BuildApp {
    fn show_build_side(&self, ctx: &egui::Context, selected: &mut BuildingKind) {
        egui::SidePanel::left("side")
            .resizable(false)
            .min_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Bauten");
                for kind in PALETTE {
                    let label = format!("{}  ({})", name(kind), cost_string(kind.spec().build_cost));
                    ui.selectable_value(selected, kind, label);
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
    }

    fn show_build_grid(&self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Linksklick: bauen · Klick auf Gebäude: abreißen · Adjazenz im Tooltip");
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
                            .on_hover_text(self.tile_tooltip(x, y, &tile));
                        if resp.clicked() {
                            actions.push(Action::Toggle(x, y));
                        }
                    }
                });
            }
        });
    }

    /// Tooltip einer Kachel inkl. aktuellem Adjazenz-Multiplikator.
    fn tile_tooltip(&self, x: u32, y: u32, tile: &gamecore::Tile) -> String {
        match tile.building {
            Some(b) => {
                let mult = self.grid.adjacency_multiplier(x, y);
                if b.kind.spec().output.is_some() {
                    format!(
                        "{} auf {}\nAdjazenz ×{:.2}",
                        name(b.kind),
                        terrain_name(tile.terrain),
                        mult
                    )
                } else {
                    format!("{} auf {}", name(b.kind), terrain_name(tile.terrain))
                }
            }
            None => terrain_name(tile.terrain).to_string(),
        }
    }

    fn show_system_side(&self, ctx: &egui::Context) {
        egui::SidePanel::left("sys_side")
            .resizable(false)
            .min_width(240.0)
            .show(ctx, |ui| {
                ui.heading(&self.system.name);
                ui.add_space(4.0);
                let t = self.sim_time;
                for body in &self.system.bodies {
                    let dist = self
                        .system
                        .position_of(body.id, t)
                        .map(|p| p.length())
                        .unwrap_or(0.0);
                    ui.label(format!("{}  ({})", body.name, kind_name(body.kind)));
                    if let Some(orbit) = &body.orbit {
                        ui.small(format!(
                            "   Abstand {:.2} Mio km · T {:.1} Tage",
                            dist / 1.0e6,
                            orbit.period() / 86_400.0
                        ));
                    } else {
                        ui.small("   Zentralkörper");
                    }
                }
                ui.add_space(8.0);
                ui.separator();
                ui.small("Schematisch — Bahnen maßstäblich, Monde zur Sichtbarkeit überhöht.");
            });
    }

    fn show_system_view(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (resp, painter) =
                ui.allocate_painter(ui.available_size(), egui::Sense::hover());
            let rect = resp.rect;
            let center = rect.center();
            let t = self.sim_time;

            // Maßstab: größte heliozentrische Halbachse füllt ~42 % der Fläche.
            let max_a = self
                .system
                .bodies
                .iter()
                .filter(|b| b.parent == Some(0))
                .filter_map(|b| b.orbit.map(|o| o.semi_major_axis))
                .fold(1.0_f64, f64::max);
            let radius_px = 0.42 * rect.size().min_elem() as f64;
            let scale = radius_px / max_a;

            let world_to_screen = |p: gamecore::Vec2| -> egui::Pos2 {
                egui::pos2(
                    center.x + (p.x * scale) as f32,
                    center.y - (p.y * scale) as f32,
                )
            };

            let faint = egui::Color32::from_gray(70);
            const MOON_VIS_R: f32 = 18.0;

            for body in &self.system.bodies {
                match body.parent {
                    // Zentralstern.
                    None => {
                        painter.circle_filled(center, 9.0, body_color(body.kind));
                        painter.text(
                            center + egui::vec2(12.0, -4.0),
                            egui::Align2::LEFT_CENTER,
                            &body.name,
                            egui::FontId::proportional(13.0),
                            egui::Color32::LIGHT_GRAY,
                        );
                    }
                    // Heliozentrisch: Bahnring um den Stern, Marker an wahrer Position.
                    Some(0) => {
                        if let Some(orbit) = &body.orbit {
                            let ring_r = (orbit.semi_major_axis * scale) as f32;
                            painter.circle_stroke(center, ring_r, egui::Stroke::new(1.0, faint));
                        }
                        if let Some(pos) = self.system.position_of(body.id, t) {
                            let p = world_to_screen(pos);
                            painter.circle_filled(p, 6.0, body_color(body.kind));
                            painter.text(
                                p + egui::vec2(9.0, -2.0),
                                egui::Align2::LEFT_CENTER,
                                &body.name,
                                egui::FontId::proportional(12.0),
                                egui::Color32::LIGHT_GRAY,
                            );
                        }
                    }
                    // Mond o. Ä.: um den Elternkörper, zur Sichtbarkeit überhöht.
                    Some(pid) => {
                        let (Some(parent_pos), Some(orbit)) =
                            (self.system.position_of(pid, t), body.orbit)
                        else {
                            continue;
                        };
                        let pscreen = world_to_screen(parent_pos);
                        painter.circle_stroke(
                            pscreen,
                            MOON_VIS_R,
                            egui::Stroke::new(1.0, faint),
                        );
                        // Richtung aus der echten Relativposition, fester Pixelradius.
                        let rel = orbit.relative_position_at(t);
                        let len = rel.length();
                        let (dx, dy) = if len > 0.0 {
                            ((rel.x / len) as f32, (rel.y / len) as f32)
                        } else {
                            (1.0, 0.0)
                        };
                        let mpos = pscreen + egui::vec2(dx * MOON_VIS_R, -dy * MOON_VIS_R);
                        painter.circle_filled(mpos, 4.0, body_color(body.kind));
                        painter.text(
                            mpos + egui::vec2(7.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            &body.name,
                            egui::FontId::proportional(11.0),
                            egui::Color32::GRAY,
                        );
                    }
                }
            }
        });
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

fn kind_name(kind: BodyKind) -> &'static str {
    match kind {
        BodyKind::Star => "Stern",
        BodyKind::Rocky => "Gesteinsplanet",
        BodyKind::Gas => "Gasplanet",
        BodyKind::Icy => "Eiswelt",
        BodyKind::Moon => "Mond",
        BodyKind::Asteroid => "Asteroid",
    }
}

fn body_color(kind: BodyKind) -> egui::Color32 {
    match kind {
        BodyKind::Star => egui::Color32::from_rgb(240, 200, 80),
        BodyKind::Rocky => egui::Color32::from_rgb(180, 150, 110),
        BodyKind::Gas => egui::Color32::from_rgb(200, 150, 90),
        BodyKind::Icy => egui::Color32::from_rgb(160, 200, 220),
        BodyKind::Moon => egui::Color32::from_rgb(160, 160, 170),
        BodyKind::Asteroid => egui::Color32::from_rgb(120, 120, 120),
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
