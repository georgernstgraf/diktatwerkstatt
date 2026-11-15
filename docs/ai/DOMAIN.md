---
type: Domain
title: Domänenmodell
description: Tabellen, Invarianten und Zustände der Diktatwerkstatt.
tags: [domain, datenmodell]
generated: { by: opencode, at: 2026-11-15T00:00:00Z }
status: draft
---

# Domänenmodell

Drei Tabellen. Eine SQLite-Datei. Kein ORM.

Speicherort: `%LOCALAPPDATA%\diktatwerkstatt\diktatwerkstatt.db` (Windows),
`~/.local/share/diktatwerkstatt/diktatwerkstatt.db` (Linux).
API-Schlüssel liegen **nicht** darin, sondern im Windows Credential Manager.

## `recording` — das Diktat

| Spalte | Typ | Bedeutung |
|---|---|---|
| `id` | INTEGER PK | |
| `created_at` | TEXT | ISO-8601, lokal |
| `closed_at` | TEXT NULL | gesetzt, sobald geschlossen |
| `status` | TEXT | `offen` \| `geschlossen` |
| `pinned` | INTEGER | 1 = vom Ring ausgenommen |
| `handed_over_seq` | INTEGER | höchste Rohtextblock-`seq`, die bereits in die Zwischenablage ging (0 = keine) |
| `polished_text` | TEXT NULL | die Politur — genau ein Slot |
| `polished_prompt_id` | INTEGER NULL | Prompt, aus dem sie entstand |
| `polished_model` | TEXT NULL | Modell, aus dem sie entstand |
| `polished_at` | TEXT NULL | Zeitpunkt des letzten Play |

`handed_over_seq` beschreibt die **Übergabe an die Zwischenablage**, nicht das
Einfügen. Ob der Text tatsächlich in der Zielanwendung gelandet ist, ist von
außen nicht feststellbar — die App behauptet es nicht.

## `chunk` — der Rohtextblock

| Spalte | Typ | Bedeutung |
|---|---|---|
| `id` | INTEGER PK | |
| `recording_id` | INTEGER FK | `ON DELETE CASCADE` |
| `seq` | INTEGER | 1-basiert, lückenlos, pro Diktat eindeutig |
| `created_at` | TEXT | ISO-8601 |
| `raw_text` | TEXT | das Transkript |
| `sprache` | TEXT NULL | von der STT-API gemeldet, nicht geraten |
| `stt_modell` | TEXT | z. B. `whisper-large-v3-turbo` |
| `dauer_ms` | INTEGER | Audioblock-Dauer |
| `quelle` | TEXT | `diktiert` \| `eingefuegt` |

## `prompt` — die Bibliothek

| Spalte | Typ | Bedeutung |
|---|---|---|
| `id` | INTEGER PK | |
| `alias` | TEXT UNIQUE | Anzeigename, z. B. „FR-böse" |
| `body` | TEXT | die Anweisung; `{{text}}` ist der Rohtext |
| `default_model` | TEXT NULL | Vorschlag, keine Bindung |
| `zielsprache` | TEXT NULL | optional, für Filter/Sortierung |
| `ton` | TEXT NULL | optional, für Filter/Sortierung |
| `archiviert` | INTEGER | 1 = nicht mehr im Dropdown |

`zielsprache` und `ton` sind bewusst **optional**: der Alias genügt heute, die
Felder kosten nichts und ersparen später eine Migration.

## Die sieben Invarianten

1. **Rohtext eines Diktats** = Verkettung seiner Blöcke nach `seq`, getrennt
   durch eine Leerzeile. Der Rohtext wird nicht redundant gespeichert.
2. **Höchstens ein Diktat ist offen** — und es ist das jüngste im Ring.
3. **Geschlossene Diktate sind unveränderlich.** Angehängt wird nur an das
   offene. Es gibt kein „Diktat wieder öffnen".
4. **Append** fügt eine Zeile in `chunk` ein (`seq = max+1`) und lässt den
   Status `offen`. Ein Append ist kein neues Diktat.
5. **Jede Änderung am Rohtext verwirft die Politur**: Append, Korrektur eines
   Blocks, Löschen eines Blocks setzen `polished_*` auf `NULL`.
6. **Eine Politur pro Diktat.** Play überschreibt sie vollständig, inklusive
   Herkunft (`polished_prompt_id`, `polished_model`, `polished_at`).
7. **Der Ring hält 5–7 Einträge** (Standard 7, konfigurierbar 5–7). Nach dem
   Schließen eines Diktats wird das älteste nicht-`pinned` gelöscht, mit allen
   Blöcken (Cascade). Es gibt kein Archiv und keine Pagination.

## Lieferung

- Nach erfolgreicher Transkription geht der **Rohtextblock** in die
  Zwischenablage; `handed_over_seq` wird auf dessen `seq` gesetzt. Tray: `READY`.
- Play legt die **Politur** in die Zwischenablage und überschreibt dort den
  Rohtext. Tray: `POLISHED`.
- Play fügt **nie** selbst in die Zielanwendung ein. Einfügen bleibt eine
  bewusste Handlung des Nutzers (Strg+V), damit ein erneuter Play keinen
  doppelten Text im Dokument erzeugt.
- Beim Schreiben in die Zwischenablage werden die Windows-Formate
  `CanIncludeInClipboardHistory=0`, `CanUploadToCloudClipboard=0` und
  `ExcludeClipboardContentFromMonitorProcessing` gesetzt: der Text ist
  einfügbar, erscheint aber nicht in `Win+V` und in keinem Clipboard-Manager.

## Lebenszyklus eines Diktats

```
neu        INSERT recording(status=offen)
sprechen   INSERT chunk(seq=1)          ← nach Stop
pausieren  kein DB-Schreibvorgang, nur Zustand
anhängen   INSERT chunk(seq=2)          ← verwirft polished_*
schließen  UPDATE recording SET status=geschlossen, closed_at=now
ring       DELETE ältestes nicht-pinned, wenn Anzahl > 7
```

## Audio

Audio wird nach erfolgreicher Transkription des Blocks verworfen
(Retention 0 als Standard). Es gibt keine Medienbibliothek und keine
Audio-Historie — der Ring hält Text, nicht Ton. Wird Evidenz gewünscht, gilt das
Session-Bundle-Format aus `polished-recognition`, damit der dortige
Kotlin-`:analyzer` die Desktop-Sitzungen mitlesen kann.

## Nicht Teil des Modells

- Run-Verlauf, Prompt-Versionierung, Fingerprints.
- Konten, Synchronisation, Cloud.
- Einfüge-Protokoll pro Zielanwendung.
