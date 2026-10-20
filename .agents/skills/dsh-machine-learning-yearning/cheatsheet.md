# Cheatsheet — Machine Learning Yearning (Andrew Ng)

## 1. The master diagnostic ladder

Measure four numbers, in this order. Each gap names one problem.

| Gap | Name | Primary fixes |
|---|---|---|
| 0 → Optimal error rate | Unavoidable bias | Nothing — it is the floor |
| Optimal error rate → Training error | **Avoidable bias** | Bigger model, less regularization, better architecture, features from error analysis |
| Training error → Training dev error | **Variance** | More training data, regularization, early stopping, (small data) feature selection |
| Training dev error → Dev error | **Data mismatch** | Understand the differing property; acquire or synthesize matching data |
| Dev error → Test error | **Dev-set overfitting** | Fresh / larger dev set |

The Training dev set is only needed when training and dev/test come from different distributions. Otherwise collapse the middle two rows: variance = dev error − training error.

## 2. Diagnosis table (optimal error rate ≈ 0%)

| Training error | Dev error | Bias | Variance | Verdict |
|---|---|---|---|---|
| 1% | 11% | 1% | 10% | High variance (overfitting) |
| 15% | 16% | 15% | 1% | High bias (underfitting) |
| 15% | 30% | 15% | 15% | High bias AND high variance |
| 0.5% | 1% | 0.5% | 0.5% | Done |

Same numbers, **optimal error rate = 14%**:

| Training error | Dev error | Avoidable bias | Variance | Verdict |
|---|---|---|---|---|
| 15% | 30% | 1% | 15% | Variance only — do not chase bias |
| 15% | 16% | 1% | 1% | Near optimal; ~2% left in total |

Three-way, with a Training dev set (optimal ≈ 0%):

| Train | Train-dev | Dev | Bias | Variance | Mismatch |
|---|---|---|---|---|---|
| 1% | 5% | 5% | 1% | 4% | 0% |
| 10% | 11% | 12% | 10% | 1% | 1% |
| 10% | 11% | 20% | 10% | 1% | 9% |
| 1% | 1.5% | 10% | 1% | 0.5% | 8.5% |

## 3. Technique selection matrix

| Technique | Bias | Variance | Notes |
|---|---|---|---|
| Increase model size | ↓ | ↑ (cancel with regularization) | Only real cost is compute |
| Add more training data | — | ↓ | "Not helpful" for bias |
| Reduce/eliminate regularization | ↓ | ↑ | |
| Add regularization (L1/L2/dropout) | ↑ | ↓ | Preferred over shrinking the model |
| Add early stopping | ↑ | ↓ | Behaves like regularization |
| Feature selection (fewer features) | ↑ (if aggressive) | ↓ | 1,000→900 negligible; 1,000→100 significant. Best with small training sets |
| Decrease model size | ↑ | ↓ | Not recommended for variance; use for compute savings |
| Modify input features from error analysis | ↓ | ↕ | Can help both |
| Modify model architecture | ↕ | ↕ | Can help both; least predictable |

## 4. Default values and thresholds

| Quantity | Default |
|---|---|
| Dev set size | 1,000–10,000 examples |
| Dev set size to detect a 0.1% improvement | ~10,000 examples |
| Blackbox dev set size | 1,000–10,000 (100 is small but useful) |
| Eyeball dev set size | Enough to contain ~50–100 *mistakes* |
| Eyeball set at 5% error rate | ~2,000 examples (0.05 × 2,000 = 100) |
| Errors to examine in error analysis | ~100 (some go to 500; rarely >1,000) |
| Time to set initial dev/test + metric | < 1 week (new projects); months acceptable for mature ones |
| Time to build the first system | A few days |
| Train/test split heuristic | 70/30 only for ~100–10,000 total examples |
| Weighting β for a 40:1 data ratio | β = 1/40 (or tune on dev) |
| Test-set progress checks | Weekly or monthly — never for decisions |

## 5. Eyeball dev set sizing (mistakes available for analysis)

| Mistakes | What you learn |
|---|---|
| ~10 | Very small; rough prioritization only |
| ~20 | Rough sense of major error sources |
| ~50 | Good sense |
| ~100 | Very good sense |
| 100–500 | No harm if you have the data |
| >1,000 | Rarely done |

## 6. Tells and smells — symptom → diagnosis → fix

| Symptom | Diagnosis | Fix |
|---|---|---|
| Great train/test numbers, poor live performance | Dev/test drawn from the wrong distribution | Rebuild dev/test from target-distribution data |
| Good on dev, bad on test | Overfit the dev set (if same distribution) | Get more/fresh dev data |
| Good on dev, bad on test, different distributions | Ambiguous — could be overfitting, a harder test set, or just a different one | Fix the distributions first; the diagnosis is unavailable until you do |
| Metric ranks A > B but the team prefers B | Unrepresentative dev set / dev overfitting / wrong metric | Update dev/test, refresh dev, or change the metric |
| Team argues about which model is better | No single-number metric | Combine (average / weighted / F1) or optimizing + satisficing |
| Training error far above target | High avoidable bias | Bigger model; more data will NOT help |
| Training error low, dev error much higher | High variance | More data, regularization, early stopping |
| Dev error curve plateaued | Data lever exhausted | Reduce bias instead |
| Training error curve already above the desired line | Provably high avoidable bias | More data cannot reach the target — do not collect it |
| Learning curves jumpy at small sizes | Sampling noise / class skew | Average 3–10 with-replacement subsets; use class-balanced subsets |
| Low train error, low train-dev error, high dev error | Data mismatch | Identify the differing property; acquire or synthesize matching data |
| Eyeball performance improving much faster than Blackbox | Overfitting the Eyeball set | Discard and rebuild the Eyeball set |
| Errors dominated by mislabeled examples as a *share* | Label quality now the bottleneck | Fix dev labels — and test labels identically |
| Search-based system emits wrong output | Search vs objective unknown | Optimization Verification test |
| Synthetic data works in training, fails on real data | Insufficient variety in the synthesis ingredients | Broaden sources (more noise hours, more cars, more 3D models) |
| Every pipeline component at human level, system is not | Flawed pipeline — a component's inputs lack information | Add a component supplying it (e.g. Detect lane markings) |
| Avoidable bias computes as negative | Over-memorized the training set | Switch to variance-reduction methods |

## 7. Metric design template

1. Name the **optimizing metric** (one number to maximize).
2. Name each **satisficing metric** with a threshold in product units.
3. Verify the dev set can *detect* the differences you care about.
4. Re-examine if the metric ever contradicts product judgment.

Worked defaults from the book:
- Precision + Recall → **F1** = 2/((1/P)+(1/R)). A: 95%/90% → 92.4%. B: 98%/85% → 91.0%.
- Accuracy vs running time → optimize accuracy subject to **≤100ms**. (A 90%/80ms, B 92%/95ms, C 95%/1,500ms → pick **B**.)
- Wakeword device → minimize false negative rate subject to **≤1 false positive per 24 hours**.
- Multi-market → weighted average across US, China, India, Other.

## 8. Error analysis spreadsheet template

| Image | Dog | Great cat | Blurry | Mislabeled | Comments |
|---|---|---|---|---|---|
| 1 | ✔ | | | | Unusual pitbull color |
| 2 | | | ✔ | | |
| 3 | | ✔ | ✔ | | Lion; zoo on rainy day |
| 4 | | ✔ | | | Panther behind tree |
| 98 | | | | ✔ | Labeler missed cat in background |
| 100 | | | | ✔ | Drawing of a cat; not a real cat |
| **% of total** | **8%** | **43%** | **61%** | **6%** | |

Rules: tick every applicable column; percentages may exceed 100%; add columns as new categories occur to you; each percentage is a *ceiling* on that project's payoff.

## 9. Mislabeled-data decision rule

| Overall dev error | Mislabeling error | Share of errors | Fix labels? |
|---|---|---|---|
| 10.0% | 0.6% | 6% | Optional |
| 2.0% | 0.6% | 30% | Yes |

Always apply the same fixing process to dev **and** test. Checking only misclassified examples is acceptable for products, biased for research.

## 10. Human-level performance selection

| Tier | Error | Use as baseline? |
|---|---|---|
| Untrained person | 15% | No |
| Junior doctor | 10% | No (fine for bulk labeling) |
| Experienced doctor | 5% | No |
| Team of doctors | 2% | **Yes** |

Rule: human-level performance = **best achievable** human performance, because it is the only tier that constrains the optimal error rate from above.

## 11. Data inclusion decision flow

1. Is there one f(x) → y that serves both sources without knowing the source?
   - **No** → exclude (or add a source feature; rarely done).
   - **Yes** → continue.
2. Does the extra data contain any transferable signal?
   - **No** (scanned documents for a cat detector) → exclude for compute reasons.
   - **Yes** → continue.
3. Can you afford a network large enough to absorb both distributions?
   - **Yes** → include at full weight.
   - **No** → include with weight β (β = 1/40 for a 40:1 ratio), or match the training distribution to dev/test more closely.

## 12. Optimization Verification test

Applicable whenever: **learned approximate Score_x(y) + approximate arg max**.

| Instance | x | y | Score | Optimizer |
|---|---|---|---|---|
| Speech recognition | Audio A | Sentence S | Score_A(S) = P(S\|A) | Beam search |
| Machine translation | Chinese C | English E | Score_C(E) = P(E\|C) | Heuristic search |
| Reinforcement learning | (none) | Trajectory T | Reward R(T) | RL algorithm |

| Result | Blame | Action |
|---|---|---|
| Score(correct) > Score(produced) | Search / optimizer | Widen beam; improve the RL algorithm |
| Score(correct) ≤ Score(produced) | Scoring / objective / reward | Improve how the score is learned or specified |

Run it across all dev-set errors and tally; a 95%/5% split caps optimizer work at ~5% of errors.

## 13. Pipeline vs end-to-end selection

| Factor | Pipeline | End-to-end |
|---|---|---|
| (input, output) pairs for the whole task | Scarce | Abundant |
| Labeled data for intermediate modules | Abundant | N/A |
| Training set size | Small | Large |
| Quality of hand-engineered abstractions | Good (MFCC, phonemes) | They cap you |
| Ng's verdict | Autonomous driving | Speech recognition |

Component selection criteria: (1) **data availability** for that component, (2) **task simplicity** of that component.

Task difficulty ladder (easiest → hardest): overexposed? → indoor/outdoor? → contains a cat? → contains a black-and-white cat? → contains a Siamese cat?

## 14. Error attribution by parts — procedure

For each dev-set mistake, with pipeline A → B → C (DAG order):
1. Replace **A**'s output with a hand-labeled perfect output; re-run B, C. Correct now? → blame **A**. Else:
2. Replace **B**'s output with a perfect output. Correct now? → blame **B**. Else:
3. Blame **C**.

Tally across ~100 errors (the book's example: 90 cat detector / 10 breed classifier → work on the detector). Then reuse the 90 failing cases for a deeper analysis of that component.

Informal alternative: how far is each component from human-level, where the human sees **only that component's inputs**? All components at human level + system below human level ⇒ the pipeline is flawed; add the missing information as a new component rather than overloading an existing one.

## 15. Project startup checklist

1. Choose dev/test from the distribution you must win on; same distribution for both. (< 1 week)
2. Choose one optimizing metric plus satisficing thresholds. (< 1 week)
3. Build a basic system. (a few days)
4. Measure training error, (training dev error), dev error; estimate the optimal error rate from best human performance.
5. Run error analysis on ~100 misclassified dev examples; split Eyeball/Blackbox if the dev set is large.
6. Apply the technique matrix to the dominant error source.
7. Plot learning curves before committing to any large data-collection effort.
8. Re-check the metric and dev/test whenever they stop pointing the team the right way.
