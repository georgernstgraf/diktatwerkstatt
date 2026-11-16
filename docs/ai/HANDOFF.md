---
type: Handoff
title: Übergabe
description: Was die nächste Sitzung zuerst tut, in welcher Reihenfolge, und was schon bekannt ist.
tags: [wissensbasis, handoff]
generated: { by: opencode/knowledge-persistence, at: 2026-11-16T05:04:02Z }
status: stable
---

# Handoff

Arbeitsverzeichnis: `~/repos/georgernstgraf/diktatwerkstatt`
Repository: https://github.com/georgernstgraf/diktatwerkstatt
Stand: **Spezifikation ausgegrillt, kein Code.** Branch `main`, Arbeitskopie
sauber. Das Grill-Ergebnis steht als Kommentar an #3 und ist in `CONTEXT.md`,
`DOMAIN.md` und `DECISIONS.md` eingearbeitet.

## ✅ Erledigt — Schritt 1: das Epic ausgegrillt

Epic #3 ist durch. Die sechs Fragen vom 2026-11-15 sind beantwortet oder
falsifiziert (Details im Kommentar an #3), das Datenmodell ist auf zwei Tabellen
geschrumpft, `pinned`, `status`, `closed_at`, `chunk`, `seq`, `handed_over_seq`,
`quelle` und die Herkunftsspalten sind gestrichen.

## 🚦 Schritt 2 — Spike (#1), jetzt auf der VM

Windows 11 in einer VM auf **think** (steht, noch ohne Gast). Geprüfte Lage:

| Punkt | Befund |
|---|---|
| CPU | i5-6500T, 4 Kerne, 2,5 GHz, `vmx` vorhanden |
| KVM | `/dev/kvm` aktiv; `georg` ist in `kvm` und `libvirt` — kein sudo für `virsh` |
| Werkzeuge | qemu 10.2.1, libvirt 12.0, virt-manager 5.1, OVMF (UEFI), swtpm (TPM 2.0) |
| Disk | 227 GB frei auf `rpool` |
| RAM | `free` zeigt 24 GB belegt, davon 14 GB ZFS-ARC — rückholbar. Vor dem Start `zfs_arc_max` begrenzen, sonst kämpft der Gast um Speicher |

**Audio ist der wunde Punkt.** `ich9-intel-hda` und `usb-audio` sind vorhanden,
aber Issue #1 verlangt „30 s ohne Aussetzer" und „Gerätewechsel überlebt". Ein
emuliertes HDA-Gerät kann den Defekt **maskieren** oder einen erfinden. Für den
Audiobaustein entweder USB-Audio per Passthrough (billiger USB-DAC) oder ein
echtes Windows. Hook, Injection, UIPI und Tray sind in der VM voll belastbar.

**Neues Abnahmekriterium** (kam aus der Grill-Recherche): Word 365 mit Build
≥ 2405, Einfügen aus einem anderen Programm, Standard „Merge Formatting" —
bleiben `h1`–`h6` echte Überschriften-Formatvorlagen oder werden sie zu
Fließtext? Davon hängt ab, ob semantisches HTML als Standardweg trägt.

Erkenntnisse danach nach `docs/ai/PITFALLS.md` (Datei anlegen).

## Schritt 3 — parallel: #4 Domäne und #5 DSP-Port

Beide brauchen keine Hardware und kein Netz, laufen also auch auf Linux.

- **#4** hat jetzt ein sehr kleines Ziel: `diktat` und `prompt` gemäß
  `DOMAIN.md`, das Arbeitsverzeichnis mit `NNN.wav`/`NNN.txt`, die zehn
  Invarianten — jede mit einem Test, der sie bricht, wenn man sie entfernt.
- **#5** hat einen fertigen Prüfstein: die Tests aus
  `polished-recognition/:core` und `aitranscribe/tests/test_fragmenter.py`. Sie
  werden portiert, nicht neu erfunden.

## Schritt 4 — der Rest

| Reihenfolge | Issue | Bedingung |
|---|---|---|
| 1 | #6 STT/LLM-Client | braucht #1 nur zum manuellen Test |
| 2 | #7 Workbench | braucht #6 (Prompt anwenden) und #4 (FIFO-Liste) |
| 3 | #8 Fast Path | braucht #1 (belegt) und #4 |
| 4 | #9 Paketierung | braucht #7 und #8 |

## Offene Entscheidungen

- **Hotkey, Einfügen, Installer, Wiederverwendung** — die vier Fragen aus
  [INTEGRATION.md](INTEGRATION.md#offene-entscheidungen) sind noch nicht
  entschieden. Sie bestimmen den Zuschnitt von #8 und #9.
- **§, Aktenzeichen und Beträge** — bewusst vertagt. Befund: Whisper hat keine
  Inverse Text Normalization, also wird „Paragraph" zum Wort und nicht zum
  Zeichen; für Aktenzeichen gibt es weltweit keine Messung. In v0.1 ist das eine
  Prompt-Aufgabe. Ob es je eigene Eingabefelder braucht, entscheidet sich nach
  dem ersten echten Protokoll.
- **Wiederaufnahme mit Rückfrage** statt automatisch (heute: automatisch).
- **Update-Kanal und Signatur (#9)** — unsigniert mit SmartScreen-Warnung oder
  Zertifikat.
- **Markenrecherche** EUIPO/AT für den Namen, vor der ersten Veröffentlichung.

## Bekannte Fallen

Die Fallstricke aus Audio, STT, Textübergabe und Bedienung stehen in
[PITFALLS.md](PITFALLS.md) — dort mit Belegen. Lies sie, bevor du einen der
vier Bausteine anfasst; sie sind die Kurzfassung von drei Recherchen und
zwei Vorgängerprojekten.

## Umgebung

- Entwicklung auf Linux (`cargo`, `node`, Tauri-CLI); Windows-Test in der VM auf
  think.
- Kein Code-Signing-Zertifikat vorhanden.
- WebView2 ist auf Windows 10/11 vorinstalliert; im Installer prüfen, nicht
  mitliefern.
