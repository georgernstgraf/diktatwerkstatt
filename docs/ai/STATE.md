---
type: State
title: Zustand
description: Was existiert, was noch nicht, und wie man es prüft.
tags: [zustand]
generated: { by: opencode/knowledge-persistence, at: 2026-11-19T04:28:38Z }
status: stable
---

# Zustand

## Existiert

| Artefakt | Stand |
|---|---|
| `README.md` | Beschreibung, Abgrenzung, Architekturbild, Lizenz |
| `CONTEXT.md` | Vokabular inkl. bewusster Abgrenzungen und Zustandsübergängen |
| `docs/ai/DOMAIN.md` | zwei Tabellen, zehn Invarianten, Abläufe, Arbeitsverzeichnis, Fehlerfälle |
| `docs/ai/INTEGRATION.md` | Referenz: wie Handy/OpenWhispr die Windows-Bausteine lösen, mit Quellen; „was wir übernehmen" ist auf Zwischenablage und Tray zusammengeschrumpft |
| `docs/ai/PITFALLS.md` | Referenz: Fallstricke aus Audio, STT, Textübergabe und Bedienung, mit Quellen |
| `docs/ai/DECISIONS.md` | 24 Entscheidungen, je mit verworfener Alternative |
| `docs/ai/index.md`, `docs/ai/log.md` | OKF-Bundle-Einstieg (`okf_version: "0.2"`) und Persistenz-Chronik |
| `docs/ai/HANDOFF.md` | Reihenfolge der nächsten Sitzung inkl. VM-Lage |
| `docs/ai/STATE.md` | diese Datei |
| `AGENTS.md` | Stack, Schichtungsregel, Verifikationsbefehle |
| `LICENSE-MIT`, `LICENSE-APACHE` | dual lizenziert, `MIT OR Apache-2.0` |
| Issues | Epic #3 mit acht offenen Sub-Issues (#1, #4–#10) und einem geschlossenen (#2); #3 ausgegrillt |

## Existiert nicht

- Kein `Cargo.toml`, kein `src/`, kein `package.json` — **null Produktcode**.
- Keine Tests, keine CI. `cargo test` läuft ins Leere, weil es kein Crate gibt.
- Keine Windows-VM auf think: qemu, libvirt, OVMF und swtpm sind installiert,
  es ist aber noch kein Gast definiert.

## Verifikation

Im Moment gibt es nichts zu verifizieren außer der Dokumentenkonsistenz:
`CONTEXT.md`, `DOMAIN.md` und `DECISIONS.md` müssen dieselben Begriffe und
Regeln nennen. `CONTEXT.md` ist dabei die Quelle der Begriffe, `DOMAIN.md` die
Quelle der Invarianten; wer einen Begriff ändert, ändert ihn zuerst dort.

Der erste grüne `cargo`-Commit kommt voraussichtlich aus **#4** (Domänenkern)
oder **#10** (Konverter), weil beide ohne Hardware und ohne Netz auskommen. Der
Spike (#1) braucht die VM und kommt danach.

## Nächster Schritt

`docs/ai/HANDOFF.md`, Schritt 3: #4 (Domänenkern) und #10 (Konverter) zuerst —
beide ohne Hardware und ohne Netz, und #10 ist der einzige Baustein, der schon
jetzt gegen eine Spezifikation testbar ist. Der Spike #1 braucht die VM auf
think; #5 läuft parallel.
