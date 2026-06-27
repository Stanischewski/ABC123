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

Das Workspace-Gerüst (Phase 0) steht:

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
    ├── server/         # Axum, autoritative Simulation (Postgres folgt)
    └── client/         # egui/eframe-Bau-Ebene (Bevy-Systemansicht folgt Phase 2)
```

Die Crate `core` ist als `gamecore` eingebunden, damit ihr Name nicht Rusts
std-`core` verdeckt. Umgesetzt:

- **`core`** (Phase 0) — Kepler-Propagation auf festen Bahnen (planar,
  Newton-Raphson), Körper-Hierarchie (Mond ⊂ Planet ⊂ Stern), das
  Ressourcenmodell (3 + 2 + 1), Energiebudget mit Priorität, Logistik-Effizienz
  (`min(1, Angebot/Bedarf)`) und die Produktionsrate-Formel.
- **`core`** (Phase 1, Bau-Ebene) — Planeten-Raster mit Gelände-Typen, Gebäude
  (Förderer, Raffinerien, Solar/Fusion, Lager) mit Platzierungsregeln,
  **Adjazenz-Boni** aus der Nachbarschaft und eine **Produktionsauflösung**, die
  ein Lager über `dt` fortschreibt — energie- und input-gedrosselt, mit an den
  Bahnradius gekoppeltem Solarertrag (`1/r²`). Alles deterministisch und getestet.
- **`server`** — Axum + optionale Postgres-Persistenz: `GET /health`,
  `GET /system` (Zustand als JSON), `GET /ws` (WebSocket-Stream der Positionen).
- **`client`** — egui/eframe-Oberfläche mit zwei Ansichten: die **Bau-Ebene**
  (Planeten-Raster mit Gelände-Farben, Gebäude-Palette, Bauen/Abreißen per Klick,
  Adjazenz im Tooltip, Lager-/Energie-Anzeige) und die **schematische
  System-Ansicht** (Körper auf ihren Kepler-Bahnen als Marker, an die Sim-Zeit
  gekoppelt). Simulations-Schritte (+1 h / +1 Tag / Auto-Tick) bewegen beides.
  Die gesamte Logik liegt im geteilten Kern — der Client stellt nur dar. Läuft
  nativ *und* im Browser (WebAssembly) aus derselben Quelle.
 
## Bauen & Ausführen

Voraussetzung: eine aktuelle Rust-Toolchain (`rustup`, Edition 2021).

```bash
# Alles bauen und die Tests des Kerns laufen lassen
cargo build
cargo test

# Server starten (lauscht auf http://127.0.0.1:8080)
cargo run -p abc123-server
#   GET /health   → "ok"
#   GET /system   → Systemzustand als JSON
#   GET /ws       → WebSocket-Stream der Körperpositionen

# Client als natives Desktop-Fenster
cargo run -p abc123-client
```

### Im Browser (WebAssembly)

Der Browser ist das primäre Ziel (DESIGN.md §5.1). Derselbe egui-Client wird per
WebAssembly gebaut; der Axum-Server liefert ihn aus — er ist damit *auch* der
Webserver der UI.

Einmalig: das Wasm-Tooling.

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk        # oder ein vorgebautes Binary von github.com/trunk-rs/trunk
```

Variante A — über den Server (UI und API unter einem Ursprung):

```bash
# UI nach crates/client/dist/ bauen …
cd crates/client && trunk build && cd ../..
# … und vom Server ausliefern: http://127.0.0.1:8080 öffnen
cargo run -p abc123-server
```

Variante B — trunk-Dev-Server mit Hot-Reload (nur UI, ohne API):

```bash
cd crates/client && trunk serve   # http://127.0.0.1:8080 (trunk-eigener Port)
```

Das gebaute `dist/` ist gitignoriert; nach einem frischen Checkout also zuerst
`trunk build`. Das Wasm-Bundle wird mit korrektem `application/wasm` ausgeliefert.
Die Bevy-Systemansicht (volle Kepler-Simulation) kommt in Phase 2.

### Datenbank (PostgreSQL in WSL)

Persistenz ist **optional**: ohne `DATABASE_URL` läuft der Server im Speicher;
mit gesetzter URL verbindet er sich, wendet die Migrationen
([crates/server/migrations/](crates/server/migrations/)) an und lädt bzw. legt
den Welt-Zustand an. Postgres läuft in WSL (Ubuntu); Windows erreicht es über das
localhost-Forwarding von WSL2.

Einmalige Einrichtung (in WSL):

```bash
sudo apt-get install -y postgresql
sudo service postgresql start
sudo -u postgres psql -c "CREATE ROLE sta LOGIN CREATEDB PASSWORD 'DEIN_PASSWORT';"
sudo -u postgres createdb -O sta abc123
```

Dann `.env.example` nach `.env` kopieren und `DATABASE_URL` setzen (Vorlage zeigt
auf `postgres://USER:PASSWORD@localhost:5432/abc123`). `.env` ist gitignoriert.

Nach jedem WSL-Neustart muss der Dienst gestartet werden:

```bash
wsl sudo service postgresql start
```
 
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
