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

## Bekannte Fallen (nicht neu entdecken)

**Audio und STT**

- Whisper **halluziniert auf Stille**: auf reinem Raumton emittiert `large-v3`
  in 61,9 % der Fälle Wörter, dokumentiert sind deutsche Untertitel-Credits
  („Untertitel der Amara.org-Community"). Ein Praxisbericht: 37 min Meeting plus
  5 h Leerlauf ergab ~600 erfundene von 963 Zeilen. Stille darf nie an die API
  gehen; es braucht einen harten Stopp, keinen „läuft weiter"-Modus.
- Das stabile Fenster ist **20–30 s**. Gemessen auf deutscher
  Verwaltungssprache: 30 s → 4,76 % WER, 45 s → 34,59 %, 60 s → 51,47 %. Die
  geerbten ~21-s-Fragmente liegen richtig.
- Prompt-Biasing bei Whisper ist auf **224 Token** begrenzt (~60–100 Fachwörter)
  und naiv injiziert gefährlich: Prompt-Echo treibt den WER von 6,5 % auf 61 %.
- OpenAI kündigt `whisper-1` und `gpt-4o-transcribe` zum **26.02.2027** ab; der
  Nachfolger hat einen eigenen `keywords`-Kanal. Nicht auf den `prompt`-Parameter
  bauen.

**Text und Einfügen**

- Word übernimmt beim Einfügen von reinem Text die Formatierung **an der
  Einfügestelle** — die Briefvorlage liefert das Aussehen. Was fehlt, ist
  Struktur, nicht Formatierung.
- **AutoCorrect greift beim Einfügen nicht.** Gerade Anführungszeichen und `--`
  bleiben stehen; das gilt auch für COM-Einfügung.
- Markdown ist gegen juristischen Text nicht neutral: `1)` wird zur Liste,
  `5*6*78` wird kursiv, `\§` bleibt als Backslash stehen. Deshalb constrained
  Markdown plus Validierung.
- Die Clipboard-Ausschlussformate wirken **global**, nicht je Datenformat —
  `CF_HTML` daneben zu legen hebt den Win+V-Ausschluss nicht auf. Drittanbieter-
  Clipboard-Manager halten sich aber nicht zwingend daran.
- UIPI: erhöht laufende Ziele erreichen weder Hook noch Injection, und
  `SendInput` meldet den Fehlschlag **nicht** — der Text verschwindet stumm.
- Google Docs ist ein Canvas: UI Automation ist wertlos, nur echte
  Tastatur-Events oder ein Paste kommen an.

**Bedienung**

- Space darf **nur im Zustand RECORDING** geschluckt werden, sonst tippt niemand
  mehr Leerzeichen.
- Die App fügt nie selbst ein. Sonst entsteht doppelter Text im fremden
  Dokument.
- Rohtext nie gewrappt speichern (Fehler aus `aitranscribe`: `PromptManager`
  wrappt beim Schreiben).
- Windows 11 versteckt neue Tray-Symbole im Überlauf ⇒ Overlay-Pille zusätzlich.

## Umgebung

- Entwicklung auf Linux (`cargo`, `node`, Tauri-CLI); Windows-Test in der VM auf
  think.
- Kein Code-Signing-Zertifikat vorhanden.
- WebView2 ist auf Windows 10/11 vorinstalliert; im Installer prüfen, nicht
  mitliefern.
