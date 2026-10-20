# Chapter 9 — Diagnosing and Addressing Data Mismatch (Book Chapters 40–43)

## Core Idea
Once training and dev/test come from different distributions, "bias vs variance" is no longer a complete diagnosis — there is a **third** failure mode, **data mismatch**: the algorithm generalizes fine to unseen data *from the training distribution* but not to the dev/test distribution. Separating it out requires a fourth dataset, the **Training dev set**. Once identified, mismatch is addressed by understanding exactly how the distributions differ and then acquiring or **synthesizing** training data that matches — carefully, because synthetic data that looks realistic to a person can still be trivially distinguishable to a learning algorithm.

## Frameworks Introduced

**The three possible problems (Ch 40).** When training = Internet + Mobile images and dev/test = Mobile only, and dev error is too high:
1. **High (avoidable) bias on the training set distribution** — it does not do well on the training set.
2. **High variance** — it does well on training but does not generalize to unseen data drawn from the *same distribution as the training set*.
3. **Data mismatch** — it generalizes well to new data from the training distribution, "but not to data drawn from the dev/test set distribution… because the training set data is a poor match for the dev/test set data."

**The four datasets (Ch 40).**
- **Training set** — what the algorithm learns from (Internet + Mobile images). Need not match the distribution you care about.
- **Training dev set** — drawn from the *same distribution as the training set*, but never trained on. "Usually smaller than the training set; it only needs to be large enough to evaluate and track the progress of our learning algorithm."
- **Dev set** — from the same distribution as the test set; reflects what you ultimately care about (Mobile images).
- **Test set** — same distribution as the dev set.

The sizing guidance from Chapters 5–7 for the dev set also applies to the training dev set.

Three measurements follow: training error (fit), training dev error (generalization within the training distribution), dev/test error (performance on the task you care about).

**The error-attribution ladder (Ch 41).** Read the gaps in order:
- optimal error rate → training error = **avoidable bias**
- training error → training dev error = **variance**
- training dev error → dev error = **data mismatch**

**The two-by-three error table (Ch 41).** Ng lays the errors out as a grid: the x-axis is the two data distributions (A = Internet images, B = Mobile images); the y-axis is three types of error — human-level error, error on examples the algorithm has trained on, and error on examples it has not trained on. The standard four cells are filled by the four datasets above. You can optionally fill the remaining two: (a) human-level performance on Mobile images, by having humans label your mobile cat images; (b) trained-on error for Mobile images, by putting a small fraction of mobile images into the training set and measuring error on that subset. "Filling in these two additional entries may sometimes give additional insight about what the algorithm is doing on the two different distributions."

**Addressing data mismatch (Ch 42).** Ng's two-step recommendation:
(i) Try to understand what properties of the data differ between the training and dev set distributions.
(ii) Try to find more training data that better matches the dev set examples your algorithm has trouble with.
Step (i) is carried out by error analysis on the dev set — manually going through ~100 examples.

**Artificial data synthesis (Ch 43).** Manufacture matched training data instead of collecting it: add downloaded car/road noise audio to clean speech recordings to synthesize in-car speech; add simulated motion blur to sharp internet cat images to match blurry cellphone dev images.

## Key Concepts
- **Data mismatch** — the named third error source; the reason the training dev set exists.
- **Cross-boundary subset comparison (Ch 42)** — "If your training and training dev sets include audio recorded within a car, you should also double-check your system's performance on this subset of data. If it is doing well on the car data in the training set but not on car data in the training dev set, then this further validates the hypothesis that getting more car data would help." This is why Chapter 36 recommended putting some dev/test-distribution data into the training set.
- **Synthetic-data overfitting** — the algorithm latches onto the finite variety in your synthetic ingredients. Ng's examples: 1,000 hours of speech mixed with only **1 hour** of car noise repeated over and over; 1,000 unique hours of car noise but taken from just **10 different cars**; 100,000 car images rendered from a video game company's ~**20 car designs**.
- **Realistic to a person ≠ realistic to a computer** — "it is sometimes easier to create synthetic data that appears realistic to a person than it is to create data that appears realistic to a computer." Humans cannot distinguish repeated car noise; a learning algorithm can.
- **Domain adaptation research** — exists, "typically applicable only in special types of problems and… much less widely used than the ideas described in this chapter."
- **No guarantees** — "if you don't have any way to get more training data that better match the dev set data, you might not have a clear path towards improving performance."

## Mental Models
- **One new dataset buys one new diagnosis.** The training dev set costs almost nothing (you already had the data; you just don't train on it) and converts an ambiguous dev-error number into a three-way attribution.
- **Mismatch is a data problem, not an algorithm problem.** The fixes are all about acquiring or manufacturing the right training examples.
- **Synthesis must cover the *distribution*, not just look right.** The 20-car video-game example is the sharpest statement: "compared to the set of all cars out on roads… this set of 20 synthesized cars captures only a minuscule fraction of the world's distribution of cars."
- **The adversary is your own learning algorithm.** "Try to avoid giving the synthesized data properties that makes it possible for a learning algorithm to distinguish synthesized from non-synthesized examples."
- **Synthesis is slow work with a large payoff.** "When working on data synthesis, my teams have sometimes taken weeks before we produced data with details that are close enough to the actual distribution… But if you are able to get the details right, you can suddenly access a far larger training set than before."

## Anti-patterns
- **Calling every dev-error gap "variance"** when training and dev distributions differ.
- **Skipping the training dev set** — leaves you unable to separate variance from mismatch.
- **Reusing 1 hour of car noise across 1,000 hours of speech** — the algorithm can overfit to that one hour and generalize poorly to a new clip whose car noise sounds different.
- **Synthesizing from 10 cars or 20 3D models** — "even if the synthesized images look very realistic, this approach (which has been independently proposed by many people) will probably not work well."
- **Synthesizing before diagnosing.** Error analysis on the dev set tells you *which* property to synthesize (car noise, motion blur); guessing wastes the weeks synthesis takes.

## Reference Tables

**Three-way attribution — the book's own numbers (Ch 41), optimal error rate ≈ 0%**

| Training error | Training dev error | Dev error | Avoidable bias | Variance | Data mismatch | Diagnosis |
|---|---|---|---|---|---|---|
| 1% | 5% | 5% | 1% | 4% | 0% | **High variance** |
| 10% | 11% | 12% | 10% | 1% | 1% | **High avoidable bias** |
| 10% | 11% | 20% | 10% | 1% | 9% | **High avoidable bias + data mismatch** (no high variance) |
| 1% | 1.5% | 10% | 1% | 0.5% | 8.5% | **Data mismatch** (Ch 40's example) |

"It is possible for an algorithm to suffer from any subset of high avoidable bias, high variance, and data mismatch."

**The gap ladder**

| Gap | Name | Fix family |
|---|---|---|
| Optimal error rate → Training error | Avoidable bias | Bigger model, less regularization, better architecture (see `ch05-bias-and-variance.md`) |
| Training error → Training dev error | Variance | More training data, regularization, early stopping |
| Training dev error → Dev error | Data mismatch | Understand the difference; acquire or synthesize matching data |
| Dev error → Test error | Dev-set overfitting | Get a fresh/larger dev set |

**The two-by-three error grid (Ch 41)**

| | Distribution A (Internet images) | Distribution B (Mobile images) |
|---|---|---|
| Human-level error | Human error on A | *Optional:* have humans label mobile images |
| Error on examples trained on | Training error | *Optional:* put a small fraction of mobile images in training and measure on that subset |
| Error on examples not trained on | Training dev error | Dev / test error |

**Synthesis pitfalls**

| Synthetic recipe | Hidden limitation | Consequence |
|---|---|---|
| 1,000h speech + 1h car noise | One noise sample repeated | Overfits to that hour's noise |
| 1,000h car noise from 10 cars | Only 10 acoustic environments | Fails on an 11th car |
| 100,000 car images from ~20 3D models | Minuscule fraction of real car designs | Fails on real-world car designs |

## Worked Example
**Diagnosing a car-noise speech system (Ch 40–42).** Humans are near-perfect on cat recognition and on clean speech transcription, so treat the optimal rate as ≈0%.

Measurements for the cat detector in Chapter 40:
- 1% error on the training set
- 1.5% error on data drawn from the same distribution as the training set that the algorithm has not seen (the **training dev set**)
- 10% error on the dev set

Attribution: avoidable bias 1%, variance 0.5%, **data mismatch 8.5%**. "In this case, you clearly have a data mismatch problem."

Now the speech version, with the fix (Ch 42). Your speech system does very well on the training set *and* the training dev set, but poorly on the dev set — a data mismatch problem.
1. **Error-analyse the dev set.** Manually go through 100 dev examples to see where mistakes happen. You find "most of the audio clips in the dev set are taken within a car, whereas most of the training examples were recorded against a quiet background. The engine and road noise dramatically worsen the performance."
2. **Name the difference.** The distributions differ in the presence of engine and road noise. That is the mismatch.
3. **Acquire matching data.** Try to get more training audio recorded in cars.
4. **Validate the hypothesis with the cross-boundary check.** If your training and training dev sets already contain some car audio, compare performance on car data in the training set vs car data in the training dev set. Doing well on the former but not the latter "further validates the hypothesis that getting more car data would help."
5. **Synthesize when collection is impractical (Ch 43).** Rather than driving around recording, download a large quantity of car/road noise clips and "add" them to your existing clean recordings of people speaking in a quiet room, producing audio that sounds as if the speaker were in a noisy car. This can generate huge amounts of matched data.
6. **Guard the synthesis.** Do not reuse a single hour of car noise across 1,000 hours of speech; do not source all your noise from 10 cars. Aim for enough source variety that the algorithm cannot tell synthesized from real.

**The motion-blur analogue.** Dev set cat images have much more motion blur because cellphone users move their phones while shooting. Take non-blurry internet training images and add simulated motion blur, making the training set more similar to the dev set.

## Key Takeaways
- Add a **Training dev set** — same distribution as training, never trained on — whenever training and dev/test distributions differ.
- Read four numbers in order: optimal rate, training error, training dev error, dev error. The three gaps name avoidable bias, variance, and data mismatch.
- Any subset of the three problems can be present at once.
- Fix mismatch by identifying the specific differing property via dev-set error analysis, then acquiring or synthesizing training data with that property.
- Synthetic data must be *distributionally* representative, not merely convincing to a human; finite source variety (1 hour of noise, 10 cars, 20 3D models) invites overfitting.
- Sometimes there is no path: if you cannot get better-matched data, mismatch may be unfixable.

## Connects To
- `ch08-training-on-different-distributions.md` — the setup that creates mismatch, and the reason to seed the training set with target-distribution data.
- `ch05-bias-and-variance.md` — the two-way diagnosis this chapter extends to three.
- `ch04-basic-error-analysis.md` — the ~100-example protocol used in step (i) of addressing mismatch.
- `ch02-dev-and-test-set-design.md` — dev-set sizing guidance that also governs the training dev set.
