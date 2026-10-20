# Chapter 1 — Musical Sound

## Core Idea
Before any theory of notes, scales, or chords can be useful, the computer musician has to know
what a *musical* sound actually is. Hewitt's answer: music is distinguished from noise by the
presence of **tone** — regular, ordered, periodic sound waves, as opposed to the "chaotic,
jumbled, and confused" waves of noise. Every musical tone has exactly three parameters —
**pitch, intensity, and tone quality** — and every knob, fader, and graph in a DAW is ultimately
manipulating one of those three.

## Frameworks Introduced

### The Three Parameters of Musical Tone
| Parameter | Physical cause | Studio equivalent |
|---|---|---|
| Pitch (frequency) | Wavelength / cycles per second | Note on the piano roll; oscillator tuning |
| Intensity (amplitude) | Wave height | Mixer fader (track); velocity (note) |
| Tone quality (timbre/waveform) | Harmonic content | Oscillator waveform, EQ, filters |

Track 2 on the book's CD presents one chord transformed three ways — pitch, then intensity,
then timbre — as an ear-training demonstration of exactly this triad of parameters.

### The Harmonic Series
A plucked string vibrates along its whole length *and* along its fractional lengths (halves,
thirds, quarters, fifths…). Each fractional length is a **mode of vibration** producing its own
frequency, called a **partial**.
- First partial = **fundamental frequency** = first harmonic. This determines the pitch we hear.
- Partials whose frequencies are whole-number multiples of the fundamental are **harmonics**;
  the succession 100, 200, 300, 400, 500 Hz… is a **harmonic series**.
- Partials that are *not* whole-number multiples are **inharmonic partials** — gongs, bells, and
  most untuned percussion — and give rise to sounds of indefinite pitch.
- Subsequent partials are much fainter than the fundamental but determine timbre. Hewitt's two
  analogies: pixels blending into one image, and an atomic nucleus (fundamental) surrounded by
  an electron cloud (harmonics).

### The Sound Envelope (ADSR)
Four components of how a sound develops through time:
- **Attack** — sound rises from an initial low to a peak value.
- **Decay** — the fade from the peak. A piano's attack is immediate then it starts fading; an
  organ has *no* decay.
- **Sustain** — the period during which the peak value is maintained.
- **Release** — the time it takes for the sound to disappear once the note is released.

### Synthesis
The **oscillator** is the sound-producing component: it generates both frequency and harmonic
content. Four basic waveforms and their uses:
| Waveform | Harmonic content | Characteristic use |
|---|---|---|
| Sine | One specific frequency only | Purest tone |
| Square | Emphasizes odd-numbered harmonics | Hollow sound; "a real treat in the basses of many different types of dance music" |
| Triangle | A few specific odd-numbered partials | Clear note; good for imitating flutes |
| Sawtooth | Extremely rich in harmonics | Strings, brass, "beautifully rich leads characteristic of trance music" |

- **Additive synthesis** — obtain the required waveform by adding harmonic waves to a given fundamental.
- **Subtractive synthesis** — begin with a harmonically rich waveform and selectively filter frequencies out.

## Key Concepts
- **Hertz (Hz)** — sound waves per second; thousands = kHz.
- **Range of human hearing** — about 20 Hz to 20 kHz. Below is **sub-audible** (elephants, male
  alligators communicate here); above is **super-audible** (dog whistles, bat pulses).
- **Musical range** — a little more than seven octaves, the range of a concert grand piano. Each
  instrument's slice of it is its **characteristic register**.
- **Master tuning standard** — note A above Middle C = **440 Hz**, universally adopted for both
  standard and electronic instruments. Deliberate mistuning gives the honky-tonk piano sound.
- **Middle C** — the universally recognized central point of the pitch register; the white key to
  the left of the two black keys nearest the lock on a piano. In notation editors it is the split
  point: Middle C and above go on the treble clef, below it on the bass clef. In MIDI convention
  Middle C is usually **C3**, the beginning of the third octave.
- **Decibels (dB)** — 0 dB is the quietest sound the human ear can pick up; average conversation
  is 70 dB; a jet taking off 200 feet away is about 120 dB.
- **Velocity** — the intensity of an individual note or drum hit, on a scale of **0 to 127**
  (0 = none, 64 = moderate, 127 = maximum). Velocity variation is what gives MIDI parts realism.
- **Psychoacoustics** — the study of how we hear and perceive these vibrations.
- **Chladni figures** — sand on bowed metal plates forms regular geometric patterns; Hewitt's
  physical demonstration that order, not chaos, is the signature of the musical.

## Mental Models
- **Order versus chaos.** Look at a waveform: erratic and unpatterned = noise; regular peaks and
  troughs with uniform phase distances = musical tone. Periodicity *is* musicality.
- **A tone is a galaxy, not a point.** What sounds like one event is "a highly complex blend of
  vibrations" extending over numerous octaves.
- **Harmonics predict harmony.** "The more harmonics the notes share, the better they get on
  together — a bit like common interests in human relationships." This one sentence links
  Chapter 1 to the whole theory of concord and discord in Chapter 8.
- **EQ is harmonic sculpting.** Boosting upper harmonics brightens; suppressing them dulls. "The
  skill of successful equalization lies in getting the balance just right."

## Anti-patterns
- Treating noise as sufficient: "If music used nothing but noises, its appeal to an audience
  would be much more limited." Percussive noise works only "within an intelligible rhythmic framework."
- Skipping note names: without them "it is more difficult to understand, retain, and utilize the
  complex information about scales, chords, keys, modes, and so on that will follow."
- Assuming synthesis is only harmonic content — ignoring the envelope produces unrealistic imitations.
- Treating electronic music as exempt from musicianship: "whatever type of music you create using
  electronic instruments, it is still music."

## Reference Tables

### Table 1.1 — Harmonic Series: The First Eight Harmonics of A
| Harmonic | Note | Frequency | Ratio |
|---|---|---|---|
| First (Fundamental) | A1 | 110 Hz | 1 |
| Second | A2 | 220 Hz | 2 |
| Third | E3 | 330 Hz | 3 |
| Fourth | A3 | 440 Hz | 4 |
| Fifth | C#4 | 550 Hz | 5 |
| Sixth | E4 | 660 Hz | 6 |
| Seventh | Fx4 (F double-sharp) | 770 Hz | 7 |
| Eighth | A4 | 880 Hz | 8 |

Ratio between first and second harmonics = 1:2; between third and fifth = 3:5. The series
"goes on ad infinitum." Audio Track 5 presents these harmonics in direct succession — and Appendix B
adds a practical detuning note: collectively they sound like a dominant seventh chord on A, but to
duplicate the harmonic series accurately the oscillators producing the **third harmonic need +2
cents, the fifth harmonic −14 cents, and the seventh harmonic −31 cents**.

### Pitch and loudness reference points
| Quantity | Value |
|---|---|
| Human hearing range | ~20 Hz – 20 kHz |
| Master tune standard (A above Middle C) | 440 Hz |
| A below Middle C | 220 Hz |
| Musical pitch range | a little over 7 octaves (concert grand) |
| Threshold of hearing | 0 dB |
| Average conversation | 70 dB |
| Jet at 200 feet | ~120 dB |
| MIDI velocity scale | 0–127 (64 = moderate) |
| Middle C in MIDI convention | C3 |

## Worked Example
**Exercise 3 (chapter exercises):** "If the frequency of the first harmonic is given as 220 Hz,
what are the frequencies of the following harmonics? Second harmonic ___ ; Fifth harmonic ___."

Worked from Table 1.1's own rule — the *n*th harmonic has *n* times the fundamental frequency:
- Second harmonic = 220 × 2 = **440 Hz**
- Fifth harmonic = 220 × 5 = **1100 Hz**

**Exercise 11** applies the same rule to a fundamental of 64 Hz shown as mode of vibration A,
with B, C, D being the second, third, and fourth modes: 128 Hz, 192 Hz, 256 Hz.

**Exercise 8:** "The master tune standard for Western electronic instruments is: A. 256 Hz
B. 332 Hz C. 192 Hz D. 440 Hz E. 368 Hz." The chapter states it directly — **D, 440 Hz**.

(The book's printed answer key was a separate download from www.courseptr.com/downloads and is
not in the book; the answers above are derived from the chapter's own stated rules and figures.)

## Key Takeaways
1. Tone — regular periodic vibration — is what separates music from noise.
2. Every musical tone has exactly three parameters: pitch, intensity, tone quality.
3. Timbre is harmonic content; harmonic content is the harmonic series; the harmonic series is
   also the physical basis of consonance.
4. Two fixed reference points anchor everything else: A = 440 Hz and Middle C = C3.
5. Velocity (0–127) is the note-level expression of intensity, and varying it is what stops MIDI
   parts sounding mechanical.
6. Synthesis = harmonic content (oscillator/waveform) + envelope (ADSR).

## Connects To
- [Chapter 2 — The Notes](ch02-the-notes.md): the 2:1 octave ratio introduced here is why
  octave-related notes share a letter name.
- [Chapter 8 — Chords](ch08-chords.md): shared harmonics explain concord and discord.
- [Chapter 25 — Intonation](ch25-intonation.md): the harmonic series is the basis of just and
  Pythagorean tuning.
