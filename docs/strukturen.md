# Strukturen & Gebäude — Katalog nach Ebene

*Begleitdokument zu `DESIGN.md` und `Oekonomie-und-System-Ebene.md`. Listet Bau- und Orbitalstrukturen nach Ebene auf — jeweils mit Rolle, Verbrauch und Ausstoß, Platzierungs-/Geländebindung und Platz in der Aufstiegsleiter. Phasen-Angaben verweisen auf die Roadmap in `DESIGN.md` §6. Bewusst unvollständig: ein lebendes Dokument, das mit dem Design wächst — bekannte Lücken sind am Ende gesammelt.*

---

## Bau-Ebene

Raster-Bau auf einer von drei Flächen — **Planet**, **Mond** oder **Orbitalstation**. Auf jeder Kachel steht ein Gebäude; **Adjazenz-Boni** belohnen kluge Nachbarschaft, das **Gelände** der Kachel (Gestein, Kristall, Gasvorkommen, Eis …) entscheidet, was wohin darf. Die Fläche ist endlich und bleibt es — der harte Engpass, den keine Energie löst (`Oekonomie-…` §11) und der Motor des ganzen Phase-1-Bogens: Geht dir der Planet aus, greifst du zu Mond, Station und System.

Die drei Flächen unterscheiden sich. **Planet und Mond** tragen Gelände und damit Förderung; ein **Mond** ist ein zweiter Körper mit eigenem Profil. Eine **Station** hat *kein* Gelände — sie kann nicht fördern, eignet sich aber für geländefreie Bauten (Solar ohne Standortzwang, die Werft) und entkommt dem endlichen Planetenraster. Die Station selbst ist eine Orbitalstruktur (siehe System-Ebene).

Jedes Gebäude entsteht über **kontinuierlichen Ressourcenfluss** statt Einmalzahlung: Eine Baustelle ist ein Produzent, der Material über die Bauzeit verbraucht und Baufortschritt erzeugt — sie kriecht bei Mangel, statt zu blockieren. Upgrades sind dieselbe Mechanik auf bestehender Kachel.

### Zentrale

**Hauptgebäude** — Das Zentrum jeder Kolonie und der erste Baustein; pro Körper genau eines. Bringt von Beginn an etwas Lagerkapazität und eine minimale Grundförderung mit, sodass man sich nie vollständig festfahren (Softlock) kann — eine ausgehungerte Wirtschaft kriecht, sie stirbt nicht. Deckt zusätzlich nur die unmittelbare **Startzone** des Nebels auf, genug für eine Startwirtschaft; den Rest der Fläche erschließt erst der erste Satellit („Blick nach unten"). Phase 1.

### Förderung (gelände-gebunden)

**Metallmine** — Fördert **Metalle** aus Gestein, den universellen Baustoff. Metalle sind der geteilte Engpass — sie fließen in Legierungen, Elektronik *und* die meisten rohen Baukosten —, daher ist Förderkapazität hier die erste Stellschraube jeder Wirtschaft. Profitiert von Adjazenz zu einem Lager. Phase 1.

**Silikat-/Kristallförderer** — Gewinnt **Silikate/Kristalle** aus seltenerem Kristallgelände, einen reinen Tech-Input (fließt nur in Elektronik). Weil Kristallgelände knapp ist, deckelt dieser Förderer indirekt die Tech-Decke und ist oft der konkrete Grund, einen zweiten Körper zu besiedeln. Phase 1.

**Gassammler** — Sammelt **Gase** (Wasserstoff, Helium-3) aus Atmosphäre und Gasvorkommen — nicht „gemint", sondern gesammelt. In Phase 1 die Basis für Fusion; später zusätzlich die Wurzel der Treibstoffschiene. Phase 1.

### Verarbeitung

**Hütte** — Verarbeitet **Metalle → Legierungen** für Rümpfe, Strukturen und Verteidigung. Verbraucht Betriebsenergie. Speist die mittleren Aufstiegssprossen (Stationsmodule, Strukturen). Phase 1.

**Elektronikfabrik** — Verarbeitet **Silikate + Metalle → Elektronik** für Schiffssysteme und Hightech-Gebäude. Verbraucht Betriebsenergie. Der erste echte Elektronik-Sink — und damit das Gebäude, das den ersten Satelliten überhaupt ermöglicht. Konkurriert über die Metalle mit der Hütte: die zentrale Frühspiel-Spannung. Phase 1.

### Energie

**Solarkollektor** — Wandelt Sonnenlicht in Energie. Gratis im Betrieb, kostet aber Gitterfläche und ist ortsgebunden; der Ertrag hängt vom Bahnradius ab (innen stark, außen schwach) — auf dem einen Startplaneten noch flach, im System die echte Standortfrage. Auf Stationen ohne Geländezwang baubar. Phase 1.

**Fusionsreaktor** — Erzeugt konstant Energie, unabhängig von Standort und Bahnradius — frisst dafür Gas, das sonst zu Treibstoff würde. Das Ventil gegen die harte Flächenknappheit: mehr Strom ohne mehr Kacheln, auf Gas-Rechnung. Leicht gegated, weil erst sinnvoll, sobald eine Gasförderung läuft. Phase 1.

### Infrastruktur

**Lager** — Hebt die Lagerkapazität (den Deckel, wie viel du horten kannst) und dient als Adjazenz-Anker für Förderer. Ausdrücklich *kein* physisches Warenlager und *kein* Routing-Ziel: Der Materialpool bleibt global und ortlos, das Lager ist ein abstrakter Kapazitätsknoten, der eine Kachel belegt. Auf der Bau-Ebene übernimmt es zugleich die Rolle, die im System ein Depot spielt. Phase 1.

**Forschungseinrichtung** — **Beschleunigt** Forschung, statt sie überhaupt erst zu ermöglichen: Projekte laufen auch ohne sie (jedes ist eine Baustelle, die statt eines Gebäudes eine *Freischaltung* erzeugt — Modell und Katalog in `forschung.md`), aber jede Einrichtung senkt deren Projektzeit. Kostet eine Kachel (Flächendruck) und verbraucht im Betrieb Elektronik/Energie — der zweite Elektronik-Sink, der die Frühwirtschaft kohärenter macht. Damit ein Beschleuniger, kein Pflicht-Schalter.
Vorbedingung für den orbitalen Forschungs-Satelliten, der dem Kachel-Kostendruck entkommt; Forschung bleibt zugleich das Rückgrat der Gebäude-Verbesserung. Phase 1.

### Aufstieg ins All

**Startrampe** — Der physische Riegel zwischen Bau- und Orbit-Ebene; ohne sie kommt nichts ins All. Diskret upgradefähig: Jede Ausbaustufe hebt die **Startklasse** — klein für Teleskop/Scan-Satellit, mittel für Sonden, groß für Stationsmodule und später den Mond-Lander — und koppelt so direkt an die zweistufige Aufklärung und die Aufstiegsleiter. Das Musterbeispiel für begrenzte, bedeutsame Upgrades statt endloser Prozentboni. Phase 1.

**Werft** — Baut Schiffe. Sitzt auf der Orbitalstation, nicht auf dem Planeten, und gehört erst in Phase 2, wenn Flotten und Kampf existieren. Hier beginnt eine Flotte als Objekt, das später zwischen den Ebenen wandert (`DESIGN.md` §5.6). Nur auf Station; Phase 2.

---

## System-Ebene

Die System-Ebene trägt eigenes ökonomisches Gewicht, schon bevor Schiffe fliegen. Ihre Strukturen sind keine Raster-Gebäude, sondern an einen Körper, einen freien Orbit oder einen Lagrange-Punkt **verankerte** Anlagen — und die Ankerwahl bestimmt die Form ihrer Effizienz über die Zeit (`Oekonomie-…` §7). In Phase 1 erscheint diese Ebene zunächst nur **schematisch** (egui, statische Marker, kein Bevy); die volle Logistik mit Geometrie, Konjunktionen und Relais ist Phase 2+.

### Aufklärung (zweistufig)

**Satellit (Scan / Forschung)** — Der erste Tritt ins All, von der Startrampe geschossen, und die erste echte Spezialisierung bei knapper Startkapazität. Ein **Scan-Satellit** kartiert Ressourcenfelder und Profile — sein erster Job ist der „Blick nach unten" auf den *eigenen* Planeten, der die eigene **Lücke** offenbart (etwa: metallreich, aber silikat-arm). Ein **Forschungssatellit** beschleunigt stattdessen Forschung aus dem Orbit. Der Satellit sät zugleich die Aufklärungsschicht, die in der Galaxie-Phase zum Hauptsystem wird — kein Wegwerf-Feature. Phase 1.

**Teleskop** — Erste Aufklärungsstufe nach außen. Deckt das **Skelett** des Systems gratis und ohne harten Riegel auf: *dass* Körper existieren, wo sie auf ihren Bahnen stehen, grob welcher Typ. Erlaubt Planung und Navigation, verrät aber nicht, was wirtschaftlich auf einem Körper liegt. Idealplatz: der abgeschattete Lagrange-Punkt L2. Phase 1.

**Sonde** — Zweite Aufklärungsstufe. Deckt die **Inhalte** eines Körpers auf — was tatsächlich dort liegt — und kostet pro Körper eine echte Sonde. Hinter diesem Schritt steckt das ökonomisch Entscheidende; bis dahin fliegt man bestenfalls blind irgendwohin. Phase 1.

### Orbitale Fläche

**Station** — Eine im Orbit errichtete Struktur, die selbst zu einer neuen **Bau-Ebene-Fläche** wird: die zweite Baufläche, die dem endlichen Planetenraster entkommt. Hat kein Gelände, eignet sich also für geländefreie Bauten — Solar ohne Standortzwang, später die Werft. Ihre Platzierung (am Körper, in freiem Orbit oder an einem Lagrange-Punkt) ist selbst eine Entscheidung mit Folgen für Geometrie und Unterhalt. Späte Phase 1 als Sprosse; volle orbitale Funktion Phase 2.

### Logistik (Phase 2+)

**Depot** — Erzeugt logistische **Reichweite und Durchsatz** — kein Warenlager, sondern ein Reichweiten-Sender (Logistik wie Bandbreite). Produzenten in Reichweite verbrauchen seinen Durchsatz nach Distanz und Geometrie; reicht er nicht, *liefert* ein ferner Vorposten weniger (weicher Abfall, keine Klippe). Phase 2+.

**Logistikzentrum** — Größere Reichweiten- und Durchsatzquelle als ein Depot, projiziert Logistik über weite Teile des Systems. Das Rückgrat eines ausgedehnten Imperiums. Phase 2+.

**Relais** — Hält eine Lieferkette „um die Sonne herum" aufrecht, wenn ein Körper in **Konjunktion** hinter der Sonne verschwindet und die Linie sonst einbräche. An einem Lagrange-Punkt (L4/L5 oder L3) verankert, gibt es dem Aufklärungs-/Relais-Netz seinen harten ökonomischen Job jenseits des Kampfes. Phase 2+.

### Megastruktur

**Dyson-Schwarm** — Erntet die gesamte Sonne, ortsgebunden an *ein* System. Keine bloße Prestige-Struktur, sondern die Infrastruktur, die das ganze System als **Einheit** rentabel betreibbar macht — die erlebte Schwelle von Typ I zu Typ II. Verlangt das Gate-Gut Komposit. Seine Beziehung zum Stellar-Triebwerk ist bewusst offen (`Oekonomie-…` §14). Endgame (Phase 4).

---

## Galaxie-Ebene

Strukturen, die auf der Hexkachel-Karte wirken: feste Verbindungen und das größte Endgame-Bauwerk. Freie Kachelbewegung und dynamische Wurmlöcher sind hier *keine* Strukturen — Bewegung bzw. flüchtige Phänomene —, daher nicht gelistet.

### Verbindung

**Massebeschleuniger** — Eine feste, gebaute Verbindung zwischen kontrollierten oder befreundeten Systemen für schnelle, direkte Sprünge (zweite der drei Bewegungs-Stufen). Schnell, aber ortsfest und ein **zerstörbares** Ziel: Kappt man die Linie, kappt man den Nachschub. Phase 4 (Voll-Multiplayer).

### Megastruktur

**Stellar-Triebwerk** — Verschiebt das eigene System um etwa **eine Kachel pro Woche** über die Galaxie. Bewusst die *letzte* Mechanik, weil sie jede Karten-Dynamik-Regel auf die Probe stellt (Kollisionen, brechende Linien, Pfadfindung). Verlangt Komposit. Steht in offener Beziehung zum Dyson-Schwarm — gemeinsame Achse (erst besitzen, dann bewegen) oder rivalisierende Pfade (Verwurzelung vs. Mobilität). Endgame-Krönung (Phase 4).

---

## Noch zu ergänzen

Bekannte Lücken, die aus den Design-Dokumenten folgen, aber noch kein konkretes Gebäude haben:

- **Verteidigungsanlage (planetar)** — die „Verteidigung", auf die Legierungen einzahlen und gegen die Flotten in Phase 2 kämpfen; Bau-Ebene, Form offen.
- **Treibstoff-Raffinerie** — Gas → Schiffstreibstoff, sobald echte Flottenbewegung existiert (Phase 2+).
- **Komposit-Werk** — Legierungen + Elektronik → Komposit (Tier 2, Gate-Gut für Top-Schiffe und Megabauten).
- **Eisförderung & -verarbeitung** — vierter Rohstoff auf kalten/äußeren Welten, spaltet in Treibstoff und Kühlung/Lebenserhaltung; gibt äußeren Welten Daseinsberechtigung.
- **Mond-Lander / Kolonieanlage** — was eine Kolonie auf einem zweiten Körper überhaupt etabliert.
- **Orbital-Forschungsstation** — die größere Orbit-Stufe, auf die die planetare Forschungseinrichtung hinführt (falls eigenständig neben dem Forschungs-Satelliten).
- **Konstruktionskapazität** — optionales Gebäude, das den gleichzeitigen Baudurchsatz hebt; nur falls Start-Spamming je unübersichtlich wird.
