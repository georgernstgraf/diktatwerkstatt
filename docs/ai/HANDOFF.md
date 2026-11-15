---
type: Handoff
title: Übergabe
description: Was die nächste Sitzung zuerst tut, in welcher Reihenfolge, und was schon bekannt ist.
tags: [wissensbasis, handoff]
generated: { by: opencode, at: 2026-11-15T00:00:00Z }
status: active
---

# Handoff

Arbeitsverzeichnis: `~/repos/georgernstgraf/diktatwerkstatt`
Repository: https://github.com/georgernstgraf/diktatwerkstatt
Stand: Spezifikation fertig, **kein Code**. Branch `main`, Arbeitskopie sauber.

## 🚦 Schritt 1 — das Epic ausgrillen (vor jedem Code)

`grilling`-Skill auf **Issue #3** anwenden. Ziel: die teuren Annahmen kippen,
bevor sie Code werden. Das Ergebnis gehört als Kommentar an #3.

Fragen, die dabei gestellt werden müssen:

1. **Ist v0.1 ohne Word-Integration für Sekretariate überhaupt benutzbar?** Die
   Zielgruppe diktiert in Word, nicht in eine App. Clipboard plus Strg+V ist
   eine Zumutung, wenn der Brief danach noch formatiert werden muss.
2. **Reicht ein Ring von 7?** Ein Vormittag hat acht Briefe. Was ist der
   Notausgang — `pinned` allein?
3. **Wer schreibt die Prompts?** Der Nutzer, oder liefern wir fünf fertige
   (DE-sachlich, EN, FR, Protokoll, E-Mail)? Ohne Beispiele ist die Bibliothek
   leer und die App wertlos.
4. **Ist eine Politur pro Diktat richtig**, wenn ein Diktat mehrere Briefe
   enthält? Oder ist das Diktat doch der falsche Container?
5. **Was passiert, wenn der API-Schlüssel mitten im Diktat abgelehnt wird?**
   Rohtext retten? In die Zwischenablage? Welcher Zustand im Tray?
6. **Ist Whisper-Deutsch gut genug** für Kanzleisprache — Eigennamen,
   Paragrafen, Aktenzeichen? Ohne Messung ist das eine Annahme, keine Tatsache.

## Schritt 2 — Spike (#1)

Ein Tag, Wegwerf-Crate, vier Bausteine: Audio 16 kHz, Hook mit
Space-Schlucken, Clipboard-Injection mit UIPI-Erkennung, Tray mit Zuständen.
Abnahmekriterien stehen im Issue. Erkenntnisse danach nach
`docs/ai/PITFALLS.md` (Datei anlegen).

## Schritt 3 — parallel: #4 Domäne und #5 DSP-Port

Beide brauchen keine Hardware und kein Netz, laufen also auch auf Linux.
#5 hat einen fertigen Prüfstein: die Tests aus `polished-recognition/:core` und
`aitranscribe/tests/test_fragmenter.py` — sie werden portiert, nicht neu erfunden.

## Schritt 4 — der Rest

| Reihenfolge | Issue | Bedingung |
|---|---|---|
| 1 | #6 STT/LLM-Client | braucht #1 nur zum manuellen Test |
| 2 | #7 Workbench | braucht #6 (Play) und #4 (Ring-Liste) |
| 3 | #8 Fast Path | braucht #1 (belegt) und #4 |
| 4 | #9 Paketierung | braucht #7 und #8 |

## Offene Entscheidungen

- **Lizenz (#2)** — blockiert jede Mitarbeit. Vor der ersten Veröffentlichung entscheiden.
- **Update-Kanal und Signatur (#9)** — unsigniert mit SmartScreen-Warnung oder Zertifikat.

## Bekannte Fallen (nicht neu entdecken)

- Windows 11 versteckt neue Tray-Symbole im Überlauf ⇒ Overlay-Pille zusätzlich.
- `Win+V` und Clipboard-Manager speichern jedes Diktat ⇒ Ausschlussformate setzen.
- UIPI: erhöht laufende Ziele erreichen weder Hook noch Injection ⇒ melden, nicht schweigen.
- Space darf **nur im Zustand RECORDING** geschluckt werden, sonst tippt niemand mehr Leerzeichen.
- Play fügt nicht ein. Sonst entsteht doppelter Text im fremden Dokument.
- Rohtext nie gewrappt speichern (Fehler aus `aitranscribe`: `PromptManager` wrappt beim Schreiben).

## Umgebung

- Entwicklung auf Linux: `cargo`, `node`, Tauri-CLI. Windows-Test in VM oder auf Zweitrechner.
- Kein Code-Signing-Zertifikat vorhanden.
- WebView2 ist auf Windows 10/11 vorinstalliert; im Installer prüfen, nicht mitliefern.
