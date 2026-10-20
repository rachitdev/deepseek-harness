---
name: dsh-music-theory-for-computer-musicians
description: "Michael Hewitt's \"Music Theory for Computer Musicians\" (Course Technology PTR / Cengage, 2008) — a 26-chapter theory course taught through piano roll, MIDI keyboard diagrams, and DAW workflow rather than classical notation. Use when working on scales, keys and key signatures, the cycle of fifths, intervals, triads and seventh/ninth/eleventh/thirteenth chords, chord progressions and root movement, cadences and modulation, the seven diatonic modes, pentatonic and exotic scales, rhythm, meter, additive rhythms, quantization and resolution, arpeggiation, voice leading, chord voicing and spacing, or microtuning and intonation for electronic music production."
---

# Music Theory for Computer Musicians

**Author**: Michael Hewitt (Dr. Mike Hewitt) · **Publisher**: Course Technology PTR, a part of
Cengage Learning, 2008 · ISBN-13 978-1-59863-503-4
**Pages**: 330 · **Chapters**: 26 (plus Appendix A: Scales, Appendix B: Audio CD sidebars)
**Generated**: 2026-08-23

## How to Use This Skill
- **No arguments** — read this file, then `cheatsheet.md` for the dense tables (semitone rule,
  key signatures, tempo bands, tells-and-smells) and `patterns.md` for the reusable moves.
- **With a topic** — check the Topic Index below, open the named chapter files, then check
  `glossary.md` for any unfamiliar term.
- **With a chapter** — open `chapters/chNN-slug.md` directly; each has Core Idea, Frameworks,
  Key Concepts, Mental Models, Anti-patterns, Reference Tables, a Worked Example from the book's
  own exercises, Key Takeaways, and cross-links.
- **Browse** — the Chapter Index is ordered as Hewitt teaches it: sound → notes → scales → rhythm
  → notation → intervals → meter → chords, then alternating harmony and rhythm chapters of
  increasing depth.

## Core Frameworks & Mental Models

**The three parameters of musical tone.** Pitch, intensity, tone quality. Every control in a DAW
manipulates one of them: piano roll and oscillator tuning (pitch), fader and velocity (intensity),
waveform, EQ and filter (timbre). Hewitt's whole book hangs off this taxonomy.

**Harmonics predict harmony.** "The more harmonics the notes share, the better they get on
together." Simpler frequency ratio → more shared harmonics → more concordant. This single physical
fact generates the perfect concords (8ve 2:1, 5th 3:2, 4th 4:3), the imperfect concords (thirds and
sixths), and the discords (seconds and sevenths) — and, in Ch 25, the case for just intonation.
Concord is nonetheless **style-relative**: "when discord becomes the norm, as in jazz, the ear
comes to accept such discords as being relatively stable harmonies."

**Pattern, not letters.** A scale is a shape you slide along the chromatic ladder. Major =
**TTSTTTS (2212221)**; natural minor = **TSTTSTT**. Learn the shape once and every key follows.
Corollary: **key** is the note you start on; **mode** is the shape you apply. Separating those two
words makes the pentatonic, diatonic-modal, and exotic material in Chs 16, 20, and 22 tractable.

**Two thirds, four triads.** A triad is two stacked thirds; a third is major (4 semitones) or minor
(3). Hence the **semitone rule**: dim **3+3**, min **3+4**, maj **4+3**, aug **4+4** — usable from
any note of the chromatic scale, and the fastest identification tool in the book.

**Emphasis makes the tonality.** A seven-note scale yields three major and three minor triads plus
one diminished. Which three you make primary decides whether the music is major or minor. Primary
triads are always on degrees **1, 4, 5**; secondary triads are the rest.

**Leaving home and coming back.** Tonality raises the tonic (I) to home and the dominant (V) to the
active pole, with the subdominant (IV) as counterbalance. Progressions go out and return; the
return is a **cadence** — **V–I perfect** (complete closure), **IV–I plagal** (mellow settling). Add
a seventh to V and the cadence sharpens further: the tritone resolves outward while the root moves
a perfect fifth.

**Root movement is progression.** Three inversions of one chord are not a progression, because the
root never changes. There are exactly three kinds of root movement, and shared-note count predicts
how each will sound: **fourths** (1 shared note, strongest), **thirds** (2, smoothest, least
contrast), **seconds** (0, maximum contrast, least smooth).

**With extended chords, decide what to leave out.** Seventh: keep the third and seventh. Ninth:
keep the seventh and ninth. Eleventh: root, third, seventh, eleventh. Thirteenth: root, seventh,
thirteenth — and drop the third. Drop the fifth first in every case. The permission for all this
freedom is a strong root scheme: "Provided that the root movement scheme is clear, the harmony's
sense of direction will be retained."

**Voicing is a separate decision from chord choice.** Inversion (which chord tone is in the bass),
octave doubling (double root and fifth, not the third), and spacing (closed vs open) are three
independent knobs. Turning them well, and holding over notes that consecutive chords share, is
**voice leading** — the difference between "awkward, blocky, and very unmusical" and "smooth,
musical, and coherent."

**Rotate the tonic to change the mood.** Keep the note set, move the tonic: seven diatonic modes,
five pentatonic modes. Each mode is a familiar scale with one signature deviation — Dorian's major
sixth, Phrygian's flat second, Lydian's sharp fourth, Mixolydian's flat seventh, Locrian's flat
fifth. Modes suit loop-based music because each supplies a fixed atmosphere over a drone tonic.
The corollary rule: **in modal writing, avoid the perfect cadence** — it collapses the mode back
into a key.

**One scale, twelve notes, one tonic.** The **harmonic form of the chromatic scale** holds all
twelve notes as intervals against a fixed tonic, so each carries a modal meaning (Db = the Phrygian
flat second, F# = the Lydian augmented fourth). Hewitt calls it "an artist's palette of color."

**Add, don't divide.** Divisive rhythm subdivides a beat into twos and threes and can never produce
7/8. Additive rhythm chains cells of two and three: 2+3 = quintuple, 3+3+2+3+2 = 13/8. It also
works inside 4/4 (3+3+2 over eight; 3+3+3+3+2+2 over sixteen).

**Rhythm first, pitch second.** Stated for motives (Ch 10) and again for rhythmic arpeggios
(Ch 24). A motive is memorable because of its rhythm; set the rhythm on one note, then add pitches.

**Motion inside stasis.** An arpeggio makes a static chord move. Non-chord tones (passing notes on
weak beats, returning tones) add melodic interest without disturbing the harmony; odd step counts
(5, 7, 11) against an even bar make the accents precess and generate inner melodies for free.

**Equal temperament is a trade, not a truth.** 1200 cents to the octave, 100 per semitone. The
tempered major third is 400 cents against a pure 386. You can retune at cent resolution — but pure
tuning works for **one** key at a time, which is why it pays off in modal music and not in
modulating tonal music.

## Chapter Index

| # | Chapter | File | Focus |
|---|---|---|---|
| 1 | Musical Sound | [ch01](chapters/ch01-musical-sound.md) | Tone vs noise, the three parameters, harmonic series, ADSR, synthesis |
| 2 | The Notes | [ch02](chapters/ch02-the-notes.md) | Locating C, the musical alphabet, sharps/flats, enharmonics, octave numbering |
| 3 | The Major Scale | [ch03](chapters/ch03-the-major-scale.md) | Key vs scale, TTSTTTS, tones and semitones, chromatic scale |
| 4 | Rhythm, Tempo, and Note Lengths | [ch04](chapters/ch04-rhythm-tempo-note-lengths.md) | Note values, dots, rests, tempo by genre, resolution/snap/quantize |
| 5 | Score Editing | [ch05](chapters/ch05-score-editing.md) | Staff, treble/bass clefs, ledger lines, accidentals, percussion staffs |
| 6 | Intervals | [ch06](chapters/ch06-intervals.md) | Melodic vs harmonic, counting rule, simple vs compound, subtract-seven |
| 7 | Meter | [ch07](chapters/ch07-meter.md) | Metric cycles, time signatures, compound time, syncopation, triplets, shuffle, cross-rhythm |
| 8 | Chords | [ch08](chapters/ch08-chords.md) | Concord/discord by ratio, triads, Roman numerals, tonality, primary triads, cadences |
| 9 | The Natural Minor Scale | [ch09](chapters/ch09-natural-minor-scale.md) | Relative minor, key vs mode, minor primary triads, modulation |
| 10 | Melody and Motives | [ch10](chapters/ch10-melody-and-motives.md) | Two axes, motives and phrases, rhythm-first motive writing |
| 11 | The Harmonic and Melodic Minor Scales | [ch11](chapters/ch11-harmonic-melodic-minor.md) | Tetrachords, sharp 7, augmented second, jazz minor, V vs v |
| 12 | Augmented and Diminished Intervals and Interval Inversions | [ch12](chapters/ch12-augmented-diminished-inversions.md) | The third interval family, 28 intervals, the inversion table |
| 13 | Chordal Inversions, Octave Doubling, and Spacing | [ch13](chapters/ch13-inversions-doubling-spacing.md) | Root/1st/2nd position, slash chords, doubling, spacing, voice leading |
| 14 | Additive Rhythms | [ch14](chapters/ch14-additive-rhythms.md) | Divisive vs additive, cells of 2 and 3, odd meters, polyrhythm |
| 15 | Expanding Your Knowledge of Keys | [ch15](chapters/ch15-expanding-keys.md) | Twelve keynotes, key signatures derived from TTSTTTS, transposition |
| 16 | The Pentatonic Scale | [ch16](chapters/ch16-pentatonic-scale.md) | Gapped scale, five modes, added-note and suspended chords, quintal harmony |
| 17 | Major, Minor, Augmented, and Diminished Triads | [ch17](chapters/ch17-four-triad-types.md) | The four triads, the semitone rule, chord symbols |
| 18 | Chord Progressions and Root Movement | [ch18](chapters/ch18-progressions-root-movement.md) | The three root movements and their shared-note signatures |
| 19 | The Cycle of Fifths | [ch19](chapters/ch19-cycle-of-fifths.md) | Sharp/flat sides, closing at F#/Gb, key relatedness, Pythagorean comma |
| 20 | The Seven Diatonic Modes | [ch20](chapters/ch20-seven-diatonic-modes.md) | Rotation of the tonic, all seven modes with chords and repertoire |
| 21 | Chords of the Seventh | [ch21](chapters/ch21-chords-of-the-seventh.md) | Building sevenths, resolving vs saturating dissonance, V7, modulation |
| 22 | Exotic Scales | [ch22](chapters/ch22-exotic-scales.md) | Two kinds of exotic scale, Neapolitan transformation, whole tone, analysis method |
| 23 | Complex Harmony | [ch23](chapters/ch23-complex-harmony.md) | Chain of thirds, ninths/elevenths/thirteenths, chromatic alteration |
| 24 | Arpeggiation | [ch24](chapters/ch24-arpeggiation.md) | Non-chord tones, step counts, five/seven-step patterns |
| 25 | Intonation | [ch25](chapters/ch25-intonation.md) | Equal temperament, just and Pythagorean tuning, cents, world tunings |
| 26 | Conclusion | [ch26](chapters/ch26-conclusion.md) | The book's pedagogical thesis |

## Topic Index

| Topic | Chapters |
|---|---|
| Acoustics, waveforms, synthesis, EQ | 1 |
| Harmonic series, partials, timbre | 1, 20, 22, 25 |
| Note names, keyboard layout, enharmonics | 2, 5 |
| MIDI conventions, velocity, sampler note maps | 1, 2 |
| Major scale, tones and semitones | 3, 15 |
| Minor scales (natural, harmonic, melodic) | 9, 11 |
| Keys, key signatures, transposition | 3, 15, 19 |
| Cycle of fifths, key relatedness | 19 |
| Modes (diatonic) | 20 |
| Pentatonic scale and modes | 16, 22 |
| Exotic, artificial, and world scales | 22, 25 |
| Whole tone scale | 17, 22 |
| Intervals — naming, quality, inversion | 3, 6, 8, 12 |
| Triads — four types, symbols | 8, 17 |
| Seventh chords | 21 |
| Ninth, eleventh, thirteenth chords | 23 |
| Added-note and suspended chords | 16 |
| Chord progressions and root movement | 8, 9, 18 |
| Cadences | 8, 11, 21 |
| Modulation | 9, 19, 21 |
| Inversions, doubling, spacing, voice leading | 13, 21, 23 |
| Rhythm, note values, dots, rests | 4 |
| Tempo and BPM by genre | 4 |
| Meter, time signatures, syncopation | 7 |
| Triplets, shuffle, cross-rhythm | 7 |
| Additive rhythm, odd meters, polyrhythm | 14, 24 |
| Melody, motives, phrases | 7, 10 |
| Arpeggiation, non-chord tones, step patterns | 24 |
| Score notation, clefs, percussion staffs | 5 |
| DAW workflow — resolution, snap, quantize, arpeggiators | 4, 7, 24 |
| Intonation, cents, microtuning | 19, 25 |

## Supporting Files
- [glossary.md](glossary.md) — ~230 terms, alphabetical, with chapter references.
- [patterns.md](patterns.md) — the book's reusable moves as When / How / Trade-offs, grouped into
  learning, rhythm, melody, harmony and voicing, extended harmony, scales and colour, and
  sound and tuning.
- [cheatsheet.md](cheatsheet.md) — fixed values, scale formulas, interval and chord tables, root
  movement and cadence selection matrices, key signatures, tempo bands, additive cell tables,
  the drum MIDI map, tuning tables, a symptom → diagnosis → fix table, and process templates.

## Scope & Limits

**Edition.** First edition, 2008 (Course Technology PTR / Cengage, ISBN-13 978-1-59863-503-4),
330 pages, 26 chapters plus two appendices. Hewitt later wrote companion volumes; none of that
material is here.

**What the book covers.** Western tonal theory taught for producers: acoustics and synthesis
fundamentals, note names, major/minor scales and the whole key system, intervals, triads and
extended chords through the thirteenth, chord progressions and voice leading, rhythm and meter
including additive rhythms, melody and motives, the diatonic modes, pentatonic and exotic scales,
arpeggiation, and microtuning.

**What it does not cover.** Song form and arrangement, mixing and mastering, sound design beyond
basic oscillator/envelope concepts, counterpoint, orchestration, serialism or set theory, and
production technique generally. Asymmetric meters are introduced only through additive rhythm;
Indian raga practice and Arabian maqam are explicitly deferred as requiring "extensive research."

**Dated material.** Software references are from 2008 — Cubase, Logic, SONAR, Reason 4.0,
Digital Performer, Sibelius, Finale, Ableton Live, Korg Triton — and named devices (Reason's Matrix
Pattern Sequencer, RPG-8, Redrum's Shuffle button; SONAR's Event editor; Cubase's micro-tuning page)
have since moved or been renamed. Table 4.1's tempo bands reflect 2008 genre conventions. The theory
itself is not dated.

**Companion media not available.** The book shipped with an **audio CD (52 tracks)**; Appendix B's
text sidebars describing each track are in the text and are cited throughout these chapter files,
but the audio is not. **Chapter exercise answers were a separate download from
www.courseptr.com/downloads and are not printed in the book.** Every "Worked Example" section here
therefore reproduces the book's own exercise and derives the answer from the chapter's own stated
rules, tables, and figures — never from an answer key, and never invented. Where an exercise is an
open composition or sequencing task, it is reproduced as such and labelled.

**Extraction limits.** Text extraction from the PDF was complete for all 26 chapters and for
Appendix B; no chapter is truncated. However, the book teaches heavily through **figures**, and
figure content is images that carry no text layer. Specifically:
- **Appendix A (Scales)** — printed entirely as notation (major, natural/harmonic/melodic minor
  scales with fingerings, the chromatic scale harmonic form, and triads and seventh chords in the
  chromatic scale). Only its headings survived extraction, so no chapter file for it exists and
  its fingering charts are not reproduced.
- **Chapter 22** — the pitch content of Figures 22.2–22.8 and 22.10 (Middle Eastern scales, Eastern
  European scales, Hindu melas, jazz and blues scales, Egyptian/Japanese/Balinese pentatonics,
  artificial scales) is not in the text layer. Only the scale names and Hewitt's prose descriptions
  of their inflections are reproduced; the scales are **not** reconstructed from outside sources.
- **Chapter 25** — the cents values in Figures 25.2–25.5 (Balinese metallophone, Arabic scale,
  Raga Bageshri, harmonic-series keyboard) are likewise images. Table 25.1 is reproduced in full.
- Elsewhere figures illustrate points the prose states explicitly, and the prose is reproduced.
  Where a chapter file gives pitches not printed as text — e.g. the seventh chords of F# major, the
  modes on C — they are **derived by applying the book's own stated procedure** to a scale the book
  itself supplies, with the derivation shown.
