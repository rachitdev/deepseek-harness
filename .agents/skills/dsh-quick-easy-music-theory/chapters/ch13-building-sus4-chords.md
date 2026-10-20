# Chapter 13 — Building Sus4 Chords

## Core Idea
The sus4 is "the most common suspended chord" and the only sus chord Steiner extends
with a seventh later. Build it exactly like a triad: alphabet check to fix the letters,
then count 5 half steps to the fourth and 2 more to the fifth.

## Frameworks Introduced

### The Sus4 Formula
```
              sus4
        1        4        5
           (5)      (2)
```

### Five-slot alphabet check for a 1-4-5 chord
Root Eb → letters E F G A B. Slots 1, 4, 5 → **E, A, B**. Every note in the chord must
be some form of those three letters.

## Key Concepts
- Root→fourth: **5 half steps**. Fourth→fifth: **2 half steps**.
- Sound description: "It sounds rather nice, kind of like clouds parting the way to
  heaven, like dreams coming true, like love blossoming in the spring air…"
- The chord is written `Ebsus4`, `Gbsus4` etc. — letter + `sus` + `4` (Ch 24).
- The homework covers 17 roots: C, C#, Db, D, D#, Eb, E, F, F#, Gb, G, G#, Ab, A, A#,
  Bb, and B.

## Mental Models
- **Same machinery, new numbers.** By this point in the book the procedure is fully
  routine: letters first, then count. Steiner says the reader "would absolutely know"
  the right spelling "through much experience building chords."
- **Start hard.** He picks Ebsus4 deliberately — "I'm throwing you in the fire once
  again" — because a flat root plus a 5-half-step jump forces two spelling decisions.

## Anti-patterns
- Spelling the fourth of Ebsus4 as G# — the letter G is not in the chord (slots are
  E, A, B).
- Spelling the fifth as A# — that would use the letter A twice.
- Measuring the fifth from the root instead of from the fourth.

## Reference Tables

**All sus4 chords (book's answer key, Chapter 13 homework 1)**

| Root | Chord | Root | Chord |
|------|-------|------|-------|
| C | C F G | G | G C D |
| C# | C# F# G# | G# | G# C# D# |
| Db | Db Gb Ab | Ab | Ab Db Eb |
| D | D G A | A | A D E |
| D# | D# G# A# | A# | A# D# E# |
| Eb | Eb Ab Bb | Bb | Bb Eb F |
| E | E A B | B | B E F# |
| F | F Bb C | | |
| F# | F# B C# | | |
| Gb | Gb Cb Db | | |

Note `Gbsus4 = Gb Cb Db` — the fourth must be a form of C, so B is written Cb.

## Worked Example
*Building an Ebsus4 — the chapter's build.*

Formula: `1 (5) 4 (2) 5`

1. **Alphabet check:** root Eb → E(1) F(2) G(3) A(4) B(5). "The root (1) letter fell on
   an E. So the fourth (4) and fifth (5) letters would have to be A and B."
2. **Fourth (4):** from Eb count up 5 half steps → E (1), F (2), F#/Gb (3), G (4),
   G#/Ab (5). Black key. It must be **Ab**, "because the letter G does not exist in
   this chord."
3. **Fifth (5):** from Ab count up 2 half steps → A (1), A#/Bb (2). Must be **Bb**,
   "because you cannot use the letter A twice."

```
             Ebsus4
        1       4       5
        Eb      Ab      Bb
```

What this means:
- Play Eb, Ab and Bb at the same time and you sound an "Eb sus4 chord."
- If someone asks for an Ebsus4, you play Eb, Ab and Bb at the same time.

## Key Takeaways
- sus4 = 1 –(5)– 4 –(2)– 5.
- Most common of the three sus types.
- Letters come from slots 1, 4, 5 of the alphabet check.
- 17 roots in the homework; Gbsus4 needs Cb.

## Connects To
- [Chapter 12 — Introduction to Suspended Chords](ch12-intro-suspended-chords.md): the
  family rule (no third).
- [Chapter 14 — Building Sus2 Chords](ch14-building-sus2-chords.md): the reversed
  intervals.
- [Chapter 16 — Building Suspended Seventh Chords](ch16-building-sus-seventh-chords.md):
  the only sus chord Steiner extends.
- [Chapter 24 — Chord Abbreviations](ch24-chord-abbreviations.md): how `sus4` is written.
