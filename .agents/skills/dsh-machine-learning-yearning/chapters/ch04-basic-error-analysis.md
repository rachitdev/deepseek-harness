# Chapter 4 — Basic Error Analysis: Build Fast, Then Count Your Errors (Book Chapters 13–19)

## Core Idea
Do not design the perfect system; **build a basic system in a few days, then let its mistakes tell you what to do next.** Error analysis — manually examining ~100 misclassified dev set examples and tallying them into categories — converts "which idea should we pursue?" into an arithmetic question about ceilings. Two hours of counting routinely saves a month of engineering.

## Frameworks Introduced

**Build first, iterate (Ch 13).** "Don't start off trying to design and build the perfect system. Instead, build and train a basic system quickly — perhaps in just a few days." Even a far-from-best system is valuable because examining how it functions surfaces clues about where to invest. (Footnote: this advice is for readers building AI applications, not those whose goal is publishing academic papers.)

**Error analysis (Ch 14).** The procedure:
1. Gather a sample of 100 dev set examples that your system **misclassified**.
2. Look at them manually and count what fraction fall into the category you are considering working on.

The count is a **ceiling** on how much the proposed work can help. Formal definition given in the book: "Error Analysis refers to the process of examining dev set examples that your algorithm misclassified, so that you can understand the underlying causes of the errors."

**Parallel evaluation via a spreadsheet (Ch 15).** Create a spreadsheet with one row per misclassified image and one column per candidate error category, plus a free-text Comments column. Tick every category that applies (one image can hit several, so column percentages need not sum to 100%). Percentages at the bottom give each idea's ceiling simultaneously. Categories are *emergent*: "once you start looking through examples, you will probably be inspired to propose new error categories" — e.g. noticing many mistakes on Instagram-filtered pictures and adding an Instagram column.

**Mislabeled-example accounting (Ch 16).** Add a **Mislabeled** column tracking examples where the human label y was wrong before the algorithm ever saw it. Decide whether to fix labels by comparing mislabeling error to the error you could otherwise be reducing.

**Eyeball / Blackbox dev set split (Ch 17–18).** Split a large dev set into:
- **Eyeball dev set** — the portion you look at with your eyes (for speech, Ng suggests calling it an "Ear dev set"). Used for error analysis.
- **Blackbox dev set** — the portion you never look at. Used for automatic evaluation, model selection, and hyperparameter tuning.

The split exists because manual inspection makes you overfit faster: "If you see the performance on the Eyeball dev set improving much more rapidly than the performance on the Blackbox dev set, you have overfit the Eyeball dev set."

## Key Concepts
- **Ceiling** — the maximum possible error reduction from fixing one category. If 5% of misclassified images are dogs, fixing dogs perfectly moves you from 10% error to at best 9.5% error (90% → 90.5% accuracy). If 50% are dogs, 10% → 5% error.
- **Emergent error categories** — Dog, Great cat (lions, panthers mistaken for house cats), Blurry, Instagram, Mislabeled.
- **Most helpful categories** — "ones that you have an idea for improving," though you should not restrict yourself only to those; the goal is intuition about promising areas.
- **Label-fixing symmetry** — "Whatever process you apply to fixing dev set labels, remember to apply it to the test set labels too so that your dev and test sets continue to be drawn from the same distribution."
- **Correct-example checking bias** — if you only re-check labels of examples the system got *wrong*, you introduce bias, because both the label and the algorithm can be wrong on an example. With 1,000 dev examples at 98% accuracy it's far easier to check the 20 errors than the 980 correct ones; that bias "is acceptable if you are interested only in developing a product," but is a problem for research papers or an unbiased test-set measure.
- **Eyeball set sizing driven by error rate** — at a 5% error rate you need ~2,000 Eyeball examples to accumulate ~100 mistakes (0.05 × 2,000 = 100). The better your classifier, the bigger the Eyeball set must be.
- **When to skip an Eyeball set** — on tasks even humans cannot do well, examining examples is much less informative; you might omit the Eyeball set entirely.

## Mental Models
- **Errors are a sample survey.** You are estimating population proportions from ~100 draws to allocate months of engineering. That is why the sample size guidance (10 / 20 / 50 / 100 mistakes) is about *estimation confidence*, not effort.
- **Two hours vs one month.** "Even if you take one minute per image, you'd be done in under two hours. These two hours could save you a month of wasted effort."
- **Manual looking is a training process for humans** — and, like training, it overfits. Hence the Blackbox reserve.
- **Label quality is a lever like any other.** Chapter 15 improves categories through algorithmic changes; Chapter 16 points out you can also improve the *Mislabeled* category — through better data labels.
- **Error analysis does not produce a formula.** "You also have to take into account how much progress you expect to make on different categories and the amount of work needed to tackle each one."

## Anti-patterns
- **Jumping straight to implementation.** "I've seen many engineers reluctant to carry out error analysis. It often feels more exciting to just jump in and implement some idea, rather than question if the idea is worth the time investment. This is a common mistake."
- **Approving a month of work on 3rd-party dog software without measuring the dog fraction first.**
- **Designing the perfect system up front** in a domain where you are not an expert.
- **Manually examining a whole 5,000-example dev set** when 500 well-chosen examples would do.
- **Fixing dev labels but not test labels** — recreates the Chapter 6 mismatch problem.
- **Fixing labels only on misclassified examples** when you need an unbiased accuracy estimate.

## Reference Tables

**The error-analysis spreadsheet (Ch 15, the book's own final tally)**

| Image | Dog | Great cat | Blurry | Comments |
|---|---|---|---|---|
| 1 | ✔ | | | Unusual pitbull color |
| 2 | | | ✔ | |
| 3 | | ✔ | ✔ | Lion; picture taken at zoo on rainy day |
| 4 | | ✔ | | Panther behind tree |
| … | … | … | … | … |
| **% of total** | **8%** | **43%** | **61%** | |

Reading: Dog work can eliminate at most 8% of errors; Great cat or Blurry are far larger. With enough people, put some engineers on Great cats and others on Blurry — the categories are independently attackable.

**Mislabeled-example decision (Ch 16)**

| Scenario | Overall dev error | Error from mislabeling | Error from other causes | Mislabeling share | Verdict |
|---|---|---|---|---|---|
| Early | 10.0% | 0.6% | 9.4% | 6% of errors | Not crucial to fix; "It might be fine not knowing whether your system has 10% or 9.4% overall error" |
| Later | 2.0% | 0.6% | 1.4% | 30% of errors | Worth fixing — decides whether true error is 1.4% or 2% |

Note the pattern: the *absolute* mislabeling error is unchanged at 0.6%; its *relative* weight grows as the system improves. "It is not uncommon to start off tolerating some mislabeled dev/test set examples, only later to change your mind."

**Eyeball dev set sizing (Ch 18)**

| Mistakes in Eyeball set | What you get |
|---|---|
| ~10 | "Very small" — hard to estimate category impact, but better than nothing for prioritization |
| ~20 | A rough sense of major error sources |
| ~50 | A good sense of major error sources |
| ~100 | A very good sense; some people analyze up to 500 |
| >1,000 | Ng has "rarely seen anyone manually analyze more than 1,000 errors" |

**Blackbox dev set sizing:** 1,000–10,000 examples is usually enough to tune hyperparameters and select models; 100 is small but still useful; more does little harm. If the dev set is too small to split, use the *entire* dev set as an Eyeball set and accept the higher overfitting risk — Ng considers the Eyeball set the more important of the two (on human-solvable tasks).

## Worked Example
**Should we spend a month on dog images?** Playing with the cat app, you notice it mistakes dogs for cats — some dogs do look like cats. A teammate proposes integrating 3rd-party software to improve dog handling. It will take a month, and they are enthusiastic.

The counting procedure:
1. Pull 100 misclassified dev examples.
2. Count the dogs.
- **If 5% are dogs:** 5% is the ceiling. At 90% accuracy (10% error), perfect dog handling yields at best 90.5% accuracy (9.5% error) — "5% less error than the original 10% error." A month for 0.5 points.
- **If 50% are dogs:** the ceiling is a 10% → 5% error drop, "a 50% relative reduction in error." Now the month is clearly worth it.

**Scaling the analysis on a 5,000-example dev set (Ch 17).** At 20% error the algorithm misclassifies ~1,000 dev images — too many to examine. Randomly take 10% of the dev set (500 examples) as the **Eyeball dev set**; you expect it to contain ~100 mistakes, exactly the analysis budget you want. The remaining 4,500 become the **Blackbox dev set** for automated evaluation and hyperparameter tuning. If Eyeball performance starts improving much faster than Blackbox performance, discard the Eyeball set and rebuild it — move more examples over from Blackbox or acquire new labeled data.

## Key Takeaways
- Ship a basic system in days; its errors are your roadmap.
- Hand-count ~100 misclassified dev examples into categories; the category percentage is the ceiling on that project's payoff.
- Use one spreadsheet to evaluate all candidate ideas in parallel; let new categories emerge while you look.
- Track a Mislabeled category; fix labels when mislabeling becomes a large *share* of remaining errors, and fix dev and test together.
- Split large dev sets into Eyeball (you look) and Blackbox (you don't) to detect and limit manual-inspection overfitting.
- Size the Eyeball set so it contains enough *mistakes* (~50–100), which means larger sets as your error rate falls.

## Connects To
- `ch03-evaluation-metrics-and-iteration.md` — error analysis is how you generate the next idea for the iteration loop.
- `ch05-bias-and-variance.md` — Chapter 26 applies the same tally to the *training* set when bias is high.
- `ch09-diagnosing-data-mismatch.md` — error analysis on the dev set is how you discover *what* differs between training and dev distributions.
- `ch12-error-analysis-by-parts.md` — the same counting logic applied to pipeline components rather than error categories.
