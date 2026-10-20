# Chapter 5 — Score Editing

## Core Idea
Software offers two ways to draw and edit notes — **piano roll** or **score edit**. Many computer
musicians prefer the piano roll precisely so they never have to read staff notation. Hewitt's
position: both are equally useful, learn enough score editing to have the option, and note the
practical payoff — "if you want to make remixes from MIDI files — information for these is often
conveyed in conventional musical notation." In score editing, duration is carried by the note
symbols from Chapter 4 and pitch by the note's **position on a staff**.

## Frameworks Introduced

### The Staff and its Clefs
The **staff** is five lines; notes sit both **on** the lines and **in the spaces between** them.
A staff is useless without a **clef**.

| Clef | Alternative name | Why so named | Covers |
|---|---|---|---|
| Treble | **G clef** | Its spiral is drawn around the line on which G above Middle C sits (second line up) | Notes above Middle C |
| Bass | **F clef** | Its symbol coils around the line on which F below Middle C sits | Notes below Middle C |
| Octave-treble | — | Ordinary treble clef with a small **8** at the bottom | Written notes sound an octave higher |
| Octave-bass | — | Bass clef with an **8** added | Same principle for the bass register |

- **Middle C on the treble staff** sits on a **ledger line** below the staff.
- **Middle C on the bass staff** sits on a ledger line immediately **above** the staff.
- Ledger lines supplement the staff instead of drawing whole extra lines, and are used at both
  extremes.
- Alternative (octave) clefs are used "when more notes would be on ledger lines than on actual
  lines of the staff."

### Reading Procedure
1. Locate Middle C on the relevant staff.
2. **Treble:** count up from Middle C following the musical alphabet; at G, return to A.
3. **Bass:** count *down* from Middle C, counting **backward** through the musical alphabet.
4. Shortcut once experienced: orient from the clef's own note — G on the treble, F on the bass —
   rather than counting all the way from Middle C.

### Accidentals on the Staff
There are only individual staff positions for the **white** keys; black keys are handled by signs.
- **Sharp (#)** before a note = play a semitone higher. F# is written as note F with a sharp before it.
- **Flat (b)** before a note = play a semitone lower. Bb is written as note B with a flat before it.
- **Natural sign** cancels a sharp or flat: the note reverts to its original white-key version.

### Rhythmic Notation Conventions
- **Tail direction:** notes **above** the middle line have tails pointing **down**; notes **below**
  the middle line have tails pointing **up**; notes on the middle line may point either way.
- **Beaming:** notes of a common value are connected within the bar. This applies to all notes
  smaller than a quarter note — eighths onward. "Don't worry about how to beam the notes unless
  you intend to produce scores of your own music. The software does it automatically for you."

### Percussion Staffs
- A **single percussion part** (e.g. a snare part) uses a **single-line staff** with the note
  values written on it. Time signature (e.g. 4/4) at the start; the short vertical bar line marks
  each complete cycle of four beats.
- A **complete drum pattern** uses a **percussion clef** with particular drums customarily
  assigned to particular lines, and a specified **note head type per drum** — for example
  **x-shaped note heads for the hi-hats**. Different note heads make drum parts much easier to read.

### The Score-Editing Symbol Menu (Logic)
Hewitt's list of the categories Logic's score editor offers: keyboard pedal signs, dynamic signs,
note values, percussion clefs, symbols, attack marks, time signatures, repeat signs, ornament
signs, BPM values, legato signs, key signatures, expression marks, adding lyrics, rests, and
sectional repeat signs. His guidance: "you will not generally use many of these because they
mostly apply to the process of producing a printed score of your work — a task that goes beyond
the scope of this book."

## Key Concepts
- **Ledger line** — a short supplementary line for notes beyond the staff.
- **Brace** — binds a treble and a bass staff into a **staff system** for keyboard instruments,
  harps, and so on.
- **Register convention** — leads ordinarily use the treble staff; basses use the bass staff.

## Mental Models
- **Two dimensions in one symbol.** In score editing "musical notes are simultaneously represented
  in two ways" — duration by the symbol's shape, pitch by its vertical position.
- **Middle C is the hinge.** It is the anchor for both clefs and the split point between them;
  everything else is counted from it until memorized.
- **The clef names itself.** G clef and F clef literally point at their own reference note — a
  faster orientation than counting from Middle C.

## Anti-patterns
- Avoiding score editing entirely and then being unable to work with notated MIDI-file material
  for remixing.
- Counting *forward* through the alphabet when descending on the bass clef — "it is important to
  remember to count backward."
- Getting bogged down in the full symbol palette, which mostly serves printed-score production.

## Reference Tables

### Clef quick reference
| | Treble (G) clef | Bass (F) clef |
|---|---|---|
| Middle C position | Ledger line **below** the staff | Ledger line **above** the staff |
| Reference note | G above Middle C, second line up | F below Middle C |
| Direction of counting | Up, forward through A–G | Down, backward through G–A |
| Typical parts | Leads | Basses |

### Accidentals
| Symbol | Name | Effect |
|---|---|---|
| # | Sharp | Play a semitone higher |
| b | Flat | Play a semitone lower |
| ♮ | Natural | Cancel a sharp or flat; play the white-key version |

## Worked Example
**Exercise 4:** "A. # is called a ___. B. b is called a ___. C. A # before a note indicates that
___. D. A b before a note indicates that ___."

From the chapter's own definitions:
- A. a **sharp** (sign)
- B. a **flat** (sign)
- C. **the note is to be played a semitone higher**
- D. **the note is to be played a semitone lower**

**Exercise 2** — "Write down both of the names by which the following clefs are known": the
treble clef is also the **G clef**; the bass clef is also the **F clef**.

**Exercise 3** — "Place note Middle C as a whole note in its correct position on the following
staffs": on the treble staff, on a ledger line immediately **below** the five lines; on the bass
staff, on a ledger line immediately **above** them (Figures 5.3 and 5.6).

(The book's answer sheet was a separate download from www.courseptr.com/downloads; the answers
above are read directly from the chapter's own statements and figures.)

## Key Takeaways
1. Score view and piano roll are alternative interfaces to the same MIDI data.
2. A staff needs a clef; treble = G clef for notes above Middle C, bass = F clef for notes below.
3. Middle C sits on a ledger line below the treble staff and above the bass staff.
4. Only white keys have staff positions; sharps, flats, and naturals supply the rest.
5. Tail direction is dictated by the middle line; beaming applies to eighths and shorter and is automatic.
6. Percussion uses a one-line staff for a single part and a percussion clef with per-drum note
   heads (x-heads for hi-hats) for full kits.

## Connects To
- [Chapter 4 — Rhythm, Tempo, and Note Lengths](ch04-rhythm-tempo-note-lengths.md): the duration symbols placed on this staff.
- [Chapter 2 — The Notes](ch02-the-notes.md): sharps, flats, and enharmonics on the keyboard.
- [Chapter 7 — Meter](ch07-meter.md): time signatures and bar lines introduced here in passing.
- [Chapter 1 — Musical Sound](ch01-musical-sound.md): Middle C as the treble/bass split point.
