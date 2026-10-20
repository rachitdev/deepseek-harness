# Chapter 24 — Chord Abbreviations

## Core Idea
Chord abbreviations are the third way to communicate a progression, and "more popular
nowadays than figured bass because they require less information, and are a little
easier to understand." The decisive difference: abbreviations name absolute chords, so
**you do not need to specify a key.**

## Frameworks Introduced

### The abbreviation grammar
Every abbreviation is a capital letter (the root) plus an optional suffix (the quality).

| Suffix | Meaning | Examples |
|--------|---------|----------|
| *(none)* | Major | Bb = Bb major · D = D major · F# = F# major |
| `m` | Minor | Dbm = Db minor · Gm = G minor · G#m = G# minor |
| `dim` or `º` | Diminished | Abdim / Abº · Cdim / Cº · D#dim / D#º |
| `aug` or `+` | Augmented | Ebaug / Eb+ · Aaug / A+ · C#aug / C#+ |
| `7` | Dominant seventh | Bb7 · D7 · F#7 |
| `sus2` / `sus4` / `sus6` | Suspended | F#sus2 · Gbsus4 · Dsus6 |

### Suffix stacking
"You may also see combinations of all such abbreviations above." Steiner's example:
**C#maj7sus4** — "this tricky little monster" — resolves to **C#, F#, G#, and B**
(a major seventh built over a sus4).

### Figured bass vs abbreviations, same progression
```
Figured Bass
Bb Major:    I     vi    iii    IV    V     I

Chord Abbreviations
             Bb    Gm    Dm     Eb    F     Bb
```
"Notice that in the chord abbreviation section, I didn't even specify what key I was in.
If I'm using that method, I don't need to specify a key. I'm simply specifying what
chords to play."

## Key Concepts
- Two notations exist for diminished (`dim`, `º`) and two for augmented (`aug`, `+`).
  Both are in circulation; read either, write either.
- A bare `7` always means **dominant** seventh, never major seventh (Ch 11).
- The uppercase/lowercase distinction that carries quality in figured bass does *not*
  apply here — the root letter is always capitalised and the suffix does the work.

## Mental Models
- **Absolute vs relative notation.** Figured bass says "the sixth chord of this key";
  abbreviations say "G minor." Relative notation transposes for free; absolute notation
  needs no context.
- **Read the suffix left to right.** `C#maj7sus4` = root C#, major 7th, sus4 — apply
  each modifier in turn.
- **Horse before the carriage.** Steiner shows the Bb progression in both notations
  before explaining the rules, betting the reader can infer them.

## Anti-patterns
- Reading a bare `7` as a major seventh. It is a dominant seventh; a major seventh is
  written `maj7`.
- Writing "Gm" and meaning G major, or "G" and meaning G minor.
- Assuming abbreviations tell you the key. They do not — and do not need to.
- Confusing `º` (diminished) with `+` (augmented).

## Reference Tables

**Full abbreviation lookup**

| Written | Read as | Chord type chapter |
|---------|---------|--------------------|
| `Bb` | Bb major | [7](ch07-building-major-chords.md) |
| `Gm` | G minor | [8](ch08-building-minor-chords.md) |
| `Caug`, `C+` | C augmented | [9](ch09-building-augmented-chords.md) |
| `Cdim`, `Cº` | C diminished | [10](ch10-building-diminished-chords.md) |
| `D7` | D dominant seventh | [11](ch11-building-seventh-chords.md) |
| `Gbsus4` | Gb sus4 | [13](ch13-building-sus4-chords.md) |
| `F#sus2` | F# sus2 | [14](ch14-building-sus2-chords.md) |
| `Dsus6` | D sus6 | [15](ch15-building-sus6-chords.md) |
| `C#maj7sus4` | C# major seventh sus4 (C# F# G# B) | [16](ch16-building-sus-seventh-chords.md) |

**Same progression, three notations**

| Method | Content |
|--------|---------|
| Figured bass | `Bb Major: I vi iii IV V I` |
| Chord abbreviations | `Bb  Gm  Dm  Eb  F  Bb` |
| "Mommy, please hold my hand" | "First play a Bb major chord (Bb–D–F), then a G minor chord (G–Bb–D)…" |

## Worked Example
*The chapter's homework — identify twelve abbreviations — with the book's answer key.*

| Abbreviation | Answer |
|--------------|--------|
| Gm | G Minor |
| Eb | Eb Major |
| F7 | F Dominant 7th |
| Bb+ | Bb Augmented |
| Dbº | Db Diminished |
| E | E Major |
| Ddim | D Diminished |
| Am | A Minor |
| Caug | C Augmented |
| F# | F# Major |
| Abm | Ab Minor |
| B7 | B Dominant 7th |

Spelling each out using the formulas from Chapters 7–11:

| Abbreviation | Notes |
|--------------|-------|
| Gm | G Bb D |
| Eb | Eb G Bb |
| F7 | F A C Eb |
| Bb+ | Bb D F# |
| Dbº | Db Fb Abb |
| E | E G# B |
| Ddim | D F Ab |
| Am | A C E |
| Caug | C E G# |
| F# | F# A# C# |
| Abm | Ab Cb Eb |
| B7 | B D# F# A |

## Key Takeaways
- Capital letter = root; suffix = quality; no suffix = major.
- `m`, `dim`/`º`, `aug`/`+`, `7`, `sus2`/`sus4`/`sus6` cover almost everything.
- Bare `7` = dominant seventh.
- No key statement required.
- Suffixes stack: `C#maj7sus4` = C# F# G# B.

## Connects To
- [Chapter 23 — Reading Figured Bass](ch23-reading-figured-bass.md): the notation this
  competes with.
- [Chapter 25 — Chord Inversions](ch25-chord-inversions.md): the slash notation added
  to these abbreviations.
- [Chapter 11 — Building Seventh Chords](ch11-building-seventh-chords.md): why `7`
  means dominant.
