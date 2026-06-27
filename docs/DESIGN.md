# Spielkonzept — [Arbeitstitel]

*Arbeitstitel, frei wählbar. Lebendes Dokument — Erstentwurf, der mit der Entwicklung wächst.*

*Begleitdokument: `Oekonomie-und-System-Ebene.md` vertieft Ressourcen, Logistik und die System-Ebene (Abschnitte 3.2, 4.1, 4.4 sowie Roadmap-Phasen 1–2).*

---

## 1. Vision

Ein browserbasiertes Strategiespiel, in dem du eine Zivilisation von einem einzelnen Planeten bis zum galaktischen Imperium führst. Es verbindet drei Vorbilder: die persistente, von Spielern geformte Welt von *EVE Online*, die asynchrone Aufbau- und Eroberungsstrategie von *Die Stämme*, und eine langsame, taktische Raumschlacht aus der Sicht eines Flottenoffiziers. Über allem steht eine Fortschrittsachse: die **Kardaschow-Skala** — du steigst auf, indem du immer mehr Energie beherrschst, von planetar über stellar bis galaktisch.

Das Spiel entfaltet sich über drei ineinander verschachtelte Ebenen — du baust auf einem **Planeten**, kommandierst Flotten in einem **Sonnensystem** und ringst um Territorium in der **Galaxie** — alle getragen von *einer* persistenten, langsam tickenden Server-Simulation, die auch weiterläuft, während du offline bist.

## 2. Design-Säulen

Fünf Prinzipien, an denen sich alle Detailentscheidungen messen lassen:

1. **Ein persistentes, langsames Universum.** Eine einzige, nie zurückgesetzte Welt im Geist von EVE. Befehle und Bewegungen dauern Minuten bis Stunden, nicht Sekunden. Diese Langsamkeit ist kein Kompromiss, sondern das zentrale Entwurfswerkzeug: Sie macht asynchrones Spiel möglich und umgeht die schwierigste Technik des Genres.
2. **Drei verschachtelte Ebenen.** Bau ⊂ System ⊂ Galaxie. Jede hat ihr eigenes Spielgefühl, aber sie teilen sich eine Simulation und eine Ökonomie.
3. **Fortschritt = Aufbau-Reihenfolge.** Die Reise des Spielers (ein Planet → Raumfahrt → interstellare Reise) ist zugleich die Entwicklungs- und Freischalt-Reihenfolge.
4. **Energie als Rückgrat.** Energie ist die wichtigste Ressource, die laufende Betriebswährung und die Metrik der Rangliste (Kardaschow).
5. **Server-autoritativ und deterministisch.** Der Server ist die alleinige Quelle der Wahrheit; Clients stellen nur dar. Das hält das Spiel fair, fälschungssicher und offline-fähig.

## 3. Die drei Ebenen

Die Ebenen teilen sich Simulation und Ökonomie, aber jede hat ihre **eigene, andersartige Knappheit** — und genau das hält sie spielerisch getrennt, obwohl auf allen „gebaut" wird: auf dem **Planeten** ist Fläche knapp (hart, fix → Layout-Puzzle), im **System** ist es Logistik und Unterhalt (weich, geometrisch → Reichweite und Takt), in der **Galaxie** Souveränität und Reichweite (sozial + räumlich). **Energie** wirkt quer über alle als Multiplikator. Kernsatz: *Auf dem Planeten ist Platz; das Limit im System ist die Logistik.* (Vertiefung im Begleitdokument §2.)

### 3.1 Bau-Ebene — Planet, Mond, Station

Der Aufbau-Kern und das ökonomische Herz des Spiels. Auf einem Raster (Grid-Tileset) errichtest du Gebäude: Förderanlagen, Raffinerien, Kraftwerke, Lager, Werften und fortgeschrittene Strukturen. **Adjazenz-Boni** belohnen kluge Platzierung — z. B. +1 Produktion für eine Mine neben einem Lager. Das **Gelände** der Kacheln ist unterschiedlich (Gestein, Eis, Gasvorkommen …), sodass die Wahl, *was wohin* kommt, eine echte Entscheidung ist und der Planet nicht generisch wird.

Diese Ebene läuft serverseitig persistent weiter — Ressourcen sammeln sich an, Bauschlangen schreiten voran, auch wenn du nicht zusiehst. Hier verbringst du die meiste Verwaltungszeit. Die UI ist `egui`-getrieben.

### 3.2 System-Ebene — Orbital-Taktik (langsames RTS)

Sobald die Raumfahrt freigeschaltet ist, öffnet sich die Sonnensystem-Ansicht. Hier bewegen sich Sonne, Planeten und Monde auf **echten Kepler-Bahnen**, und du kommandierst Flotten aus der Sicht eines hohen Offiziers: „Flotte X, gehe in Orbit um Planet P bei Radius R und eröffne das Feuer auf die Verteidigung."

Es ist bewusst *langsam* — ein Manöver oder eine Planetenumrundung dauert Minuten. Kämpfe lösen sich über Reichweite und Sichtlinie auf, mit Geschossen, die echte Flugzeit haben. Weil die Simulation server-autoritativ und deterministisch ist, kannst du einer Schlacht live in weicher Zeitlupe zusehen — oder sie wird zu Ende gerechnet, während du weg bist, und du liest hinterher den Gefechtsbericht. Diese Ebene wird mit Bevy gerendert. Sie ist das taktische Herz.

Doch sie ist **mehr als eine Kampfplattform**: Schon bevor Schiffe fliegen, trägt sie eigenes ökonomisches Gewicht — Logistik, Geometrie und Aufklärung (siehe 4.1 und Begleitdokument §6–9). Während die Bau-Ebene *statischen* Raum hat (eine Kachel bleibt eine Kachel), ist hier der Raum *dynamisch*: Position ist relativ und wandert, weil alles kreist; das Puzzle ist Geometrie über Zeit. Deshalb wird eine **schematische** Form dieser Ansicht bereits in Phase 1 geboren (egui, statische Marker, *kein* Bevy, *keine* Schiffe) und dient als Heimatbasis und Dashboard; die volle Kepler-Simulation mit Flotten und Gefecht ist Phase 2.

### 3.3 Galaxie-Ebene — Hex-Strategiekarte (dynamischer Graph)

Mit interstellarer Reise öffnet sich die Galaxie: eine Hexkachel-Karte im Stil von *Die Stämme*, aber lebendig. Das Universum ist in der **Mitte alt** und nach **außen immer neuer**; neue Spieler beginnen an der frischen Außenkante, etablierte Mächte ringen im verkrusteten Kern. Hier gibt es Territorium, Allianzen, Handel und die großen strategischen Bewegungen. Die Karte selbst verändert sich (siehe 4.6). Technisch ist sie ein **dynamischer Graph** mit wandernden Knoten und Kanten, der überwiegend serverseitig lebt. Sie ist die strategische und soziale Meta-Ebene.

## 4. Kernmechaniken

### 4.1 Ressourcen & Ökonomie

**Leitgedanke: allokieren statt routen.** Die Wirtschaft soll Tiefe haben, aber **nicht** Factorio sein. Der Trick ist, zwei Arten von „schwer" zu trennen, die Factorio vermischt: *Kettentiefe* (viele Verarbeitungsstufen) und *räumliche Logistik* (Bänder verlegen, Durchsätze in Echtzeit balancieren). Wir nehmen eine **flache Kette mit dichter Verflechtung** und ersetzen das Band-Routing durch **Allokation** — du balancierst abstrakte Flüsse auf Lager-Ebene mit Prioritäten, verlegst aber nie ein Band. Das einzige räumliche Puzzle auf der Bau-Ebene ist die *Platzierung* (Adjazenz + Gelände). Formeln und Herleitung im Begleitdokument `Oekonomie-und-System-Ebene.md` (§1, §3, §5).

**Energie (das Rückgrat).** Aus Kollektoren und Reaktoren — und schon hier eine echte Portfolio-Entscheidung: **Solar** (ertragsstark im inneren System, gratis im Betrieb, aber flächen- und ortsgebunden — koppelt an den Bahnradius) gegen **Fusion** (konstant überall, frisst aber Gas, das sonst Treibstoff würde). Die meisten Produktionsgebäude *verbrauchen* Energie im Betrieb — dein Energiebudget ist eine eigene Entscheidungsschicht (was läuft, was nicht), und Energie ist die Metrik der Kardaschow-Rangliste. Vor allem aber ist Energie der **Multiplikator**, der die Grenzen aller anderen Ebenen verschiebt (siehe 4.7).

**Das Modell: 3 roh + 2 veredelt + 1 Gate-Gut.** Flach genug, um nie nach Factorio zu riechen, dicht genug für echte Tradeoffs schon auf einem einzelnen Planeten. Jeder Rohstoff hängt an *einem* Gelände und *einer* Rolle, damit Förder-Platzierung zählt und Planeten unterschiedliche Profile bekommen.

**Roh-Ressourcen:**
- **Metalle** — aus Gestein und Asteroiden. Der universelle Baustoff: geht in Legierungen, Elektronik *und* die meisten rohen Baukosten → der **geteilte Engpass**.
- **Silikate / Kristalle** — aus seltenerem Kristallgelände. Reiner Tech-Input (nur Elektronik); ihre Gelände-Knappheit deckelt die Tech-Decke und treibt zum zweiten Körper, später zum Handel.
- **Gase** (Wasserstoff, Helium-3) — Energie- und Treibstoff-Basis.

**Veredelte Ressourcen** (alle energiekostend):
- **Legierungen** ← Metalle — für Rümpfe, Strukturen, Verteidigung.
- **Elektronik** ← Silikate + Metalle — für Schiffssysteme und Hightech-Gebäude.

**Gate-Gut:**
- **Komposit** ← Legierungen + Elektronik — *ein* Schlüsselgut für Top-Schiffe, Megabauten und das Stellar-Triebwerk. Tier 2, kommt bewusst spät.

Der Baum ist **3 → 2 → 1**, keine Rekursion, keine Bänder. Die **Konvergenz bei Metalle** (Elektronik *und* Legierungen ziehen daran) ist die zentrale Frühspiel-Spannung: Jede Tonne ist eine Entscheidung, überinvestierst du in eine Schiene, hungert die andere. Die „Schwere" kommt aus vier Hebeln, keiner davon Logistik-Routing: **Metalle-Engpass, Energiebudget, Gelände-Knappheit → Planetenprofile, und Platzierung.**

**Spätere Schichten (Galaxie-/Multiplayer-Phase):** vierter Rohstoff (Eis/Wasser für äußere Welten, spaltet in Treibstoff + Kühlung), Nebenprodukte, Treibstoff für Schiffsbewegung — und vor allem **Spezialisierung und Spielerhandel**. Weil verschiedene Regionen (alter Kern vs. frischer Rand) unterschiedliche Roh-Profile haben, entstehen natürliche Handelsrouten, betrieben über Massebeschleuniger und Wurmlöcher. Die *tiefe* ökonomische Komplexität kommt also bewusst erst, wenn es Handelspartner und Transportwege gibt — vorher bleibt die Wirtschaft solo abstimmbar.

### 4.2 Bauen & Adjazenz-Boni

Raster-Bau auf Planeten, Monden und Orbitalstationen. Gebäude erhalten Boni abhängig von Nachbarschaft und Gelände. Das macht Platzierung zu einem Puzzle, gibt jeder Welt einen eigenen Charakter und liefert einen Grund, mehrere Körper (und später Systeme) mit unterschiedlichen Profilen zu besiedeln.

### 4.3 Schiffe, Flotten & Kampf

**Schiffsklassen über Rollen** — Langstrecken-Artillerie, Nahkampf-Brecher, abschirmende Punktverteidigung, Träger/Support — definiert über Reichweite, Tempo, Bewaffnung und Panzerung. Es gibt **keine** starre Schere-Stein-Papier-Tabelle; **Konter entstehen aus der Positions-Simulation**: Eine Glaskanone auf Distanz wird vernichtet, wenn Brecher die Lücke schließen; Raketenschiffe werden von Punktverteidigung gekontert, die Geschosse im Flug abfängt. Taktische Tiefe entsteht so aus Positionierung und Flottenmischung, nicht aus abstrakten Boni. (Falls sich emergente Konter im Test als zu subtil erweisen, können später gezielte Boni nachgelegt werden.)

Bewegung folgt dem **Manöver-Modell** (kein voll-newtonsches Delta-v-Management): Ein Schiff bekommt ein Ziel oder einen Orbit-Befehl, plottet einen Anflug auf den (sich bewegenden) Körper und parkt in einer Kreisbahn, die dessen Position mitführt. Sieht physikalisch echt aus, bleibt aber direkt kommandierbar.

### 4.4 Bewegung & Reise

Drei Technologie-Stufen, die eine geschichtete Bewegungs-Meta ergeben:

1. **Freie Kachelbewegung** (Sublicht) — Flotten ziehen frei von Hex zu Hex; können **Blockaden** errichten (Zone-of-Control). Langsam, aber berechenbar.
2. **Massebeschleuniger / Sprungnetz** — zwischen kontrollierten/befreundeten Systemen baust du **feste Verbindungen** für schnelle, direkte Sprünge. Schnell, aber ortsfest und ein zerstörbares Ziel: Kappt man die Linie, kappt man den Nachschub.
3. **Dynamische Wurmlöcher** — erscheinen und vergehen, eröffnen weite Handelswege und Kriegsoffensiven. Mächtig, aber unberechenbar — und **zweischneidig**: Auch der Feind kann *dein* Wurmloch nutzen.

**Logistik ≠ Flottenbewegung.** Davon getrennt steht die *Versorgungs*-Logistik. Roh-Material fließt **abstrakt und global** (kein Routing, keine Bänder), aber wie viel von einem fernen Vorposten den globalen Pool *erreicht*, hängt von einer eigenen **räumlichen Kapazitätsschicht** ab — wie Bandbreite: Depots und Relais *erzeugen* Reichweite und Durchsatz, Produzenten *verbrauchen* sie nach Distanz und Geometrie. Die Effizienz fällt weich (`min(1, Angebot/Bedarf)`), nicht auf eine Klippe. Weil alles kreist, *pulsiert* sie mit der synodischen Periode: Steht ein Körper hinter der Sonne (Konjunktion), bricht die Lieferung ein — es sei denn, ein **Relais** (etwa an einem Lagrange-Punkt) hält den Link „um die Sonne herum". Das gibt dem Aufklärungs-/Relais-Netz einen harten ökonomischen Job jenseits des Kampfes. Primär Phase 2+; Herleitung und Lagrange-Taxonomie im Begleitdokument §7–8.

### 4.5 Persistentes Universum

Eine einzige, dauerhafte Welt. **Alters-Gradient:** Mitte alt, Rand neu — neue Spieler spawnen an der frischen Kante, was das Verkrustungsproblem *räumlich* löst statt über Saison-Resets. **Verlust und Neustart:** Wer ausgelöscht wird, beginnt neu; man kann auch *freiwillig* neu starten (für Prestige). **Prestige ist Identität, nicht Macht:** Es gibt Kosmetik, Titel oder seitwärtige Optionen, aber keine rohe Stärke — sonst würde es genau die faire Außenkante zerstören. **Wachstum** des Universums folgt der Spielerzahl (eine neue Ringschicht entsteht, wenn die Grenze gefüllt ist), damit der Rand frisch *und* bevölkert ist.

### 4.6 Lebende Galaxie

Die Karte ist kein statisches Gitter, sondern verändert sich:

- **Kosmische Ereignisse** sortieren Regionen der Karte um — ein sanfter Anti-Stagnations-Reset ohne Fortschritts-Wipe. Wucht und Häufigkeit sind der bewusste Regler zwischen „persistentes Universum" und „periodischer Aufruhr".
- **Stellar-Triebwerk (Endgame-Krönung):** Ein gebautes Megaprojekt verschiebt das eigene System um **1 Kachel pro Woche** über die Galaxie. Bewusst als *letzte* Mechanik, weil es jede Karten-Dynamik-Regel auf die Probe stellt (Kollisionen, brechende Linien, Pfadfindung).
- **Dyson-Schwarm (stellare Energie-Krönung):** Die andere große Megastruktur — erntet die ganze Sonne, **ortsgebunden** an *ein* System, und macht es als Einheit rentabel betreibbar (die erlebte Typ-I→II-Schwelle, siehe 4.7). Wie Dyson und Stellar-Triebwerk zueinander stehen — *dieselbe* Achse (erst besitzen, dann bewegen) oder *rivalisierende* Endgame-Pfade (Verwurzelung vs. Mobilität) — ist **bewusst noch offen** (siehe 7 und Begleitdokument §14).

### 4.7 Fortschritt & Rangliste

Technologie schaltet die Ebenen frei (Raumfahrt → System; interstellare Reise → Galaxie) und vertieft jede Schicht. Die **Rangliste** misst beherrschte Energie entlang der **Kardaschow-Skala** — das übergreifende „Wofür spiele ich".

Wichtig: Die **Energiequellen *sind* die Kardaschow-Leiter** (planetar: Solar + Fusion → stellar: Dyson-Schwarm → galaktisch: verteilte Netze), und jede Stufe schiebt den **rentablen Radius** nach außen — du kannst weiter expandieren, härtere Geometrie überbrücken, tiefer veredeln. Energie ist damit der **Multiplikator**, der die Grenzen aller anderen Ebenen verschiebt — aber sie *kauft nicht alles*: Fläche bleibt hart, knappe Lagrange-Punkte bleiben knapp, fehlendes Material bleibt fehlend. So bleibt „mehr Energie = mehr rausholen" wahr, ohne „mehr Energie = gewinnt alles" zu werden — der Schutz der fairen Außenkante (4.5). Herleitung im Begleitdokument §9–11.

### 4.8 Soziales

Allianzen und Diplomatie als tragende Säule: geteilte Verteidigung, geteilte Karten-Intel, koordinierte Offensiven, interne Politik. Lebt auf der Galaxie-Ebene.

## 5. Technische Architektur

### 5.1 Plattform

**Browser** als primäres Ziel — wegen der Distribution (überall spielbar, keine Installation, asynchron), die für dieses Genre den Kern ausmacht. Da der Stack aus *einer* Codebasis sowohl ein Wasm-Web-Target als auch ein natives Desktop-Binary erzeugt, ist eine Desktop-Version später nahezu geschenkt.

### 5.2 Technologie-Stack

- **Sprache:** Rust, durchgängig (Client und Server).
- **Engine (System-Ansicht):** **Bevy** — ECS, läuft nativ und im Browser (Wasm via `wgpu`). Das datenorientierte ECS passt gut zu einer simulationslastigen Welt mit vielen Entities.
- **UI-Schicht:** **egui / eframe** — trägt die gesamte Verwaltungs- und Bau-Ebene (Menüs, Panels, Bauschlangen, Tech-Baum). Läuft ebenfalls im Browser.
- **Backend:** **Axum** (oder Actix-web) + WebSockets.
- **Datenbank:** **PostgreSQL** — der persistente Welt-Zustand.
- **Gemeinsame Core-Crate:** Spielregeln, Entity-Definitionen, Ressourcenformeln, Kepler-Mathematik, Serialisierung (`serde`) — genutzt von Server *und* Client.

### 5.3 Server-autoritatives Modell

Der Server simuliert autoritativ; Clients stellen nur dar und senden Befehle. Das umgeht das schlimmste RTS-Problem — **deterministischen Lockstep über verschiedene Plattformen** (Float-Determinismus ist berüchtigt) — denn nur die *server-interne* Simulation muss konsistent sein, was `f64` problemlos leistet. Es hält das Spiel fair und fälschungssicher und ermöglicht, dass Schlachten offline aufgelöst werden.

### 5.4 Simulationsmodell

Ein **Hybrid**, abgestimmt auf die Langsamkeit:

- **Himmelskörper:** analytisch über Kepler (geschlossene Funktion der Zeit) — kein Dauer-Tick nötig, driftet nie.
- **Aktive Simulation (Schiffe im Gefecht):** fester Zeitschritt, aber nur, wenn ein System „wach" ist (ein Kampf läuft oder ein Spieler sieht zu). Ruhende Systeme ticken nur ökonomisch.
- **Ökonomie & Bauschlangen:** ereignisbasiert / bei Bedarf berechnet (Zustand zum Abrufzeitpunkt), statt jede Sekunde zu ticken.

Das skaliert auf tausende Systeme, weil die teure Detailsimulation nur lokal und nur bei Bedarf läuft.

### 5.5 Orbitalmechanik

- **Himmelskörper auf festen Kepler-Bahnen** („on rails"): je Körper ein Satz Bahnelemente; Position = Lösung der Kepler-Gleichung (Newton-Raphson). Monde umkreisen Planeten, Planeten die Sonne. **Kein** n-Body — das wäre chaotisch und ohne spielerischen Gewinn.
- **2D (planar)** für die Systemansicht — konsistent mit Hex-Galaxie und Raster-Bau und deutlich einfacher. 3D bleibt eine spätere Option.
- **Schiffe** über das Manöver-Modell (siehe 4.3).
- **Geschenkte Mehrskaligkeit:** Niedrige Planeten-Orbits sind physikalisch schnell (Minuten), heliozentrische Bahnen langsam (Tage/Wochen). Aus *demselben* Modell fallen flotte taktische Orbits und langsame strategische Planetenwanderung — und die Wanderung erzeugt echte Transferfenster zwischen Planeten.

### 5.6 Datenfluss & gemeinsame Core-Crate

Die Kepler-Propagation lebt in der Core-Crate und läuft identisch auf Server und Client. Der Client berechnet Körperpositionen **selbst** aus den Bahnelementen — der Server streamt also keine Planetenpositionen, nur Schiffszustände und Ereignisse. Bei den langsamen Geschwindigkeiten genügen Updates alle paar Sekunden; der Client interpoliert weich dazwischen. Eine **Flotte ist *ein* Objekt**, das zwischen den Ebenen wandert: Werft im Raster → Schiff im System → Flotte auf der Galaxie-Hex → beim Ankommen zurück ins System.

## 6. Roadmap

Leitprinzip: **Jede Phase endet in etwas Lauffähigem und Teilbarem** (vertikale Schnitte). Da Einzelspieler-Server und späterer Multiplayer-Server *derselbe* autoritative Kern sind, ist Multiplayer eine **additive** Phase, kein Rewrite.

- **Phase 0 — Fundament.** Simulationsmodell festlegen und umsetzen (der Hybrid aus 5.4). Workspace-Gerüst: Core-Crate, Server, Client-Skelett, Postgres-Grundlage. Noch kein Spiel, aber das Skelett läuft.
- **Phase 1 — Vom Planeten in den Orbit (Bau-Ebene + schematisches System).** Ein Planet, Raster-Bau, das Ressourcenmodell (3 + 2 + 1), Adjazenz-Boni, Bauschlangen, serverseitig persistent — und dann der **Aufstieg ins All**: Satelliten für Sicht und Forschung, **zweistufige Aufklärung** (erst den eigenen Planeten kartieren → Profil und Lücke; dann Teleskope für das System-Skelett; dann Sonden, die Inhalte aufdecken), erste Stationen, Mond als zweiter Körper. Leitfiguren: ein **Aufstiegs-Gradient** (immer eine sichtbare nächste Sprosse; jede verlangt eine kleine *Umkonfiguration* der Wirtschaft, kein bloßer Timer wie bei *Die Stämme*) und **Breite vor Tiefe** (erst das System dünn bespannen, dann Körper ausbauen). Die System-Ansicht wird hier schon geboren — aber **schematisch** (egui, statische Marker, *kein* Bevy, *keine* Schiffe) und dient als Heimatbasis/Dashboard. Einzelspieler. Das erste *teilbare* Artefakt endet nicht bei „ein Planet läuft", sondern bei „**Orbit erreicht, System gescannt, Expansion begonnen**". Details: Begleitdokument §12–13.
- **Phase 2 — Heimatsystem, voll simuliert (System-Ebene).** Der Übergang aus Phase 1 ist **fließend**. Jetzt kommt die volle Kepler-Simulation: Körper auf echten Bahnen, fliegende Flotten, langsame Taktik-Ansicht, Schiffsbau, Gefecht gegen NPC-/Planetenverteidigung, erste Schiffsklassen — und die **volle System-Ökonomie**: geometrie-gekoppelte Logistik, Konjunktionen, Relais, Lagrange-Punkte, rentabler Radius, Treibstoff/Komposit. Bevy kommt hinzu. Hier zeigt sich, ob sich das Manöver *gut anfühlt*.
- **Phase 3 — Galaxie + Multiplayer (Galaxie-Ebene).** Hex-Karte, Flottenbewegung, erste Reise-Stufe — und der additive Schritt zum geteilten Server: mehrere Spieler, persistentes Universum, Alters-Gradient, Spawn am Rand, Verlust/Neustart, spezialisierte Rohstoffe + Handel. Teilbar in 3a (Karte + Bewegung) und 3b (Multiplayer + PvP + Handel).
- **Phase 4 — Lebende Galaxie + Endgame.** Beschleuniger-Netz, dynamische Wurmlöcher, Blockaden, kosmische Ereignisse, Stellar-Triebwerk, Allianzen/Diplomatie, Kardaschow-Endgame. Dauerbetrieb und Feintuning.

## 7. Bewusst aufgeschoben & offene Fragen

Reiner Inhalt bzw. Tuning — wird *während* der Entwicklung entschieden, nicht davor:

- Ressourcen-**Struktur entschieden** (3 roh + 2 veredelt + 1 Gate-Gut „Komposit"; Metalle als geteilter Engpass; Energie als Solar-vs-Fusion-Portfolio). Offen bleibt die *Balance* (Förderraten, Flächen, Energiekosten) und der *Zeitpunkt* späterer Stufen (Eis/Wasser, Nebenprodukte, Treibstoff). Tech-Baum im Detail offen.
- **Dyson-Schwarm vs. Stellar-Triebwerk:** dieselbe Achse (erst das System besitzen, dann bewegen) oder rivalisierende Endgame-Pfade (maximale Verwurzelung vs. Mobilität/Reichweite)? Entscheidet, ob das Endgame ein einzelner Gipfel oder eine Weggabelung ist — und ob ein Spieler beide Kronen tragen kann (siehe Begleitdokument §14).
- Konkretes Schiffsklassen-Roster und ihre Werte.
- Raten der Karten-Dynamik: Universums-Wachstum, Wurmloch-Frequenz, Tempo des Stellar-Triebwerks, Wucht/Häufigkeit kosmischer Ereignisse.
- Was genau einen Neustart überlebt; Ausgestaltung von Prestige.
- Galaxie-Topologie im Detail (frei Hex-zu-Hex vs. Sprungnetz-Chokepoints; Spawn-Logik).
- Allianz-Mechaniken im Detail.
- Zeitskalen-Regler: Manöver-Tempo der Schiffe relativ zur Planetenbewegung; Tick-Rate.
- Monetarisierung (Open Source — vorerst keine) und Anti-Cheat / Multi-Accounting (real, aber später).
- 2D jetzt; 3D-Systemansicht als spätere Option.

## 8. Designprinzipien für Balance

- **Langsamkeit ist Freund.** Sie ermöglicht asynchrones Spiel und erspart EVEs härteste Technik (Echtzeit-Massenschlacht-Synchronisation, Time-Dilation).
- **Energie bleibt König** — als Betriebswährung und Kardaschow-Metrik durchgehend.
- **Faire Außenkante schützen.** Logarithmische Machtkurven, Anfängerschutz und Comeback-Mechaniken, damit Spätstarter gegen kompoundierende Produktion eine Chance haben. Prestige niemals als rohe Macht.
- **Inkrementell abstimmbar.** Ökonomie und Kampf wachsen in Komplexität mit den Phasen; tiefe Verflechtung (viele Ketten, Spielerhandel) erst, wenn die Partner und Wege existieren.
- **Manche Fragen beantwortet nur der Prototyp** — Spielgefühl der Manöver, richtige Tick-Rate, ob die Ökonomie Spaß macht. Die frühen Phasen sind teils genau solche Experimente.
