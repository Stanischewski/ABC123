-- Phase-0-Grundlage: der persistente Welt-Zustand.
--
-- Die Welt ist (vorerst) ein Singleton: eine einzige, nie zurückgesetzte
-- Simulation (DESIGN.md §2, §4.5). Wir halten Sim-Zeit und Systemzustand als
-- JSONB — flexibel, solange das Schema in `core` noch in Bewegung ist. Sobald
-- sich die Typen stabilisieren, werden einzelne Aggregate in echte Tabellen
-- normalisiert (Körper, Gebäude, Flotten ...).
CREATE TABLE IF NOT EXISTS world (
    id         smallint    PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    sim_time   double precision NOT NULL DEFAULT 0,
    system     jsonb       NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);
