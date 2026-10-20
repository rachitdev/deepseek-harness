# Chapter 14 — Building Sus2 Chords

## Core Idea
A sus2 has a root, a second and a fifth — the sus4's intervals in reverse order: 2 half
steps up to the second, then 5 more to the fifth. Because the second sits right next to
the root, the alphabet check usually resolves itself with no accidental drama.

## Frameworks Introduced

### The Sus2 Formula
```
              sus2
        1        2        5
           (2)      (5)
```

### Alphabet check for a 1-2-5 chord
Root Bb → letters B C D E F. Slots 1, 2, 5 → **B, C, F**. "The chord will contain some
spelling of B, D, and F" is how the book phrases the target letters after the grid; the
grid itself puts C in slot 2 and F in slot 5, which is what the build then confirms.

## Key Concepts
- Root→second: **2 half steps**. Second→fifth: **5 half steps**.
- Total span is the same as sus4 (7 half steps) — only the internal split differs.
- Written `Bbsus2`, `F#sus2`, etc.
- 17 roots in the homework, the same list used for sus4, sus6 and 7sus4.

## Mental Models
- **"Set up the alphabet right away."** Steiner's explicit instruction opening this
  chapter — the letter grid is drawn before any counting.
- **Easy after the hard one.** He again picks a flat root ("not the easiest chord to
  build, but after building this chord you will find most of the rest to be quite a bit
  easier"). This is a recurring pedagogical move across Chapters 10, 13, 14 and 15.

## Anti-patterns
- Counting 5 half steps first and 2 second — that produces a sus4, not a sus2.
- Skipping the alphabet grid because the second "is obviously the next letter." It is
  the next letter, but the accidental still has to be checked.

## Reference Tables

**All sus2 chords (book's answer key, Chapter 14 homework 1)**

| Root | Chord | Root | Chord |
|------|-------|------|-------|
| C | C D G | G | G A D |
| C# | C# D# G# | G# | G# A# D# |
| Db | Db Eb Ab | Ab | Ab Bb Eb |
| D | D E A | A | A B E |
| D# | D# E# A# | A# | A# B# E# |
| Eb | Eb F Bb | Bb | Bb C F |
| E | E F# B | B | B C# F# |
| F | F G C | | |
| F# | F# G# C# | | |
| Gb | Gb Ab Db | | |

Note `A#sus2 = A# B# E#` — every letter takes a sharp; B# and E# are required by the
alphabet check even though they sound like C and F.

**sus2 vs sus4 span**

| Type | Interval 1 | Interval 2 | Total |
|------|-----------|-----------|-------|
| sus2 | 2 | 5 | 7 |
| sus4 | 5 | 2 | 7 |

## Worked Example
*Building a Bbsus2 — the chapter's build.*

Formula: `1 (2) 2 (5) 5`

1. **Alphabet check:** root B → B(1) C(2) D(3) E(4) F(5). Slots 1, 2, 5 give the letters
   **B, C, F**.
2. **Second (2):** from Bb count up 2 half steps → B (1), C (2). "Piece of cake!"
   **Answer: C.**
3. **Fifth (5):** from C count up 5 half steps → C#/Db (1), D (2), D#/Eb (3), E (4),
   F (5). **Answer: F.**

```
             Bbsus2
        1       2       5
        Bb      C       F
```

What this means:
- Play Bb, C and F at the same time and you sound a "Bbsus2 chord."
- If someone asks for a Bbsus2 chord, you play Bb, C and F at the same time.

## Key Takeaways
- sus2 = 1 –(2)– 2 –(5)– 5.
- Same total span as sus4, opposite split.
- Alphabet grid before counting, every time.
- Sharp roots (A#, G#, D#) force B#, E# and similar spellings.

## Connects To
- [Chapter 12 — Introduction to Suspended Chords](ch12-intro-suspended-chords.md)
- [Chapter 13 — Building Sus4 Chords](ch13-building-sus4-chords.md): the mirrored
  formula.
- [Chapter 15 — Building Sus6 Chords](ch15-building-sus6-chords.md): the third sus type.
