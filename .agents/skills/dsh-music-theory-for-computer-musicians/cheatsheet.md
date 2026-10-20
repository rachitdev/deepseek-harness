# Cheatsheet — Music Theory for Computer Musicians (Michael Hewitt)

Dense reference. Every value is the book's own.

---

## Fixed reference values

| Quantity | Value | Ch |
|---|---|---|
| Master tune standard, A above Middle C | **440 Hz** | 1 |
| A below Middle C | 220 Hz | 1 |
| Middle C in MIDI convention | **C3** | 1, 2 |
| Human hearing range | ~20 Hz – 20 kHz | 1 |
| Musical pitch range | a little over 7 octaves | 1 |
| Threshold of hearing | 0 dB | 1 |
| Average conversation | 70 dB | 1 |
| Jet at 200 ft | ~120 dB | 1 |
| MIDI velocity scale | 0–127 (0 none, 64 moderate, 127 max) | 1 |
| Average human heart | ~80 BPM (the calm/excite pivot) | 4 |
| Semitones per octave | 12 | 3 |
| Cents per octave / per semitone | 1200 / 100 | 25 |
| Total intervals in the system | 28 | 12 |
| Diatonic modes × keys | 7 × 12 = 84 | 20 |
| Pentatonic modes × keys | 5 × 12 = 60 | 16 |
| Microtones for pure tuning in all keys | ~60 | 25 |

---

## Scale formulas

| Scale | Pattern | On C / A |
|---|---|---|
| **Major (Ionian)** | T T S T T T S (2212221) | C D E F G A B |
| **Natural minor (Aeolian)** | T S T T S T T | A B C D E F G |
| **Harmonic minor** | natural minor, **♯7** | A B C D E F **G#** |
| **Melodic minor asc.** | natural minor, **♯6 ♯7** | A B C D E **F# G#** |
| **Melodic minor desc.** | = natural minor | A G F E D C B |
| **Pentatonic** | T T m3 T m3 | C D E G A |
| **Whole tone** | T T T T T T | C D E F# G# A# |
| **Chromatic** | all 12 semitones | — |

### The seven diatonic modes
| Mode | White-key tonic | Signature deviation | Use |
|---|---|---|---|
| Ionian | C | = major | Everything tonal |
| Dorian | D | major 6th in a minor context | Dm–G; "House of the Rising Sun," "Mad World" |
| Phrygian | E | **flat 2nd** | Em–F (F = Neapolitan triad); Goa / psy-trance |
| Lydian | F | **sharp 4th** | Euphoric; *The Simpsons* theme |
| Mixolydian | G | **flat 7th** | Folk flavor; G–F–Dm; *Star Wars*, *Star Trek* |
| Aeolian | A | = natural minor | Most rock, pop, dance |
| Locrian | B | **flat 5th**, no tonic triad | Techno, drum & bass, hard trance |

**Derived:** Spanish Phrygian = Phrygian + ♯3 (Flamenco). Harmonic scale = Lydian + ♭7.

### Exotic scale inflections (Ch 22)
| Scale | Definition |
|---|---|
| Hungarian Major | ♯2, ♯4, ♭7 |
| Hungarian Minor (Gypsy scale) | harmonic minor + ♯4 |
| Gypsy I | natural minor + ♯4 |
| Gypsy II | major + ♭2, ♭6 |
| Neapolitan (any) | any mode + ♭2 |
| Ancient Egyptian pentatonic | pentatonic mode 4 + ♭2, ♭5 (hemitonic) |
| Blues (hexatonic/7-note/9-note) | blue notes Eb, F#, Bb |

---

## Interval reference

### Semitone counts and qualities (Table 8.2)
| Interval | Mode | Quality | Semitones |
|---|---|---|---|
| First | Perfect | Perfect concord | 0 |
| Minor second | Imperfect | Discord (sharp) | 1 |
| Major second | Imperfect | Discord (mild) | 2 |
| Minor third | Imperfect | Concord | 3 |
| Major third | Imperfect | Concord | 4 |
| Perfect fourth | Perfect | Perfect concord | 5 |
| Augmented fourth | Chromatic | Discord (sharp) | 6 |
| Perfect fifth | Perfect | Perfect concord | 7 |
| Minor sixth | Imperfect | Concord | 8 |
| Major sixth | Imperfect | Concord | 9 |
| Minor seventh | Imperfect | Discord (mild) | 10 |
| Major seventh | Imperfect | Discord (sharp) | 11 |
| Octave | Perfect | Perfect concord | 12 |

### Ratios
| Interval | Ratio |
|---|---|
| Octave | 2:1 |
| Perfect fifth | 3:2 |
| Perfect fourth | 4:3 |
| Major third | 5:4 |
| Minor third | 6:5 |
| Major sixth | 5:3 |
| Minor sixth | 8:5 |

### Rules
- **Counting:** include the starting note. C D E F G = fifth.
- **Compound → simple:** subtract 7, repeatedly. 24 → 17 → 10 → 3 (a compound third).
- **Augment:** only perfect and **major** intervals. **Diminish:** only perfect and **minor**.
- **Inversion:** numbers sum to 9; major↔minor, augmented↔diminished, perfect↔perfect.
- **Forms available:** imperfect intervals 4 (dim/min/maj/aug); perfect intervals 3 (dim/perf/aug).

---

## Chord reference

### The semitone rule (Ch 17)
| Triad | Formula | Symbol |
|---|---|---|
| Diminished | **3 + 3** | C° / Cdim |
| Minor | **3 + 4** | Cm |
| Major | **4 + 3** | C |
| Augmented | **4 + 4** | C+ |

### Triads by scale degree
| Scale | I | ii | iii | IV | V | vi | vii |
|---|---|---|---|---|---|---|---|
| **C major** | C | Dm | Em | F | G | Am | B° |
| **A natural minor** | Am (i) | B° (ii) | C (III) | Dm (iv) | Em (v) | F (VI) | G (VII) |
| **Harmonic minor** | minor | **dim** | **aug** | minor | **major** | major | **dim** |
| **Melodic minor asc.** | — | — | **aug** | **aug** | — | — | — |

Harmonic minor common triads: **I, IV, V, VI**. Diminished: **II, VII**. Augmented: **III**.

### Seventh chords in C major
| Degree | Chord | Notes |
|---|---|---|
| I | Cmaj7 | C E G B |
| ii | Dm7 | D F A C |
| iii | Em7 | E G B D |
| IV | Fmaj7 | F A C E |
| V | **G7** (dominant) | G B D F |
| vi | Am7 | A C E G |
| vii | Bm7♭5 | B D F A |

**Symbols:** Cmaj7 = major triad + major 7th · C7 = major triad + minor 7th · Cm7 = minor triad +
minor 7th · Cm/maj7 = minor triad + major 7th.

### Extended chords — what to keep
| Chord | Notes | Essential members | Drop first |
|---|---|---|---|
| Triad | 3 | root, third, fifth | — |
| Seventh | 4 | **third, seventh** | fifth |
| Ninth | 5 | **seventh, ninth** | fifth |
| Eleventh | 6 | **root, third, seventh, eleventh** | fifth, ninth |
| Thirteenth | 7 | **root, seventh, thirteenth** | **third** (clashes with 11th) |

### Positions
| Notes in chord | Positions | Slash notation |
|---|---|---|
| 3 (triad) | root, 1st, 2nd | C, C/E, C/G |
| 4 (seventh) | root, 1st, 2nd, 3rd | Cmaj7, /E, /G, /B |
| 5 (ninth) | root + 4 inversions | — |

**Procedure:** to advance one inversion, move the lowest note up an octave.

### Doubling and spacing
| Note | Double? | Why |
|---|---|---|
| Root | Best | Identity |
| Fifth | Best | Backbone |
| Third | Sparingly | The "sweet note"; carries major/minor |

Closed spacing = crowded. Open spacing = wide spread.

---

## Root movement selection matrix (Ch 18)

| Movement | Shared notes | Smoothness | Contrast | Verdict |
|---|---|---|---|---|
| **Fourths** | 1 | good | good | **Strongest**; use sequentially (cycle of fourths) |
| **Thirds** | 2 | highest | lowest | Smoothest connection |
| **Seconds** | 0 | lowest | highest | Maximum contrast |

Inversions of one chord = **no root movement = no progression**.

---

## Cadence selection

| Want | Use | Notes |
|---|---|---|
| Complete, unambiguous closure | **V – I** (perfect) | Strengthen further with **V7 – I**: tritone resolves outward + perfect fifth root move |
| Mellow settling | **IV – I** (plagal / Amen) | Bob Marley, "No Woman No Cry" |
| Decisive close in a minor key | **V (major) – i** | Requires harmonic minor's ♯7 |
| Mellow close in a minor key | **v (minor) – i** | Natural minor |
| Shift major → relative minor | **vii – vi** | Leading tone triad resolving to vi (Track 39) |
| Modal writing | **avoid any perfect cadence** | It collapses the mode into a key |

**Modulation:** target the **dominant seventh of the new key**, not its tonic.

---

## Key signatures and the cycle of fifths

| Sharps | Major key | Relative minor | Flats | Major key | Relative minor |
|---|---|---|---|---|---|
| 0 | C | A m | 0 | C | A m |
| 1 (F#) | G | E m | 1 (Bb) | F | D m |
| 2 (+C#) | D | B m | 2 (+Eb) | Bb | G m |
| 3 (+G#) | A | F# m | 3 (+Ab) | Eb | C m |
| 4 (+D#) | E | C# m | 4 (+Db) | Ab | F m |
| 5 (+A#) | B | G# m | 5 (+Gb) | Db | Bb m |
| 6 (+E#) | **F#** | D# m | 6 (+Cb) | **Gb** | Eb m |

- Up a fifth = +1 sharp. Down a fifth = +1 flat.
- **F# major = Gb major** (enharmonic; the cycle closes). Under pure tuning they differ by the
  **Pythagorean comma, ~1/5 semitone**.
- Relative minor = **down 3 notes (or up 6)** from the major tonic; same key signature.
- Closest keys = circle neighbours (most shared notes). Most distant from C = **Gb**.
- Sharps read bright, flats read dark, **C is the pivot**.

---

## Rhythm and meter

### Note values
| Note | Fraction | Flags | Per 4/4 bar | Rest exists |
|---|---|---|---|---|
| Whole | 1 | — | 1 | yes |
| Half | 1/2 | — | 2 | yes |
| Quarter | 1/4 | — | 4 | yes |
| Eighth | 1/8 | 1 | 8 | yes |
| 16th | 1/16 | 2 | 16 | yes |
| 32nd | 1/32 | 3 | 32 | yes |
| 64th | 1/64 | 4 | 64 | yes |

**Dot** = +half the note's value. Dotted half = 3 quarters. Dotted quarter = 3 eighths.
Dotted whole = 6 quarters. Dotted 16th = 3 thirty-seconds.

### Tempo by genre (Table 4.1)
| Genre | BPM |
|---|---|
| Ambient | 50–100 |
| Hip-hop | 70–95 |
| Deep house | 110–130 |
| Trance | 130–145 |
| Hard dance / hardcore | 145–170 |
| Drum and bass | 160–180 |

### Meters
| Meter | Stress | Compound form | Beats |
|---|---|---|---|
| Duple | s w | 6/8 | 6 |
| Triple | s w m | 9/8 | 9 |
| Quadruple | s w m w | 12/8 | 12 |

### Additive cells → meters
| Grouping | Total |
|---|---|
| 2+2 | 4 |
| 2+3 / 3+2 | 5 (quintuple) |
| 3+3 | 6 |
| 2+2+3 / 2+3+2 / 3+2+2 | 7 |
| 2+3+3 / 3+2+3 / 3+3+2 | 8 |
| 3+3+3 | 9 |
| 3+3+3+2 (and rotations) | 11 |
| 3+3+2+3+2 | 13 |
| 3+3+3+2+2+2 | 15 |
| Inside 4/4: 3+3+2 | 8 steps |
| Inside 4/4: 3+3+3+3+2+2 | 16 steps |

Cells: 2 → 4 combinations · 3 → 8 · 4 → 16 · 5 → 32.

### Triplet naming by DAW
| Software | Eighth triplets | Sixteenth triplets |
|---|---|---|
| **Logic** | "twelfths" | "twenty-fourths" |
| **Cakewalk SONAR** | "triplets" | "triplets" |
| **Reason** | 1/8T | 1/16T (+ dedicated Shuffle button on Redrum) |

### Arpeggio step counts
| Steps | Effect |
|---|---|
| 4, 8, 16, 32 | In sync with the bar |
| **5, 7, 11** | Out of sync; accents precess, generating inner melodies |

---

## Drum kit MIDI map (the book's own, Ch 2 Ex. 10)

| Note | Sound | Note | Sound |
|---|---|---|---|
| C3 | Kick drum | C#4 | Crash cymbal |
| D3 | Snare | D#4 | Ride cymbal |
| D#3 | Clap | F#4 | Tambourine |
| F#3 | Closed hi-hat | G#4 | Vocal hit |
| A#3 | Open hi-hat | | |

**Basic drum template (Appendix B, Track 8):** bass drum on **1 and 3**, snare on **2 and 4** (the
**backbeat**). "Although capable of infinite variance, this cycle provides the basic template for
most drum patterns." Faster pace → hi-hats on sixteenths (Track 9). Building excitement → kick
moves through quarters → eighths → sixteenths → 32nds → 64ths (Track 10).

---

## Tuning (Table 25.1, cents)

| Note | Equal | Just (ratio / cents) | Pythagorean (ratio / cents) |
|---|---|---|---|
| C | 0 | 1/1 · 0 | 1/1 · 0 |
| Db | 100 | 16/15 · 112 | 256/243 · 90 |
| D | 200 | 9/8 · 204 | 9/8 · 204 |
| Eb | 300 | 6/5 · 316 | 32/27 · 294 |
| E | 400 | 5/4 · **386** | 81/64 · **408** |
| F | 500 | 4/3 · 498 | 4/3 · 498 |
| F# | 600 | 45/32 · 590 | 729/512 · 612 |
| G | 700 | 3/2 · 702 | 3/2 · 702 |
| Ab | 800 | 8/5 · 814 | 128/81 · 792 |
| A | 900 | 5/3 · 884 | 27/16 · 906 |
| Bb | 1000 | 9/5 · 1018 | 16/9 · 996 |
| B | 1100 | 15/8 · 1088 | 243/128 · 1110 |
| C | 1200 | 2/1 · 1200 | 2/1 · 1200 |

| System | Notes/octave |
|---|---|
| Western equal temperament | 12 |
| Arabian (Pythagorean-based) | 17 |
| N. Hindustani raga source scale | 22 |
| Thai | 7 equal (1/7 octave each) |
| Balinese | 7 unequal, locality-dependent |

**Harmonic series detuning (Appendix B, Track 5):** 3rd harmonic **+2 cents**, 5th **−14 cents**,
7th **−31 cents**.

---

## Tells and smells — symptom → diagnosis → fix

| Symptom | Diagnosis | Fix |
|---|---|---|
| MIDI part sounds rigid and mechanical | Over-quantized, uniform velocity | Reduce quantize; vary velocity across 0–127; add subtle panning (Ch 1, 4) |
| Chord sounds bare and empty | Third is missing | Restore the third — the "sweet note" (Ch 13) |
| Chord has richness but no backbone | Third alone, no fifth | Add the fifth (Ch 13) |
| Progression sounds blocky and disjointed | Ignored inversions, doubling, spacing | Choose inversions that share notes with the neighbouring chord; double root and fifth; score four parts (Ch 13) |
| "Progression" has no forward movement | Root is not changing (inversions only) | Change the root (Ch 18) |
| Dense harmony sounds like mush | Weak or unclear root scheme | Fix a clear root movement scheme first, then colour (Ch 18, 23) |
| Extended chord is muddy | Too many notes retained | Trim to the chord's essential members (see table above) |
| Eleventh chord clashes internally | Natural 11th against the third | Sharpen the eleventh (Lydian) or drop the third (Ch 23) |
| Seventh sounds like a mistake | Unresolved in a context with few sevenths | Resolve stepwise downward to a concord, or saturate the texture with sevenths (Ch 21) |
| Minor-key ending sounds unfinished | Using minor v | Substitute major V (harmonic minor's ♯7) (Ch 11) |
| Key change didn't land | Aimed at the new tonic without a cadence | Target the **dominant seventh of the new key**; the ear needs a perfect cadence there (Ch 21) |
| Modal piece keeps sounding "in a key" | A perfect cadence has crept in | Remove it; use a drone and the mode's characteristic progression instead (Ch 21) |
| Locrian progression won't resolve | No proper tonic triad (B D F is diminished) | Don't seek tonal resolution; exploit the B–F tritone axis as a dark bass feature (Ch 20) |
| Diminished chords sound comic | Silent-movie cliché from overuse | Use with "extreme caution"; consider the diminished triad as a dominant substitute instead (Ch 17) |
| Sound is too bright / too dull | Upper harmonic balance | EQ: attenuate upper harmonics to brighten, suppress to dull (Ch 1) |
| Synth imitation sounds unconvincing | Envelope ignored | Emulate attack, decay, sustain, release as well as harmonic content (Ch 1) |
| Shuffle won't come out on the drum machine | Wrong resolution | Set triplet resolution (or use Redrum's Shuffle button) (Ch 7) |
| Arpeggio feels static across bars | Even step count locked to the bar | Use 5, 7, or 11 steps (Ch 24) |
| Track feels stuck in a rut | Same scale, same meter, same triads | Change mode, use additive rhythm, add pentatonic added-note chords, or adopt an exotic scale (Ch 14, 16, 22) |

---

## Process templates

### Learning a new key (Ch 15)
1. Play the scale through. 2. Define its seven triads. 3. Transpose progressions you already have
into it.

### Learning a new scale (Ch 11)
1. Play it up and down until fluent (Appendix A has fingerings). 2. Learn its chords as arpeggios.
3. Build three-chord progressions from the primary chords. 4. Add secondary chords and hear
substitutions.

### Building a chord vocabulary in a key (Ch 18)
1. Play the scale. 2. Take each degree as a root, add a third and a fifth, staying in key → seven
triads. 3. Play those seven in all three root-movement orders (seconds, thirds, fourths).

### Writing a motive (Ch 10, 24)
1. Rhythm first, on a single note. 2. Ask: rising or falling? major or minor? leaping or stepwise?
3. Choose intervals to answer. 4. Pair two statements as question and answer → a phrase.

### Building an arpeggio (Ch 24)
1. Lay down the harmony to be arpeggiated. 2. Choose a step count (4/8/16/32 for lock, 5/7/11 for
precession). 3. Add non-chord tones — passing notes on weak beats between chord tones a third
apart; returning tones a step away and back. 4. For rhythmic arpeggios, set the rhythm on one note
before adding pitches.

### Adopting an exotic scale (Ch 22)
1. List the interval of each note against the tonic. 2. Find the conspicuous triads. 3. Identify
the signature two- or three-note melodic groups. 4. Set a drone on the tonic and write melodically.

### Building an extended chord (Ch 21, 23)
1. Build the triad. 2. Add the seventh above the root, staying in scale. 3. Add further thirds for
ninth / eleventh / thirteenth. 4. Trim to essential members. 5. Chromatically alter if wanted,
keeping the third and seventh intact on dominants.
