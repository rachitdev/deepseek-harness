# Chapter 9 — Experience Doesn't Reliably Ensure Expertise

*Part III — Feedback: Learning from Experience. Maxim 9 of 12.*

> "True intuitive expertise is learned from prolonged experience with good feedback on mistakes." — Daniel Kahneman

Chapter's opening questions: What are the prerequisites for intuitive expertise? How can we improve in uncertain environments with noisy feedback? When should you trust your gut, and when should you stick to the numbers?

## Core Idea

Practice does not make perfect — **practice plus the right kind of feedback** makes perfect. Whether raw experience turns into genuine skill depends entirely on the *learning environment*. Where cues are valid and feedback is fast, experience compounds into real intuitive expertise (online poker). Where cues are weak, feedback is delayed, noisy, or absent, decades of experience produce confidence without accuracy (Meehl's clinicians, Tetlock's pundits). The practical move is not to trust or distrust intuition wholesale, but to **diagnose the environment** and, where it is unfriendly, deliberately tame it: substitute models, upgrade feedback, and think in groups.

## Frameworks Introduced

### The Klein–Kahneman conditions for intuitive expertise
From "Conditions for Intuitive Expertise: A Failure to Disagree" — two conditions must both hold for an intuitive judgment (recognition) to be genuinely skilled:

1. **The environment must provide adequately valid cues** to the nature of the situation.
2. **People must have an opportunity to learn the relevant cues** (sufficient experience with rapid feedback).

Their division of labor: *"Where simple and valid cues exist, humans will find them if they are given sufficient experience and enough rapid feedback to do so. Statistical analysis is more likely to identify weakly valid cues, and a prediction algorithm will maintain above-chance accuracy by using such cues consistently."*

| Cue structure | Winner | Why |
|---|---|---|
| A few **strong, stable** cues | Human intuition does fine | Recognition finds them; experience encodes them |
| Many **individually weak** cues that must be summed | Simple formula | Humans grasp vivid rarities and neglect mundane aggregation |

### Clinical vs. statistical prediction (Paul Meehl, 1954)
- **Clinical method** — the subjective impression of the doctor, counselor, teacher, or parole judge.
- **Statistical / actuarial method** — a simple formula fed basic data.
- Meehl's 1954 review covered fewer than two dozen studies and he stayed open to a niche for clinicians. Four decades and 100+ studies later he wrote: *"conducting an investigation in which informal clinical judgment would perform better than the equation was almost impossible."* Tally: **~2/5 of studies roughly equal, ~3/5 significantly favoring the actuarial method** — never a reliable clinical win.
- **The broken-leg cue** — Meehl's own hoped-for exception: a formula gives .90 that Professor A attends Friday's movie; the clinician knows he just broke his leg, dropping it to ~zero. Rare but highly informative cues *should* favor the human. In practice, the effect ran the wrong way — Eric Johnson's think-aloud transcripts of residency-admissions committees found judges *"seem to use information in quite a different manner"* than a linear model. It is **as if intuitive experts spend all their time hunting for broken legs and ignore the mundane fact that the professor usually goes to the movies on Fridays.**
- Adding **longer interviews** — rich narrative that can't fit an equation — made clinicians *worse*.

### Wicked vs. kind learning environments (the poker/pundit contrast)
| Feature | Poker (kind-ish) | Political forecasting / clinical judgment (wicked) |
|---|---|---|
| Cue validity | High | Low; causes are multiple and interacting |
| Feedback speed | Instant, per hand | Years, or never |
| Repetition | Millions of hands, multi-tabled | History doesn't repeat |
| Formal theory | Probability + game theory | None strong enough |
| Record-keeping | Hand histories on a hard drive | Fallible memory, selective |

### Tetlock's four superforecaster strategies
1. **Break larger judgments into smaller ones.** Intuition performs a bait-and-swap, substituting an easier similar-sounding question. On "will a radioactive poison be detected in Yasser Arafat's remains?", the naive forecaster substitutes "do I think Israeli spies poisoned him?"; the good forecaster asks how the poison decays, how detectable it is after years, and by what routes it could enter the body.
2. **Make use of base rates.** Explicitly establish the overall probability for events of that type — how often do military coups succeed? how often does the NASDAQ close higher a year later? — to start in the right ballpark before fine adjustment.
3. **Form discussion groups for constructive disagreement.** Teams beat individuals; debate aggregates information and blocks single-viewpoint leaps.
4. **Keep score and calibrate.** Predict on an explicit probability scale. Tetlock's analogy: a world where people love to run but never agreed to stay on the track, start at a gun, end at a fixed distance, with no timekeepers — *"How likely is it that running times are improving in this world? Not very."*

### The four strategies for learning under uncertainty (the chapter's own prescription)
1. **Use a model** — don't feel out whether a bet is right-sized when you can compute pot odds. Models need not be complicated; counting factors for and against, or a weighted sum in a spreadsheet, beats subjective judgment. Even when not the final arbiter, a model gives a defensible starting point to adjust up or down.
2. **Get more than just outcome feedback** — track decisions so memory can't distort them, and get feedback on *calibration*, not just direction.
3. **Build a brain trust** — aggregate information and sharpen reasoning through friendly debate.
4. **Know when (and when not) to trust your gut** — apply the Klein–Kahneman test before deciding which mode to use.

## Key Concepts

- **Chess vs. poker.** Chess is deterministic: same moves, same outcome. Poker is stochastic — even holding two aces, the best starting hand in Texas Hold'em, you lose to a random hand **one time in six**. Randomness makes error attribution hard: did you lose because you were unlucky or unskilled?
- **Two remedies for randomness.** (a) More experience, until luck averages out — but the space of poker hands is too vast for this alone. (b) **Compute the correct move with probability theory and ignore the actual outcome** — count your outs, count the hands that beat you, compare odds to bet size to see whether a bet is for *value* or a *bluff*.
- **Hidden information and calibration.** Poker's second difficulty is not luck but concealment. Optimal play depends on the odds against the hands your opponent *might* hold, and on what your opponent thinks *you* hold. Bet only with good cards and opponents fold to your raises; bluff too often and they call. **Game-theory optimal (GTO)** play seeks the exact bluff/value calibration that leaves no exploitable pattern — some players literally randomize using the second hand of a watch (even/odd) to escape mind-readers like Doyle Brunson. MIT's Sloan School has offered a poker theory course.
- **The Moneymaker effect (2003).** Chris Moneymaker, a full-time accountant, won a $39 online satellite into the World Series of Poker and beat 839 entrants (each paying $10,000) for **$2.5 million**. Online poker exploded.
- **Why online play accelerated skill.** Multi-tabling compresses experience — Daniel Negreanu: *"some of these guys are playing twelve games at once… A guy like Doyle Brunson, who's eighty-four years old right now, he's been playing every day pretty much for fifty, sixty years. He still hasn't played as many hands as some of these twenty-three-year-old kids."* More important than volume: **feedback**. Players replace frail recollection with a hard drive, downloading every hand and running it through analysis software to find both probability errors and exploitable patterns in their own play.
- **The old psychology-first tradition.** Doyle Brunson, author of one of the first popular strategy books: *"Poker is a game of people. A man's true feelings come out in a poker game."* He mixed sound probability with belief in extrasensory perception and trusting gut over analysis — seeding the still-popular perception that poker is psychology rather than probability. Online play, where all you see is a screen name, removed the psychoanalysis and forced the math forward.
- **The conjunction fallacy (Linda).** "Linda is 31 years old, single, outspoken, and very bright. She majored in philosophy… deeply concerned with issues of discrimination and social justice, and also participated in anti-nuclear demonstrations." Asked whether (1) Linda is a bank teller or (2) Linda is a bank teller *and* active in the feminist movement is more likely, many choose (2) — but the feminist bank tellers are a **subset** of bank tellers, so (1) must be at least as probable. Intuition is a superb storyteller and a poor aggregator.
- **Wisdom of crowds (Francis Galton).** At a county fair, no individual guessed the ox's weight correctly, but the **average of the guesses was nearly perfect**.
- **Debate beats averaging (Moshman & Geil).** On the Wason four-card task (see chapter 6), only **9 percent** of individuals answered correctly. In small discussion groups the success rate jumped to **75 percent** — far above what averaging could produce, because the correct minority could convince the majority. Some groups even reached the right answer *when no member had initially proposed it*: *"levels of understanding that are difficult to elicit in situations of individual performance may emerge in the context of collaborative reasoning."*
- **Reasoning is social (Sperber & Mercier).** Human reasoning is adapted more for justifying one's beliefs to others than for solitary truth-finding — which is why debate sharpens thinking.
- **Intellectual humility beats intellectual confidence.** Tetlock: there was *"a curious inverse relationship between how well forecasters thought they were doing and how well they did."* Bad forecasters fit everything into one overarching worldview (great for forceful essays and citations, bad for reality). Good forecasters were Truman's despised "one-handed economists" in reverse — always saying "well, on the one hand…"

## Mental Models

- **Poker vs. the pundit.** Ask of any skill: is this more like poker (valid cues, instant feedback, formal theory, logged history) or more like political punditry (weak cues, decade-long horizons, no reference class)? The answer dictates how much to trust felt experience.
- **The broken-leg hunter.** A vivid image for expert failure: an expert who spends all attention scanning for the rare, dramatic exception while under-weighting the boring regularity that carries most of the predictive power.
- **The hard drive replaces the memory.** The core mechanism behind the online poker generation's speed — and a transferable move. Anything you can log, you can later analyze; anything only remembered will be re-remembered in your favor.
- **The untimed running world.** Tetlock's picture of a domain with no scorekeeping. If you can't say who ran what in how long, nobody gets faster.
- **The bait-and-swap.** When a question is too hard, the mind quietly answers an easier neighboring question and reports the answer with the hard question's confidence.

## Anti-patterns

- **Treating years of experience as proof of expertise.** In Tetlock's project, experts beat chance only barely and beat undergraduates, but **lost to simple models that extrapolated past trends**, and even underperformed smart non-experts answering questions outside their field.
- **Relying on outcome feedback alone.** A 55/45 edge may take hundreds of hands to become visible, yet it is exactly what separates a winning from a losing player. Research shows outcome feedback alone doesn't reliably improve forecasting — in one experiment **people got worse with more experience**.
- **Asymmetric outcome sampling.** Hiring managers congratulate themselves on the talent they scouted, and never learn about the gems they rejected after a lackluster interview. If you only observe outcomes for the option you chose, your feedback loop is broken by construction.
- **Adding more narrative to a judgment problem.** Longer interviews made clinicians worse, not better. Rich stories inflate confidence without adding predictive validity.
- **One-worldview thinking.** Coherence sells; it does not forecast.
- **Believing intuition is a fallible version of a linear model.** Johnson's transcripts say otherwise — the failure mode is *categorically different* information use, so "just be more careful" doesn't fix it.
- **Groupthink masquerading as a brain trust.** Discussion doesn't guarantee convergence on truth — ideology, groupthink, and domineering members derail debate. It is only *more likely* than thinking alone.

## Reference Tables

### Decision rule — trust gut or use the model?
| Condition | Verdict |
|---|---|
| Cues valid + fast feedback + many repetitions | Intuition is likely genuine skill; use it, but let math overrule it |
| Prediction requires summing many weak cues | Use a formula |
| Rare, highly informative cue present (real broken leg) | Adjust the model's output, don't discard the model |
| No reference class, decade-long horizon | Expect near-chance accuracy; forecast in probabilities and keep score |

### Numbers cited in the chapter
| Fact | Figure |
|---|---|
| Annette Obrestad's WSOP Europe win | Sept 17, 2007, day before her 19th birthday; beat 362 entrants at £10,000 each; £1 million prize |
| Obrestad's live-game career winnings | Over $3.9 million |
| Moneymaker's 2003 WSOP win | $39 satellite → beat 839 entrants → $2.5 million |
| Losing with pocket aces vs. a random hand | 1 in 6 |
| Burgess parole study | 3,000 parole decisions, 21 factors, unweighted sum |
| Meehl's follow-up tally | ~2/5 of studies equal, ~3/5 favoring actuarial |
| Wason task, individuals vs. small groups | 9% → 75% correct |
| Tetlock's IARPA superforecasters | Beat the control group by **60 to 78 percent**, out-predicting teams with access to classified data |

## Worked Example — Burgess's parole tally vs. three psychiatrists

The concrete study Meehl used to make the actuarial case, reproduced as the book reports it:

1. **The question.** Predict recidivism for criminal offenders coming up for parole.
2. **The data.** Sociologist Ernest Burgess examined **three thousand parole decisions**.
3. **The "model."** Burgess listed **twenty-one basic factors** about each offender — age, previous offenses, nature of the crime, and so on. The algorithm was arithmetic at its crudest: **add one for each factor in the parolee's favor, subtract one for each factor against.** No weights. No regression. No interaction terms.
4. **The comparison group.** The expert opinion of **three psychiatrists** reviewing the same cases.
5. **The handicap — running against the formula.** The tally was applied to *every* case; the psychiatrists were allowed to **omit an opinion on some of the harder cases**, keeping only the ones they felt able to judge.
6. **The result.** Burgess's unweighted sum **slightly underperformed the psychiatrists at predicting success**, but **greatly surpassed them at predicting failure** — the side of the ledger where a parole board's errors actually cost something.
7. **The generalization.** At publication, fewer than two dozen head-to-head studies existed. Over the following decades **more than one hundred** accumulated, consistently favoring simple formulas across a wide range of decisions under uncertainty.
8. **Meehl's prescription.** Replace intuitive judgment with simple rules and models where the evidence supports it — psychiatric diagnosis by symptom checklist rather than the therapist's gut. Where formulas *beat* clinicians, accuracy improves; where they merely *tie*, the saving is enormous, since the status quo is lengthy deliberation by highly paid experts and the alternative is a little data entry in a spreadsheet.

**Where this has already happened:** bank loans are now issued on actuarial formulas, not lenders' intuitions, and the profession is better for it. Formulas did not remove loan officers — they removed a source of bias and error from bank policy.

**The counterweight:** skilled poker players are the marked contrast. Until deep learning and supercomputer-fueled solvers arrived, top human players could **edge out sophisticated poker software** — because poker satisfies both Klein–Kahneman conditions and the clinicians' environment does not.

## Key Takeaways

1. Experience only becomes expertise when the environment supplies valid cues *and* an opportunity to learn them. Check both before trusting anyone's gut, including your own.
2. Where prediction depends on summing many weak cues, a crude formula beats an expert — reliably, across 100+ studies.
3. Confidence and accuracy are inversely related in wicked environments. Treat your own certainty as a warning light, not a signal.
4. Outcome feedback alone is not enough. Log decisions, score them on a probability scale, and check calibration, not just direction.
5. You can partly tame a wicked environment: decompose the question, anchor on base rates, argue it out in a group, and keep score.
6. Poker's lesson is that **technology can upgrade the environment** — hand histories and analysis software turned a game of "psychology" into a game of calibration. Ask what the equivalent log is in your field.
7. Groups beat individuals not by averaging but by letting a correct minority persuade — 9 percent to 75 percent on the Wason task.
8. Intuition and models are not rivals to choose between once. The expert's edge is knowing, case by case, which one to defer to.

## Connects To

- [Chapter 4 — Knowledge Becomes Invisible with Experience](ch04-knowledge-becomes-invisible-with-experience.md) — Klein's firefighters and recognition-primed decision making, the case *for* expert intuition that this chapter bounds.
- [Chapter 6 — The Mind Is Not a Muscle](ch06-the-mind-is-not-a-muscle.md) — the Wason four-card task, reused here as the group-reasoning experiment.
- [Chapter 5 — The Difficulty Sweet Spot](ch05-the-difficulty-sweet-spot.md) — feedback as the third element of the practice loop; this chapter is about when that feedback is trustworthy.
- [Chapter 10 — Practice Must Meet Reality](ch10-practice-must-meet-reality.md) — the next feedback problem: practice that never contacts the real physical and social setting.
- [Chapter 11 — Improvement Is Not a Straight Line](ch11-improvement-is-not-a-straight-line.md) — misconceptions that survive because feedback never disconfirms them.
