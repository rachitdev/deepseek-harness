# Chapter 6 — Learning Curves: Will More Data Actually Help? (Book Chapters 28–32)

## Core Idea
Training and dev error at a single training-set size give you a point estimate; a **learning curve** — dev error and training error plotted against the number of training examples, with the desired error rate drawn in — gives you a *trajectory*. Plotting both curves together is what lets you say with confidence whether collecting twice as much data can reach the goal, or whether it provably cannot. "Looking at the learning curve might therefore help you avoid spending months collecting twice as much training data, only to realize it does not help."

## Frameworks Introduced

**The dev-error learning curve (Ch 28).** Train separate copies of the algorithm on increasing subset sizes — e.g. with 1,000 examples, train on 100, 200, 300, …, 1000 — and plot dev set error against training set size. As training size increases, dev error should decrease.

**The desired error rate line (Ch 28).** Draw a horizontal target on the plot. Sources for that target:
- Human-level performance, if that is what you hope to match.
- Product intuition about what user experience requires (e.g. for delivering cat pictures).
- Experience on a long-running application: what progress is reasonable in the next quarter/year.

Then visually extrapolate the dev curve. If it is still descending, doubling the training set may plausibly reach the target. If it has **plateaued** (flattened out), "you can immediately tell that adding more data won't get you to your goal."

**Adding the training-error curve (Ch 29).** Two structural facts make the pair far more informative than the dev curve alone:
- Training error **increases** with training set size (harder to fit more examples).
- The dev error curve "usually lies strictly above the blue training error curve."

**The three canonical readings (Ch 30–31).** See the Reference Table. The decisive case is high bias: once training error is *above* the desired line and still rising, more data cannot possibly work — the dev curve lives above training error, and training error can only get worse.

**Noise reduction for small-data curves (Ch 32).** Two remedies, to be used only if you have already plotted curves and found them too noisy:
- Train several (say 3–10) models per training-set size on different randomly chosen subsets **sampled with replacement**, and plot the *average* training and dev error.
- If the class distribution is skewed or there are many classes, choose a **balanced** subset rather than a purely random one — e.g. force 2/10 positive and 8/10 negative to match the 20%/80% overall split. "More generally, you can make sure the fraction of examples from each class is as close as possible to the overall fraction in the original training set."

## Key Concepts
- **Plateau** — a flattened dev error curve; the signal that additional data has stopped paying.
- **Why training error rises with data** — Ng's ladder: with 2 examples (one cat, one non-cat), the algorithm memorizes both and gets 0% training error even if they are mislabeled. With 100 examples, some mislabeled or ambiguous (very blurry images even humans can't call), memorizing gets harder and training accuracy drops slightly. With 10,000, "it becomes even harder for the algorithm to perfectly fit all 10,000 examples."
- **Sampling with replacement** — pick 10 examples out of 100 for the first subset; for the second subset pick 10 again *without regard to* what was already chosen, so an example may appear in both. (Without replacement, the second subset would be drawn from only the remaining 90.) "In practice, sampling with or without replacement shouldn't make a huge difference, but the former is common practice."
- **Skew-driven noise** — if 80% of examples are negative (y=0) and 20% positive (y=1), a random 10-example subset may contain only negatives, "making it very difficult for the algorithm to learn something meaningful." Same risk with many classes (e.g. 100 animal species).
- **Log-spaced training sizes** — instead of evenly spaced sizes, train at 1,000 / 2,000 / 4,000 / 6,000 / 10,000 to cut cost while still showing the trend.

## Mental Models
- **Two curves converge on the truth; one curve invites wishful extrapolation.** "One downside of this process is that if you only look at the dev error curve, it can be hard to extrapolate and predict exactly where the red curve will go."
- **Training error is a floor that keeps rising.** Since dev error sits above training error, and training error only worsens with more data, a training curve already above the target is a proof of impossibility for the data lever.
- **Gap = variance, distance-to-target = avoidable bias, read graphically.** The classic high-bias picture: at the largest training size there is "a large gap between the training error and the desired performance, indicating large avoidable bias… the gap between the training and dev curves is small, indicating small variance."
- **A learning curve is an experiment budget, not a free plot.** Training ten models at 1,000…10,000 examples costs real compute; use log spacing and only denoise when the noise actually obscures the trend.

## Anti-patterns
- **Extrapolating the dev curve alone**, especially with a small (and therefore noisy) dev set.
- **Buying a second dataset before plotting.** The chapter's whole economic case is avoiding months of collection that provably cannot reach the target.
- **Denoising prematurely.** "I would not bother with either of these techniques unless you have already tried plotting learning curves and concluded that the curves are too noisy to see the underlying trends." With >10,000 training examples and an unskewed class distribution, "you probably won't need these techniques."
- **Evenly spaced training sizes on expensive models**, when log spacing gives the same information.
- **Reading a single rightmost point as the whole story.** "Previously, we were measuring training and dev set error only at the rightmost point of this plot, which corresponds to using all the available training data. Plotting the full learning curve gives us a more comprehensive picture."

## Reference Tables

**Interpreting the curve pair (Ch 30–31)**

| Training error curve | Dev error curve | Gap | Distance of training curve from desired level | Diagnosis | Action |
|---|---|---|---|---|---|
| Low | Much higher than training | Large | Small | Small bias, **large variance** | "Adding more training data will probably help close the gap" |
| High, rising toward/above desired level | Slightly above training, plateaued | Small | Large | **Large avoidable bias**, small variance | More data cannot help by itself — reduce bias (bigger model, less regularization, better architecture) |
| High | Much higher than training | Large | Large | **Significant bias and significant variance** | "You will have to find a way to reduce both" |

**Structural facts you can rely on when reading any learning curve**

| Fact | Consequence |
|---|---|
| Dev error decreases as training set grows | A still-descending dev curve leaves room for the data lever |
| Training error *increases* as training set grows | The training curve can only move further from the target |
| Dev curve lies above the training curve | If training error > desired level, dev error can never reach it via more data |

**Noise controls (Ch 32)**

| Symptom | Remedy |
|---|---|
| Curves jumpy at small training sizes | Average 3–10 models per size, subsets sampled with replacement |
| Heavily skewed classes or many classes | Use class-balanced subsets matching the overall class fractions |
| Training all sizes is too expensive | Log-space the sizes (1,000 / 2,000 / 4,000 / 6,000 / 10,000) |
| Training set >10,000 and not skewed | Do nothing — you probably don't need these |

## Worked Example
**Proving that more data cannot save a high-bias system (Ch 30).** You plot dev error against training set size and it appears to plateau. You are unsure — the dev set is small, so the curve could be noisy, and the extrapolation is guesswork.

Now add the training error curve. Suppose the desired performance line is your estimate of the optimal error rate, and the training curve sits well *above* that line at the largest training size. Ng's two-step proof that data will not rescue you:

1. "As we add more training data, training error can only get worse. Thus, the blue training error curve can only stay the same or go higher, and thus it can only get further away from the (green line) level of desired performance."
2. "The red dev error curve is usually higher than the blue training error. Thus, there's almost no way that adding more data would allow the red dev error curve to drop down to the desired level of performance when even the training error is higher than the desired level of performance."

Conclusion: "you can be absolutely sure that adding more data will not, by itself, be sufficient." This is the textbook high-avoidable-bias learning curve: large distance from training curve to desired level (large avoidable bias), small gap between training and dev curves (small variance). Spend the quarter on model size and architecture, not on data collection.

**Plotting on a 100-example training set (Ch 32).** Train on randomly chosen subsets of 10, 20, 30, …, 100 examples — ten data points. The curve looks noisy at the small end: a 10-example draw may be unlucky (many ambiguous or mislabeled examples) or lucky. If your task is 80% negative / 20% positive, a 10-example random draw might contain only negatives. Fix by (a) averaging over 3–10 with-replacement draws per size and (b) forcing each 10-example subset to hold 2 positives and 8 negatives.

## Key Takeaways
- Plot **both** dev error and training error against training set size, with a horizontal desired-error line.
- A plateaued dev curve, or a training curve already above the desired line, means more data alone will not reach the goal.
- Gap between the curves = variance; distance from the training curve to the target = avoidable bias.
- Training error rises with dataset size — this is expected, and it is what makes the two-curve proof work.
- Denoise (averaged with-replacement subsets, class-balanced subsets) only when noise actually hides the trend; log-space sizes to control compute.

## Connects To
- `ch05-bias-and-variance.md` — the numeric version of the same diagnosis; the curve is its graphical extension.
- `ch07-human-level-performance.md` — the usual source of the "desired error rate" line.
- `ch04-basic-error-analysis.md` — what to do once the curve says data is not the answer.
- `ch08-training-on-different-distributions.md` — when the extra data you would add comes from a different distribution.
