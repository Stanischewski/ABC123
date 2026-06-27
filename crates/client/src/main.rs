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

// Im Release nativ ohne Konsolenfenster (auf Wasm bedeutungslos → ausgeschlossen).
#![cfg_attr(
    all(not(debug_assertions), not(target_arch = "wasm32")),
    windows_subsystem = "windows"
)]

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

/// Nativer Einstiegspunkt (Desktop-Fenster).
#[cfg(not(target_arch = "wasm32"))]
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

/// Browser-Einstiegspunkt (Wasm): hängt die App an die `<canvas>` in index.html.
#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|_cc| Ok(Box::new(BuildApp::new()))),
            )
            .await
            .expect("eframe konnte im Browser nicht starten");
    });
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
    Build(u32, u32, BuildingKind),
    Demolish(u32, u32),
    SetEnabled(u32, u32, bool),
    SetPriority(u32, u32, i32),
}

/// Auswählbare Prioritätsstufen (höher = wird bei Energie-Knappheit zuerst
/// bedient). Werte passen zur Drossel-Logik in `core`.
const PRIORITY_LEVELS: [(&str, i32); 3] = [("Hoch", 10), ("Normal", 0), ("Niedrig", -10)];

fn priority_name(priority: i32) -> String {
    PRIORITY_LEVELS
        .iter()
        .find(|(_, v)| *v == priority)
        .map(|(label, _)| label.to_string())
        .unwrap_or_else(|| priority.to_string())
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
            log: vec!["Willkommen. Linksklick baut · Rechtsklick öffnet das Menü.".into()],
        }
    }

    /// Schreibt die Simulation um `dt` Sekunden fort.
    fn step(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        self.sim_time += dt;
        let _ = gamecore::resolve_step(&self.grid, &mut self.stock, self.orbit_radius_km, dt);
    }

    /// Live-Vorschau einer Sim-Stunde auf einer Lager-Kopie — speist die
    /// Netto-Raten (+x/h) und den Energie-Balken, ohne den Zustand zu ändern.
    fn preview(&self) -> (Stockpile, StepReport) {
        let mut s = self.stock.clone();
        let rep = gamecore::resolve_step(&self.grid, &mut s, self.orbit_radius_km, 3_600.0);
        (s, rep)
    }

    /// Platziert ein bestimmtes Gebäude (Gelände/Belegung/Budget vorausgesetzt).
    fn build(&mut self, x: u32, y: u32, kind: BuildingKind) {
        let Some(tile) = self.grid.tile(x, y).copied() else {
            return;
        };
        if tile.building.is_some() {
            return;
        }
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

    /// Reißt das Gebäude an `(x, y)` ab (volle Kostenerstattung).
    fn demolish(&mut self, x: u32, y: u32) {
        if let Some(b) = self.grid.remove(x, y) {
            refund(&mut self.stock, b.kind.spec().build_cost);
            self.log
                .push(format!("Abgerissen: {} @ ({x},{y})", name(b.kind)));
        }
    }

    /// Schaltet ein Gebäude ein/aus.
    fn set_enabled(&mut self, x: u32, y: u32, on: bool) {
        if self.grid.set_enabled(x, y, on) {
            let what = if on { "Eingeschaltet" } else { "Ausgeschaltet" };
            self.log.push(format!("{what} @ ({x},{y})"));
        }
    }

    /// Setzt die Drossel-Priorität eines Gebäudes.
    fn set_priority(&mut self, x: u32, y: u32, p: i32) {
        if self.grid.set_priority(x, y, p) {
            self.log
                .push(format!("Priorität {} @ ({x},{y})", priority_name(p)));
        }
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
                self.show_build_grid(ctx, selected, &mut actions);
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
                Action::Build(x, y, kind) => self.build(x, y, kind),
                Action::Demolish(x, y) => self.demolish(x, y),
                Action::SetEnabled(x, y, on) => self.set_enabled(x, y, on),
                Action::SetPriority(x, y, p) => self.set_priority(x, y, p),
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

                // Live-Vorschau (1 Sim-Stunde): Netto-Raten und Energie-Bilanz.
                let (preview, rep) = self.preview();

                ui.separator();
                ui.heading("Lager");
                egui::Grid::new("stock").striped(true).num_columns(3).show(ui, |ui| {
                    for r in Resource::ALL {
                        ui.label(format!("{r:?}"));
                        ui.label(format!("{:.0}", self.stock.get(r)));
                        // Netto-Änderung über die nächste Sim-Stunde.
                        let rate = preview.get(r) - self.stock.get(r);
                        let (txt, col) = rate_label(rate);
                        ui.colored_label(col, txt);
                        ui.end_row();
                    }
                });

                ui.separator();
                ui.heading("Energie");
                let supply = rep.energy_supply;
                let demand = rep.energy_demand;
                let frac = if supply > 0.0 {
                    (demand / supply).min(1.0)
                } else if demand > 0.0 {
                    1.0
                } else {
                    0.0
                };
                let ok = rep.energy_satisfied();
                let fill = if ok {
                    egui::Color32::from_rgb(80, 170, 90)
                } else {
                    egui::Color32::from_rgb(200, 70, 70)
                };
                ui.add(
                    egui::ProgressBar::new(frac as f32)
                        .fill(fill)
                        .text(format!("Verbrauch {demand:.1} / {supply:.1} /s")),
                );
                if !ok {
                    ui.colored_label(fill, "KNAPP — Produktion gedrosselt");
                }

                ui.separator();
                ui.heading("Log");
                for line in self.log.iter().rev().take(8) {
                    ui.small(line);
                }
            });
    }

    fn show_build_grid(&self, ctx: &egui::Context, selected: BuildingKind, actions: &mut Vec<Action>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!(
                "Linksklick: {} bauen / abreißen · Rechtsklick: Menü",
                name(selected)
            ));
            ui.add_space(6.0);
            let cell = egui::vec2(56.0, 56.0);
            for y in 0..self.grid.height {
                ui.horizontal(|ui| {
                    for x in 0..self.grid.width {
                        let tile = self.grid.tile(x, y).copied().unwrap();
                        // Gebäude-Kürzel; ausgeschaltet → ausgegraut.
                        let (text, text_col) = match tile.building {
                            Some(b) if b.enabled => (short(b.kind), egui::Color32::WHITE),
                            Some(b) => (short(b.kind), egui::Color32::from_gray(120)),
                            None => ("", egui::Color32::WHITE),
                        };
                        let btn = egui::Button::new(egui::RichText::new(text).strong().color(text_col))
                            .fill(terrain_color(tile.terrain))
                            .min_size(cell);
                        let resp = ui
                            .add_sized(cell, btn)
                            .on_hover_text(self.tile_tooltip(x, y, &tile));

                        // Linksklick: schnelle Aktion (bauen / abreißen).
                        if resp.clicked() {
                            match tile.building {
                                Some(_) => actions.push(Action::Demolish(x, y)),
                                None => actions.push(Action::Build(x, y, selected)),
                            }
                        }

                        // Rechtsklick: Kontextmenü.
                        resp.context_menu(|ui| match tile.building {
                            None => {
                                ui.label(terrain_name(tile.terrain));
                                ui.menu_button("Bauen", |ui| {
                                    let mut any = false;
                                    for kind in PALETTE {
                                        if kind.can_build_on(tile.terrain) {
                                            any = true;
                                            let label = format!(
                                                "{}  ({})",
                                                name(kind),
                                                cost_string(kind.spec().build_cost)
                                            );
                                            if ui.button(label).clicked() {
                                                actions.push(Action::Build(x, y, kind));
                                                ui.close_menu();
                                            }
                                        }
                                    }
                                    if !any {
                                        ui.label("(kein Gebäude für dieses Gelände)");
                                    }
                                });
                            }
                            Some(b) => {
                                ui.label(name(b.kind));
                                let toggle = if b.enabled { "Ausschalten" } else { "Einschalten" };
                                if ui.button(toggle).clicked() {
                                    actions.push(Action::SetEnabled(x, y, !b.enabled));
                                    ui.close_menu();
                                }
                                // Priorität nur für Energieverbraucher sinnvoll.
                                if b.kind.spec().energy_demand > 0.0 {
                                    ui.menu_button(
                                        format!("Priorität: {}", priority_name(b.priority)),
                                        |ui| {
                                            for (label, val) in PRIORITY_LEVELS {
                                                if ui
                                                    .selectable_label(b.priority == val, label)
                                                    .clicked()
                                                {
                                                    actions.push(Action::SetPriority(x, y, val));
                                                    ui.close_menu();
                                                }
                                            }
                                        },
                                    );
                                }
                                if ui.button("Abreißen").clicked() {
                                    actions.push(Action::Demolish(x, y));
                                    ui.close_menu();
                                }
                            }
                        });
                    }
                });
            }
        });
    }

    /// Tooltip einer Kachel inkl. Zustand und aktuellem Adjazenz-Multiplikator.
    fn tile_tooltip(&self, x: u32, y: u32, tile: &gamecore::Tile) -> String {
        match tile.building {
            Some(b) => {
                let state = if b.enabled { "" } else { " — aus" };
                let spec = b.kind.spec();
                if spec.output.is_some() {
                    let mult = self.grid.adjacency_multiplier(x, y);
                    let mut s = format!(
                        "{}{state} auf {}\nAdjazenz ×{:.2}",
                        name(b.kind),
                        terrain_name(tile.terrain),
                        mult
                    );
                    if spec.energy_demand > 0.0 {
                        s.push_str(&format!("\nPriorität: {}", priority_name(b.priority)));
                    }
                    s
                } else {
                    format!("{}{state} auf {}", name(b.kind), terrain_name(tile.terrain))
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

/// Beschriftung + Farbe für eine Netto-Rate pro Stunde (`+x/h` grün, `-x/h` rot).
fn rate_label(rate_per_hour: f64) -> (String, egui::Color32) {
    if rate_per_hour.abs() < 0.5 {
        ("0/h".to_string(), egui::Color32::from_gray(120))
    } else if rate_per_hour > 0.0 {
        (
            format!("+{:.0}/h", rate_per_hour),
            egui::Color32::from_rgb(90, 180, 100),
        )
    } else {
        (
            format!("{:.0}/h", rate_per_hour),
            egui::Color32::from_rgb(200, 90, 80),
        )
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
