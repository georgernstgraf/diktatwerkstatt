---
type: State
title: Zustand
description: Was existiert, was noch nicht, und wie man es prüft.
tags: [zustand]
generated: { by: opencode, at: 2026-11-15T00:00:00Z }
status: active
---

# Zustand

## Existiert

| Artefakt | Stand |
|---|---|
| `README.md` | Beschreibung, Abgrenzung, Architekturbild |
| `CONTEXT.md` | Vokabular inkl. bewusster Abgrenzungen und Zustandsdiagramm |
| `docs/ai/DOMAIN.md` | drei Tabellen, sieben Invarianten, Lieferregeln, Lebenszyklus |
| `docs/ai/DECISIONS.md` | 12 Entscheidungen, je mit verworfener Alternative |
| `docs/ai/HANDOFF.md` | Reihenfolge der nächsten Sitzung |
| `AGENTS.md` | Stack, Schichtungsregel, Verifikationsbefehle |
| Issues | Epic #3 mit acht Sub-Issues (#1, #2, #4–#9) |

## Existiert nicht

- Kein `Cargo.toml`, kein `src/`, kein `package.json` — **null Produktcode**.
- Keine Tests, keine CI. `cargo test` läuft ins Leere, weil es kein Crate gibt.
- Keine Lizenz (Issue #2).

## Verifikation

Im Moment gibt es nichts zu verifizieren außer der Dokumentenkonsistenz:
`CONTEXT.md`, `DOMAIN.md` und `DECISIONS.md` müssen dieselben Begriffe und
Regeln nennen. Der erste grüne Commit wird der Spike (#1).

## Nächster Schritt

`docs/ai/HANDOFF.md`, Schritt 1: Epic #3 ausgrillen.
