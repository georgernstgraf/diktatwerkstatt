---
type: State
title: Zustand
description: Was existiert, was noch nicht, und wie man es prüft.
tags: [zustand]
generated: { by: opencode/knowledge-persistence, at: 2026-11-16T05:04:02Z }
status: stable
---

# Zustand

## Existiert

| Artefakt | Stand |
|---|---|
| `README.md` | Beschreibung, Abgrenzung, Architekturbild, Lizenz |
| `CONTEXT.md` | Vokabular inkl. bewusster Abgrenzungen und Zustandsübergängen |
| `docs/ai/DOMAIN.md` | zwei Tabellen, zehn Invarianten, Abläufe, Arbeitsverzeichnis, Fehlerfälle |
| `docs/ai/DECISIONS.md` | 20 Entscheidungen, je mit verworfener Alternative |
| `docs/ai/HANDOFF.md` | Reihenfolge der nächsten Sitzung inkl. VM-Lage |
| `docs/ai/STATE.md` | diese Datei |
| `AGENTS.md` | Stack, Schichtungsregel, Verifikationsbefehle |
| `LICENSE-MIT`, `LICENSE-APACHE` | dual lizenziert, `MIT OR Apache-2.0` |
| Issues | Epic #3 mit acht Sub-Issues (#1, #2, #4–#9); #3 ausgegrillt |

## Existiert nicht

- Kein `Cargo.toml`, kein `src/`, kein `package.json` — **null Produktcode**.
- Keine Tests, keine CI. `cargo test` läuft ins Leere, weil es kein Crate gibt.
- Kein `docs/ai/PITFALLS.md` — entsteht mit dem Spike (#1).
- Keine Windows-VM auf think: qemu, libvirt, OVMF und swtpm sind installiert,
  es ist aber noch kein Gast definiert.

## Verifikation

Im Moment gibt es nichts zu verifizieren außer der Dokumentenkonsistenz:
`CONTEXT.md`, `DOMAIN.md` und `DECISIONS.md` müssen dieselben Begriffe und
Regeln nennen. `CONTEXT.md` ist dabei die Quelle der Begriffe, `DOMAIN.md` die
Quelle der Invarianten; wer einen Begriff ändert, ändert ihn zuerst dort.

Der erste grüne `cargo`-Commit wird voraussichtlich **#4** (Domänenkern), weil
er ohne Hardware und ohne Netz auskommt. Der Spike (#1) braucht die VM und
kommt danach.

## Nächster Schritt

`docs/ai/HANDOFF.md`, Schritt 2: Spike #1 in der VM auf think. Parallel #4 und
#5, beide plattformunabhängig.
