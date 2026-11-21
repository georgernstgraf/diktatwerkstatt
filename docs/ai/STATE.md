---
type: State
title: Zustand
description: Was existiert, was noch nicht, und wie man es prüft.
tags: [zustand]
generated: { by: opencode/general, at: 2026-11-21T08:40:06Z }
status: stable
---

# Zustand

## Existiert

| Artefakt | Stand |
|---|---|
| `Cargo.toml` | Workspace (Edition 2024, `resolver = "3"`, `rust-version` 1.89), Abhängigkeiten zentral: `rusqlite` 0.43 (`bundled`), `chrono`, `thiserror` |
| `crates/core` | **Domäne aus #4**: `store.rs` (zwei `STRICT`-Tabellen, `user_version = 1`, WAL/`synchronous = FULL`), `fragments.rs` (Arbeitsverzeichnis `NNN.wav`/`NNN.txt`, `.tmp` + `rename`), `error.rs` (typisierte Fehler) |
| `crates/core/tests/invarianten.rs` | 15 Tests: einer je Invariante, dazu die halb geschriebene `.txt`, die leere `.txt`, die 999-Grenze, die Wiederaufnahme, die Prompt-Bibliothek |
| `README.md` | Beschreibung, Abgrenzung, Architekturbild, Lizenz — drei Ziele, ein Kern |
| `CONTEXT.md` | Vokabular inkl. **Plattformschale**, **Fähigkeitsleiter**, **Diktierplatz**, bewusster Abgrenzungen und Zustandsübergängen |
| `docs/ai/DOMAIN.md` | zwei Tabellen, zehn Invarianten, **Fähigkeitsleiter**, Abläufe, Arbeitsverzeichnis, Lieferformate je Plattform, Fehlerfälle |
| `docs/ai/INTEGRATION.md` | Referenz: wie Handy/OpenWhispr die Bausteine lösen, mit Linux- und macOS-Spalte, mit Quellen; „was wir übernehmen" ist auf Zwischenablage, Tray und Hotkeys zusammengeschrumpft |
| `docs/ai/PITFALLS.md` | Referenz: Fallstricke aus Audio, STT, Textübergabe, Bedienung, **Windows**, **Linux/Wayland**, **Linux/X11** und **macOS**, mit Quellen; X11 und Windows gemessen, macOS und Wayland-Fragen 3/5/7 noch aus dem Bau |
| `docs/ai/DECISIONS.md` | 34 Entscheidungen, je mit verworfener Alternative — darunter die Beschlusslage E1–E10 vom 2026-11-16 |
| `docs/ai/index.md`, `docs/ai/log.md` | OKF-Bundle-Einstieg (`okf_version: "0.2"`) und Persistenz-Chronik |
| `docs/ai/HANDOFF.md` | Reihenfolge der nächsten Sitzung inkl. VM-Lage und Provisioning-Befund |
| `docs/ai/STATE.md` | diese Datei |
| `AGENTS.md` | Stack, Schichtungsregel, Verifikationsbefehle |
| `LICENSE-MIT`, `LICENSE-APACHE` | dual lizenziert, `MIT OR Apache-2.0` |
| Issues | Epic #3 mit offenen Sub-Issues #1, #4–#10 und #12–#18; #2, #11, #19 und #20 geschlossen (#19 claw, #20 think provisioniert); #3 ausgegrillt |

## Existiert nicht

- Kein `apps/` — keine Plattformschale, keine Workbench, kein Tray, keine
  Zwischenablage, kein Audio. Der Kern ist da, die Schalen fehlen.
- Keine CI: die drei `cargo`-Befehle laufen von Hand, vor jedem Commit.
- Keine Windows-VM auf think: qemu, libvirt, OVMF und swtpm sind installiert,
  es ist aber noch kein Gast definiert.
- Kein Netz im Kern: `rusqlite` ist gebündelt, `cargo test` braucht nach dem
  ersten Bau weder Netz noch Hardware noch eine laufende App.

## Verifikation

Im Workspace-Wurzelverzeichnis, auf **think** mit
`export PATH="$HOME/.cargo/bin:$PATH"` (die Agent-Shell findet `cargo` sonst
nicht, siehe `PITFALLS.md`, „Linux und Wayland"):

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Grün heißt hier: die fünfzehn Tests in `crates/core/tests/invarianten.rs` laufen
durch. Ein Test zählt nur, wenn er die Invariante bricht — am 2026-11-21 an drei
Invarianten gegengeprüft (Append-Sperre, Lückenprüfung, Reihenfolge im
Abschluss).

Daneben bleibt die Dokumentenkonsistenz: `CONTEXT.md` ist die Quelle der Begriffe,
`DOMAIN.md` die Quelle der Invarianten; wer einen Begriff ändert, ändert ihn
zuerst dort.

Ohne Hardware testbar bleibt der Fast Path (#8), sobald die **Fähigkeitsleiter**
existiert: ein Fake mit `{foreground_window: nein, insert_keystroke: nein}` ist
Wayland, einer mit `{ja, ja}` ist X11 oder Windows.

## Nächster Schritt

**#5 DSP-Port** in dasselbe Crate — Fragmentierung, Naht, Q-Gate; der Prüfstein
sind die Tests aus `polished-recognition/:core` und
`aitranscribe/tests/test_fragmenter.py`, sie werden portiert, nicht neu erfunden.
Danach **#10 Konverter** (`deliver(markdown) -> Delivery { html, text }`); #4, #5
und #10 liegen alle in `crates/core`, laufen also nacheinander.

Parallel dazu die Spikes (`docs/ai/HANDOFF.md`, Schritt 4): **#12 Linux-Spike**,
dessen X11-Teil gemessen ist (elite, 2026-11-21: `XGrabKey`,
`_NET_ACTIVE_WINDOW`, XTEST, Zwischenablage, Tray, Audio). Offen sind die
Wayland-Fragen 2 (Erst-Einwilligung des Portals über einen Neustart), 3
(Injection in GTK/Writer/Brave) und 5/7 auf GNOME.
