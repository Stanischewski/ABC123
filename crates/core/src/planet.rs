//! Die Bau-Ebene: ein Planeten-Raster aus gelände-typisierten Kacheln, auf die
//! Gebäude gesetzt werden.
//!
//! Das ökonomische Herz des Spiels (DESIGN.md §3.1, §4.2). Fläche ist hart
//! begrenzt, Gelände bindet jeden Rohstoff an *einen* Kacheltyp, und
//! **Adjazenz-Boni** belohnen kluge Platzierung — das einzige räumliche Puzzle
//! der Bau-Ebene ist die *Platzierung*, kein Transport (Begleitdokument §1).
//!
//! Reine, deterministische Datenstruktur; die Zeit-Integration der Produktion
//! lebt in [`crate::production`].

use serde::{Deserialize, Serialize};

use crate::economy::Stockpile;
use crate::resource::Resource;

/// Geländeprofil einer Kachel. Bindet Rohstoffe an Orte (DESIGN.md §4.1):
/// jeder Rohstoff hängt an *einem* Gelände, damit Förder-Platzierung zählt und
/// Planeten unterschiedliche Profile bekommen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    /// Gestein → Metalle.
    Rock,
    /// Kristallgelände → Silikate (seltener, deckelt die Tech-Decke).
    Crystal,
    /// Gasvorkommen → Gase.
    GasField,
    /// Eis → Wasser/Treibstoff (spätere Phase); in Phase 1 nur Baugrund.
    Ice,
    /// Karger Grund — kein Abbau, aber freie Baufläche (Kraftwerke, Lager).
    Barren,
}

impl Terrain {
    /// Der Rohstoff, der auf diesem Gelände förderbar ist (falls überhaupt).
    pub fn raw_resource(self) -> Option<Resource> {
        match self {
            Terrain::Rock => Some(Resource::Metals),
            Terrain::Crystal => Some(Resource::Silicates),
            Terrain::GasField => Some(Resource::Gases),
            Terrain::Ice | Terrain::Barren => None,
        }
    }
}

/// Gebäudetypen der Bau-Ebene (Phase 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingKind {
    // Förderer (gelände-gebunden)
    MetalMine,
    CrystalExtractor,
    GasCollector,
    // Raffinerien (veredeln, energiekostend)
    Smelter,        // Metalle → Legierungen
    ElectronicsFab, // Silikate + Metalle → Elektronik
    CompositeFab,   // Legierungen + Elektronik → Komposit
    // Energie (Portfolio: Solar vs. Fusion, DESIGN.md §4.1)
    SolarCollector,
    FusionReactor,
    // Logistik/Lager — erzeugt Adjazenz-Durchsatz für Nachbarn.
    Depot,
}

/// Statische Kennwerte eines Gebäudetyps. Platzhalter-Balance für Phase 1
/// (Tuning offen, DESIGN.md §7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildingSpec {
    /// Erforderliches Gelände (`None` = beliebige Baufläche).
    pub required_terrain: Option<Terrain>,
    /// Erzeugter Stoff (`None` = Kraftwerk/Lager).
    pub output: Option<Resource>,
    /// Basisrate in Output-Einheiten/Sekunde bei voller Versorgung.
    pub base_rate: f64,
    /// Betriebsenergie/Sekunde, solange das Gebäude läuft (Verbraucher).
    pub energy_demand: f64,
    /// Erzeugte Bruttoenergie/Sekunde (Kraftwerke).
    pub energy_output: f64,
    /// Gas/Sekunde bei voller Last (Fusion); sonst 0.
    pub fuel_rate: f64,
    /// Skaliert der Ertrag mit `1/r²` zum Bahnradius? (Solar, DESIGN.md §4.1).
    pub solar: bool,
    /// Baukosten — über die Bauzeit als kontinuierlicher Fluss verbraucht
    /// (kein Einmalkauf; strukturen.md §Bau-Ebene), nicht als Sofortzahlung.
    pub build_cost: &'static [(Resource, f64)],
    /// Bauzeit in Sekunden bei voller Materialversorgung.
    pub build_time: f64,
}

impl BuildingKind {
    /// Kennwerte dieses Gebäudetyps.
    pub fn spec(self) -> BuildingSpec {
        use BuildingKind::*;
        use Resource::*;
        match self {
            MetalMine => BuildingSpec {
                required_terrain: Some(Terrain::Rock),
                output: Some(Metals),
                base_rate: 1.0,
                energy_demand: 2.0,
                energy_output: 0.0,
                fuel_rate: 0.0,
                solar: false,
                build_cost: &[(Metals, 50.0)],
                build_time: 7_200.0,
            },
            CrystalExtractor => BuildingSpec {
                required_terrain: Some(Terrain::Crystal),
                output: Some(Silicates),
                base_rate: 0.6,
                energy_demand: 2.0,
                energy_output: 0.0,
                fuel_rate: 0.0,
                solar: false,
                build_cost: &[(Metals, 60.0)],
                build_time: 9_000.0,
            },
            GasCollector => BuildingSpec {
                required_terrain: Some(Terrain::GasField),
                output: Some(Gases),
                base_rate: 0.8,
                energy_demand: 2.0,
                energy_output: 0.0,
                fuel_rate: 0.0,
                solar: false,
                build_cost: &[(Metals, 60.0)],
                build_time: 9_000.0,
            },
            Smelter => BuildingSpec {
                required_terrain: None,
                output: Some(Alloys),
                base_rate: 0.5,
                energy_demand: 3.0,
                energy_output: 0.0,
                fuel_rate: 0.0,
                solar: false,
                build_cost: &[(Metals, 80.0)],
                build_time: 10_800.0,
            },
            ElectronicsFab => BuildingSpec {
                required_terrain: None,
                output: Some(Electronics),
                base_rate: 0.4,
                energy_demand: 4.0,
                energy_output: 0.0,
                fuel_rate: 0.0,
                solar: false,
                build_cost: &[(Metals, 80.0), (Alloys, 20.0)],
                build_time: 14_400.0,
            },
            CompositeFab => BuildingSpec {
                required_terrain: None,
                output: Some(Composite),
                base_rate: 0.2,
                energy_demand: 6.0,
                energy_output: 0.0,
                fuel_rate: 0.0,
                solar: false,
                build_cost: &[(Alloys, 60.0), (Electronics, 60.0)],
                build_time: 28_800.0,
            },
            SolarCollector => BuildingSpec {
                required_terrain: None,
                output: None,
                base_rate: 0.0,
                energy_demand: 0.0,
                energy_output: 10.0,
                fuel_rate: 0.0,
                solar: true,
                build_cost: &[(Metals, 40.0), (Electronics, 10.0)],
                build_time: 7_200.0,
            },
            FusionReactor => BuildingSpec {
                required_terrain: None,
                output: None,
                base_rate: 0.0,
                energy_demand: 0.0,
                energy_output: 15.0,
                fuel_rate: 1.0,
                solar: false,
                build_cost: &[(Metals, 100.0), (Alloys, 40.0)],
                build_time: 18_000.0,
            },
            Depot => BuildingSpec {
                required_terrain: None,
                output: None,
                base_rate: 0.0,
                energy_demand: 0.0,
                energy_output: 0.0,
                fuel_rate: 0.0,
                solar: false,
                build_cost: &[(Metals, 30.0)],
                build_time: 3_600.0,
            },
        }
    }

    /// Ob dieses Gebäude auf das gegebene Gelände gesetzt werden darf.
    pub fn can_build_on(self, terrain: Terrain) -> bool {
        match self.spec().required_terrain {
            Some(req) => req == terrain,
            None => true,
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_progress() -> f64 {
    1.0
}

/// Eine platzierte Struktur samt Drossel-Priorität (höher = wird bei
/// Energie-/Input-Knappheit zuerst bedient), Ein/Aus-Schalter und
/// Baufortschritt. Ein ausgeschaltetes *oder* noch im Bau befindliches Gebäude
/// ist inert: es produziert nichts, verbraucht keine Energie und trägt keinen
/// Adjazenz-Bonus bei.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub kind: BuildingKind,
    pub priority: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Baufortschritt `0.0..=1.0`. `< 1.0` = Baustelle (inert), `1.0` = fertig.
    #[serde(default = "default_progress")]
    pub progress: f64,
}

impl Building {
    /// Ein fertiges, betriebsbereites Gebäude (Fortschritt 1.0).
    pub fn new(kind: BuildingKind) -> Self {
        Building {
            kind,
            priority: 0,
            enabled: true,
            progress: 1.0,
        }
    }

    pub fn with_priority(kind: BuildingKind, priority: i32) -> Self {
        Building {
            kind,
            priority,
            enabled: true,
            progress: 1.0,
        }
    }

    /// Eine frische Baustelle (Fortschritt 0.0); wird über die Bauzeit
    /// fertiggestellt (siehe [`Grid::advance_construction`]).
    pub fn construction_site(kind: BuildingKind) -> Self {
        Building {
            kind,
            priority: 0,
            enabled: true,
            progress: 0.0,
        }
    }

    /// Betriebsbereit = fertig gebaut *und* eingeschaltet.
    pub fn is_operational(&self) -> bool {
        self.enabled && self.progress >= 1.0
    }

    /// Noch im Bau (Fortschritt < 1.0).
    pub fn under_construction(&self) -> bool {
        self.progress < 1.0
    }
}

/// Eine Rasterkachel: Gelände plus optionales Gebäude.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: Terrain,
    pub building: Option<Building>,
}

/// Fehler beim Platzieren eines Gebäudes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceError {
    OutOfBounds,
    Occupied,
    WrongTerrain,
}

/// Das Planeten-Raster: endliche Fläche (DESIGN.md §3.1), Reihen-weise (row-major).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    tiles: Vec<Tile>,
}

impl Grid {
    /// Legt ein Raster an, dessen Kacheln alle dasselbe Gelände tragen.
    pub fn new(width: u32, height: u32, terrain: Terrain) -> Self {
        let tiles = vec![
            Tile {
                terrain,
                building: None,
            };
            (width * height) as usize
        ];
        Grid {
            width,
            height,
            tiles,
        }
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }

    /// Kachel an `(x, y)`.
    pub fn tile(&self, x: u32, y: u32) -> Option<&Tile> {
        self.index(x, y).map(|i| &self.tiles[i])
    }

    /// Setzt das Gelände einer Kachel (für die Welterzeugung).
    pub fn set_terrain(&mut self, x: u32, y: u32, terrain: Terrain) -> bool {
        if let Some(i) = self.index(x, y) {
            self.tiles[i].terrain = terrain;
            true
        } else {
            false
        }
    }

    /// Platziert ein Gebäude, prüft Grenzen, Belegung und Gelände.
    pub fn place(&mut self, x: u32, y: u32, building: Building) -> Result<(), PlaceError> {
        let i = self.index(x, y).ok_or(PlaceError::OutOfBounds)?;
        let tile = &self.tiles[i];
        if tile.building.is_some() {
            return Err(PlaceError::Occupied);
        }
        if !building.kind.can_build_on(tile.terrain) {
            return Err(PlaceError::WrongTerrain);
        }
        self.tiles[i].building = Some(building);
        Ok(())
    }

    /// Entfernt ein Gebäude und gibt es zurück.
    pub fn remove(&mut self, x: u32, y: u32) -> Option<Building> {
        let i = self.index(x, y)?;
        self.tiles[i].building.take()
    }

    /// Schaltet ein Gebäude ein oder aus. Gibt `true` bei Erfolg.
    pub fn set_enabled(&mut self, x: u32, y: u32, enabled: bool) -> bool {
        if let Some(i) = self.index(x, y) {
            if let Some(b) = &mut self.tiles[i].building {
                b.enabled = enabled;
                return true;
            }
        }
        false
    }

    /// Setzt die Drossel-Priorität eines Gebäudes. Gibt `true` bei Erfolg.
    pub fn set_priority(&mut self, x: u32, y: u32, priority: i32) -> bool {
        if let Some(i) = self.index(x, y) {
            if let Some(b) = &mut self.tiles[i].building {
                b.priority = priority;
                return true;
            }
        }
        false
    }

    /// Schreitet alle Baustellen um `dt` Sekunden voran.
    ///
    /// Bauen ist **kontinuierlicher Materialfluss**, kein Einmalkauf
    /// (strukturen.md §Bau-Ebene): eine Baustelle zieht ihre Baukosten über die
    /// Bauzeit aus dem globalen Lager. Fehlt Material, **kriecht** der Bau
    /// proportional zum knappsten Input, statt zu blockieren. Gibt die Zahl der
    /// in diesem Schritt fertiggestellten Bauten zurück.
    pub fn advance_construction(&mut self, stock: &mut Stockpile, dt: f64) -> usize {
        let dt = dt.max(0.0);
        let mut completed = 0;
        for tile in &mut self.tiles {
            let Some(b) = &mut tile.building else { continue };
            if b.progress >= 1.0 {
                continue;
            }
            let spec = b.kind.spec();
            if spec.build_time <= 0.0 {
                b.progress = 1.0;
                completed += 1;
                continue;
            }

            // Angestrebter Fortschritt dieses Schritts bei voller Versorgung.
            let want = (dt / spec.build_time).min(1.0 - b.progress);
            if want <= 0.0 {
                continue;
            }

            // Materiallimit über alle Baukosten (weicher Abfall, keine Klippe).
            let mut frac = 1.0_f64;
            for (res, qty) in spec.build_cost {
                let need = qty * want;
                if need > 0.0 {
                    frac = frac.min(stock.get(*res) / need);
                }
            }
            let actual = want * frac.clamp(0.0, 1.0);
            if actual <= 0.0 {
                continue;
            }

            for (res, qty) in spec.build_cost {
                stock.add(*res, -qty * actual);
            }
            b.progress += actual;
            if b.progress >= 1.0 {
                b.progress = 1.0;
                completed += 1;
            }
        }
        completed
    }

    /// Die orthogonalen Nachbar-Gebäude von `(x, y)` (4er-Nachbarschaft).
    /// Ausgeschaltete Gebäude zählen nicht (sie sind inert).
    pub fn neighbor_buildings(&self, x: u32, y: u32) -> Vec<BuildingKind> {
        let mut out = Vec::new();
        let deltas: [(i64, i64); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dx, dy) in deltas {
            let nx = x as i64 + dx;
            let ny = y as i64 + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            if let Some(tile) = self.tile(nx as u32, ny as u32) {
                if let Some(b) = tile.building {
                    if b.is_operational() {
                        out.push(b.kind);
                    }
                }
            }
        }
        out
    }

    /// Iteriert über alle belegten Kacheln als `(x, y, Building)`.
    pub fn buildings(&self) -> impl Iterator<Item = (u32, u32, Building)> + '_ {
        self.tiles.iter().enumerate().filter_map(move |(i, t)| {
            t.building.map(|b| {
                let i = i as u32;
                (i % self.width, i / self.width, b)
            })
        })
    }

    /// Adjazenz-Multiplikator (≥ 1.0) für den Produzenten an `(x, y)`.
    ///
    /// Phase-1-Regeln (bewusst schlicht, DESIGN.md §4.2 / Begleitdokument §13):
    /// ein **Lager** in der Nachbarschaft beschleunigt jeden Produzenten
    /// („Mine neben Lager"); eine **Raffinerie** profitiert zusätzlich von einem
    /// benachbarten Förderer, der einen ihrer Eingänge liefert (Kolokation).
    /// Gedeckelt, damit Adjazenz würzt, aber nicht dominiert.
    pub fn adjacency_multiplier(&self, x: u32, y: u32) -> f64 {
        const PER_NEIGHBOR: f64 = 0.10;
        const CAP: f64 = 1.5;

        let Some(building) = self.tile(x, y).and_then(|t| t.building) else {
            return 1.0;
        };
        let Some(output) = building.kind.spec().output else {
            return 1.0;
        };
        let recipe_inputs = output.recipe().map(|r| r.inputs).unwrap_or(&[]);

        let mut bonus = 0.0;
        for neighbor in self.neighbor_buildings(x, y) {
            if neighbor == BuildingKind::Depot {
                bonus += PER_NEIGHBOR;
            } else if let Some(nout) = neighbor.spec().output {
                if recipe_inputs.iter().any(|(r, _)| *r == nout) {
                    bonus += PER_NEIGHBOR;
                }
            }
        }
        (1.0 + bonus).min(CAP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_binds_raw_resources() {
        assert_eq!(Terrain::Rock.raw_resource(), Some(Resource::Metals));
        assert_eq!(Terrain::Crystal.raw_resource(), Some(Resource::Silicates));
        assert_eq!(Terrain::GasField.raw_resource(), Some(Resource::Gases));
        assert_eq!(Terrain::Barren.raw_resource(), None);
    }

    #[test]
    fn placement_respects_terrain_and_occupancy() {
        let mut g = Grid::new(3, 3, Terrain::Barren);
        g.set_terrain(1, 1, Terrain::Rock);

        // Mine braucht Gestein.
        assert_eq!(
            g.place(0, 0, Building::new(BuildingKind::MetalMine)),
            Err(PlaceError::WrongTerrain)
        );
        assert!(g.place(1, 1, Building::new(BuildingKind::MetalMine)).is_ok());
        // Kachel jetzt belegt.
        assert_eq!(
            g.place(1, 1, Building::new(BuildingKind::Depot)),
            Err(PlaceError::Occupied)
        );
        // Außerhalb.
        assert_eq!(
            g.place(9, 9, Building::new(BuildingKind::Depot)),
            Err(PlaceError::OutOfBounds)
        );
    }

    #[test]
    fn depot_neighbour_grants_adjacency_bonus() {
        let mut g = Grid::new(3, 3, Terrain::Rock);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        // Ohne Nachbarn: neutral.
        assert!((g.adjacency_multiplier(0, 0) - 1.0).abs() < 1e-9);
        // Lager daneben: +10 %.
        g.set_terrain(1, 0, Terrain::Barren);
        g.place(1, 0, Building::new(BuildingKind::Depot)).unwrap();
        assert!((g.adjacency_multiplier(0, 0) - 1.1).abs() < 1e-9);
    }

    #[test]
    fn refinery_benefits_from_adjacent_input_extractor() {
        // Smelter (← Metalle) neben einer Mine (Metalle).
        let mut g = Grid::new(2, 1, Terrain::Barren);
        g.set_terrain(0, 0, Terrain::Rock);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        g.place(1, 0, Building::new(BuildingKind::Smelter)).unwrap();
        // Smelter zieht Bonus aus der benachbarten Metallquelle.
        assert!((g.adjacency_multiplier(1, 0) - 1.1).abs() < 1e-9);
    }

    #[test]
    fn adjacency_is_capped() {
        // Mine von vier Lagern umgeben → +40 %, aber Deckel greift erst > 50 %.
        let mut g = Grid::new(3, 3, Terrain::Barren);
        g.set_terrain(1, 1, Terrain::Rock);
        g.place(1, 1, Building::new(BuildingKind::MetalMine)).unwrap();
        for (x, y) in [(0, 1), (2, 1), (1, 0), (1, 2)] {
            g.place(x, y, Building::new(BuildingKind::Depot)).unwrap();
        }
        assert!((g.adjacency_multiplier(1, 1) - 1.4).abs() < 1e-9);
    }

    #[test]
    fn construction_consumes_materials_and_completes() {
        let mut g = Grid::new(1, 1, Terrain::Barren);
        g.place(0, 0, Building::construction_site(BuildingKind::Depot))
            .unwrap();
        // Depot: 30 Metalle Baukosten, 3600 s Bauzeit.
        let mut stock = Stockpile::new();
        stock.set(Resource::Metals, 1000.0);

        // Halbe Bauzeit → 50 % Fortschritt, 15 Metalle verbraucht.
        let done = g.advance_construction(&mut stock, 1800.0);
        assert_eq!(done, 0);
        let b = g.tile(0, 0).unwrap().building.unwrap();
        assert!((b.progress - 0.5).abs() < 1e-9);
        assert!((stock.get(Resource::Metals) - 985.0).abs() < 1e-6);
        assert!(!b.is_operational());

        // Rest der Bauzeit → fertig und betriebsbereit.
        let done = g.advance_construction(&mut stock, 1800.0);
        assert_eq!(done, 1);
        assert!(g.tile(0, 0).unwrap().building.unwrap().is_operational());
        assert!((stock.get(Resource::Metals) - 970.0).abs() < 1e-6);
    }

    #[test]
    fn construction_crawls_without_materials() {
        let mut g = Grid::new(1, 1, Terrain::Rock);
        g.place(0, 0, Building::construction_site(BuildingKind::MetalMine))
            .unwrap();
        // Leeres Lager → trotz langer Zeit kein Fortschritt (kriecht, blockt nicht).
        let mut empty = Stockpile::new();
        let done = g.advance_construction(&mut empty, 1_000_000.0);
        assert_eq!(done, 0);
        assert_eq!(g.tile(0, 0).unwrap().building.unwrap().progress, 0.0);
    }

    #[test]
    fn construction_site_grants_no_adjacency_until_complete() {
        let mut g = Grid::new(2, 1, Terrain::Rock);
        g.place(0, 0, Building::new(BuildingKind::MetalMine)).unwrap();
        // Baustelle als Nachbar zählt noch nicht.
        g.place(1, 0, Building::construction_site(BuildingKind::Depot))
            .unwrap();
        assert!((g.adjacency_multiplier(0, 0) - 1.0).abs() < 1e-9);
        // Fertigstellen → Bonus greift.
        let mut stock = Stockpile::new();
        stock.set(Resource::Metals, 1000.0);
        g.advance_construction(&mut stock, 10_000.0);
        assert!((g.adjacency_multiplier(0, 0) - 1.1).abs() < 1e-9);
    }

    #[test]
    fn grid_round_trips_through_json() {
        let mut g = Grid::new(2, 2, Terrain::Rock);
        g.place(0, 0, Building::with_priority(BuildingKind::MetalMine, 5))
            .unwrap();
        let json = serde_json::to_string(&g).unwrap();
        let back: Grid = serde_json::from_str(&json).unwrap();
        assert_eq!(g, back);
    }
}
