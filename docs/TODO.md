# TODO

*Lebende Aufgabenliste. Gegliedert nach Roadmap-Phasen (`DESIGN.md` §6). Erledigtes
ist abgehakt, damit der Fortschritt sichtbar bleibt; Offenes verweist, wo nützlich,
auf die Design-Dokumente. Kein fester Zeitplan — Reihenfolge ist eine Empfehlung.*

Legende: `[x]` erledigt · `[ ]` offen · `[~]` teilweise.

---

## Phase 0 — Fundament

- [x] Workspace-Gerüst (`core` / `server` / `client`), CI-fähiger `cargo build`/`test`
- [x] Simulationsmodell: Kepler-Bahnen „on rails" (planar, Newton-Raphson)
- [x] Körper-Hierarchie (Mond ⊂ Planet ⊂ Stern), absolute Positionen
- [x] Ressourcenmodell 3 + 2 + 1; Energiebudget mit Priorität; Logistik-Effizienz
- [x] Server-Skelett: Axum, `/health`, `/system`, `/ws` (Positions-Stream)
- [x] Postgres-Grundlage: optionale Persistenz, Migration `world` (Singleton)
- [x] Browser-Ziel (WebAssembly) + Server liefert die UI aus

## Phase 1 — Vom Planeten in den Orbit

### Bau-Ebene (ökonomisches Herz) — weitgehend da

- [x] Raster mit Gelände-Typen, Platzierungsregeln (Gelände/Belegung/Eindeutigkeit)
- [x] Gebäude: Förderer, Raffinerien, Solar/Fusion, Lager, Komposit-Werk
- [x] Hauptgebäude (Anti-Softlock, Lager-/Adjazenz-Anker, eines pro Körper)
- [x] Adjazenz-Boni inkl. Aufschlüsselung; Energie- und Input-Drosselung nach Priorität
- [x] Bauschlange als kontinuierlicher Ressourcenfluss (kriecht bei Mangel)
- [x] egui-UI: Bauen/Abreißen/Ein-Aus, Prioritäten, Lager-Raten (+x/h), Energie-Balken
- [x] Platzierungs-Vorschau (Hover) + Gebäude-Info (Rechtsklick)
- [x] Schematische System-Ansicht (egui-Marker, an Sim-Zeit gekoppelt)
- [x] **Lagerkapazität**: Deckel je Stoff; Lager/Hauptgebäude heben ihn; volle Lager drosseln die Produktion
- [x] **Forschung als Freischaltungs-Baum** (Design: `forschung.md`) — Material-finanziert,
  Energie als Fluss, kein Punkte-Modell mehr (`Resource::Research` entfernt):
  - [x] Forschungsprojekt als Baustelle (kriecht bei Mangel, zieht Energie unter Priorität)
  - [x] Knoten Legierungen / Elektronik / Komputertechnik / Triebwerktechnik / Raketen /
    Satelliten samt Voraussetzungen
  - [x] Forschungseinrichtung vom Punkte-Produzenten zum **Beschleuniger** umgebaut
    (senkt Projektzeit, frisst im Betrieb Elektronik + Energie)
  - [x] Gebäude-Freischaltung verdrahtet: Hütte ← *Legierungen*, Elektronikfabrik ← *Elektronik*
  - [x] egui-Forschungs-Tab: Baum mit Status, Projekt starten/abbrechen, Energie-Priorität
  - [x] Stufen-Forschung: Ausbaustufe II/III heben die erlaubte Gebäude-Stufe
  - [~] Aufstiegs-Freischaltungen (Startrampe-Startklasse, Satellit-Nutzlasten) als
    Fähigkeit im Baum vermerkt; konkrete Gebäude/Subsystem folgen (s. Aufstieg)
- [x] **Gebäude-Upgrades**: dieselbe Bau-Mechanik auf bestehender Kachel (Stufen 1–3);
  Upgrade macht das Gebäude inert (wie Baustelle), höhere Stufen skalieren Output/Lager/
  Energie; Stufen per Forschung (Ausbaustufe II/III) freigeschaltet; Ausbau abbrechbar
- [ ] **Balance-Pass**: Förderraten, Energiekosten, Bauzeiten, Adjazenz-Stärke (Platzhalter → tunen)
- [ ] Startrampe (Riegel zur Orbit-Ebene, upgradebare Startklasse) — braucht Aufstiegs-Subsystem
- [ ] Verteidigungsanlage (planetar, Legierungs-Sink) — erst mit Kampf (Phase 2) sinnvoll

### Aufstieg ins All & zweistufige Aufklärung (`Oekonomie-…` §12)

- [ ] **Fog-of-War** auf dem Raster: nur Startzone sichtbar; Rest verdeckt
- [ ] **1. Satellit – Blick nach unten**: eigenen Planeten kartieren → Profil & Lücke
- [ ] **Scan- vs. Forschungs-Satellit**: erste echte Spezialisierung bei knapper Startkapazität
- [ ] **Teleskop**: System-Skelett aufdecken (Körper, Bahnen, grober Typ) — gratis
- [ ] **Sonde**: Inhalte eines Körpers aufdecken (kostet je Körper eine Sonde)
- [ ] **Station** als zweite Baufläche im Orbit (kein Gelände; geländefreie Bauten)
- [ ] **Mond** als zweiter Körper mit eigenem Profil → erste Mehr-Kolonie-Verwaltung
- [ ] Aufstiegs-Gradient: jede Sprosse verlangt eine kleine Umkonfiguration, kein bloßer Timer

### Server & Persistenz

- [ ] **Bau-Zustand persistieren**: Raster + Lager + Sim-Zeit in `world` (derzeit nur `System`)
- [ ] **Client ↔ Server**: UI lädt Zustand vom Server und schickt Befehle (statt rein lokal)
- [ ] Autoritative Befehlsverarbeitung (Bauen/Abreißen/Schalten serverseitig validiert)
- [ ] Bau-Zustand serverseitig „bei Abruf" fortrechnen (ereignisbasiert, DESIGN §5.4)

## Phase 2 — Heimatsystem, voll simuliert

- [ ] Volle Kepler-Simulation mit Bevy-Renderung (nativ + Wasm)
- [ ] Fliegende Flotten (Manöver-Modell), langsame Taktik-Ansicht
- [ ] Schiffsbau (Werft auf Station), erste Schiffsklassen über Rollen
- [ ] Gefecht gegen NPC-/Planetenverteidigung (Positions-Simulation, Geschoss-Flugzeit)
- [ ] Volle System-Ökonomie: Logistik als Kapazität, Distanz-/Geometrie-Gewichtung
- [ ] Konjunktionen + Relais (Lieferketten „um die Sonne herum")
- [ ] Lagrange-Punkte (L1–L5) mit Charakteren; rentabler Radius
- [ ] Treibstoffschiene (Gas → Treibstoff); Komposit als Gate-Gut im Einsatz
- [ ] Vierter Rohstoff Eis/Wasser (Treibstoff + Kühlung) — falls Phase 1 zu dünn

## Phase 3 — Galaxie + Multiplayer

- [ ] Hex-Strategiekarte; freie Kachelbewegung (Sublicht), Zone-of-Control
- [ ] Massebeschleuniger / Sprungnetz (feste, zerstörbare Verbindungen)
- [ ] Geteilter Server, persistentes Universum, Alters-Gradient, Spawn am Rand
- [ ] Verlust/Neustart; Prestige als Identität, nicht Macht
- [ ] Spezialisierte Regionen → Spielerhandel; Nebenprodukte

## Phase 4 — Lebende Galaxie + Endgame

- [ ] Dynamische Wurmlöcher; Blockaden
- [ ] Kosmische Ereignisse (Anti-Stagnation ohne Wipe)
- [ ] Allianzen & Diplomatie
- [ ] Dyson-Schwarm (stellare Energie-Krönung; Typ-I→II-Schwelle)
- [ ] Stellar-Triebwerk (System bewegen) — Verhältnis zu Dyson noch offen (`Oekonomie-…` §14)
- [ ] Kardaschow-Endgame & Rangliste

## Querschnitt / Technik

- [ ] Determinismus-Tests des Kerns über lange Zeiträume
- [ ] `cargo clippy` und `fmt` in den Workflow aufnehmen
- [ ] CI (build + test) einrichten
- [ ] `trunk build --release` (wasm-opt) — Bundle-Größe drücken, wenn relevant
- [ ] systemd-Autostart für Postgres in WSL (optional, Komfort)
- [ ] Lizenz endgültig festlegen (`LICENSE` ist MIT, README erwähnt „noch offen")
- [ ] Anti-Cheat / Multi-Accounting (erst mit Multiplayer real)

## Offene Design-Entscheidungen (aus den Docs)

- [ ] Dyson-Schwarm vs. Stellar-Triebwerk: gemeinsame Achse oder rivalisierende Pfade?
- [ ] Konkretes Schiffsklassen-Roster und Werte
- [ ] Raten der Karten-Dynamik (Universums-Wachstum, Wurmloch-Frequenz, Triebwerks-Tempo)
- [ ] Was einen Neustart überlebt; Ausgestaltung von Prestige
- [ ] Galaxie-Topologie (frei Hex-zu-Hex vs. Sprungnetz-Chokepoints)
- [ ] Zeitskalen-Regler (Manöver-Tempo vs. Planetenbewegung, Tick-Rate)
- [ ] 3D-Systemansicht als spätere Option
