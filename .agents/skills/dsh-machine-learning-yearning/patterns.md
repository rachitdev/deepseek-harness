# Patterns — Machine Learning Yearning (Andrew Ng)

The book's reusable moves, grouped by its own parts. Each is **When / How / Trade-offs**.

---

## Setting Up Development and Test Sets (book ch 5–12)

### Pick dev/test from the future distribution, not from what you happen to have
**When** — Starting any project where deployment data may differ from your collected data (website images vs mobile uploads).
**How** — State the distribution you must win on. Draw dev and test from it. Pre-launch, approximate it (ask friends to shoot mobile cat photos); post-launch, refresh from real user data. Keep the mismatched data as *training* data.
**Trade-offs** — Requires judgment about how much to invest in dev/test quality; may leave you with a smaller dev set than a naive split would give; if you truly cannot approximate the future distribution, you accept generalization risk knowingly.

### Draw dev and test from the same distribution as each other
**When** — Always, unless you are pinned to a third-party benchmark.
**How** — Never assign different segments (US+India vs China+Other) to dev and test; sample both from the full target population.
**Trade-offs** — On a benchmark whose creators mismatched them, "luck, rather than skill, will have a greater impact on your performance," and you may have to live with it.

### Size the dev set by the smallest difference you must detect
**When** — Choosing dev set size.
**How** — 1,000–10,000 examples is common; ~10,000 gives a good chance of detecting a 0.1% improvement; mature, high-revenue applications may need far more to detect 0.01%.
**Trade-offs** — Larger dev sets cost labeling effort and evaluation compute; "there is no need to have excessively large dev/test sets beyond what is needed."

### Collapse multiple goals into one optimizing metric
**When** — Your team is comparing models on two or more numbers and cannot agree on a ranking.
**How** — Average, weighted-average, or F1 the components; or take a weighted average across market segments.
**Trade-offs** — Any combination embeds a value judgment; a weighted average across four markets hides which market regressed.

### Optimizing + satisficing metrics
**When** — Criteria are incommensurable (accuracy vs milliseconds vs model file size) and a formula would be arbitrary.
**How** — Declare N−1 satisficing thresholds in product-meaningful units ("≤100ms", "no more than one false positive every 24 hours"), then maximize the remaining metric subject to them.
**Trade-offs** — The thresholds are still judgment calls; a candidate barely failing a threshold is discarded even if it is much better on the optimizing metric.

### Set metric and dev/test fast, then change them without ceremony
**When** — New project (target: under one week). Not for mature applications, where teams may spend months.
**How** — Ship an imperfect metric and dev/test set. Revisit when the metric's ranking contradicts the team's product judgment.
**Trade-offs** — Early imperfection means some early experiments are misdirected; the alternative — weeks of debate — costs more.

### Change the metric when it misranks the product
**When** — The metric ranks A above B but the team believes B is better (e.g. A lets pornographic images through).
**How** — Diagnose which of the three causes applies (unrepresentative distribution / dev-set overfitting / wrong metric) and apply the matching fix. For a wrong metric, define a new one that heavily penalizes the failure and announce the new goal.
**Trade-offs** — Changing the metric invalidates historical comparisons; leaving it in place is worse — you drift into "manually choosing among classifiers."

---

## Basic Error Analysis (book ch 13–19)

### Build a basic system in days, then iterate
**When** — Starting a new application, especially outside your expertise.
**How** — Ship the simplest working end-to-end system in a few days; use its errors as the roadmap.
**Trade-offs** — Explicitly for practitioners, not for academic publication; the first system will be far from best, and you must resist polishing it.

### Count 100 misclassified examples to get a ceiling
**When** — Anyone proposes a project whose payoff is unmeasured (e.g. a month integrating dog-detection software).
**How** — Sample 100 misclassified dev examples, count how many fall in the proposed category, and convert to an error-reduction ceiling.
**Trade-offs** — ~2 hours of manual work; sampling noise at 100 examples; the count does not include how hard or how partial the fix will be.

### Parallel idea evaluation in one spreadsheet
**When** — Several candidate directions are on the table at once.
**How** — Rows = misclassified examples; columns = candidate categories plus Comments; tick all that apply; add new columns as categories emerge; read the column percentages as ceilings.
**Trade-offs** — Percentages can exceed 100% (multi-category examples), so they are ceilings, not a partition; the process is iterative and you may need to re-examine earlier images against new categories.

### Track and (sometimes) fix mislabeled examples
**When** — Error analysis reveals a nontrivial Mislabeled column.
**How** — Add a Mislabeled column. Compare mislabeling error against the error you could otherwise reduce. Fix when mislabeling becomes a large *share* of remaining errors. Apply the same fixing process to dev *and* test.
**Trade-offs** — Checking only misclassified examples is cheaper but biases your accuracy estimate — acceptable for a product, not for a research paper.

### Split the dev set into Eyeball and Blackbox
**When** — The dev set is large enough that you cannot examine all its errors (e.g. 1,000 errors out of 5,000 examples).
**How** — Randomly move ~10% into the Eyeball set (sized to contain ~50–100 mistakes), leave the rest as Blackbox for automated evaluation and tuning. Rebuild the Eyeball set when its performance outpaces Blackbox.
**Trade-offs** — Splits your data; a small dev set cannot support the split, in which case use it all as Eyeball and accept higher overfitting risk.

---

## Bias and Variance (book ch 20–27)

### Read training error and the train→dev gap before choosing a tactic
**When** — Every time you ask "what next?"
**How** — Bias ≈ training error; variance ≈ dev error − training error; avoidable bias = training error − optimal error rate.
**Trade-offs** — The definitions are informal and differ from the statistics literature; they are chosen for actionability, not rigor.

### Estimate the optimal error rate before judging bias
**When** — The task may have irreducible error (noisy audio, ambiguous images).
**How** — Have humans label data and measure their accuracy; use the best achievable human performance.
**Trade-offs** — Impossible or unreliable on tasks humans do poorly; expensive when top-tier labelers are scarce.

### Bigger model for bias, more data for variance
**When** — After the diagnosis above.
**How** — High avoidable bias → increase model size (and add regularization to absorb the variance increase), reduce regularization, modify features from error-analysis insight, change architecture. High variance → add training data, add regularization, add early stopping, consider feature selection with small data.
**Trade-offs** — Bigger models cost compute; more data may be unobtainable; regularization raises bias; feature selection can raise bias sharply at large reductions.

### Prefer regularization over shrinking the model
**When** — Tempted to reduce network size to fix overfitting.
**How** — Keep the size, tune L2 or dropout on the dev set.
**Trade-offs** — Shrinking is still legitimate when your actual goal is faster/cheaper training.

### Error analysis on the training set
**When** — Training error is high (bias problem).
**How** — Tally ~100 poorly handled training examples into categories, exactly as with the Eyeball dev set. Also check that a human could solve those examples at all.
**Trade-offs** — Only informative when the algorithm underfits; on human-impossible examples the category is unactionable.

---

## Learning Curves (book ch 28–32)

### Plot dev *and* training error against training set size
**When** — Before committing months to data collection.
**How** — Train at several subset sizes, plot both curves plus a horizontal desired-error line, and read the gap (variance) and the training-curve-to-target distance (avoidable bias).
**Trade-offs** — Costly — you train many models. Mitigate with log-spaced sizes (1,000 / 2,000 / 4,000 / 6,000 / 10,000).

### Use the two-curve proof to rule out the data lever
**When** — The dev curve looks like it might plateau but you are unsure.
**How** — If the training curve is already above the desired level, more data cannot help: training error only worsens with more data, and dev error sits above it.
**Trade-offs** — Depends on the desired-error line being a credible target (typically the optimal error rate).

### Denoise small-data learning curves
**When** — Curves are too jumpy to read, typically below ~10,000 examples or with skewed classes.
**How** — Average 3–10 models per training size, subsets drawn with replacement; and/or use class-balanced subsets matching the overall class fractions.
**Trade-offs** — Multiplies training cost; only worth it after you have confirmed the noise obscures the trend.

---

## Comparing to Human-Level Performance (book ch 33–35)

### Define human-level performance as the best achievable
**When** — Multiple human skill tiers exist (untrained person 15%, junior doctor 10%, experienced doctor 5%, team of doctors 2%).
**How** — Use the lowest error (2%) as both the optimal-rate proxy and the desired performance level.
**Trade-offs** — The best tier is the most expensive to obtain labels from; mitigate by tiering — bulk labeling by juniors, hard cases escalated.

### Hunt for subsets where humans still win
**When** — Your average performance has surpassed human level.
**How** — Find a definable slice (rapidly spoken speech) where humans beat the system; reapply all three uses of human performance on that slice.
**Trade-offs** — Progress is inherently slower past human level; some slices may not be cleanly definable.

---

## Training and Testing on Different Distributions (book ch 36–43)

### Assign data by role, never by shuffle
**When** — You have a small target-distribution dataset and a large foreign one.
**How** — Dev/test from the target distribution; training gets the foreign data plus a slice of target data (5,000 of 10,000 user images into dev/test, the other 5,000 into training).
**Trade-offs** — Shrinks dev/test; creates a train/dev distribution gap that you must then diagnose with a Training dev set.

### Apply the consistency test before adding foreign data
**When** — Deciding whether to merge datasets.
**How** — Ask whether one f(x) predicts y for both sources without knowing the source. Yes (internet vs mobile cats) → include. No (NYC vs Detroit prices) → exclude, or add a disambiguating feature (rarely done in practice).
**Trade-offs** — Including consistent foreign data consumes representational capacity; safe only with a big enough network and enough compute.

### Down-weight an over-represented data source
**When** — The extra data outnumbers target data heavily (40:1) and you lack the compute for a proportionally larger network.
**How** — Add weight β to the foreign source's loss term; β = 1/40 equalizes the two; tune β on the dev set.
**Trade-offs** — Needed "only when you suspect the additional data has a very different distribution than the dev/test set, or if the additional data is much larger"; a mis-set β wastes the foreign data's value.

### Add a Training dev set
**When** — Training and dev/test distributions differ.
**How** — Hold out data drawn from the training distribution but never train on it. Read four numbers: optimal rate, training error, training dev error, dev error.
**Trade-offs** — Consumes a little training data; sized like a dev set, so it need not be large.

### Diagnose mismatch by error-analysing the dev set
**When** — Training dev error is low but dev error is high.
**How** — Manually examine ~100 dev examples to identify the differing property (car noise, motion blur), then acquire matching data. Validate by comparing performance on that property's subset across the training/training-dev boundary.
**Trade-offs** — "There are no guarantees" — if better-matched data is unobtainable, there may be no clear path.

### Synthesize matched training data
**When** — Collecting real matched data is impractical.
**How** — Combine ingredients (clean speech + car noise; sharp images + simulated blur) to manufacture data resembling the dev distribution.
**Trade-offs** — Realistic to a person ≠ realistic to an algorithm. Finite source variety (1 hour of noise, 10 cars, 20 3D car models) invites overfitting; getting the details right "sometimes took weeks," but the payoff is a far larger training set.

---

## Debugging Inference Algorithms (book ch 44–46)

### Run the Optimization Verification test
**When** — Your system learns an approximate score and maximizes it with an approximate search (speech recognition, machine translation, reinforcement learning).
**How** — For each dev-set error, compare Score(correct output) vs Score(produced output). Correct scores higher → search problem (e.g. widen the beam). Otherwise → objective/scoring problem. Tally the split across the dev set.
**Trade-offs** — Requires a known-superior reference output — though it need not be optimal (a human trajectory suffices) — and the ability to score arbitrary candidates.

---

## End-to-End Deep Learning (book ch 47–52)

### Choose end-to-end only with abundant both-ends data
**When** — Deciding between one learned mapping and a hand-built pipeline.
**How** — Ask whether you have a large dataset of (input, output) pairs. Speech does (audio, transcript); driving does not (image, steering direction).
**Trade-offs** — With small data the end-to-end system lacks the human knowledge that MFCCs and phonemes supply; with large data those same components cap performance.

### Design pipeline components around available data
**When** — You have chosen a pipeline.
**How** — Prefer components with public datasets or crowdsourceable labels (car detection, pedestrian detection via Mechanical Turk).
**Trade-offs** — Optimizing for label availability may not produce the theoretically cleanest decomposition.

### Design pipeline components for task simplicity
**When** — Same.
**How** — Split hard tasks into easier ones (cat detector + breed classifier instead of a direct Siamese-cat classifier). Easier ≈ learnable with fewer examples ≈ shallower computation.
**Trade-offs** — More components means more interfaces and more places to introduce error; ML has no formal definition of task difficulty, so this stays a judgment call.

---

## Error Analysis by Parts (book ch 53–58)

### Attribute each error to exactly one component
**When** — You have a pipeline and must choose which part to improve.
**How** — Informally, inspect each stage's output. Formally, replace an upstream component's output with a hand-labeled perfect output and re-run; if the system now succeeds, blame that component. Generalize with the A → B → C ladder under DAG ordering.
**Trade-offs** — Manual hand-labeling of perfect intermediate outputs is laborious; "if the number of ambiguous cases is small, you can make whatever decision you want and get a similar result."

### Benchmark each component against a human given the same input
**When** — You want a faster, informal read than full attribution.
**How** — Ask how far each component is from human-level, and how far the overall system is from a human restricted to the same inputs as that component.
**Trade-offs** — Only works where humans can do the component's task.

### Diagnose and redesign a flawed pipeline
**When** — Every component is at human level but the system is not.
**How** — Ask what other information a skilled human uses (lane markings) and add a component that supplies it.
**Trade-offs** — Redesign is expensive; the tempting shortcut — feeding raw inputs into a downstream module — violates the task-simplicity principle.

### Distribute chapters to teammates
**When** — You need a specific prioritization change adopted by the team.
**How** — Print and hand over the 1–2 pages that make that argument, which is why the chapters are short.
**Trade-offs** — None stated; Ng's closing pitch (book ch 58) is that a superhero team beats a superhero.
