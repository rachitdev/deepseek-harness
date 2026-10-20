---
name: dsh-machine-learning-yearning
description: "Machine Learning Yearning (draft) by Andrew Ng, deeplearning.ai, 2018 — a strategy manual for prioritizing work on machine learning projects. Use when setting up dev/test sets, choosing a single-number evaluation metric, running error analysis, diagnosing bias vs variance vs data mismatch, reading learning curves, comparing to human-level performance, training and testing on different distributions, applying the Optimization Verification test, or deciding between end-to-end learning and a hand-designed ML pipeline."
---

# Machine Learning Yearning

**Author**: Andrew Ng (a deeplearning.ai project) · **Pages**: 118 · **Chapters**: 58 book chapters, grouped here into 12 study files · **Generated**: 2026-08-23

## How to Use This Skill
- **No arguments** — read this file top to bottom, then `cheatsheet.md` for the diagnostic ladder and decision tables.
- **With a topic** — check the Topic Index below, then open the named chapter file(s) plus the matching section of `patterns.md`.
- **With a chapter** — open `chapters/chNN-slug.md` directly; each contains the book's own numbers and worked examples.
- **Browsing** — `glossary.md` for terms, `patterns.md` for reusable When/How/Trade-offs moves, `cheatsheet.md` for thresholds and symptom→fix tables.

## Core Frameworks & Mental Models

**1. The diagnostic ladder.** Almost every "what should we do next?" question is answered by measuring four numbers and reading the gaps between them:

| Gap | Name | Fix family |
|---|---|---|
| 0 → optimal error rate | Unavoidable bias | None — it is the floor |
| optimal error rate → training error | **Avoidable bias** | Bigger model, less regularization, better architecture |
| training error → training dev error | **Variance** | More data, regularization, early stopping |
| training dev error → dev error | **Data mismatch** | Acquire or synthesize training data that matches dev |
| dev error → test error | Dev-set overfitting | Fresh/larger dev set |

The Training dev set is only needed when training and dev/test come from different distributions; otherwise variance = dev error − training error. Bias ≈ training error, informally. The same training error means opposite things at different optimal error rates: 15% training error is a bias emergency at a 0% Bayes rate and nearly optimal at a 14% Bayes rate.

**2. Dev/test sets are the steering wheel, not a leftover split.** "Choose dev and test sets to reflect data you expect to get in the future and want to do well on." Dev and test must share a distribution with each other (so a dev–test gap unambiguously means dev overfitting), but the *training* set need not match either. Size the dev set by the smallest difference you must detect (~10,000 examples to see 0.1%), not by a 70/30 ritual.

**3. One optimizing metric, N−1 satisficing metrics.** Two numbers give no ranking, so the team cannot decide. Combine (average, weighted average, F1) or declare thresholds in product units — "≤100ms", "no more than one false positive every 24 hours of operation" — and maximize the remaining metric under them. Never invent a coefficient like `Accuracy − 0.5×RunningTime`. Set the metric within a week and change it the moment it misranks the product.

**4. Error analysis converts opinions into ceilings.** Sample ~100 misclassified dev examples, tally them into categories in a spreadsheet, and read each category's percentage as the maximum error reduction that project can deliver. If 5% of errors are dogs, perfect dog handling moves 10% error to at best 9.5%. Two hours of counting routinely prevents a month of wasted engineering. Split large dev sets into an **Eyeball** set (you look; sized to hold ~50–100 mistakes) and a **Blackbox** set (you never look; 1,000–10,000) so you can detect manual-inspection overfitting.

**5. Build fast, then iterate.** Ship a basic system in a few days rather than designing the perfect one. Its mistakes are the roadmap. The loop is idea → code → experiment, and dev set + metric is what makes the experiment step cheap enough to run dozens of times.

**6. Learning curves settle "will more data help?" before you buy the data.** Plot dev error *and* training error against training-set size, with a desired-error line. Training error rises with data and dev error sits above it — so if the training curve is already above the target, more data provably cannot reach it. A plateaued dev curve says the same.

**7. Human-level performance = best achievable human performance.** Not average. A team of doctors at 2% proves the optimal rate ≤ 2%; a junior doctor's 10% constrains nothing. Human performance buys three things: high-quality labels, intuition about why an example is hard, and an optimal-error-rate estimate. After you surpass humans on average, find a subset where they still win and keep using all three.

**8. Consistency decides what data to include.** You may train on a different distribution than you test on. Include foreign data when a single f(x) serves both sources without knowing the source (internet vs mobile cat images); exclude it when the same x implies a different y (New York City vs Detroit house prices). Big networks make mixed training safe; if the ratio is extreme and compute is short, down-weight the foreign source (β = 1/40 for 40:1) instead of discarding it.

**9. Optimization Verification test.** Whenever a system learns an approximate score and maximizes it with an approximate search, one comparison assigns blame: if Score(correct output) > Score(produced output), the *search* failed; otherwise the *scoring function* is wrong. Tally the split across all dev-set errors before allocating engineers. Works for speech recognition (beam search), machine translation, and reinforcement learning (reward function vs RL algorithm) — and the reference output need only be *better*, not optimal.

**10. End-to-end vs pipeline is a data question.** End-to-end wins with abundant labeled data for *both ends*; hand-engineered components (MFCC, phonemes) supply human knowledge when data is scarce and cap performance when it is plentiful. Choose pipeline components by data availability and task simplicity — a cat detector plus a breed classifier each need far less data than a direct Siamese-cat classifier.

**11. Error analysis by parts, and the flawed-pipeline test.** Attribute each pipeline error to exactly one component by hand-labeling a "perfect" output for an upstream stage and re-running the rest. Then benchmark each component against a human *given the same inputs as that component*. If every component is at human level and the system is not, the pipeline itself is flawed — a component is being starved of information (e.g. lane markings) and you should add a component that supplies it, not overload an existing one.

## Chapter Index

| # | File | Book chapters | Covers |
|---|---|---|---|
| 1 | [chapters/ch01-ml-strategy-and-scale.md](chapters/ch01-ml-strategy-and-scale.md) | 1–4 | Why ML strategy, using the book with your team, prerequisites, scale as the driver of progress |
| 2 | [chapters/ch02-dev-and-test-set-design.md](chapters/ch02-dev-and-test-set-design.md) | 5–7 | Train/dev/test roles, same-distribution rule, dev and test set sizing |
| 3 | [chapters/ch03-evaluation-metrics-and-iteration.md](chapters/ch03-evaluation-metrics-and-iteration.md) | 8–12 | Single-number metrics, F1, optimizing vs satisficing, iteration speed, when to change metrics |
| 4 | [chapters/ch04-basic-error-analysis.md](chapters/ch04-basic-error-analysis.md) | 13–19 | Build fast then iterate, the ~100-example tally, mislabeled data, Eyeball/Blackbox split |
| 5 | [chapters/ch05-bias-and-variance.md](chapters/ch05-bias-and-variance.md) | 20–27 | Bias/variance definitions, avoidable bias, the modern tradeoff, technique menus |
| 6 | [chapters/ch06-learning-curves.md](chapters/ch06-learning-curves.md) | 28–32 | Plotting dev and training error vs data size, the three readings, noise control |
| 7 | [chapters/ch07-human-level-performance.md](chapters/ch07-human-level-performance.md) | 33–35 | The three uses of human performance, defining it, surpassing it |
| 8 | [chapters/ch08-training-on-different-distributions.md](chapters/ch08-training-on-different-distributions.md) | 36–39 | Role-based data allocation, the consistency test, capacity, weighting with β |
| 9 | [chapters/ch09-diagnosing-data-mismatch.md](chapters/ch09-diagnosing-data-mismatch.md) | 40–43 | Training dev set, three-way attribution, addressing mismatch, artificial data synthesis |
| 10 | [chapters/ch10-optimization-verification-test.md](chapters/ch10-optimization-verification-test.md) | 44–46 | Search vs objective blame assignment, general form, the RL/helicopter case |
| 11 | [chapters/ch11-end-to-end-deep-learning.md](chapters/ch11-end-to-end-deep-learning.md) | 47–52 | End-to-end vs pipelines, data availability, task simplicity, rich outputs |
| 12 | [chapters/ch12-error-analysis-by-parts.md](chapters/ch12-error-analysis-by-parts.md) | 53–58 | Component-level attribution, DAG ordering, flawed pipelines, conclusion |

## Topic Index

| Topic | Chapters |
|---|---|
| Artificial data synthesis | 9 |
| Avoidable vs unavoidable bias | 5, 7 |
| Bayes / optimal error rate | 5, 6, 7 |
| Beam search, search vs objective | 10 |
| Bias and variance diagnosis | 5, 6, 9 |
| Dev/test set design and sizing | 2, 3 |
| Distribution mismatch (train vs dev) | 8, 9 |
| Early stopping | 5 |
| End-to-end learning | 11 |
| Error analysis (dev set) | 4, 9 |
| Error analysis (training set) | 5 |
| Error analysis by parts | 12 |
| Eyeball / Blackbox dev sets | 4 |
| F1 score, precision, recall | 3 |
| Human-level performance | 7, 12 |
| Iteration speed | 1, 3 |
| Learning curves | 6 |
| Mislabeled examples | 4 |
| Model size and capacity | 5, 8 |
| Optimization Verification test | 10 |
| Optimizing and satisficing metrics | 3 |
| Pipeline design (data availability, task simplicity) | 11, 12 |
| Regularization (L1, L2, dropout) | 5 |
| Reinforcement learning / reward functions | 10 |
| Rich outputs (captioning, translation, TTS) | 11 |
| Scale (data + compute) | 1 |
| Single-number evaluation metric | 3 |
| Speech recognition examples | 5, 7, 9, 10, 11 |
| Team persuasion and adoption | 1, 12 |
| Training dev set | 9 |
| Weighting data (β) | 8 |

## Supporting Files
- [glossary.md](glossary.md) — ~70 terms with definitions and chapter references.
- [patterns.md](patterns.md) — the book's reusable moves as When / How / Trade-offs, grouped by the book's own parts.
- [cheatsheet.md](cheatsheet.md) — diagnostic ladder, technique matrix, default thresholds, tells-and-smells tables, templates.

## Scope & Limits
- **Edition**: the free draft edition of *Machine Learning Yearning*, © 2018 Andrew Ng, a deeplearning.ai project. Every page is watermarked "Machine Learning Yearning-Draft." The source PDF is titled `Ng-MLY01-13.pdf` but contains all **58 chapters** across 118 pages, not just chapters 1–13.
- **Text extraction**: the PDF has a clean text layer; all 58 chapters extracted completely. No OCR was required.
- **Figures not reproduced**: the book relies on ~20 diagrams that carry no extractable text — the performance-vs-data curves (ch 4), all learning curve plots (ch 28–32), the bias/variance/mismatch 2×3 grid (ch 41), the pipeline block diagrams (ch 47–52, 57), and the Siamese cat bounding-box images (ch 53–54). Their content is described in words in the chapter files, but the visual originals are not included.
- **Also missing from extraction**: the squared-error objective formulas in ch 39 and the arg max expressions in ch 44–45 appear as images in the PDF; they are described in prose here rather than reproduced as equations.
- **What is covered**: ML project *strategy* — prioritization, dataset design, metrics, diagnosis, pipeline architecture.
- **What is not covered**: how to implement any algorithm. There is no code in the book and no `## Code Examples` section in these chapter files. Optimization algorithms, network architectures, and training procedures are assumed knowledge; Ng points readers to the Coursera Machine Learning course (http://ml-class.org) and the Deep Learning specialization (http://deeplearning.ai).
- **What dates the material**: written in 2018. Examples predate the transformer/foundation-model era — the speech pipeline discussion assumes MFCC and phoneme stages, "large neural network" means something far smaller than a modern LLM, and the end-to-end-vs-pipeline verdict for autonomous driving reflects 2018 data availability. The diagnostic framework (bias/variance/mismatch, error analysis, metric design) transfers to modern practice; the specific technology judgments should be re-examined. The book also predates now-common concerns such as fairness auditing, privacy constraints on data collection, and pretrained-model fine-tuning, none of which it addresses.
