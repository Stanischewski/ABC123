# Forschungen — Phase 1

*Begleitdokument zu `strukturen.md` und `DESIGN.md`. Listet den Forschungs-Stummel
der Phase 1 auf — die Freischaltungen, die den Aufstiegsbogen vom nackten Planeten
bis zum ersten Blick in den Orbit tragen. Forschung ist hier **Freischaltung, kein
Prozentbonus** (Schutz der fairen Außenkante, `DESIGN.md` §4.5). Bewusst klein: die
Epochen II/III (Dyson, galaktische Netze) existieren im Baum nur als gesperrte Tore.
Lebendes Dokument; alle Mengen sind Platzhalter (→ tunen).*

---

## Modell

Forschung ist **dieselbe Mechanik wie Bauen**: Ein Forschungsprojekt ist eine
Baustelle, die statt eines Gebäudes eine *Freischaltung* erzeugt. Sie verbraucht
Material über die Zeit als kontinuierlichen Fluss und **kriecht bei Mangel, statt
zu blockieren**.

- **Bezahlt mit Material**, nicht mit Forschungspunkten — jedes Projekt zieht die
  unten gelisteten Stoffe.
- **Energie ist ein Fluss, kein Vorrat.** Ein laufendes Projekt zieht Energie aus
  dem Budget, solange es läuft — unter einer **Priorität, die du setzt**. Forschung
  konkurriert so mit Produktion um Strom: allokieren statt routen. Darum taucht
  Energie unten *nicht* als Kostenposten auf.
- Die **Forschungseinrichtung** ist kein Punkte-Produzent, sondern ein
  **Beschleuniger**: Sie senkt die Projektzeit und bleibt im Betrieb ein
  Elektronik-Sink (Elektronik + Energie). Optional, nicht Pflicht.

---

## Struktur

```
Legierungen ──► Hütte
     │
     └─► Triebwerktechnik ──► Raketen ──┐
                                        ├──► Orbit erreicht
Elektronik ──► Elektronikfabrik          │
     │                                   │
     └─► Komputertechnik ──► Satelliten ─┘
```

Zwei Linien, ein Zusammenlauf: **Antrieb** (Legierungen → Triebwerk → Raketen)
bringt etwas *hoch*, **Rechentechnik** (Elektronik → Komputer → Satelliten) gibt
ihm *Augen*. Für echten Blick in den Orbit brauchst du beide — früh treibst du nur
eine Linie schnell, also entscheidet die Reihenfolge, *wie* du den Orbit zuerst
erreichst.

---

## Wurzeln — billig, ab Start erreichbar

### Legierungen
- **Voraussetzung:** —
- **Kosten:** Metall, Zeit *(gering)*
- **Schaltet frei:** Hütte (Metalle → Legierungen)

### Elektronik
- **Voraussetzung:** —
- **Kosten:** Kristall, Zeit *(gering)*
- **Schaltet frei:** Elektronikfabrik (Silikate + Metalle → Elektronik)

> Beide Wurzeln kosten nur **Rohstoffe**, die ab dem ersten Förderer fließen.

---

## Stämme — verlangen veredelte Güter

### Komputertechnik
- **Voraussetzung:** Elektronik
- **Kosten:** Elektronik, Legierungen, Zeit *(mittel)*
- **Öffnet:** Satelliten (Folge-Forschung)

### Triebwerktechnik
- **Voraussetzung:** Legierungen
- **Kosten:** Metall, Legierungen, Gas, Zeit *(mittel)*
- **Öffnet:** Raketen (Folge-Forschung)



---

## Krone — der Schritt ins All

### Raketen
- **Voraussetzung:** Triebwerktechnik
- **Kosten:** Legierungen, Gas, Elektronik, Zeit *(hoch)*
- **Schaltet frei:** Startrampe (Startklasse *klein* — der Riegel zur Orbit-Ebene)

### Satelliten
- **Voraussetzung:** Komputertechnik
- **Kosten:** Elektronik, Legierungen, Zeit *(hoch)*
- **Schaltet frei:** Satellit-Nutzlasten — **Scan-Satellit** (Blick nach unten, deckt
  das eigene Profil und den Nebel auf) und **Forschungs-Satellit** (beschleunigt
  Forschung aus dem Orbit)

---

## Danach — nächste Sprossen, noch nicht Phase 1

- **Sonden** → hebt die Startklasse auf *mittel*; deckt die *Inhalte* fremder Körper
  auf (zweite Aufklärungsstufe).
- **Stationsbau** → Startklasse *groß*, Stationsmodule als zweite Baufläche.
- **Komposit** / **Eis** (4. Rohstoff) → Übergang zu Phase 2 und zur Typ-I-Schwelle.
- **Epoche II / III** — Dyson-Schwarm, Massebeschleuniger, Stellar-Triebwerk —
  bleiben im Baum sichtbare, **gesperrte Tore** (Zugkraft, kein Inhalt).
