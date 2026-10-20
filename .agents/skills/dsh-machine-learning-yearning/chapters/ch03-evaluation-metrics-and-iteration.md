# Chapter 3 — Single-Number Metrics, Optimizing vs Satisficing, and Fast Iteration (Book Chapters 8–12)

## Core Idea
A team can only move fast in one direction at a time, and a **single-number evaluation metric** is what makes a direction unambiguous. When you genuinely care about several things, do not invent a weighted formula out of thin air — declare one **optimizing metric** and turn the rest into **satisficing metrics** with thresholds. Set the metric and dev/test sets fast (target: under a week), and change them without embarrassment the moment they stop pointing the team the right way.

## Frameworks Introduced

**Single-number evaluation metric (Ch 8).** Classification accuracy is the canonical example: run the classifier on the dev set, get one number. "Having a single-number evaluation metric such as accuracy allows you to sort all your models according to their performance on this metric, and quickly decide what is working best." Precision and Recall together are *not* a single-number metric — two numbers give no ranking.

Standard combinations Ng endorses:
- Average of the components.
- **F1 score** — "a modified way of computing their average, and works better than simply taking the mean." Footnote: it is the harmonic mean, `2 / ((1/Precision) + (1/Recall))`.
- Average or **weighted average** across segments (e.g. accuracy in US, China, India, Other) — "one of the most common ways to combine multiple metrics into one."

**Optimizing and satisficing metrics (Ch 9).** With N criteria, make N−1 of them satisficing (must clear a threshold) and 1 optimizing (maximize it subject to the thresholds). Ng explicitly rejects fabricating a formula like `Accuracy - 0.5*RunningTime` as "unnatural."

**The iteration loop (Ch 10).** The build cycle Ng describes:
1. Start off with some **idea** on how to build the system.
2. Implement the idea in **code**.
3. Carry out an **experiment** which tells you how well the idea worked. ("Usually my first few ideas don't work!")
Then generate more ideas and keep iterating. "The faster you can go round this loop, the faster you will make progress." Dev set + metric is the mechanism that makes step 3 cheap.

**Three causes of a bad metric verdict (Ch 11).** When your metric ranks classifier A above B but the team believes B is better for the product, the cause is one of exactly three things — see the Reference Table below.

## Key Concepts
- **Satisficing metric** — a criterion your classifier "just has to be good enough on." Example: runs in at most 100ms.
- **Optimizing metric** — the one you maximize subject to the satisficing constraints. Example: accuracy.
- **Metric/dev-set velocity** — Ng asks teams for an initial dev/test set and metric "in less than one week — rarely longer. It is better to come up with something imperfect and get going quickly, rather than overthink this." Exception: mature applications (anti-spam is his example) where teams "spend months to acquire even better dev/test sets."
- **Dev set overfitting detected via test set** — if dev performance is much better than test performance, you have overfit the dev set; get a fresh dev set.
- **Test set hygiene under progress tracking** — you may evaluate on the test set weekly or monthly to track progress, "but do not use the test set to make any decisions regarding the algorithm, including whether to roll back to the previous week's system." Otherwise you begin overfitting the test set and lose the unbiased estimate.
- **Accumulating 0.1% wins** — "a lot of progress in your system will be made by gradually accumulating dozens of these 0.1% improvements," which is only possible if you can measure them.

## Mental Models
- **A ranking, not a report card.** The metric's job is to impose a total order on candidate models so a decision is instant. Two numbers = no order = a meeting.
- **Thresholds beat coefficients.** Nobody can defend the 0.5 in `Accuracy − 0.5×RunningTime`. Everyone can defend "100ms is acceptable."
- **A metric is a hypothesis about the product.** It can be wrong. Ng's stance is not to tolerate a distrusted metric: "I would strongly recommend picking a new metric and using the new metric to explicitly define a new goal for the team, rather than proceeding for too long without a trusted metric and reverting to manually choosing among classifiers."
- **Cheap measurement changes what you can attempt.** Without a dev set you must ship each candidate into the app and "play with the app for a few hours" — and still miss a 95.0% → 95.1% gain.

## Anti-patterns
- **Carrying multiple headline metrics indefinitely** — "Having multiple-number evaluation metrics makes it harder to compare algorithms."
- **Inventing an arbitrary scalarization** that mixes incommensurable units (accuracy and milliseconds) with a made-up coefficient.
- **Overthinking the initial metric** for weeks on a new project.
- **Using the test set to make decisions**, including rollback decisions.
- **Continuing to trust a metric that has visibly failed the product** — the pornography example below.
- **Manually eyeballing candidate classifiers** as a substitute for fixing the metric.

## Reference Tables

**Precision/Recall vs F1 — the book's own numbers (Ch 8)**

| Classifier | Precision | Recall | F1 score |
|---|---|---|---|
| A | 95% | 90% | 92.4% |
| B | 98% | 85% | 91.0% |

Precision/Recall alone: "neither classifier is obviously superior." F1 ranks A above B — a decision.

**Optimizing vs satisficing — the book's own numbers (Ch 9)**

| Classifier | Accuracy | Running time |
|---|---|---|
| A | 90% | 80ms |
| B | 92% | 95ms |
| C | 95% | 1,500ms |

With running time as satisficing (≤100ms) and accuracy as optimizing: C is disqualified, and B wins.

**Three causes of a wrong metric verdict, and their fixes (Ch 11)**

| Cause | Symptom | Fix |
|---|---|---|
| 1. The actual distribution you need to do well on differs from dev/test | Dev set is mostly adult cats; users upload kittens | Update dev/test to be representative |
| 2. You have overfit the dev set | Dev performance ≫ test performance | Get a fresh dev set / more dev data |
| 3. The metric measures the wrong thing | A is more accurate but lets pornographic images through | Change the metric — e.g. heavily penalize letting through pornographic images |

**Chapter 12 takeaways checklist (the book's own summary of this part)**
- Choose dev/test from the future distribution you care about — may differ from training data.
- Choose dev and test from the same distribution if possible.
- Choose a single-number metric; combine multiple goals by averaging or by optimizing/satisficing.
- ML is highly iterative — dozens of ideas before one satisfies.
- Dev/test + single-number metric → faster evaluation → faster iteration.
- Establish dev/test and metric in under a week on a new application; longer is acceptable for mature ones.
- The 70/30 heuristic does not apply when you have a lot of data.
- Dev set: big enough to detect meaningful accuracy changes, not much larger. Test set: big enough for a confident final estimate.
- If dev/metric stop pointing the right way, change them fast: (i) overfit dev → more dev data; (ii) wrong distribution → new dev/test data; (iii) wrong metric → new metric.

## Worked Example
**The wakeword device.** You are building a hardware device whose microphone listens for a wakeword — Amazon Echo listening for "Alexa," Apple Siri for "Hey Siri," Android for "Okay Google," Baidu apps for "Hello Baidu." Two error types matter:
- **False positive rate** — the system wakes up when nobody said the wakeword.
- **False negative rate** — it fails to wake when someone did.

Ng's formulation: **minimize the false negative rate (optimizing metric), subject to there being no more than one false positive every 24 hours of operation (satisficing metric).** Note the satisficing threshold is expressed in product-meaningful units (one spurious wake per day), not as a rate on a dev set — which is exactly what makes it defensible to a whole team.

**The pornography incident (Ch 11), as a metric-change decision.** Metric = classification accuracy. It ranks classifier A above classifier B. You then run both and find A "is allowing occasional pornographic images to slip through." The bad impression makes A unacceptable despite higher accuracy. The metric has failed cause #3. Action: change the metric so it heavily penalizes letting through pornographic images, announce the new goal to the team, and re-rank. Do *not* keep the old metric and override it by hand each time.

**A generalizable multi-criteria setup.** Criteria: binary file size of the model (users don't want to download large apps), running time, accuracy. Set thresholds for file size and running time (satisficing); optimize accuracy under those constraints.

## Key Takeaways
- One number, one direction. If you have two numbers, combine them (average, weighted average, F1) or split them into optimizing + satisficing.
- Satisficing thresholds should be stated in product terms ("≤100ms", "≤1 false wake per 24h").
- Get an imperfect metric and dev/test set within a week; iterate on them later.
- Track progress on the test set at most weekly/monthly, and never let it drive decisions.
- Changing dev/test or the metric mid-project is normal, not a failure — just make sure the team knows the new direction.

## Connects To
- `ch02-dev-and-test-set-design.md` — the sets the metric is computed on.
- `ch04-basic-error-analysis.md` — what to do once the metric says you are stuck.
- `ch07-human-level-performance.md` — using a human baseline to set the "desired error rate" the metric aims at.
- `ch01-ml-strategy-and-scale.md` — why prioritization, not ideas, is the bottleneck.
