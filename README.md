# [Arbeitstitel]
 
> Ein browserbasiertes Weltraum-Strategiespiel: führe eine Zivilisation von einem einzelnen Planeten bis zum galaktischen Imperium.
 
**Status:** Phase 0 – Fundament · sehr frühe Entwicklung · noch nicht spielbar
 
Inspiriert von *EVE Online* (persistente, von Spielern geformte Welt), *Die Stämme* (asynchroner Aufbau und Eroberung) und langsamer, taktischer Raumschlacht. Über allem steht die **Kardaschow-Skala** als Fortschrittsachse — du steigst auf, indem du immer mehr Energie beherrschst.
 
## Überblick
 
Das Spiel entfaltet sich über drei verschachtelte Ebenen, getragen von *einer* persistenten, langsam tickenden Server-Simulation, die auch weiterläuft, während du offline bist:
 
- **Planet** — Raster-Aufbau mit Adjazenz-Boni und Geländeprofilen. Das ökonomische Herz.
- **Sonnensystem** — langsame Orbital-Taktik auf echten Kepler-Bahnen; Logistik und Aufklärung tragen schon vor dem Kampf eigenes Gewicht.
- **Galaxie** — Hexkachel-Strategiekarte, lebendig und ständig in Bewegung; Territorium, Allianzen, Handel.
Leitidee: **langsam und asynchron**. Befehle dauern Minuten bis Stunden, nicht Sekunden — das macht asynchrones Spiel möglich und umgeht die schwierigste Technik des Genres.
 
## Technologie
 
- **Sprache:** Rust (durchgängig, Client und Server)
- **Engine (Systemansicht):** Bevy (ECS, nativ + Wasm)
- **UI / Bau-Ebene:** egui / eframe
- **Backend:** Axum + WebSockets
- **Datenbank:** PostgreSQL
- **Ziel:** Browser (WebAssembly); Desktop-Binary nahezu geschenkt
- **Gemeinsame `core`-Crate:** Spielregeln, Kepler-Mathematik, Serialisierung — von Server *und* Client genutzt
## Projektstruktur
 
(geplant)
 
```
.
├── README.md
├── LICENSE
├── Cargo.toml          # Workspace-Manifest
├── docs/
│   ├── DESIGN.md                      # Kanonisches Design-Dokument
│   └── Oekonomie-und-System-Ebene.md  # Vertiefung: Ökonomie, Logistik, System-Ebene
└── crates/
    ├── core/           # Spielregeln, Kepler, geteilte Typen (Server + Client)
    ├── server/         # Axum, autoritative Simulation, Postgres
    └── client/         # Bevy + egui, Wasm-Ziel
```
 
## Bauen & Ausführen
 
Sobald das Workspace-Gerüst steht (Phase 0):
 
```bash
# Server + Tests
cargo build
cargo test
 
# Client im Browser (Beispiel mit trunk)
trunk serve
```
 
Bis dahin enthält das Repo nur die Design-Dokumentation.
 
## Dokumentation
 
Das Design ist die derzeitige Hauptsubstanz des Projekts:
 
- **[docs/DESIGN.md](docs/DESIGN.md)** — Vision, die drei Ebenen, Kernmechaniken, Architektur, Roadmap.
- **[docs/Oekonomie-und-System-Ebene.md](docs/Oekonomie-und-System-Ebene.md)** — vertieft Ressourcen, Produktionsketten, Logistik als räumliche Kapazität, Lagrange-Punkte und den rentablen Radius.
## Roadmap (Kurzfassung)
 
- **Phase 0 — Fundament:** Simulationsmodell, Workspace-Gerüst, Postgres-Grundlage.
- **Phase 1 — Vom Planeten in den Orbit:** Raster-Bau, Ressourcenmodell (3 + 2 + 1), zweistufige Aufklärung, schematische Systemansicht.
- **Phase 2 — Heimatsystem (voll simuliert):** Kepler-Simulation, fliegende Flotten, Gefecht, volle System-Ökonomie.
- **Phase 3 — Galaxie + Multiplayer:** Hex-Karte, geteilter Server, persistentes Universum, Handel.
- **Phase 4 — Lebende Galaxie + Endgame:** Wurmlöcher, kosmische Ereignisse, Stellar-Triebwerk, Dyson-Schwarm, Kardaschow-Endgame.
Vollständige Roadmap in [docs/DESIGN.md](docs/DESIGN.md#6-roadmap).
 
## Lizenz
 
Open Source. Die konkrete Lizenz wird noch festgelegt (siehe `LICENSE`).
 
## Mitwirken
 
Aktuell ein Solo-Projekt ohne festen Zeitplan. Issues und Diskussionen sind willkommen.
