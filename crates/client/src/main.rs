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
use gamecore::{
    Building, BuildingKind, BodyKind, Grid, PlaceError, ResearchId, ResearchState, Resource,
    StepReport, Stockpile, System, Terrain, Unlock,
};

/// Gebäude in der Palette, in Bau-Reihenfolge.
const PALETTE: [BuildingKind; 11] = [
    BuildingKind::Headquarters,
    BuildingKind::MetalMine,
    BuildingKind::CrystalExtractor,
    BuildingKind::GasCollector,
    BuildingKind::Smelter,
    BuildingKind::ElectronicsFab,
    BuildingKind::CompositeFab,
    BuildingKind::ResearchLab,
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
    Research,
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
    StartResearch(ResearchId),
    CancelResearch,
    SetResearchPriority(i32),
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
    research: ResearchState,
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

        // Startbestand innerhalb der Anfangskapazität (Zentrale 500 + Basis 100).
        let mut stock = Stockpile::new();
        stock.set(Resource::Metals, 500.0);
        stock.set(Resource::Alloys, 100.0);
        stock.set(Resource::Electronics, 80.0);
        stock.set(Resource::Gases, 150.0);

        BuildApp {
            view: View::Build,
            grid: gamecore::demo_home_planet(),
            system,
            stock,
            research: ResearchState::new(),
            sim_time: 0.0,
            orbit_radius_km,
            selected: BuildingKind::MetalMine,
            auto: false,
            log: vec!["Willkommen. Linksklick baut · Rechtsklick öffnet das Menü.".into()],
        }
    }

    /// Schreibt die Simulation um `dt` Sekunden fort (Produktion + Forschung +
    /// Baufortschritt). Loggt frisch abgeschlossene Forschungen.
    fn step(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        self.sim_time += dt;
        let before = self.research.completed().len();
        let _ = gamecore::advance(
            &mut self.grid,
            &mut self.stock,
            &mut self.research,
            self.orbit_radius_km,
            dt,
        );
        if self.research.completed().len() > before {
            if let Some(id) = self.research.completed().last() {
                self.log.push(format!("Forschung abgeschlossen: {}", id.name()));
            }
        }
    }

    /// Struktureller Sim-Bericht für ein gegebenes Raster — eine 1-Stunden-
    /// Vorschau auf einer Lager-Kopie, ohne den echten Zustand zu ändern. Nutzt
    /// [`gamecore::resolve_step`] (forschungsfrei), da nur der Netto-Fluss je
    /// Stoff (Überschuss/Defizit) und das Energiebudget interessieren.
    fn grid_report(&self, grid: &Grid) -> StepReport {
        let mut s = self.stock.clone();
        gamecore::resolve_step(grid, &mut s, self.orbit_radius_km, 3_600.0)
    }

    /// Projizierte Auswirkung, wenn `kind` an `(x, y)` gebaut würde — die
    /// *eingeschwungene* Änderung von Saldo (Überschuss/Defizit) und Energie
    /// (das Gebäude sofort betriebsbereit gedacht), gegen den aktuellen Zustand.
    fn placement_tooltip(&self, x: u32, y: u32, kind: BuildingKind) -> String {
        let base = self.grid_report(&self.grid);
        let mut g = self.grid.clone();
        let _ = g.place(x, y, Building::new(kind));
        let hyp = self.grid_report(&g);

        let mut s = format!(
            "Bauen: {} ({})\n— Auswirkung auf den Saldo —",
            name(kind),
            cost_string(kind.spec().build_cost)
        );
        let mut any = false;
        for (i, r) in Resource::ALL.iter().enumerate() {
            // Änderung des strukturellen Saldos (Einheiten/Stunde).
            let d = (hyp.net_flow[i] - base.net_flow[i]) * 3_600.0;
            if d.abs() > 0.5 {
                s.push_str(&format!("\n{r:?} {d:+.0}/h"));
                any = true;
            }
        }
        let ds = hyp.energy_supply - base.energy_supply;
        let dd = hyp.energy_demand - base.energy_demand;
        if ds.abs() > 0.05 {
            s.push_str(&format!("\nEnergie-Angebot {ds:+.1}/s"));
            any = true;
        }
        if dd.abs() > 0.05 {
            s.push_str(&format!("\nEnergie-Bedarf {dd:+.1}/s"));
            any = true;
        }
        if kind.storage() > 0.0 {
            s.push_str(&format!("\nLagerkapazität +{:.0}", kind.storage()));
            any = true;
        }
        if !any {
            s.push_str("\n(noch keine Wirkung — evtl. fehlt Energie oder Input)");
        }
        s
    }

    /// Setzt eine Baustelle (kein Einmalkauf — Material fließt über die Bauzeit).
    fn build(&mut self, x: u32, y: u32, kind: BuildingKind) {
        if !self.research.is_building_unlocked(kind) {
            if let Some(id) = kind.required_research() {
                self.log
                    .push(format!("{}: erst {} erforschen", name(kind), id.name()));
            }
            return;
        }
        match self.grid.place(x, y, Building::construction_site(kind)) {
            Ok(()) => self.log.push(format!("Baustelle: {} @ ({x},{y})", name(kind))),
            Err(PlaceError::WrongTerrain) => self
                .log
                .push(format!("{}: falsches Gelände @ ({x},{y})", name(kind))),
            Err(PlaceError::Occupied) => {
                self.log.push(format!("{}: Feld belegt", name(kind)))
            }
            Err(PlaceError::AlreadyPresent) => self
                .log
                .push(format!("{}: existiert bereits (nur eines erlaubt)", name(kind))),
            Err(PlaceError::OutOfBounds) => {}
        }
    }

    /// Reißt ein Gebäude ab bzw. bricht eine Baustelle ab. Erstattet das bereits
    /// verbaute Material (Kosten × Fortschritt) zurück.
    fn demolish(&mut self, x: u32, y: u32) {
        if let Some(b) = self.grid.remove(x, y) {
            for (r, q) in b.kind.spec().build_cost {
                self.stock.add(*r, q * b.progress);
            }
            let verb = if b.under_construction() {
                "Bau abgebrochen"
            } else {
                "Abgerissen"
            };
            self.log.push(format!("{verb}: {} @ ({x},{y})", name(b.kind)));
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

    /// Startet ein Forschungsprojekt (sofern keines läuft und freigeschaltet).
    fn start_research(&mut self, id: ResearchId) {
        if self.research.start(id) {
            self.log.push(format!("Forschung gestartet: {}", id.name()));
        }
    }

    /// Bricht das laufende Forschungsprojekt ab (Fortschritt verfällt).
    fn cancel_research(&mut self) {
        if let Some(a) = self.research.cancel() {
            self.log
                .push(format!("Forschung abgebrochen: {}", a.id.name()));
        }
    }

    /// Setzt die Energie-Priorität der Forschung.
    fn set_research_priority(&mut self, p: i32) {
        self.research.set_priority(p);
        self.log
            .push(format!("Forschungs-Priorität: {}", priority_name(p)));
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
                ui.selectable_value(&mut view, View::Research, "Forschung");
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
            View::Research => {
                self.show_research_side(ctx, &mut actions);
                self.show_research_view(ctx, &mut actions);
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
                Action::StartResearch(id) => self.start_research(id),
                Action::CancelResearch => self.cancel_research(),
                Action::SetResearchPriority(p) => self.set_research_priority(p),
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
                    if self.research.is_building_unlocked(kind) {
                        ui.selectable_value(selected, kind, label);
                    } else {
                        let req = kind.required_research().map(|r| r.name()).unwrap_or("");
                        ui.add_enabled(
                            false,
                            egui::SelectableLabel::new(false, format!("🔒 {label}")),
                        )
                        .on_disabled_hover_text(format!("Erst {req} erforschen"));
                    }
                }

                // Live-Vorschau (1 Sim-Stunde): struktureller Saldo + Energie.
                let rep = self.grid_report(&self.grid);

                ui.separator();
                let cap = self.grid.storage_capacity();
                ui.heading(format!("Lager (Kap. {cap:.0})"));
                ui.small("Saldo = Überschuss/Defizit, unabhängig vom Lagerstand.");
                egui::Grid::new("stock").striped(true).num_columns(3).show(ui, |ui| {
                    for (i, r) in Resource::ALL.iter().enumerate() {
                        ui.label(format!("{r:?}"));
                        let amt = self.stock.get(*r);
                        let full = amt >= cap - 0.5;
                        let col = if full {
                            egui::Color32::from_rgb(200, 90, 80)
                        } else {
                            ui.visuals().text_color()
                        };
                        ui.colored_label(col, format!("{amt:.0} / {cap:.0}"));
                        // Struktureller Saldo (Einheiten/Stunde), unabhängig vom Lager.
                        let (txt, col) = rate_label(rep.net_flow[i] * 3_600.0);
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
                        // Baustelle → Fortschritt in %; sonst Kürzel
                        // (ausgeschaltet → ausgegraut).
                        let (text, text_col) = match tile.building {
                            Some(b) if b.under_construction() => (
                                format!("{:.0}%", b.progress * 100.0),
                                egui::Color32::from_rgb(230, 180, 80),
                            ),
                            Some(b) if b.enabled => (short(b.kind).to_string(), egui::Color32::WHITE),
                            Some(b) => (short(b.kind).to_string(), egui::Color32::from_gray(120)),
                            None => (String::new(), egui::Color32::WHITE),
                        };
                        let btn = egui::Button::new(egui::RichText::new(text).strong().color(text_col))
                            .fill(terrain_color(tile.terrain))
                            .min_size(cell);
                        let resp = ui.add_sized(cell, btn);
                        // Tooltip nur fürs gehoverte Feld berechnen (sonst teuer):
                        // belegt → Gebäude-Info; leer & baubar → Platzierungs-Vorschau.
                        let resp = if resp.hovered() {
                            let tip = match tile.building {
                                Some(_) => self.tile_tooltip(x, y, &tile),
                                None if selected.can_build_on(tile.terrain)
                                    && self.research.is_building_unlocked(selected) =>
                                {
                                    self.placement_tooltip(x, y, selected)
                                }
                                None => terrain_name(tile.terrain).to_string(),
                            };
                            resp.on_hover_text(tip)
                        } else {
                            resp
                        };

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
                                        if !kind.can_build_on(tile.terrain) {
                                            continue;
                                        }
                                        any = true;
                                        let label = format!(
                                            "{}  ({})",
                                            name(kind),
                                            cost_string(kind.spec().build_cost)
                                        );
                                        if self.research.is_building_unlocked(kind) {
                                            if ui.button(label).clicked() {
                                                actions.push(Action::Build(x, y, kind));
                                                ui.close_menu();
                                            }
                                        } else {
                                            let req =
                                                kind.required_research().map(|r| r.name()).unwrap_or("");
                                            ui.add_enabled(false, egui::Button::new(format!("🔒 {label}")))
                                                .on_disabled_hover_text(format!(
                                                    "Erst {req} erforschen"
                                                ));
                                        }
                                    }
                                    if !any {
                                        ui.label("(kein Gebäude für dieses Gelände)");
                                    }
                                });
                            }
                            // Baustelle: nur Abbrechen.
                            Some(b) if b.under_construction() => {
                                ui.label(format!(
                                    "{} — im Bau {:.0}%",
                                    name(b.kind),
                                    b.progress * 100.0
                                ));
                                if ui.button("Bau abbrechen").clicked() {
                                    actions.push(Action::Demolish(x, y));
                                    ui.close_menu();
                                }
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
                                // Info: Boni und Abzüge aufgeschlüsselt.
                                ui.menu_button("Info", |ui| {
                                    let spec = b.kind.spec();
                                    if let Some(out) = spec.output {
                                        ui.label(format!("Basis: {:.2}/s → {out:?}", spec.base_rate));
                                        let mult = self.grid.adjacency_multiplier(x, y);
                                        ui.label(format!("Adjazenz: ×{mult:.2}"));
                                        let contribs = self.grid.adjacency_contributions(x, y);
                                        if contribs.is_empty() {
                                            ui.label("   (keine Nachbar-Boni)");
                                        } else {
                                            for (k, bonus) in contribs {
                                                ui.label(format!("   +{:.0}%  {}", bonus * 100.0, name(k)));
                                            }
                                        }
                                        ui.label(format!("Effektiv: {:.2}/s", spec.base_rate * mult));
                                    }
                                    if spec.energy_demand > 0.0 {
                                        ui.label(format!("Energiebedarf: −{:.1}/s", spec.energy_demand));
                                    }
                                    if spec.energy_output > 0.0 {
                                        let note = if spec.solar {
                                            "  (×1/r²)"
                                        } else if spec.fuel_rate > 0.0 {
                                            "  (frisst Gas)"
                                        } else {
                                            ""
                                        };
                                        ui.label(format!("Energie: +{:.1}/s{note}", spec.energy_output));
                                    }
                                    if b.kind.storage() > 0.0 {
                                        ui.label(format!("Lagerkapazität: +{:.0}", b.kind.storage()));
                                    }
                                });
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
            Some(b) if b.under_construction() => {
                let cost = cost_string(b.kind.spec().build_cost);
                format!(
                    "{} — im Bau {:.0}%\nauf {}\nBaukosten: {}",
                    name(b.kind),
                    b.progress * 100.0,
                    terrain_name(tile.terrain),
                    cost
                )
            }
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

    /// Linkes Panel der Forschungs-Ansicht: aktives Projekt, Priorität, Hinweise.
    fn show_research_side(&self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        egui::SidePanel::left("research_side")
            .resizable(false)
            .min_width(240.0)
            .show(ctx, |ui| {
                ui.heading("Forschung");
                ui.small("Projekte sind Baustellen: Material fließt über die Zeit und kriecht bei Mangel.");
                ui.separator();

                match self.research.active() {
                    Some(a) => {
                        ui.label(format!("Aktiv: {}", a.id.name()));
                        ui.add(
                            egui::ProgressBar::new(a.progress as f32)
                                .text(format!("{:.0}%", a.progress * 100.0)),
                        );
                        ui.add_space(4.0);
                        ui.label("Energie-Priorität:");
                        ui.horizontal(|ui| {
                            for (label, val) in PRIORITY_LEVELS {
                                if ui
                                    .selectable_label(self.research.priority() == val, label)
                                    .clicked()
                                {
                                    actions.push(Action::SetResearchPriority(val));
                                }
                            }
                        });
                        ui.add_space(4.0);
                        if ui.button("Abbrechen").clicked() {
                            actions.push(Action::CancelResearch);
                        }
                    }
                    None => {
                        ui.label("Kein Projekt aktiv.");
                        ui.small("Wähle rechts einen Knoten und starte ihn.");
                    }
                }

                ui.separator();
                ui.heading("Forschungseinrichtung");
                ui.small(
                    "Senkt im Betrieb die Projektzeit (Beschleuniger) und frisst dabei \
                     Elektronik + Energie. Optional — Projekte laufen auch ohne sie.",
                );

                ui.separator();
                ui.heading("Log");
                for line in self.log.iter().rev().take(8) {
                    ui.small(line);
                }
            });
    }

    /// Zentrale Forschungs-Ansicht: der Freischaltungs-Baum als Knotenliste mit
    /// Status, Kosten und Start-Knopf.
    fn show_research_view(&self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Freischaltungs-Baum (Phase 1)");
            ui.small("Forschung schaltet frei — kein Prozentbonus. Genau ein Projekt zur Zeit.");
            ui.add_space(6.0);

            let busy = self.research.active().is_some();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for id in gamecore::research::ALL {
                    let node = id.node();
                    let done = self.research.is_done(id);
                    let active = self.research.active().map(|a| a.id) == Some(id);
                    let can_start = self.research.can_start(id);

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let (badge, col) = if done {
                                ("✓", egui::Color32::from_rgb(90, 180, 100))
                            } else if active {
                                ("▶", egui::Color32::from_rgb(230, 180, 80))
                            } else if can_start {
                                ("○", ui.visuals().text_color())
                            } else {
                                ("🔒", egui::Color32::from_gray(130))
                            };
                            ui.colored_label(col, badge);
                            ui.strong(node.name);
                        });
                        ui.label(node.desc);
                        ui.small(format!(
                            "Kosten: {} · Zeit {:.1} h",
                            cost_string(node.cost),
                            node.time / 3_600.0
                        ));
                        ui.small(unlock_text(node.unlock));

                        if done {
                            ui.colored_label(egui::Color32::from_rgb(90, 180, 100), "erforscht");
                        } else if active {
                            let p = self.research.active().map(|a| a.progress).unwrap_or(0.0);
                            ui.add(
                                egui::ProgressBar::new(p as f32)
                                    .desired_width(180.0)
                                    .text(format!("{:.0}%", p * 100.0)),
                            );
                        } else if can_start {
                            if ui
                                .add_enabled(!busy, egui::Button::new("Starten"))
                                .on_disabled_hover_text("Erst das laufende Projekt beenden")
                                .clicked()
                            {
                                actions.push(Action::StartResearch(id));
                            }
                        } else {
                            let missing: Vec<&str> = node
                                .prereqs
                                .iter()
                                .filter(|p| !self.research.is_done(**p))
                                .map(|p| p.name())
                                .collect();
                            ui.small(format!("benötigt: {}", missing.join(", ")));
                        }
                    });
                    ui.add_space(4.0);
                }
            });
        });
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
        Headquarters => "Hauptgebäude",
        ResearchLab => "Forschungseinrichtung",
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
        Headquarters => "HQ",
        ResearchLab => "Fo",
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

/// Beschreibt, was ein Forschungsknoten freischaltet.
fn unlock_text(unlock: Option<Unlock>) -> String {
    match unlock {
        Some(Unlock::Building(k)) => format!("→ schaltet {} frei", name(k)),
        Some(Unlock::Ascent(s)) => format!("→ schaltet {s} frei"),
        None => "→ öffnet Folge-Forschung".to_string(),
    }
}

fn cost_string(cost: &[(Resource, f64)]) -> String {
    cost.iter()
        .map(|(r, q)| format!("{:.0} {:?}", q, r))
        .collect::<Vec<_>>()
        .join(", ")
}

