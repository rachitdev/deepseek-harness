# Chapter 23 — Reading Figured Bass

## Core Idea
Figured bass is "an ultra fast form of communication": name a key, write a few Roman
numerals, and another musician knows exactly what to play. It compresses an entire
chord progression into a line of text — but it is meaningless without the key, because
the same numerals produce completely different chords in different keys.

## Frameworks Introduced

### The format
```
    <Key>:    I    vi    iii    IV    V    I
```
Key first, then numerals in playing order. Case and the º sign carry the chord quality
(Ch 18).

### Two ways of saying the same thing
Steiner contrasts figured bass with what he calls the **"Mommy, please hold my hand"**
method — spelling every chord out longhand:

> "First, play an E major chord (E – G# – B), then a C# minor chord (C# – E – G#), then a
> G# minor chord (G# – B – D#), then an A major chord (A – C# – E), and then a B major
> chord (B – D# – F#), and finally an E major chord (E – G# – B). Good job, Junior."

Then the punchline: "Unfortunately, Mommy can't come to band practice."

### The transposition property
"Every time you change key, you must re-write your figured bass... However, the
integrity of the chord progression will not change. The progression will sound the
same; it simply starts in a different place."

## Key Concepts
- **Figured Bass** — a way to write music based on numbers that refer to a specific
  chord in a scale.
- Prerequisite: "excellent knowledge of all major and minor scales, as well as the
  diatonic triads contained within them."
- "Without the key, the roman numerals will mean absolutely nothing."
- Real-world usage: one musician says "Play a one, four, five in the key of G major" and
  both players produce the same thing.

## Mental Models
- **Numerals are relative; letters are absolute.** The numeral encodes the *function*;
  the key supplies the *pitches*. This is what makes transposition a mechanical lookup.
- **The lookup table is the harmonisation grid.** Reading figured bass is just indexing
  into the Chapter 18/19/21 tables.
- **Study cost pays off.** "This probably took both of them quite some time to study and
  practice, but the results of that will produce a top-notch musician."

## Anti-patterns
- Writing numerals without stating the key.
- Longhand chord-by-chord dictation ("Mommy, please hold my hand") when a numeral line
  would do.
- Assuming a figured bass line survives a key change unedited. The numerals stay; the
  chord letters must be rewritten.

## Reference Tables

**E major diatonic triads (the chapter's lookup grid)**

| | I | ii | iii | IV | V | vi | viiº |
|--|---|----|-----|----|---|----|------|
| 5 | B | C# | D# | E | F# | G# | A |
| 3 | G# | A | B | C# | D# | E | F# |
| 1 | E | F# | G# | A | B | C# | D# |

**Bb major diatonic triads (the chapter's transposition grid)**

| | I | ii | iii | IV | V | vi | viiº |
|--|---|----|-----|----|---|----|------|
| 5 | F | G | A | Bb | C | D | Eb |
| 3 | D | Eb | F | G | A | Bb | C |
| 1 | Bb | C | D | Eb | F | G | A |

**The same progression, two keys**

| Numerals | I | vi | iii | IV | V | I |
|----------|---|----|-----|----|---|---|
| E major | E major | C# minor | G# minor | A major | B major | E major |
| Bb major | Bb major | G minor | D minor | Eb major | F major | Bb major |

## Worked Example
*The chapter's four homework progressions, with the book's answer key. Each row of the
answer tables is a voice; read a column top-to-bottom as one chord.*

**1. `I V vi ii IV I` in D major**

| Numeral | I | V | vi | ii | IV | I |
|---------|---|---|----|----|----|---|
| 5 | A | E | G | B | D | A |
| 3 | F# | C# | E | G | B | F# |
| 1 | D | A | C# | E | G | D |

Chords: D major, A major, C# minor, E minor, G major, D major.

**2. The same progression in Ab major**

| Numeral | I | V | vi | ii | IV | I |
|---------|---|---|----|----|----|---|
| 5 | Eb | Bb | G | F | Ab | Eb |
| 3 | C | G | Eb | Db | F | C |
| 1 | Ab | Eb | C | Bb | Db | Ab |

Chords: Ab major, Eb major, C minor, Bb minor, Db major, Ab major.

**3. `i VI iv VII V i` in A harmonic minor**

| Numeral | i | VI | iv | VII | V | i |
|---------|---|----|----|-----|---|---|
| 5 | E | C | A | D | B | E |
| 3 | C | A | F | B | G# | C |
| 1 | A | F | D | G | E | A |

Chords: A minor, F major, D minor, G major, **E major** (the raised third G# — see
Chapter 21), A minor.

**4. The same progression in G harmonic minor**

| Numeral | i | VI | iv | VII | V | i |
|---------|---|----|----|-----|---|---|
| 5 | D | Bb | G | C | A | D |
| 3 | Bb | G | Eb | A | F# | Bb |
| 1 | G | Eb | C | F | D | G |

Chords: G minor, Eb major, C minor, F major, **D major** (raised third F#), G minor.

## Key Takeaways
- Format: key, then Roman numerals in order.
- No key = no meaning.
- Numerals are portable; chord letters are not — rewrite on transposition.
- Reading figured bass = indexing the diatonic-triad grid for the stated key.
- In harmonic minor keys the V is major (raised third).

## Connects To
- [Chapter 18 — Diatonic Triads (Major)](ch18-diatonic-triads-major.md): where the
  numerals come from.
- [Chapter 21 — Diatonic Triads (Harmonic Minor)](ch21-diatonic-triads-harmonic-minor.md):
  the minor V used in homework 3 and 4.
- [Chapter 24 — Chord Abbreviations](ch24-chord-abbreviations.md): the more popular
  alternative.
- [Chapter 25 — Chord Inversions](ch25-chord-inversions.md): adding 6 and 64 to these
  numerals.
