# Chapter 10 — Debugging Inference: The Optimization Verification Test (Book Chapters 44–46)

## Core Idea
A huge class of AI systems shares one design pattern: **learn an approximate scoring function, then run an approximate search to maximize it.** When such a system produces a wrong output, the fault lies either in the search or in the score — and guessing wrong wastes months. The **Optimization Verification test** settles it with one comparison: score the *correct* output and the *produced* output. If the correct output already scores higher, the search algorithm is at fault. If not, the scoring function is at fault.

## Frameworks Introduced

**The pattern (Ch 44).** A speech recognizer inputs an audio clip **A** and computes Score_A(**S**) for each possible output sentence **S** — for example Score_A(S) = P(S|A), the probability that the correct transcription is S given audio A. It then must find the S maximizing that score. With a 50,000-word English vocabulary there are (50,000)^N possible sentences of length N — "far too many to exhaustively enumerate." So you use an approximate search such as **beam search**, which keeps only the K top candidates during search. Such algorithms "are not guaranteed to find the value of S that maximizes Score_A(S)."

**The two possible faults (Ch 44).**
1. **Search algorithm problem** — the approximate search (beam search) failed to find the S that maximizes Score_A(S).
2. **Objective (scoring function) problem** — the estimates of Score_A(S) = P(S|A) were inaccurate; the score failed to recognize the correct transcription as best.

**The test (Ch 44).** Let S_out be the produced output and S* the correct output. Compute Score_A(S*) and Score_A(S_out), and check whether Score_A(S*) > Score_A(S_out).
- **Case 1: Score_A(S*) > Score_A(S_out).** The learning algorithm correctly ranked S* higher, yet search returned S_out. **Search problem** — e.g. try increasing the beam width of beam search.
- **Case 2: Score_A(S*) ≤ Score_A(S_out).** The scoring function fails to give a strictly higher score to the correct output. **Objective problem** — improve how you learn or approximate Score_A(·).

**Aggregating over the dev set (Ch 44).** Do not decide on one example. "To apply the Optimization Verification test in practice, you should examine the errors in your dev set. For each error, you would test whether Score_A(S*) > Score_A(S_out)." Each example where the inequality holds is counted as an optimization-algorithm error; each where it fails is counted as a scoring-function error. The resulting split is a ceiling calculation exactly like error analysis: "suppose you find that 95% of the errors were due to the scoring function Score_A(·), and only 5% due to the optimization algorithm. Now you know that no matter how much you improve your optimization procedure, you would realistically eliminate only ~5% of our errors."

**General form (Ch 45).** Applicable whenever: given input **x**, you can compute Score_x(**y**) rating how good response y is; you use an approximate algorithm to find arg max_y Score_x(y); and you suspect the search sometimes misses the maximum. With y* the correct output and y_out the produced one, test whether Score_x(y*) > Score_x(y_out). Holds → blame the optimization algorithm. Otherwise → blame the computation of Score_x(y).

**The reinforcement-learning variant (Ch 46).** When there is no explicit input x, the pattern reduces to Score(·) — for RL, the reward function R(T) over trajectories T, maximized by the RL algorithm. The comparison output need not be optimal: "so long as you have some y* … that is a superior output to the performance of your current learning algorithm — even if it is not the 'optimal' output — then the Optimization Verification test can indicate whether it is more promising to improve the optimization algorithm or the scoring function."

## Key Concepts
- **Beam search** — an approximate search keeping only the K top candidates. Widening the beam is the canonical fix when the test blames search.
- **Score_A(S) = P(S|A)** — the probabilistic instantiation of the scoring function in speech recognition.
- **Arg max intractability** — (50,000)^N candidate sentences is why the approximation exists at all.
- **Reward function R(·)** — the RL scoring function, "typically chosen by hand to quantify how desirable different trajectories T are." Ng: "It is not easy to design good reward functions."
- **Autorotation** — the helicopter maneuver used as the RL example: landing with the engine turned off, which "allows helicopters to land even if their engine unexpectedly fails. Human pilots practice this maneuver as part of their training."
- **Design-pattern recognition** — "It is a very common 'design pattern' in AI to first learn an approximate scoring function Score_x(·), then use an approximate maximization algorithm. If you are able to spot this pattern, you will be able to use the Optimization Verification test to understand your source of errors."

## Mental Models
- **Two suspects, one measurement.** The test is a single inequality that partitions blame; no instrumentation of the search internals is needed.
- **You need a better reference output, not a perfect one.** S*, E*, or T_human — anything known to be superior to what the system produced — is enough to run the test.
- **The score is a model; the search is an engineer's approximation.** Improving them are entirely different projects (better learned objective vs wider beam / better search), staffed by different work.
- **Random allegiance is the failure the test prevents.** "Some researchers will randomly decide to work on the search algorithm; others will randomly work on a better way to learn values for Score_A(S). But unless you know which of these is the underlying cause of the error, your efforts could be wasted."

## Anti-patterns
- **Choosing search-vs-objective by taste or team specialty.**
- **Concluding from a single example.** Errors mix; run the test over the dev set errors and count.
- **Widening the beam when 95% of errors are scoring-function errors** — a maximum ~5% payoff.
- **Blaming the RL algorithm when the reward function is miscalibrated** (or vice versa) — the helicopter case below.
- **Assuming the comparison output must be optimal.** T_human is "pretty good, even if not optimal," and that suffices.

## Reference Tables

**The test, in all three instantiations**

| Setting | Input x | Output y | Scoring function | Approximate optimizer | Test |
|---|---|---|---|---|---|
| Speech recognition (Ch 44) | Audio clip A | Sentence S | Score_A(S) = P(S\|A) | Beam search | Score_A(S*) > Score_A(S_out)? |
| Machine translation, Chinese→English (Ch 45) | Chinese sentence C | English translation E | Score_C(E) = P(E\|C) | Heuristic search | Score_C(E*) > Score_C(E_out)? |
| Reinforcement learning, helicopter (Ch 46) | (none) | Trajectory T | Reward R(T) | RL algorithm | R(T_human) > R(T_out)? |

**Verdict table**

| Result | Meaning | Work on |
|---|---|---|
| Score(correct) > Score(produced) | The score ranked correctly; the optimizer failed to find it | The search / optimization algorithm (e.g. increase beam width; improve the RL algorithm) |
| Score(correct) ≤ Score(produced) | The score misranks the correct output | The scoring / objective / reward function |

**Dev-set aggregation output**

| Errors attributed to | Share | Implication |
|---|---|---|
| Scoring function | 95% | Ceiling for optimizer work is ~5%; focus on estimating Score_A(·) better |
| Optimization algorithm | 5% | |

## Worked Example
**"I love machine learning" → "I love robots" (Ch 44).** An audio clip A records someone saying "I love machine learning," but the system outputs "I love robots."

- S_out = "I love robots" (the produced transcription).
- S* = "I love machine learning" (the correct transcription).

Compute both scores and compare:
- If **Score_A(S*) > Score_A(S_out)**: the learned model already knows the correct sentence is better, but beam search never surfaced it. This is a search algorithm problem — increase the beam width.
- If **Score_A(S*) ≤ Score_A(S_out)**: the model genuinely believes "I love robots" is the better transcription of this audio. No amount of search improvement helps. Work on how Score_A(·) is learned or approximated.

Then repeat across all dev-set errors and tally the two buckets before allocating engineers.

**The autorotation helicopter (Ch 46).** You are teaching a helicopter to fly an autorotation — landing with the engine off. You hand-design a reward function R(T) over trajectories: a crash might score R(T) = −1,000, a safe landing a positive value depending on smoothness. R(·) "has to trade off how bumpy the landing was, whether the helicopter landed in exactly the desired spot, how rough the ride down was for passengers, and so on."

The learned controller performs far worse than a human pilot — bumpier, less safe landings. Which is at fault: the RL algorithm (failing to achieve max_T R(T)) or the reward function (mis-specifying what a good landing is)?

Let T_human be the human pilot's trajectory and T_out the algorithm's. By assumption T_human is superior. Test whether **R(T_human) > R(T_out)**:
- **Case 1 (inequality holds):** R(·) correctly rates T_human above T_out, but the RL algorithm is producing the inferior T_out. "This suggests that working on improving our reinforcement learning algorithm is worthwhile."
- **Case 2 (inequality fails):** R(T_human) ≤ R(T_out) — the reward function assigns a worse score to the better trajectory. "You should work on improving R(·) to better capture the tradeoffs that correspond to a good landing."

Note the structural difference from the speech case: you compare against *human-level* performance rather than a known optimal output, and that is sufficient.

**Machine translation (Ch 45).** Input Chinese sentence C, compute Score_C(E) = P(E|C) for candidate English translations E; the set of all English sentences is too large, so you rely on a heuristic search. The system outputs E_out instead of a correct E*. Test Score_C(E*) > Score_C(E_out): if it holds, "the Score_C(·) correctly recognized E* as a superior output to E_out; thus, you would attribute this error to the approximate search algorithm. Otherwise, you attribute this error to the computation of Score_C(·)."

## Key Takeaways
- Recognize the pattern: learned approximate score + approximate arg max. It covers speech recognition, machine translation, and reinforcement learning.
- One inequality — Score(correct) vs Score(produced) — assigns blame between search and objective.
- Run it over all dev-set errors and count; the split is a ceiling on each line of work.
- A merely *superior* reference output (a human trajectory) is enough; it need not be optimal.
- In RL the scoring function is the hand-designed reward, and it is a genuine suspect: "It is not easy to design good reward functions."

## Connects To
- `ch04-basic-error-analysis.md` — the same count-the-errors-and-compute-a-ceiling discipline.
- `ch07-human-level-performance.md` — the source of T_human as a reference output.
- `ch12-error-analysis-by-parts.md` — blame attribution when the system is a multi-component pipeline rather than a score-and-search.
- `ch03-evaluation-metrics-and-iteration.md` — a reward function that misranks is a metric problem in the training loop, mirroring a dev metric that misranks classifiers.
