# Chapter 25 — Intonation

## Core Idea
"Intonation concerns **the fine points of how a scale is tuned**." Western music's equal
temperament — twelve mathematically equal semitones — "represents only one option and represents a
**compromise** to what is actually possible." Modern fine-tune options divide the octave into
**1,200 cents** and let you move any key by a single cent, which makes just intonation,
Pythagorean tuning, and the tunings of Bali, Arabia, and Hindustan directly available to a
computer musician.

## Frameworks Introduced

### The Three Tuning Systems
| System | Principle |
|---|---|
| **Equal temperament** | "The octave is divided up into **12 mathematically equal semitones**." The Western default. |
| **Just intonation** | "Favors **pure intervals as defined by their ratios** — in other words, perfect fifths of 3/2, major thirds of 5/4, major sixths of 5/3, and so on." |
| **Pythagorean intonation** | "**Each note of the scale is tuned in pure fifths of ratio 3/2.**" "Also used in traditional Chinese and Arabian music, it generates its own unique interval set." |

### The Cents System
"The octave is divided up into **1,200 cents**, in which **each equal semitone is worth 100**. By
providing a fine-tune option on a synthesizer or sampler, it is possible to **adjust each semitone
up or down in increments of a single cent**. The principal advantage of this is that it becomes
possible to use pure intervals."

**The size of the compromise, worked:** "Compare, for example, a pure major third with an equally
tempered major third. The **tempered third is equivalent to 400 cents, while the pure major third
is 384 cents — a difference of some 16 cents**."

### Why Alternative Tunings Are Hard
> "The central problem with alternative tunings to equal temperament is that **it is not possible
> to construct a pure scale for more than one diatonic key**. And, even then, the requirements of
> producing pure-sounding chords necessitate having **more than one type of Bb**, for example.
> **To create pure organic music in all keys, you would need a scale of about 60 microtonal notes.**
> It is just not always practical."

**The exception:** "With **modal** music it is more viable because **modal music tends to stay in
the one key**. You can obtain some good results."

### World Tunings a Computer Musician Can Reach
| Tradition | Structure |
|---|---|
| **Balinese** | "A **seven-tone scale** tuned quite unlike any other tuning system in the world." Figure 25.2 gives cents measurements for a dedicated Balinese keyboard. "Precise tunings can vary depending upon the locality," so it is "merely a sample tuning." |
| **Thai** | **Seven-tone equal temperament** — "a unique tuning in which the gaps between the seven notes of the scale are tuned as **equal intervals (that is, 1/7 of an octave)**." (Track 52.) |
| **Arabian** | "The basic scale of Arabian music has **17 notes based on Pythagorean tuning**." |
| **Hindustani** | "In Northern Hindustan, the notes that belong to each **raga** are selected from a **microtonal scale of 22 notes to the octave**." Figure 25.4 fine-tunes **Raga Bageshri**, "which is very similar to our C major scale." |
| **Overtone melodies** | "Use only the **overtones or partials of a fundamental**" — didgeridoo, Jew's harp, Buddhist overtone singing. "**Equal temperament does not give the correct pitches for overtone melodies.**" Figure 25.5's fine-tuned keyboard "gives every harmonic **up to the twenty-first**." |

### Ragas Are More Than Scales
"In general, **ragas are much more than just a mode**. Certain conventions exist with regard to
each raga in terms of **important notes, characteristic melodic turns, the time of day it should be
used, the ideal ethos**, and so on. To discover what these conventions are, you would need to do
extensive research."

## Key Concepts
- **Cent** — 1/1200 of an octave; 100 cents = one equally tempered semitone.
- **Historical experiment:** "During the second half of the last century, a lot of experimentation
  took place… American musicians such as Californian **La Monte Young** became very interested in
  the use of pure intervals; his improvisatory **1964 work *The Well Tuned Piano*** required that
  the piano be tuned in **pure thirds (5/4), fifths (3/2) and harmonic sevenths (7/4)**."
- **Software support:** "**Steinberg's Cubase** offers a remarkably effective and very
  straightforward way of tuning your intervals, enabling you to tune each key up or down in
  increments of a single cent." The **Korg Triton** "offers this option in the form of slots for a
  dozen or so **'user scales.'**"
- **Instrument matching matters:** for Balinese Pelog, use "a **Balinese metallophone program**
  that will show this tuning at its best" (Chapter 22).
- **Buddhist overtone singing** — monks "have developed remarkable singing techniques where they
  can sing two notes at the same time — the fundamental and the harmonics."

## Mental Models
- **Equal temperament is a trade, not a truth.** It buys the ability to play in all twelve keys at
  the cost of every interval being slightly impure.
- **Pure tuning wants one key.** The more keys you want pure, the more microtones you need —
  ~60 for all keys. Modal music, which stays put, is where pure tuning pays off.
- **Cents make tuning editable.** Once every key can be nudged by a cent, historical and world
  tunings become presets rather than research projects. "You can also **invent original
  tunings**… Who knows — you could come up with the next big thing to bring music into a higher
  state of evolution!"

## Anti-patterns
- Attempting pure tuning across a modulating, multi-key piece — impractical without ~60 microtones.
- Playing overtone melodies in equal temperament, which "does not give the correct pitches."
- Treating a raga as merely a scale, ignoring its melodic turns, characteristic notes, and time-of-day convention.
- Treating a single published Balinese tuning as canonical — "precise tunings can vary depending
  upon the locality."

## Reference Tables

### Table 25.1 — Twelve-Toned Chromatic Scale Tuned in Equal Temperament, Just Intonation, and Pythagorean Intonation
| Note | Equal temperament (cents) | Just intonation ratio | Just (cents) | Pythagorean ratio | Pythagorean (cents) |
|---|---|---|---|---|---|
| C | 0 | 1/1 | 0 | 1/1 | 0 |
| Db | 100 | 16/15 | 112 | 256/243 | 90 |
| D | 200 | 9/8 | 204 | 9/8 | 204 |
| Eb | 300 | 6/5 | 316 | 32/27 | 294 |
| E | 400 | 5/4 | 386 | 81/64 | 408 |
| F | 500 | 4/3 | 498 | 4/3 | 498 |
| F# | 600 | 45/32 | 590 | 729/512 | 612 |
| G | 700 | 3/2 | 702 | 3/2 | 702 |
| Ab | 800 | 8/5 | 814 | 128/81 | 792 |
| A | 900 | 5/3 | 884 | 27/16 | 906 |
| Bb | 1000 | 9/5 | 1018 | 16/9 | 996 |
| B | 1100 | 15/8 | 1088 | 243/128 | 1110 |
| C | 1200 | 2/1 | 1200 | 2/1 | 1200 |

### Pure interval ratios (just intonation)
| Interval | Ratio |
|---|---|
| Perfect fifth | 3/2 |
| Perfect fourth | 4/3 |
| Major third | 5/4 |
| Major sixth | 5/3 |
| Harmonic seventh (La Monte Young) | 7/4 |
| Octave | 2/1 |

### Microtonal scale sizes
| System | Notes per octave |
|---|---|
| Western equal temperament | 12 |
| Arabian (Pythagorean-based) | **17** |
| Northern Hindustani (raga source scale) | **22** |
| Pure organic music in all keys | **~60** |
| Thai | 7 (equal 1/7-octave steps) |
| Balinese | 7 (unequal, locality-dependent) |

> **Extraction note:** the specific cents values in Figure 25.2 (Balinese metallophone tuning),
> Figure 25.3 (fine-tuned Arabic scale), Figure 25.4 (Raga Bageshri), and Figure 25.5 (harmonic
> series fine-tuned keyboard) are printed as images and did not survive text extraction. Only
> Table 25.1's values, reproduced in full above, are available from the text layer.

## Worked Example
**The chapter's own worked comparison — how far equal temperament is from pure.**

Take the major third above C:
- **Equally tempered E** = 4 semitones × 100 cents = **400 cents**.
- **Pure (just) major third**, ratio **5/4** = **386 cents** (Table 25.1).
- Difference: **14 cents by the table, which Hewitt rounds to "some 16 cents"** in the prose
  comparison of a pure and tempered third.

The same comparison across the three systems, using Table 25.1 for the perfect fifth:
| System | G above C | Cents |
|---|---|---|
| Equal temperament | — | 700 |
| Just intonation | 3/2 | 702 |
| Pythagorean | 3/2 | 702 |

The fifth is nearly identical in all three (2 cents sharp of tempered in both pure systems) — which
is why the fifth is the interval both pure systems build from. The **third** is where they diverge
sharply: just intonation puts E at **386** (14 flat of tempered), Pythagorean at **408** (8 sharp of
tempered) — a 22-cent gap between the two pure systems on a single note.

**Exercise 1**, answered from the chapter's own text: "Intonation concerns the fine points of how a
scale is **tuned**. The 12-tone chromatic scale used in Western music is tuned according to the
system of **equal temperament**. In this system, the octave is divided up into 12 mathematically
equal **semitones**."

**Exercise 2:** "Name three kinds of intonation" — **A. equal temperament, B. just intonation,
C. Pythagorean intonation.**

**Exercise 3:** "For the purposes of fine-tuning, the octave is divided up into **twelve** hundred
cents, in which each equal semitone is worth **100** cents. By providing a fine-tune option on a
synthesizer or sampler, it is possible to adjust each semitone up or down in increments of a single
**cent**."

**Exercise 5:** "**Just** intonation uses pure intervals as defined by their mathematical ratios in
the **harmonic** series — such as perfect fifths of ratio 3:2, major thirds of ratio **5:4**, major
sixths of ratio 5:3, and so on."

**Exercise 4:** "Name three kinds of music in which alternative tunings might be useful" — from the
chapter: **modal music** (it stays in one key), **Balinese/gamelan music**, **Arabian and
Hindustani music**, and **overtone melodies**.

(The book's answer key was a separate download from www.courseptr.com/downloads; the answers above
are read from the chapter's own statements and Table 25.1.)

## Key Takeaways
1. Equal temperament divides the octave into 12 equal semitones — a workable compromise, not a pure system.
2. Just intonation uses pure ratios (5/4, 3/2, 5/3); Pythagorean tunes everything from pure 3/2 fifths.
3. 1200 cents to the octave, 100 per tempered semitone; cent-level fine-tuning is the enabling technology.
4. The tempered major third is 400 cents against a pure 386 — the largest everyday discrepancy.
5. Pure tuning cannot span multiple keys; ~60 microtones would be needed. Modal music, staying in
   one key, is where it works.
6. Arabian scale = 17 notes (Pythagorean); Northern Hindustani raga source scale = 22 notes; Thai =
   seven equal 1/7-octave steps; Balinese = seven unequal, locality-dependent.
7. Overtone melodies (didgeridoo, Jew's harp, Buddhist overtone singing) require harmonic-series
   tuning, not equal temperament.

## Connects To
- [Chapter 1 — Musical Sound](ch01-musical-sound.md): the harmonic series and partials that overtone melodies use.
- [Chapter 8 — Chords](ch08-chords.md): the interval ratios (2:1, 3:2, 4:3, 5:4, 6:5, 5:3, 8:5) that just intonation makes exact.
- [Chapter 19 — The Cycle of Fifths](ch19-cycle-of-fifths.md): the Pythagorean comma and why F# ≠ Gb under pure tuning.
- [Chapter 22 — Exotic Scales](ch22-exotic-scales.md): the Balinese Pelog, Arabian, and Hindu mela scales this chapter tunes.
- [Chapter 20 — The Seven Diatonic Modes](ch20-seven-diatonic-modes.md): modal music, the case where pure tuning is practical.
- [Chapter 16 — The Pentatonic Scale](ch16-pentatonic-scale.md): gamelan and pentatonic traditions with their own tunings.
