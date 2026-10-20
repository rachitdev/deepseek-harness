---
name: dsh-how-to-lie-with-statistics
description: "How to Lie with Statistics by Darrell Huff (Victor Gollancz 1954; Penguin/Pelican edition 1973, reprinted 1991), the classic primer on statistical deception and how to detect it. Use when evaluating survey results, polls, averages, percentages, charts and graphs, correlation-versus-causation claims, sample bias, margins of error, index numbers, or advertising and media statistics; when asked to critique or fact-check a statistic; or when Huff's five questions -- \"Who says so? How does he know? What's missing? Did somebody change the subject? Does it make sense?\" -- are wanted."
---

# How to Lie with Statistics

**Author**: Darrell Huff (pictures by Mel Calman in this edition; original illustrations by Irving Geis)
**Pages**: 129 (Penguin/Pelican edition)
**Chapters**: 10
**Generated**: 2026-08-23

## How to Use This Skill

- **No arguments** — start with the five questions below, then skim `cheatsheet.md` for the tells-and-smells table.
- **With a topic** (samples, averages, graphs, correlation, percentages) — go to the chapter named in the Topic Index, then to the matching pattern in `patterns.md`.
- **With a chapter** — open `chapters/chNN-slug.md` for study-depth notes: core idea, frameworks, the book's own cases and numbers, a worked example, and cross-links.
- **Browse** — `glossary.md` for terminology, `patterns.md` for the reusable moves as When/How/Trade-offs, `cheatsheet.md` for decision tables and thresholds.

## Core Frameworks & Mental Models

### The five questions (Chapter 10 — the book's payoff)

Run them in order on any statistic that matters. Source → method → omissions → substitution → plausibility.

1. **Who says so?** Look for bias — "the laboratory with something to prove for the sake of a theory, a reputation, or a fee; the newspaper whose aim is a good story; labour or management with a wage level at stake." Conscious bias includes selection of favourable data and shifted units of measurement. **Unconscious bias "is often more dangerous."** Beware the **O.K. name**: "When an O.K. name is cited, make sure that the authority stands behind the information, not merely somewhere alongside it."
2. **How does he know?** Is the sample biased, or self-selected? Is it large enough to permit any reliable conclusion? Is the correlation big enough to mean anything?
3. **What's missing?** The number of cases; the probable or standard error; the range; the base of an index; the comparison figure; the factor that actually caused the change; the kind of average.
4. **Did somebody change the subject?** "Watch out for a switch somewhere between the raw figure and the conclusion. One thing is all too often reported as another" — reported cases for cases, saying for doing, a redefined unit, a component for the whole, *post hoc* for *propter hoc*.
5. **Does it make sense?** Test the unproved assumption; compute the implied totals; round spurious precision back; refuse naive extrapolation.

### The two ways a sample fails (Ch 1, 3)
**Selection** and **size**. The random-sample test: *does every name or thing in the whole group have an equal chance to be in the sample?* Stratified random sampling is the affordable substitute and leaks at four points (wrong proportions, misclassification, quota gaming, non-random selection within strata). Assume non-responders differ, and that poll bias runs toward the richer, better-educated, more conventional respondent. **"The result of a sampling study is no better than the sample it is based on."**

### The three averages, and when the choice matters (Ch 2)
**Mean** (add and divide), **median** (half above, half below), **mode** (most frequent). They coincide only on a normal distribution. On skewed data — above all money — they diverge, and "nearly everybody is below average." Decision rule: *identify the distribution shape first; then demand the average by name, plus "average of what? who's included?"*

### Think in ranges, not points (Ch 3, 4)
Every sample figure carries an error term: **probable error** (covers half the cases) or **standard error** (about two-thirds). Two filters, in order: *is the difference real?* then *is it big enough to matter?* — "A difference is a difference only if it makes a difference." Significance floor: 5 per cent (19 in 20); 1 per cent is "practically certain." And an average without a range can be worse than useless.

### Pictures lie while every number stays true (Ch 5, 6, 9)
**Truncation** removes the zero line; **ordinate rescaling** compounds it; together they make 10 per cent out-shout 100 per cent. "Nothing has been falsified — except the impression that it gives," and it works better than adjectives because "it contains no adjectives or adverbs to spoil the illusion of objectivity." **The cube law**: scale a drawn object's height by *k* and the reader receives *k²* on the page, *k³* in the mind. Honest pictograph rule: *repeat an identical icon; never resize it.* Maps encode area, not the variable of interest.

### The semi-attached figure (Ch 7)
**"If you can't prove what you want to prove, demonstrate something else and pretend that they are the same thing."** The measurement is genuine; the *join* to the claim is false. Three probes: **"So what?"** · **"More than what?"** · **get the denominator** (per passenger-mile, per person at risk). Then rule out reporting artefacts — a change in definitions, diagnosis, or legal reporting duty produces a change in the count.

### Correlation triage (Ch 8)
Three types: **chance**, **real but direction unknown**, **real but neither causes the other**. Four rivals to dispose of before any causal language: chance, reverse causation, a third factor, and **the trend of the times** — "look first of all to see if it is not one of this type, produced by the stream of events." Also check cohort vs ageing, selection into the treatment, and the observed range (rain helps corn until it ruins it). "A correlation may be real and based on real cause and effect — and still be almost worthless in determining action in any single case."

### Percentage discipline (Ch 9)
Name the base, every time. Never add percentage increases. State whether profit is on cost or on selling price. A 20 per cent cut needs a 25 per cent rise to reverse; 50 per cent needs 100 per cent. Percentage points ≠ per cent. With small n, publish the count. Spurious decimals are bluff. Index numbers dance to the chosen base period and averaging method — one dataset yields "up 25%," "down 25%," or "no change."

### The stance
Huff is not an eliminativist. "Arbitrarily rejecting statistical methods makes no sense either. That is like refusing to read because writers sometimes use words to hide facts and relationships rather than to reveal them." The goal is "how to recognize sound and usable data in that wilderness of fraud." And the honest presenter's move, when two valid methods disagree, is to publish both.

## Chapter Index

| # | Chapter | Core idea | File |
|---|---|---|---|
| 1 | The Sample with the Built-in Bias | A result is no better than the sample; bias is built in and runs in a known direction | [chapters/ch01-sample-with-built-in-bias.md](chapters/ch01-sample-with-built-in-bias.md) |
| 2 | The Well-Chosen Average | Mean, median and mode are three answers; the unqualified word "average" hides which | [chapters/ch02-well-chosen-average.md](chapters/ch02-well-chosen-average.md) |
| 3 | The Little Figures That Are Not There | Omission — of n, significance, range, axis labels — is the deception | [chapters/ch03-little-figures-that-are-not-there.md](chapters/ch03-little-figures-that-are-not-there.md) |
| 4 | Much Ado about Practically Nothing | Probable error; a difference must be both real and large enough to matter | [chapters/ch04-much-ado-about-practically-nothing.md](chapters/ch04-much-ado-about-practically-nothing.md) |
| 5 | The Gee-Whiz Graph | Truncation and axis rescaling lie while every plotted number stays true | [chapters/ch05-gee-whiz-graph.md](chapters/ch05-gee-whiz-graph.md) |
| 6 | The One-Dimensional Picture | Scaling a drawn object by *k* delivers *k²* on the page and *k³* in the mind | [chapters/ch06-one-dimensional-picture.md](chapters/ch06-one-dimensional-picture.md) |
| 7 | The Semi-attached Figure | A genuine number attached to a different question from the one being answered | [chapters/ch07-semi-attached-figure.md](chapters/ch07-semi-attached-figure.md) |
| 8 | Post Hoc Rides Again | Association is not causation; storks, lice, smoking and grades, college and income | [chapters/ch08-post-hoc-rides-again.md](chapters/ch08-post-hoc-rides-again.md) |
| 9 | How to Statisticulate | The catalogue: maps, decimals, shifting bases, bogus addition, percentiles, indexes | [chapters/ch09-how-to-statisticulate.md](chapters/ch09-how-to-statisticulate.md) |
| 10 | How to Talk Back to a Statistic | **The five questions** — the book's defensive method | [chapters/ch10-how-to-talk-back-to-a-statistic.md](chapters/ch10-how-to-talk-back-to-a-statistic.md) |

## Topic Index

| Topic | Chapters |
|---|---|
| Sampling, sample bias, self-selection, non-response | 1, 3, 10 |
| Polls, opinion research, questionnaires, interviewer effects | 1, 7, 10 |
| Mean, median, mode; skewed distributions | 2, 9 |
| Sample size, significance testing, the law of averages | 3, 4, 10 |
| Range, deviation, probable and standard error | 3, 4 |
| IQ and test scores | 4 |
| Line graphs, axes, truncation, aspect ratio | 5 |
| Bar charts, pictographs, three-dimensional distortion | 6 |
| Maps as misleading pictures | 9 |
| Irrelevant and mismatched figures; rates vs counts | 7 |
| Reporting artefacts, changed definitions, diagnosis effects | 7, 10 |
| Correlation vs causation, confounders, cohort effects | 8 |
| Percentages, bases, discounts, percentage points, percentiles | 9, 10 |
| Index numbers, base periods, geometric mean | 9 |
| Spurious precision and decimals | 1, 9, 10 |
| Extrapolation and forecasting | 10 |
| Advertising and medical claims | 3, 4, 7 |
| Corporate and labour statistics, profits, wages | 2, 7, 9 |
| Interest rates and instalment buying | 10 |

## Supporting Files

- [glossary.md](glossary.md) — Huff's vocabulary, alphabetical, with chapter references.
- [patterns.md](patterns.md) — 26 reusable moves as **When / How / Trade-offs**, grouped by sample, summary number, picture, join, causation, and defence.
- [cheatsheet.md](cheatsheet.md) — the five questions, a tells-and-smells table, thresholds and defaults, the averages selection matrix, the cube-law table, correlation triage, percentage discipline, and a nine-step audit pipeline.

## Scope & Limits

**Edition.** Notes were taken from the Penguin/Pelican edition (first published by Victor Gollancz 1954; Pelican 1973; Penguin reprint 1991), 129 pages, with pictures by Mel Calman. Copyright 1954 by Darrell Huff and Irving Geis. This British edition converts many of Huff's original US dollar examples to pounds sterling and localises some references, so figures here may differ from the American edition; where the text itself quotes American sources, dollars are retained.

**Source was OCRed.** The PDF supplied had no text layer and was processed with `ocrmypdf`/Tesseract before extraction. OCR introduced numeric and typographic errors, most of them obvious (transposed digits, `1` for `l`, dropped decimal points, `ro` for `10`). Figures have been checked for internal arithmetic consistency and against the book's own worked calculations wherever possible, and obvious artefacts were corrected silently. A few numbers in the original are reported inconsistently across the OCR text — notably the Yale Class of '24 average income (rendered variously as $25,111, $24,111 and $26,111; **$25,111** is used here) and the Rotundian weekly wage in Chapter 6 (rendered as both £15 and £16; **£15** is used, since it is the figure the doubling argument requires). Index-number percentages in Chapter 9 appear as 26 and 24 in the OCR where the arithmetic gives **25**; 25 is used. Treat any individual figure as approximately reliable and verify against a printed copy before quoting it as fact.

**What is covered.** All ten chapters plus the Introduction and Acknowledgements. Every chapter file contains the book's own named cases and numbers.

**What is not covered.** The book's illustrations are described but not reproduced; several arguments (the moneybags, the three drawings of the national-income graph, the Oklahoma City temperature chart, the darkening-shadow maps, the *Time* dual chart) depend on pictures that these notes render verbally. Huff supplies no formulae beyond the arithmetic shown; there is no coverage of statistical computation, hypothesis-test mechanics, regression, Bayesian reasoning, or modern causal inference, and this skill does not add any. The brief's **Code Examples** section is intentionally omitted — this is not a technical book.

**What dates the material.** The examples are drawn from the 1930s–early 1950s American and British press: the *Literary Digest* 1936 poll, the Dewey and Truman campaigns, Kinsey, polio vaccine trials, cigarette advertising, wartime opinion research. Dollar and pound amounts are unadjusted for inflation. Several passages use racial terminology ("Negro," "black people," "coloured") that reflects mid-twentieth-century usage; quotations preserve the book's wording. Some statistical conventions have shifted since 1954 — probable error has largely been displaced by standard error and confidence intervals (Huff notes the transition in Chapter 4), "the Revised Stanford-Binet" has been superseded, and the discussion of IQ and of Gesell's norms reflects the psychometrics of its day. The reasoning, the taxonomy of tricks, and the five questions are the durable content.
