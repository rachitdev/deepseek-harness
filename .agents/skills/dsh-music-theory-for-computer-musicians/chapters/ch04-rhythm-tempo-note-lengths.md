# Chapter 4 — Rhythm, Tempo, and Note Lengths

## Core Idea
Duration is the parameter that gives every event "its value in the dimension of musical time,"
and the succession of those values is rhythm. Hewitt insists the computer musician learn the
**American fractional naming system** (whole note, half note, quarter note…) rather than the
English one (crotchet, quaver, minim), because that is what music software uses — and because
resolution, snap-to-grid, and quantization settings are meaningless without it.

## Frameworks Introduced

### Beat → Tempo → Relative Beat Length
- **Beat/pulse** — musical time divided into regular pulses of consistent duration, emulating
  natural rhythms (seasons, tides, heartbeat, breath, walking).
- **Tempo** — how fast or slow the beat is, measured in beats per minute. "It has been
  scientifically proven that beats faster than the average speed of the human heart (about 80
  beats per minute) tend to stimulate and excite, whereas those slower than the heartbeat have a
  calming effect."
- **Relative beat length** — the length of a beat *relative to others* in the composition, e.g.
  two hi-hat beats per kick-drum beat = hi-hats are half the length.

### The American Note-Value System
Each note's value is a proportion of a whole note.
- **Whole note** — empty circle. Equals a whole bar in 4/4.
- **Half note** — empty circle with a tail. Two per whole note; equals half a bar.
- **Quarter note** — like a half note but with a black-filled circle. Four per whole note. "Most
  electronic music tracks use four quarter notes as a standard division of musical time. Each
  series of four quarters is called a bar." In the piano roll the quarters are the vertical grid lines.
- **Eighth note** — half a quarter; a quarter note with **one flag**. A run of eighths is **beamed**.
- **16th note** — **two flags**. **32nd note** — **three flags**. 64ths four flags, 128ths five.
  "For most tracks it is very rare to see any note shorter than a 32nd" — 32nds are typically
  confined to things like a programmed snare roll.

### Dotted Notes
A dot "extends the value of a note category by another half."
- Dotted half note = a three-quarter note (3 quarter notes).
- Dotted quarter note = a three-eighths note (3 eighth notes).

### Rests
"Rhythm can be defined as a pattern of sounds and silences… it is the gaps between the notes that
give the rhythm its shape and identity." Every note value has a corresponding rest symbol.
Rests only exist in programs with score-writing facilities — SONAR, Logic, Cubase, Sibelius,
Finale do; **Propellerhead's Reason does not**, because it has no score view. In piano roll,
rests simply appear as empty space.

### Resolution / Snap to Grid / Quantization
| Function | What it does |
|---|---|
| **Resolution** | The value of notes drawn with the Pencil tool. Set to an eighth → pencils in an eighth note on the selected pitch. |
| **Snap to grid** | Written notes automatically land at the start of the beat, the beat being whatever the resolution is set to. |
| **Quantization** | A snap-to-grid function that operates *while recording*: set to an eighth, every recorded note is moved to the nearest eighth. |

## Key Concepts
- **Harmonic rhythm** — "the rhythm engendered by the chord changes in a song." Chords have
  rhythm too.
- **Bar** — each series of four quarter notes in the standard electronic-music division.
- **Beaming** — connecting the flags of eighths and shorter notes.
- **Guiro** — Hewitt's example of studying real instruments: "a guiro, by the way, is simply a
  washboard, and a stick is used to scrape it."

## Mental Models
- **Rhythm is prior to everything.** "Without rhythm there probably could not be any music," and
  in dance music "the drum track is probably the most important feature. After all, it is the
  beat that we dance to."
- **Fractions map to bars in 4/4.** A quarter note is a quarter of a bar; a half note is half a
  bar; a whole note is a whole bar. This is Hewitt's memory hook.
- **The two-hand tap.** Tap once per second with the right hand = whole notes; tap twice as fast
  with the left = half notes. The relationship, not the absolute speed, is what the notation encodes.
- **Rhythm carries style.** "Styles of music (such as house, hard dance, techno, and drum and
  bass) are often distinguished by the driving rhythms of their bass lines."

## Anti-patterns
- Relying on other people's loops: "These days it is not enough to rely on other people's loops…
  Any producer worth his salt will produce his own drum and percussion tracks."
- Over-quantizing: quantization "is not always desirable though, because it can lead to a rigid,
  mechanical sound."
- Learning only the English names (crotchet/quaver/minim) when software speaks American fractions.

## Reference Tables

### Table 4.1 — Various Tempos in Beats per Minute (BPM)
| Genre | BPM |
|---|---|
| Ambient | 50–100 BPM |
| Hip-hop | 70–95 BPM |
| Deep house | 110–130 BPM |
| Trance | 130–145 BPM |
| Hard dance/hardcore | 145–170 BPM |
| Drum and bass | 160–180 BPM |

Reference point: average human heart ≈ 80 BPM — the perceptual dividing line between calming
and stimulating tempos.

### Table 4.2 — Kick Drum Quarters and Hi-Hat Eighths
| Step | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|
| Kick drum | ● | | ● | | ● | | ● | |
| Hi-hat | ● | ● | ● | ● | ● | ● | ● | ● |

The kick marks the pulse by which tempo is gauged; the hi-hat marks a two-to-one rhythm.
(Audio Track 7; Tracks 8, 9, 10 plus Appendix B give further drumming patterns.)

### Note values and flags
| Note | Fraction of whole note | Symbol cue | Per bar in 4/4 |
|---|---|---|---|
| Whole | 1 | Empty circle | 1 |
| Half | 1/2 | Empty circle + tail | 2 |
| Quarter | 1/4 | Filled circle + tail | 4 |
| Eighth | 1/8 | + 1 flag | 8 |
| 16th | 1/16 | + 2 flags | 16 |
| 32nd | 1/32 | + 3 flags | 32 |
| 64th | 1/64 | + 4 flags | 64 |
| 128th | 1/128 | + 5 flags | 128 |

## Worked Example
**Exercise 7:** "A. There are ___ sixteenth notes in a half note. B. Three quarter notes are
equivalent to ___ eighth notes. C. There are ___ thirty-second notes in an eighth note."

Worked from the chapter's own doubling rule (each value is half the one above it):
- A half note = 2 quarters = 4 eighths = **8 sixteenths**.
- One quarter = 2 eighths, so three quarters = **6 eighth notes**.
- One eighth = 2 sixteenths = **4 thirty-second notes**.

**Exercise 9**, applying the chapter's dot rule ("extends the value of a note category by another half"):
- A. A dotted quarter note = a quarter + an eighth = **3 eighth notes** (the chapter states this
  outright: "a dotted quarter note is equivalent to a three-eighths note").
- B. A dotted whole note = a whole + a half = **6 quarter notes**.
- C. A dotted sixteenth note = a sixteenth + a thirty-second = **3 thirty-second notes**.

(Answers to the printed exercises were a separate download from www.courseptr.com/downloads and
are not in the book; the above are derived from the chapter's own stated rules.)

## Key Takeaways
1. Learn the American fractional note names — they are what DAWs use.
2. Each value is half the one above; flags count the halvings past a quarter (1 flag = 8th, 2 = 16th, 3 = 32nd).
3. A dot adds half the note's value.
4. Rests are silences with note-value equivalents; they only appear in score-capable software.
5. Resolution sets what you draw, snap-to-grid sets where drawn notes land, quantization moves
   recorded notes — all three are expressed in note values.
6. Tempo has genre-typical bands (Table 4.1) and a perceptual pivot near the 80 BPM heartbeat.

## Connects To
- [Chapter 5 — Score Editing](ch05-score-editing.md): these same symbols placed on a staff.
- [Chapter 7 — Meter](ch07-meter.md): how these beats group into bars and metric cycles.
- [Chapter 14 — Additive Rhythms](ch14-additive-rhythms.md): asymmetric cycles built from these values.
- [Chapter 10 — Melody and Motives](ch10-melody-and-motives.md): rhythm as a component of a strong motive.
