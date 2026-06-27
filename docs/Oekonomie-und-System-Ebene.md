# Ökonomie, Logistik & System-Ebene — Design-Vertiefung

*Begleitdokument zu `DESIGN.md`. Hält die Entscheidungen und Erkenntnisse fest, die in den Sitzungen zu Ressourcen, Produktionsketten, System-Logistik und Expansion gefallen sind. Vertieft vor allem die Abschnitte 3.2, 4.1, 4.2 und 4.4 des Hauptdokuments. Lebendes Dokument — Entscheidungen sind als getroffen markiert, Offenes ist am Ende gesammelt.*

---

## 1. Leitgedanke: Allokieren statt Routen

Die zentrale Frage war: schwer genug, dass der Aufbau Tiefe hat, aber **nicht** Factorio. Dafür müssen zwei Arten von „schwer" sauber getrennt werden — Factorio vermischt beide.

- **Kettentiefe** — wie viele Verarbeitungsstufen zwischen Rohstoff und Endprodukt liegen und wie stark sie sich verschachteln. Factorio treibt das weit.
- **Räumliche Logistik** — Bänder verlegen, Durchsätze balancieren, Spaghetti entwirren, in Echtzeit. Das ist Factorios eigentlicher Kern und passt **überhaupt nicht** zu einem asynchronen, langsam tickenden Spiel.
- **Verflechtungsdichte** — wie viele *echte Entscheidungen* dir wenige Rohstoffe abringen. *Die Stämme* hat hier null: drei Rohstoffe, keine Transformation, drei Regler. Das ist „zu leicht".

**Entscheidung:** Der Sweet Spot ist **flache Kette + dichte Verflechtung + Allokation statt Logistik**. Du balancierst abstrakte Flüsse auf Lager-Ebene mit Prioritäten — du verlegst **nie** ein Band. Das einzige räumliche Puzzle auf der Bau-Ebene ist die *Platzierung* (Adjazenz + Gelände), nicht der Transport. So wird es „nicht leicht", ohne nach Factorio zu riechen.

Dieser Leitsatz trägt das ganze Dokument: Materialfluss bleibt abstrakt und global; alles Räumliche wird zu *Kapazität* (Logistik) und *Geometrie* (Orbits), nie zu Mikro-Routing.

---

## 2. Drei Formen von Knappheit

Aus den Sitzungen kristallisierte sich heraus, dass es im Spiel **drei verschiedene Formen von Ressource** gibt, jede mit anderer Geometrie. Das ist das mentale Gerüst, an dem alles Weitere hängt:

```
Materialien   →  GLOBALER POOL     eine Menge je Stoff, fungibel, ortlos
                                   — wird NIE geroutet
Energie       →  FLACHES BUDGET    Angebot vs. Verbrauch, ortlos, mit Priorität
Logistik      →  RÄUMLICHES NETZ   Angebot vs. Bedarf, distanz-/geometrie-gewichtet
                                   — „das Energiebudget, nur zählt Distanz"
```

Der Lerntrick dahinter: **Logistik ist mechanisch dasselbe wie das Energiebudget** — Angebot gegen Bedarf, bei Knappheit drosselt es nach Priorität. Der einzige Unterschied ist die Gewichtung: Energiebedarf ist flach (ein Gebäude verbraucht, egal wo), Logistikbedarf ist **räumlich** (ein ferner Vorposten verbraucht mehr). Wer das Energiebudget verstanden hat, versteht Logistik in einem Satz.

Daraus folgt die zweite, übergreifende Erkenntnis: **Jede Ebene hat ihre eigene, andersartige Knappheit** — und genau das hält sie spielerisch getrennt, obwohl auf allen „gebaut" wird.

```
EBENE          KNAPPE RESSOURCE      ENGPASS-NATUR        DAS PUZZLE
Planet         Fläche (Kacheln)      hart, fix            Layout: was wohin
System         Logistik + Unterhalt  weich, geometrisch   Reichweite, Takt
Galaxie        Souveränität/Reichw.  sozial + räumlich    wie weit, mit wem
Übergreifend   Energie               Multiplikator        wie viel Maschine
```

**Kernsatz:** *Auf dem Planeten ist Platz; das Limit im System ist die Logistik und der Unterhalt.* Der Planet ist **innen** begrenzt (endliches Raster, Adjazenz erzwingt Tetris). Im System ist Fläche kein Thema mehr — die Frage wechselt von „habe ich Platz?" zu „**trägt sich das?**". Dieser Wechsel der Frage ist der Grund, warum sich System-Expansion anders *anfühlt* als Planeten-Bau.

---

## 3. Ressourcenmodell

**Entscheidung:** Ein gestuftes Modell aus **3 roh + 2 veredelt + 1 Gate-Gut** — flach genug, um nie nach Factorio zu riechen, dicht genug, dass schon ein einzelner Planet echte Tradeoffs erzwingt. Jeder Rohstoff ist an *ein* Gelände und *eine* Rolle gebunden, damit Förder-Platzierung zählt und Planeten unterschiedliche Profile bekommen.

**Roh-Ressourcen:**
- **Metalle** — aus Gestein und Asteroiden. Der universelle Baustoff. Geht in Legierungen, Elektronik *und* die meisten rohen Gebäudekosten → der **geteilte Engpass** (siehe unten).
- **Silikate / Kristalle** — aus seltenerem Kristallgelände. Reiner Tech-Input, nur für Elektronik. Ihre Gelände-Knappheit **deckelt die Tech-Decke** und treibt zum zweiten Körper, später zum Handel.
- **Gase** (Wasserstoff, Helium-3) — aus Atmosphäre und Gasvorkommen. Energie- und Treibstoff-Basis.

**Veredelte Ressourcen** (alle energiekostend):
- **Legierungen** ← Metalle — für Rümpfe, Strukturen, Verteidigung.
- **Elektronik** ← Silikate + Metalle — für Schiffssysteme und Hightech-Gebäude.

**Gate-Gut:**
- **Komposit** ← Legierungen + Elektronik — *ein* Schlüsselgut für Top-Schiffe, Megabauten und das Stellar-Triebwerk. Tier 2, kommt erst spät.

```
Metalle ──┬─────────────────→ Legierungen ──┐
          │      (+ Energie)                 │
          │                                  ├──→ Komposit
          │                    ┌─────────────┘     (Tier 2: Top-Schiffe,
Silikate ─┴─→ Elektronik ──────┘   (+ viel        Megabauten, Stellar-Triebwerk)
   (+ Metalle + Energie)            Energie)

Gase ──→ Energie (Fusion)
Gase ──→ Treibstoff (spätere Phase)
```

Der ganze Baum ist **3 → 2 → 1**: keine Rekursion, keine Bänder. Wichtig ist die **Konvergenz** bei Metalle: Weil Elektronik *auch* Metalle braucht, konkurriert sie mit Legierungen um denselben Rohstoff.

### Woraus die „Schwere" kommt — vier Hebel, keiner davon Logistik

1. **Metalle-Engpass.** Metalle gehen in beide veredelten Güter *und* in rohe Baukosten. Jede Tonne ist eine Entscheidung; überinvestierst du in eine Schiene, hungert die andere.
2. **Energiebudget.** Jede Raffinerie-Stufe kostet Betriebsenergie. Mehr Produktion heißt mehr Kraftwerke, die Fläche fressen (Solar) oder Gas verbrennen (Fusion). Der weiche Logistik-Druck — aber als *eine Budgetzahl mit Priorität*, nicht als physische Verlegung.
3. **Gelände-Knappheit → Profile.** Kein Planet hat alles. Eine gas-reiche, silikat-arme Welt muss spezialisieren oder einen zweiten Körper besiedeln — der Motor für die spätere Handelsphase.
4. **Platzierung.** Endliche, gelände-typisierte Kacheln plus Adjazenzboni machen Layout zum Puzzle. Das ist das „Spaghetti", aber als Stadtbau-Puzzle, nicht als Bandführung.

### Spätere Tiefe — bewusst dosiert

- **Vierter Rohstoff: Eis / Wasser** auf kalten/äußeren Welten → spaltet in Wasserstoff (Treibstoffschiene) und Sauerstoff/Wasser (Lebenserhaltung, *Kühlung* für energiestarke Endgame-Bauten). Gibt äußeren Welten Daseinsberechtigung. **Erst nachlegen, wenn Phase 1 sich zu dünn anfühlt.**
- **Nebenprodukte** — z. B. He-3 (Premium) *und* H (Allerwelts) aus derselben Gas-Raffination, oder Schlacke beim Schmelzen. Erzeugt „du machst B, ob du willst oder nicht"-Verflechtung. Stark für die Handelsphase, vorher eher Buchhaltung.
- **Tier-2 und Treibstoff** — Komposit und Schiffstreibstoff erst, wenn Megabauten bzw. echte Flottenbewegung existieren (Phase 2+). In Phase 1 reicht 3 → 2.

---

## 4. Energie — Portfolio und Multiplikator

Energie ist **nicht** eine flache Zahl, sondern auf zwei Ebenen ein Designwerkzeug.

### Als Portfolio-Entscheidung (eine der reichsten frühen Tradeoffs)

- **Solarkollektoren** — Ertrag abhängig vom Bahnradius (innen stark, außen schwach → koppelt direkt an die Kepler-Ebene; wenn das Stellar-Triebwerk das System verschiebt, ändert sich die Solarausbeute). Gratis im Betrieb, aber kostet Gitterfläche und ist ortsgebunden.
- **Fusionsreaktoren** — konstant, egal wo, aber fressen Gas, das sonst für Treibstoff gespart würde.

Solar = gratis, aber standort-/platzabhängig. Fusion = verlässlich, aber verbraucht knappen Rohstoff.

### Als Multiplikator über alle Ebenen

**Erkenntnis:** Energie ist der Hebel, der die *Grenzen aller anderen Ebenen verschiebt*. Mehr Energie heißt nicht stumpf „mehr Zahl", sondern:
- **weiter** expandieren (Unterhalt fernerer Vorposten tragen),
- **härtere** Geometrie überbrücken (in der Konjunktion mehr Energie reindrücken, statt Effizienz zu verlieren),
- **tiefer** veredeln (die energiehungrige zweite Stufe und Komposit fahren).

Energie ist das **Lösungsmittel, das die Logistik-Wand aufweicht**. Das macht die Kardaschow-Achse zu echtem Gameplay statt zu einer Punkteanzeige (siehe Abschnitt 9).

---

## 5. Der Produktions-Mechanismus

Die Gameplay-Antwort auf „wie werden Rohstoffe weiterverarbeitet" — und sie passt exakt zum ereignisbasierten Sim-Modell aus `DESIGN.md` 5.4.

Jeder Produzent hat eine Rate:

```
Rate = Basis × Adjazenz × Energie-verfügbar × Input-verfügbar
```

Lager **integrieren über die Zeit** und werden **beim Abruf** berechnet (kein Sekunden-Tick). Läuft ein Input oder Energie leer, **drosselt** der Produzent nach einer **Priorität, die du setzt**.

Das *ist* das Balancing-Gameplay: Verhältnisse so einstellen, dass nichts verhungert; unter Knappheit priorisieren; Engpässe asynchron beim nächsten Login flicken. **Du allokierst, du routest nicht.**

---

## 6. Statischer vs. dynamischer Raum

Die gesuchte Identität, die die System-Ebene von der Bau-Ebene trennt:

- **Unten — statischer Raum.** Eine Kachel ist eine Kachel, Adjazenz ist für immer, das Puzzle ist einmalige Platzierung.
- **Oben — dynamischer Raum.** Position ist relativ und *wandert*, weil alles kreist. Das Puzzle ist **Geometrie über Zeit**: Ausrichtung, Sichtlinie, Transferfenster.

**Kernpunkt:** Es gibt keine *dauerhaft* richtige Seite des Systems. Die Distanz zwischen Asteroiden-Vorposten und Heimatknoten ist kein fester Wert, sondern ein **Takt** — mal stehen beide auf derselben Sonnenseite (kurze Wege), mal trennt sie die Sonne (lange Wege, im Extrem blockierte Sichtlinie). Die Effizienz **pulsiert** mit der synodischen Periode.

Damit ist die System-Ebene **mehr als eine taktische Kampfplattform** — sie trägt eigenes ökonomisches Gewicht.

---

## 7. Verankerung & Lagrange-Punkte

Damit Platzierung im System eine *Entscheidung* ist und nicht bloß etwas, das einem passiert, gilt eine klare Regel: **Alles, was du baust, ankert an etwas**, und die Ankerwahl bestimmt die Form der Effizienzkurve über die Zeit.

- **Am Körper selbst** (Vorposten auf dem Asteroiden) → bei der Förderung immer effizient, nur der Heimweg pulsiert. Der sichere, lesbare Standardfall.
- **In freiem Orbit** (Depot mit eigener Bahn) → eigene Periode, schlägt mit *jedem* Ziel anders. Flexibel, aber schwer zu lesen.
- **An einem Lagrange-Punkt** → fest relativ zum Körperpaar, also **stabile Geometrie zu einem Planeten**. Das Premium-Slot.

### Die fünf Lagrange-Punkte, jeder mit eigener Funktion

```
   Planetenbahn um die Sonne (2D-Aufsicht):

                  ◇ L4  ·· 60° voraus, STABIL → Vorposten / Staging
                 ╱
   ◇ ········· ☉ ········· ◇ ─●─ ◇  ──► Bahnrichtung
   L3         Sonne        L1 │ L2
   (Gegenseite,            │  │  └ anti-Sonne, abgeschattet:
    Sichtlinie             │  │     Teleskop / Scan-Sat
    blockiert)             │  └ der Planet
                           └ sonnenwärts: Solar-Premium / Frühwarnung
                 ╲
                  ◇ L5  ·· 60° dahinter, STABIL → Vorposten / Staging
```

- **L1** (sonnenwärts) — sieht das innere System, Frühwarnung, Solar-Bestlage.
- **L2** (anti-sonnenwärts, abgeschattet) — *der* Platz für den Scan-/Forschungs-Satelliten. Wertet die „Scan-Sat vs. Forschungs-Sat"-Achse rückwirkend auf.
- **L3** (Gegenseite der Sonne) — die Sonne blockt die Sichtlinie zum eigenen Planeten, schwer zu nutzen → der **Stealth-/Hinterhalt-Slot** im Multiplayer.
- **L4 / L5** (60° voraus/dahinter) — stabil, teilen die Planetenbahn ohne zu driften (hier sammeln sich real die Trojaner) → **Premium-Immobilie** für Vorposten und Flotten-Staging.

### Station-Keeping-Gradient (drei Tiers)

- **Gewöhnlicher Orbit** — gratis & reichlich, aber ohne Sonder-Geometrie.
- **L1–L3** — verfügbar, kosten aber **laufend Treibstoff/Energie** zum Halten.
- **L4 / L5** — gratis zu halten, aber **extrem knapp** und damit umkämpft.

Von „billig-beliebig" bis „teuer-prestigeträchtig".

---

## 8. Logistik als räumliche Kapazität

**Entscheidung (die zentrale dieses Strangs):** *Keine* 10.000 Lager/Depots mit Einzelwerten, in die ständig geliefert werden muss — das wäre ein logistischer Alptraum. Stattdessen werden **zwei Dinge getrennt, die Factorio verschmilzt**: der *Materialfluss* (abstrakt und global) und die *räumliche Logistik* (eine eigene Kapazitätsschicht).

Logistik wird **nicht** als gelagertes Material behandelt (sonst hätte man wieder einen Stoff zum Herumschieben), sondern als **Kapazität wie Bandbreite**: Depots und Logistikzentren *erzeugen* Durchsatz und Reichweite, Produzenten *verbrauchen* ihn nach Distanz und Geometrie. **Ein Depot ist kein Warenlager, sondern ein Reichweiten-Sender.**

```
PRODUZENT         LOGISTIK-GATE                      GLOBALER POOL
(Mine/Vorposten)  (Kapazität vs. Bedarf)             (fungibel, ortlos)

 fördert  ─────►  Bedarf  = Output × Distanz × 1/Geometrie
 100% lokal       Angebot = Depots + Zentren + Relais in Reichweite
                       │
                  Effizienz = min(1, Angebot / Bedarf)   ← weicher Abfall
                       │
                       ▼
                  geliefert ──────────────────────►  + Silikat, + Metall …
```

**Wichtige Formulierung:** Es ist nicht „die Mine läuft langsamer", sondern „**die Mine liefert weniger**". Sie fördert lokal voll, aber nur ein Bruchteil erreicht den Pool, wenn die Logistik den Durchsatz nicht trägt. Das ist physikalisch ehrlich (die abstrakten Frachter schaffen es nicht alle nach Hause).

### Kopplung an Kepler: das Konjunktions-Problem

Die `1/Geometrie` im Bedarf **ist** der Konjunktions-Effekt. Steht der Silikat-Asteroid auf der Sonnenseite → Bedarf niedrig, voller Durchsatz. Wandert er hinter die Sonne (obere Konjunktion) → Bedarf schießt hoch oder die Linie reißt ganz, die Lieferung bricht ein — *es sei denn*, ein **Relais** an L4/L5 oder L3 hält den Link „um die Sonne herum" aufrecht.

Damit hat das Aufklärungs-/Relais-Netz seinen **harten ökonomischen Job**: Es ist nicht Deko, es hält die Lieferketten über die Konjunktionen am Leben. (Vorbild: die realen Konjunktions-Blackouts der Mars-Sonden.)

Weil die Körper analytisch *on-rails* sind, ist die Zukunft **gratis berechenbar**: Der Client kann eine Zeitleiste kommender Transferfenster und Konjunktionen rendern, ohne dass der Server etwas simuliert — z. B. *„Vorposten X geht in 4 Tagen in Konjunktion; ohne Relais fällt die Lieferung für 6 Tage auf 30 %."* Aus der Lesbarkeits-Schwäche der dynamischen Geometrie wird so ein **Verkaufspunkt**, und er fällt unmittelbar aus dem Kepler-on-rails-Entwurf.

### Zwei Verfeinerungen

- **Weicher Abfall, keine Klippe.** `min(1, Angebot/Bedarf)` fällt proportional, nicht auf null. Passt zu „nicht zu schwer": Man kommt nach dem Wochenende zurück und der Vorposten lief mit 60 %, keine Katastrophe. Vielleicht mit einem **Boden** (ein Produzent geht nie ganz auf null, lokaler Restbetrieb).
- **Optionaler Auto-Puffer.** Ein Depot darf einen kleinen Puffer halten, der den Output bei schwachem Link absorbiert und bei starkem wieder abgibt — der Vorposten *reitet* kurze Konjunktionen aus. Entscheidend: Du legst **nie** fest, *was* reingeht, du wählst nur die *Kapazität*. Es füllt und leert sich selbst → ein Mechanik-Element, keine Fummel-Aufgabe. Gehört vermutlich erst in Phase 2+.

### Kapital- vs. Betriebskosten

**Tendenz-Entscheidung:** das **geometrie-modulierte Hybrid**. Bauen kostet einmal (schafft Reichweite); Halten kostet laufend Energie + Treibstoff — und zwar **mehr, je härter die Geometrie gegen dich arbeitet**. Bei günstiger Stellung ist der Unterhalt billig; in der Konjunktion zahlst du *mehr*, um denselben Durchsatz durchzudrücken — oder du lässt die Effizienz fallen, um zu sparen. Du wählst selbst, wo auf der Kurve du sitzt.

Drei Effekte: (1) koppelt Logistik in bestehende Loops zurück (Energie → Produktion *und* Logistik; Treibstoff → Schiffe *und* Hauler); (2) macht ein weit verstreutes Imperium *laufend* teuer statt nur einmal; (3) begrenzt Überdehnung ganz ohne künstliche Obergrenze — ein EVE-iges „Souveränität kostet Unterhalt".

---

## 9. Rentabilität & der rentable Radius

**Leitsatz:** *Warless* (ziellos) expandieren lohnt sich nicht — man muss sich fragen, ob es **rentabel** ist, dort etwas hinzubauen. Jeder Vorposten hat eine Bilanz:

```
Ertrag    = Förderrate × Logistik-Effizienz(Geometrie, Takt)
Kosten    = Unterhalt(Energie + Treibstoff, geometrie-moduliert)
Rentabel  ⇔  Ertrag − Kosten  >  Opportunitätskosten
                                  (was dieselben Inputs woanders bringen)
```

Der entscheidende Teil ist die letzte Zeile — die **Opportunitätskosten**. Ein Vorposten muss sich nicht nur selbst tragen, er muss sich gegen die *beste Alternative* für dieselben Inputs (Energie, Fläche, Logistik) behaupten. Das macht „ziellos expandieren lohnt nicht" mathematisch wahr: Liefert der ferne Asteroid netto positiv, aber dieselbe Energie/Logistik brächte daheim *mehr*, expandierst du trotzdem nicht.

**Folge:** Expansion ist **bedarfsgetrieben statt giergetrieben**. Du dehnst dich nur dorthin aus, wo der ferne Körper etwas bietet, das daheim *fehlt* — das knappe Silikat, ein L-Punkt, ein Solar-Premiumplatz. Das **Profil-Modell** (jeder Körper hat andere Stärken) ist der Motor, der überhaupt Gründe zum Ausgreifen liefert. Ohne Profil-Unterschiede gäbe es nie einen Grund, die teure Logistik zu zahlen.

**Selbstregulierende Expansions-Grenze (ohne künstliches Limit):** Du wächst genau bis zu dem Radius, wo Grenzertrag = Grenz-Unterhalt. Näher dran lohnt fast alles; weiter draußen frisst der geometrie-modulierte Unterhalt den Ertrag. Wer überdehnt, blutet laufend Energie/Treibstoff für Vorposten, die kaum liefern. Und: Die Grenze ist **persönlich** — ein energiereicher Spieler kann sie nach außen schieben.

---

## 10. Die Kardaschow-Leiter als Energiequellen-Folge

Hier schließt sich der Kreis zur Fortschrittsstruktur: **Die Energiequellen *sind* die Kardaschow-Leiter**, und jede hat eine andere Geometrie und einen anderen rentablen Radius.

```
planetar     Solar (orbit-/flächenabhängig) + Fusion (frisst Gas)
                 → rentabler Radius: das innere System
                          │
stellar      Dyson-Schwarm / Stern-Ernte — Energie ≫, an EINE Sonne gebunden
                 → rentabler Radius: das ganze System, harte Geometrie bezahlbar
                          │
galaktisch   verteilte Netze, Transfer zwischen Systemen
                 → rentabler Radius: interstellar
```

**Der Dyson-Schwarm ist kein bloßes Prestige-Bauwerk**, sondern die Infrastruktur, die das *ganze* System rentabel erschließbar macht — plötzlich ist genug Energie da, um jeden Winkel über jede Konjunktion zu versorgen. Das ist eine **qualitative Schwelle**, kein quantitativer Sprung: Vorher rechnest du bei jedem fernen Vorposten spitz und lässt halbe Konjunktionen durchhängen; nachher betreibst du das System als **Einheit**. Genau der erlebte Übergang von Typ I („planetare Zivilisation, die mühsam ins All greift") zu Typ II („stellare Zivilisation, die ihr System besitzt").

So wird die Kardaschow-Achse zu echtem Gameplay statt zur Punkteanzeige: Du kletterst sie nicht für eine höhere Zahl, sondern weil jede Stufe deinen **rentablen Radius nach außen schiebt**.

---

## 11. Schutz der fairen Außenkante

**Spannung (ehrlich benannt):** Wenn Energie *all diese* Wände aufweicht, droht sie zur Universal-Währung zu werden, die alle anderen Knappheiten trivialisiert — dann ist ein energiereicher Spieler nicht *anders*, sondern in jeder Dimension überlegen, was die faire Außenkante (`DESIGN.md` 4.5) bedroht.

**Schutz:** Energie kauft eben **nicht** alles. Sie weicht Logistik und Veredelungs-Tiefe auf, aber:
- **Fläche bleibt hart** — auch mit Dyson hat dein Planet endliche Kacheln.
- **Knappe L-Punkte bleiben knapp** — Energie verschafft dir keinen zweiten L4.
- **Material bleibt unersetzlich** — ein Dyson-Schwarm gibt dir keine einzige Tonne Silikat, wenn dein System keins hat.

Die Grenze eines energiereichen Imperiums verschiebt sich nach außen, aber sie *verschwindet nicht* — sie stößt nur an die *nächste* Knappheit, die Energie nicht lösen kann. Das ist der Mechanismus, der „mehr Energie = mehr rausholen" wahr sein lässt, ohne dass es „mehr Energie = gewinnt alles" wird.

---

## 12. Der Aufstiegs-Bogen in Phase 1

Die frühe Schwierigkeit liegt **nicht in Knappheit, sondern in einem Aufstiegs-Gradienten**: Der Planetenstart ist sanft, aber es gibt immer eine sichtbare nächste Sprosse, die zieht. Man steckt nie an einer Wand fest — man klettert.

**Schutz gegen „zu leicht":** Jede Sprosse verlangt eine kleine **Umkonfiguration** der Wirtschaft, nicht bloß einen vollen Lagerbalken (das ist der Unterschied zu *Die Stämme*, wo „die nächste Stufe" nur ein Timer ist). Für den Satelliten musst du Metalle in die Elektronik-Schiene umlenken und neu balancieren; für den Mond brauchst du echten Überschuss *und* eine Spezialisierungs-Entscheidung.

### Breite vor Tiefe + zweistufige Aufklärung

**Entscheidung:** Erst legst du eine *dünne Schicht* über das ganze System (Sicht, dann Präsenz), *danach* gehst du in die Tiefe und baust einzelne Körper aus. Thematisch natürlicher (erst Territorium beanspruchen, dann entwickeln) — und macht das System zum Protagonisten der späten Phase 1.

Die Aufklärung ist **zweistufig** und beginnt beim eigenen Planeten:

```
                  Blick nach unten          Blick nach außen
Heimatplanet ─► 1. Sat: eigenen      ─► Teleskop: System-   ─► Sonden: Inhalte
(nur Startzone)    Planeten kartieren     Skelett (was/wo,      der Körper
                   → Profil, Lücke        grob)                 aufdecken
                        │                                          │
                        └──────────── BREITE ──────────┬──────────┘
                                                        ▼
                          System als Rahmen: Präsenz, Stationen, Relais
                                                        │
                                                     TIEFE
                                                        ▼
                          Körper kolonisieren & ausbauen (nach Profil)
                                                        ▼
                          volle Systemkontrolle ─► Phase 2 / 3
```

1. **Erster Satellit blickt nach unten** und kartiert den *eigenen* Planeten — dabei entdeckst du dein volles Profil und damit deine **Lücke** (z. B. metallreich, aber silikat-arm). Der erste „aha, ich muss raus"-Moment — verdient, nicht skriptet. Lehrt den Nebel-Mechanismus am sicheren, billigen, lokalen Fall.
2. **Teleskope blicken nach außen** — zeigen das **Skelett** (*dass* Körper existieren, wo sie auf ihren Bahnen stehen, grob welcher Typ). Gratis, du kannst planen und navigieren.
3. **Sonden decken Inhalte auf** — *was* auf einem Körper ist, kostet eine echte Sonde.

**„Zweistufig informiert"** (löst die alte Frage „Zugang gaten vs. Information gaten"): Teleskop gibt Position + grobe Art gratis (kein harter Riegel), aber das *ökonomisch Entscheidende* steckt hinter dem Sondieren. Du *darfst* blind irgendwohin fliegen, tust es dann aber ohne Ahnung vom Wert. Sanfter, aber nicht trivialer Boden.

### Die Leiter (jede Sprosse eine neue *Art* Entscheidung)

```
Planet          Wirtschaft aufbauen — sanfter Einstieg, Adjazenz & Energie lernen
   │  (Elektronik fließt)
   ▼
Satellit        Sicht & Forschung — erster Tritt ins All.
                Scan-Sat (Ressourcenfelder/Profile) vs. Forschungs-Sat —
                erste echte Spezialisierung bei knapper Startkapazität
   │  (Legierungen)
   ▼
Station         zweite Baufläche im Orbit — entkommt dem endlichen Raster.
                Solar ohne Geländezwang, später die Werft
   │  (echter Überschuss)
   ▼
Mond            zweiter Körper, anderes Profil — erste Mehr-Kolonie-Verwaltung,
                hier zahlt sich das Profil-Modell aus
   │
   ▼
Systemkontrolle Sprungbrett zu Kampf (Phase 2) und interstellar (Phase 3)
```

**Der Satellit arbeitet dreifach:** (1) gibt einen *Grund*, ins All zu wollen, der nicht „weil der Tech-Baum es sagt" ist; (2) ist der erste echte **Elektronik-Sink** und macht die Frühwirtschaft kohärenter; (3) sät die Aufklärungs-/Fog-of-War-Schicht, die in der Galaxie-Phase zum Hauptsystem wird — kein Wegwerf-Feature.

### Preis und Gegenmittel

Breite-zuerst **diffundiert den Fokus** — man hat womöglich einen halbfertigen Planeten, drei sondierte-aber-leere Körper und zwei Stationen, nichts fühlt sich „fertig" an. Für ein langsames Async-Spiel kann das gut sein (immer etwas zum Anstupsen) oder leicht zerfasern. **Gegenmittel:** Die **System-Ansicht wird zur Heimatbasis und zum Dashboard** — die Karte, die zeigt „das ist mein System, das ist entwickelt, das ist als Nächstes dran". Breite-zuerst erhöht damit die Wichtigkeit der System-Ansicht — was glücklich damit zusammenfällt, dass sie ohnehin schon in Phase 1 geboren wird.

---

## 13. Auswirkung auf die Phasen-Roadmap

Der Übergang von Phase 1 zu Phase 2 ist **sehr fließend**. Es macht sogar mehr Sinn, *zuerst das System zu entwickeln* (Sicht, Präsenz, Rahmen) und *dann* die Planeten/Monde im System auszubauen.

**Verschiebung gegenüber `DESIGN.md`:**
- **Phase 1 endet nicht** bei „ein Planet funktioniert", sondern bei „du hast den Orbit erreicht, scannst das System und breitest dich aus".
- Die **System-Ebene wird schon in Phase 1 geboren** — als **schematische** Ansicht (egui, statische Marker für Körper und Ressourcenfelder, **kein** Bevy, **keine** Schiffe). Die volle Kepler-Simulation mit fliegenden Flotten und Gefecht bleibt Phase 2.
- Der **System-Rahmen** (Sicht, Präsenz, Scouting, Stationen) ist spätes Phase 1; die **tiefe Mehr-Körper-Kolonisation** die hintere Hälfte des Übergangs.

**Phasen-Staffelung der Logistik/Geometrie:**
- **Phase 1 (schlank):** Produzent braucht ein Depot in Reichweite; Distanz zählt mild; Geometrie höchstens angedeutet. Scan-Sat zeigt Profile; Sonden decken Körper auf; Vorposten mit simpler Positions-Effizienz; ein, zwei L-Slots als „gute Plätze".
- **Phase 2+ (voll):** geometrie-gekoppelte, konjunktions-bewusste, relais-gestützte Logistik; volle L-Punkt-Taxonomie mit Charakteren; Auto-Puffer; Station-Keeping-Kosten; Treibstoffschiene; Tier-2/Komposit. Verzahnt sich dann ohnehin mit Kampf und Flotten-Staging.

---

## 14. Offene Fragen

**Bewusst offen gehalten — Dyson vs. Stellar-Triebwerk:** Wie stehen die zwei größten Megabauten zueinander? Der Dyson-Schwarm ist die *stellare* Stufe (Energie ≫, aber **ortsgebunden** an dein System); das Stellar-Triebwerk verschiebt das *ganze* System übers Hexfeld.
- **Variante A — dieselbe Achse:** erst Dyson (besitze dein System), dann Triebwerk als Krönung (bewege das System, das du besitzt). Ein Strang von planetar über stellar zu „mobile stellare Zivilisation".
- **Variante B — rivalisierende Endgame-Pfade:** Dyson = Wurzeln schlagen, maximale Ausbeute *an einem Ort*, gegen Triebwerk = Mobilität/Reichweite/Aggression, *Verzicht* auf maximale Verwurzelung. Die stellare Stufe *gabelt* sich → zwei Spielstile, zwei Typ-II-Identitäten.

Das entscheidet, ob das Endgame ein einzelner Gipfel oder eine Weggabelung ist — und ob ein Spieler beide Kronen tragen kann oder sich für eine Zivilisations-Philosophie entscheiden muss. *(Noch nicht entschieden.)*

**Tuning, das erst der Prototyp beantwortet:**
- Genaue Förderraten, Flächen, Energiekosten — und damit der konkrete Schwierigkeits-Boden in Phase 1.
- Stärke der Distanz-/Geometrie-Gewichtung in der Logistik-Formel; Höhe des Effizienz-Bodens.
- Kapital- vs. Betriebskosten-Mischung im Detail (Tendenz: geometrie-moduliertes Hybrid).
- Synodische Perioden / Bahnradien relativ zum Spieltempo — wie oft und wie hart Konjunktionen beißen.
- Wie hart der Scan greift (Tendenz: nur Information, nicht Zugang) und ob L-Slots in Phase 1 schon eine Rolle spielen.
- Anzahl/Balance der Ressourcen-Stufen und Zeitpunkt für Eis/Wasser, Nebenprodukte, Treibstoff, Komposit.
