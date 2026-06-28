//! Forschung als **Freischaltungs-Baum** (Phase 1, `forschung.md`).
//!
//! Forschung ist hier **Freischaltung, kein Prozentbonus** (Schutz der fairen
//! Außenkante, DESIGN.md §4.5). Sie nutzt **dieselbe Mechanik wie Bauen**: ein
//! Projekt ist eine Baustelle, die statt eines Gebäudes eine Freischaltung
//! erzeugt — es verbraucht Material über die Zeit als kontinuierlichen Fluss und
//! **kriecht bei Mangel, statt zu blockieren**. Bezahlt wird mit **Material**,
//! nicht mit Punkten.
//!
//! Energie ist ein **Fluss**, kein Vorrat: Eine laufende Forschung zieht Energie
//! nur über die **Forschungseinrichtung** (Beschleuniger, siehe
//! [`crate::production`]). Ohne Einrichtung läuft ein Projekt trotzdem — nur
//! langsamer und ohne Stromkosten.
//!
//! In Phase 1 läuft **genau ein** Projekt zur Zeit; weitere Slots schaltet
//! späteres Werk frei.

use serde::{Deserialize, Serialize};

use crate::planet::BuildingKind;
use crate::resource::Resource;

/// Geschwindigkeits-Beitrag je voll versorgter Forschungseinrichtung. Eine
/// Einrichtung verdoppelt grob das Tempo (×(1 + 1.0)); zwei ×3 usw. Platzhalter.
pub const LAB_SPEEDUP_PER_LAB: f64 = 1.0;
/// Elektronik/Sekunde, die eine voll laufende Einrichtung im Betrieb frisst
/// (Elektronik-Sink). Skaliert mit Energie- und Elektronik-Verfügbarkeit.
pub const LAB_ELECTRONICS_RATE: f64 = 0.3;

/// Die Forschungsknoten der Phase 1 (`forschung.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResearchId {
    /// Wurzel: schaltet die Hütte frei.
    Alloys,
    /// Wurzel: schaltet die Elektronikfabrik frei.
    Electronics,
    /// Stamm: öffnet Satelliten (Folge-Forschung).
    ComputerTech,
    /// Stamm: öffnet Raketen (Folge-Forschung).
    DriveTech,
    /// Krone: schaltet die Startrampe (Startklasse klein) frei.
    Rockets,
    /// Krone: schaltet die Satelliten-Nutzlasten frei.
    Satellites,
    /// Ausbau: erlaubt Gebäude-Stufe 2.
    UpgradeII,
    /// Ausbau: erlaubt Gebäude-Stufe 3.
    UpgradeIII,
}

/// Alle Knoten in kanonischer Reihenfolge (Wurzeln → Stämme → Krone → Ausbau).
pub const ALL: [ResearchId; 8] = [
    ResearchId::Alloys,
    ResearchId::Electronics,
    ResearchId::ComputerTech,
    ResearchId::DriveTech,
    ResearchId::Rockets,
    ResearchId::Satellites,
    ResearchId::UpgradeII,
    ResearchId::UpgradeIII,
];

/// Was ein abgeschlossener Knoten freischaltet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unlock {
    /// Macht ein Gebäude baubar.
    Building(BuildingKind),
    /// Hebt die erlaubte Gebäude-Ausbaustufe auf den angegebenen Wert
    /// (`crate::planet::MAX_LEVEL` ist die harte Obergrenze).
    UpgradeLevel(u32),
    /// Eine Aufstiegs-Fähigkeit. Das zugehörige Subsystem (Startrampe,
    /// Satelliten-Nutzlasten) ist eigenes, späteres Werk — hier nur als
    /// erreichte Fähigkeit vermerkt.
    Ascent(&'static str),
}

/// Statische Kennwerte eines Forschungsknotens. Mengen/Zeiten sind
/// Platzhalter-Balance (Tuning offen, `forschung.md`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResearchNode {
    /// Anzeigename (deutsch).
    pub name: &'static str,
    /// Vorausgesetzte Knoten (alle müssen abgeschlossen sein).
    pub prereqs: &'static [ResearchId],
    /// Materialkosten — über die Projektzeit als Fluss verbraucht (kein Einmalkauf).
    pub cost: &'static [(Resource, f64)],
    /// Projektzeit in Sekunden bei voller Materialversorgung, *ohne* Beschleuniger.
    pub time: f64,
    /// Was der Abschluss freischaltet (`None` = öffnet nur Folge-Forschung).
    pub unlock: Option<Unlock>,
    /// Kurzbeschreibung für die UI.
    pub desc: &'static str,
}

impl ResearchId {
    /// Kennwerte dieses Knotens.
    pub fn node(self) -> ResearchNode {
        use Resource::*;
        match self {
            ResearchId::Alloys => ResearchNode {
                name: "Legierungen",
                prereqs: &[],
                cost: &[(Metals, 100.0)],
                time: 1_800.0,
                unlock: Some(Unlock::Building(BuildingKind::Smelter)),
                desc: "Schaltet die Hütte frei (Metalle → Legierungen).",
            },
            ResearchId::Electronics => ResearchNode {
                name: "Elektronik",
                prereqs: &[],
                cost: &[(Silicates, 100.0)],
                time: 1_800.0,
                unlock: Some(Unlock::Building(BuildingKind::ElectronicsFab)),
                desc: "Schaltet die Elektronikfabrik frei (Silikate + Metalle → Elektronik).",
            },
            ResearchId::ComputerTech => ResearchNode {
                name: "Komputertechnik",
                prereqs: &[ResearchId::Electronics],
                cost: &[(Electronics, 80.0), (Alloys, 60.0)],
                time: 5_400.0,
                unlock: None,
                desc: "Öffnet die Satelliten-Forschung.",
            },
            ResearchId::DriveTech => ResearchNode {
                name: "Triebwerktechnik",
                prereqs: &[ResearchId::Alloys],
                cost: &[(Metals, 80.0), (Alloys, 60.0), (Gases, 40.0)],
                time: 5_400.0,
                unlock: None,
                desc: "Öffnet die Raketen-Forschung.",
            },
            ResearchId::Rockets => ResearchNode {
                name: "Raketen",
                prereqs: &[ResearchId::DriveTech],
                cost: &[(Alloys, 120.0), (Gases, 80.0), (Electronics, 60.0)],
                time: 10_800.0,
                unlock: Some(Unlock::Ascent("Startrampe (Startklasse klein)")),
                desc: "Schaltet die Startrampe frei — den Riegel zur Orbit-Ebene.",
            },
            ResearchId::Satellites => ResearchNode {
                name: "Satelliten",
                prereqs: &[ResearchId::ComputerTech],
                cost: &[(Electronics, 120.0), (Alloys, 80.0)],
                time: 10_800.0,
                unlock: Some(Unlock::Ascent("Satellit-Nutzlasten (Scan + Forschung)")),
                desc: "Schaltet Scan- und Forschungs-Satellit frei.",
            },
            ResearchId::UpgradeII => ResearchNode {
                name: "Ausbaustufe II",
                prereqs: &[ResearchId::Alloys],
                cost: &[(Metals, 120.0), (Alloys, 40.0)],
                time: 5_400.0,
                unlock: Some(Unlock::UpgradeLevel(2)),
                desc: "Erlaubt den Ausbau von Gebäuden auf Stufe 2 (mehr Leistung je Kachel).",
            },
            ResearchId::UpgradeIII => ResearchNode {
                name: "Ausbaustufe III",
                prereqs: &[ResearchId::UpgradeII],
                cost: &[(Alloys, 120.0), (Electronics, 80.0)],
                time: 9_000.0,
                unlock: Some(Unlock::UpgradeLevel(3)),
                desc: "Erlaubt den Ausbau von Gebäuden auf Stufe 3.",
            },
        }
    }

    /// Anzeigename — Kurzform für [`ResearchNode::name`].
    pub fn name(self) -> &'static str {
        self.node().name
    }
}

/// Ein laufendes Forschungsprojekt (genau eines in Phase 1).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ActiveProject {
    pub id: ResearchId,
    /// Fortschritt `0.0..=1.0`.
    pub progress: f64,
}

/// Der Forschungszustand einer Kolonie: abgeschlossene Knoten, das aktive
/// Projekt und die Energie-Priorität der Forschung.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ResearchState {
    completed: Vec<ResearchId>,
    active: Option<ActiveProject>,
    /// Energie-Priorität der Forschung (höher = wird bei Knappheit zuerst
    /// bedient), analog zur Gebäude-Priorität.
    priority: i32,
}

impl ResearchState {
    pub fn new() -> Self {
        ResearchState::default()
    }

    /// Ob ein Knoten bereits erforscht ist.
    pub fn is_done(&self, id: ResearchId) -> bool {
        self.completed.contains(&id)
    }

    /// Die abgeschlossenen Knoten.
    pub fn completed(&self) -> &[ResearchId] {
        &self.completed
    }

    /// Das aktive Projekt, falls eines läuft.
    pub fn active(&self) -> Option<&ActiveProject> {
        self.active.as_ref()
    }

    /// Energie-Priorität der Forschung.
    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
    }

    /// Ob ein Knoten startbar ist: alle Voraussetzungen erfüllt, noch nicht
    /// erforscht und nicht gerade aktiv.
    pub fn can_start(&self, id: ResearchId) -> bool {
        if self.is_done(id) {
            return false;
        }
        if self.active.map(|a| a.id) == Some(id) {
            return false;
        }
        id.node().prereqs.iter().all(|p| self.is_done(*p))
    }

    /// Alle gerade startbaren Knoten (Voraussetzungen erfüllt, offen).
    pub fn available(&self) -> Vec<ResearchId> {
        ALL.iter().copied().filter(|id| self.can_start(*id)).collect()
    }

    /// Startet ein Projekt, sofern startbar und kein anderes läuft. Gibt `true`
    /// bei Erfolg.
    pub fn start(&mut self, id: ResearchId) -> bool {
        if self.active.is_some() || !self.can_start(id) {
            return false;
        }
        self.active = Some(ActiveProject { id, progress: 0.0 });
        true
    }

    /// Bricht das aktive Projekt ab (Fortschritt geht verloren). Gibt das
    /// abgebrochene Projekt zurück.
    pub fn cancel(&mut self) -> Option<ActiveProject> {
        self.active.take()
    }

    /// Markiert das aktive Projekt als abgeschlossen und räumt den Slot. Intern
    /// von [`crate::production`] aufgerufen, wenn der Fortschritt 1.0 erreicht.
    pub(crate) fn complete_active(&mut self) -> Option<ResearchId> {
        let done = self.active.take()?;
        if !self.completed.contains(&done.id) {
            self.completed.push(done.id);
        }
        Some(done.id)
    }

    /// Setzt den Fortschritt des aktiven Projekts (intern).
    pub(crate) fn set_active_progress(&mut self, progress: f64) {
        if let Some(a) = &mut self.active {
            a.progress = progress;
        }
    }

    /// Ob ein Gebäude baubar ist: entweder ohne Forschungsvoraussetzung oder
    /// deren Knoten ist erforscht.
    pub fn is_building_unlocked(&self, kind: BuildingKind) -> bool {
        match kind.required_research() {
            None => true,
            Some(id) => self.is_done(id),
        }
    }

    /// Die höchste Gebäude-Ausbaustufe, die die bisherige Forschung erlaubt
    /// (Stufe 1 ohne Forschung; [`Unlock::UpgradeLevel`]-Knoten heben sie an).
    pub fn max_building_level(&self) -> u32 {
        let mut level = 1;
        for id in &self.completed {
            if let Some(Unlock::UpgradeLevel(l)) = id.node().unlock {
                level = level.max(l);
            }
        }
        level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_are_available_branches_are_gated() {
        let r = ResearchState::new();
        let avail = r.available();
        assert!(avail.contains(&ResearchId::Alloys));
        assert!(avail.contains(&ResearchId::Electronics));
        // Stämme/Krone brauchen Voraussetzungen.
        assert!(!avail.contains(&ResearchId::ComputerTech));
        assert!(!avail.contains(&ResearchId::Rockets));
    }

    #[test]
    fn completing_a_prereq_opens_the_next_node() {
        let mut r = ResearchState::new();
        assert!(r.start(ResearchId::Electronics));
        r.set_active_progress(1.0);
        r.complete_active();
        assert!(r.is_done(ResearchId::Electronics));
        // Jetzt ist Komputertechnik startbar.
        assert!(r.can_start(ResearchId::ComputerTech));
        // Satelliten erst nach Komputertechnik.
        assert!(!r.can_start(ResearchId::Satellites));
    }

    #[test]
    fn only_one_active_project_at_a_time() {
        let mut r = ResearchState::new();
        assert!(r.start(ResearchId::Alloys));
        // Zweiter Start scheitert, solange eines läuft.
        assert!(!r.start(ResearchId::Electronics));
        r.cancel();
        assert!(r.start(ResearchId::Electronics));
    }

    #[test]
    fn upgrade_levels_unlock_via_research() {
        let mut r = ResearchState::new();
        assert_eq!(r.max_building_level(), 1);
        // Ausbaustufe II braucht Legierungen.
        assert!(!r.can_start(ResearchId::UpgradeII));
        for id in [ResearchId::Alloys, ResearchId::UpgradeII] {
            assert!(r.start(id));
            r.set_active_progress(1.0);
            r.complete_active();
        }
        assert_eq!(r.max_building_level(), 2);
        // Stufe III erst nach II.
        assert!(r.can_start(ResearchId::UpgradeIII));
        r.start(ResearchId::UpgradeIII);
        r.set_active_progress(1.0);
        r.complete_active();
        assert_eq!(r.max_building_level(), 3);
    }

    #[test]
    fn buildings_locked_until_researched() {
        let mut r = ResearchState::new();
        // Hütte/Fab anfangs gesperrt, Mine/Solar frei.
        assert!(!r.is_building_unlocked(BuildingKind::Smelter));
        assert!(!r.is_building_unlocked(BuildingKind::ElectronicsFab));
        assert!(r.is_building_unlocked(BuildingKind::MetalMine));
        assert!(r.is_building_unlocked(BuildingKind::SolarCollector));
        // Nach „Legierungen" ist die Hütte frei.
        r.start(ResearchId::Alloys);
        r.set_active_progress(1.0);
        r.complete_active();
        assert!(r.is_building_unlocked(BuildingKind::Smelter));
        assert!(!r.is_building_unlocked(BuildingKind::ElectronicsFab));
    }
}
