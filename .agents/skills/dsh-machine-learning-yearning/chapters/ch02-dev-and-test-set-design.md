# Chapter 2 — Designing Dev and Test Sets: Distribution and Size (Book Chapters 5–7)

## Core Idea
The dev and test sets are not leftovers from a data split — they are the **steering wheel of the project**. "The purpose of the dev and test sets are to direct your team toward the most important changes to make to the machine learning system." Three design rules follow: choose them to reflect the future data you care about, draw them from the *same* distribution as each other, and size them by the smallest performance difference you need to detect — not by a fixed percentage.

## Frameworks Introduced

**The three-set vocabulary (Ch 5).**
- **Training set** — which you run your learning algorithm on.
- **Dev (development) set** — which you use to tune parameters, select features, and make other decisions about the learning algorithm. Also called the **hold-out cross validation set**.
- **Test set** — which you use to evaluate performance, but *not* to make any decisions regarding what algorithm or parameters to use.

**The selection rule (Ch 5), stated as a boxed principle in the book:**
> Choose dev and test sets to reflect data you expect to get in the future and want to do well on.

Corollary: "your test set should not simply be 30% of the available data, especially if you expect your future data (mobile phone images) to be different in nature from your training set (website images)."

**Same-distribution rule (Ch 6).** Dev and test must come from the same distribution, for two reasons: (1) the dev set defines what the team optimizes, so it must cover *all* of what you care about; (2) mismatched dev/test makes a dev-set-good / test-set-bad result undiagnosable.

**Sizing heuristics (Ch 7).** Size the dev set by the resolution of the decisions it must support; size the test set by the confidence you need in the final number.

## Key Concepts
- **Distribution mismatch at deployment** — the opening failure: a team downloads cat/non-cat pictures from websites, splits 70/30, gets good train and test numbers, deploys to the mobile app, and performance is "really poor." Cause: users upload phone pictures — lower resolution, blurrier, poorly lit — a different distribution from website images.
- **Obsolescence of the 70/30 rule** — "Before the modern era of big data, it was a common rule in machine learning to use a random 70%/30% split." It still works when you have 100 to 10,000 examples, but the *fraction* devoted to dev/test has been shrinking as absolute dataset sizes grow into the billions.
- **Detection resolution** — a 100-example dev set cannot detect the difference between 90.0% and 90.1% accuracy. With 10,000 examples "you will have a good chance of detecting an improvement of 0.1%."
- **Overfitting the dev set as a clean diagnosis** — only available when dev and test share a distribution: "You have overfit the dev set. The obvious cure is to get more dev set data."
- **Statistical significance testing** — Ng's footnote: in theory you could test whether a change is statistically significant on the dev set; "in practice, most teams don't bother with this (unless they are publishing academic research papers), and I usually do not find statistical significance tests useful for measuring interim progress."

## Mental Models
- **The dev set is a target, not a sample.** Whatever you put in the dev set is what the team will spend the next months optimizing. Ask "is this the thing I want six engineers pointed at?" rather than "is this a fair random sample of my hard drive?"
- **Approximate the future when you cannot observe it.** Pre-launch, you may have no user data. Ng's fallback ladder: ask friends to take mobile phone pictures of cats and send them to you → launch and update dev/test with actual user data → as a last resort use website images, "aware of the risk of this leading to a system that doesn't generalize well."
- **Mismatch multiplies uncertainty.** "Working on machine learning applications is hard enough." Mismatched dev/test adds a second unknown on top of every experiment.

## Anti-patterns
- **Random 70/30 on whatever data you happen to have**, when the deployment distribution differs from it.
- **Splitting by segment.** The book's explicit counter-example: with data segmented into US, China, India, Other, putting US + India in dev and China + Other in test. The dev set then fails to represent the goal ("to do well on all four geographies, and not only two").
- **Assuming your training distribution equals your test distribution.** "Try to pick test examples that reflect what you ultimately want to perform well on, rather than whatever data you happen to have for training."
- **Excessively large dev/test sets.** "There is no need to have excessively large dev/test sets beyond what is needed to evaluate the performance of your algorithms."
- **Trusting a 3rd-party benchmark's split.** If the benchmark's dev and test sets come from different distributions, "luck, rather than skill, will have a greater impact on your performance."

## Reference Tables

**Dev set sizing**

| Situation | Dev set size | Rationale |
|---|---|---|
| Detecting a 0.1% difference | ~10,000 examples | "good chance of detecting an improvement of 0.1%" |
| Typical ML project | 1,000–10,000 examples | "Dev sets with sizes from 1,000 to 10,000 examples are common" |
| 100 examples | too small | cannot detect 90.0% vs 90.1% |
| Mature, high-stakes apps (advertising, web search, product recommendations) | much larger than 10,000 | teams chase 0.01% improvements with direct profit impact |

**Test set sizing**

| Data volume | Guidance |
|---|---|
| 100–10,000 examples total | The 30% heuristic still works well |
| Millions to a billion+ examples | Fraction shrinks; absolute size grows; size it to "give high confidence in the overall performance of your system" |

**Three failure modes when dev and test distributions differ (Ch 6)**

| # | Possibility | Implication |
|---|---|---|
| 1 | You overfit the dev set | Get more dev data — fixable |
| 2 | The test set is *harder* than the dev set | Algorithm may be doing as well as possible; no further improvement available |
| 3 | The test set is just *different* | "A lot of your work to improve dev set performance might be wasted effort" |

## Worked Example
**The cat app that broke on launch.** Your team downloads cat (positive) and non-cat (negative) pictures from various websites, splits the dataset 70%/30% into training and test, and builds a detector that "works well on the training and test sets." Deployed into the mobile app, performance is poor.

Diagnosis walk-through:
1. Compare the two distributions. Training/test = website images. Live traffic = phone images: lower resolution, blurrier, poorly lit.
2. The classifier "did not generalize well to the actual distribution you care about: mobile phone pictures." The 30% test set measured the wrong thing — it certified performance on a distribution nobody was serving.
3. Fix: rebuild dev and test from phone pictures. Pre-launch, approximate by asking friends to shoot mobile cat photos. Post-launch, refresh dev/test from real user uploads.
4. Keep website images — as *training* data. (Chapter 36 in `ch08-training-on-different-distributions.md` shows this is legitimate and often desirable; only dev/test must match the target distribution.)

**The four-market split, done wrong and right.** Data is segmented into (i) US, (ii) China, (iii) India, (iv) Other. Wrong: dev = {US, India}, test = {China, Other}. Right: dev and test are each drawn from all four markets, so the dev set encodes the actual goal of doing well everywhere, and a dev-vs-test gap unambiguously means dev overfitting.

## Key Takeaways
- Dev/test define the target; pick them from the distribution you want to win on, even if that differs from your training data.
- Dev and test must come from the same distribution as each other, or a dev-test gap becomes uninterpretable.
- Size the dev set by the smallest improvement you need to detect (commonly 1,000–10,000; ~10,000 to see 0.1%).
- Size the test set for confidence in the final number, not by a 30% ritual.
- Never use the test set to make algorithm decisions — that is exactly what the dev set is for.

## Connects To
- `ch03-evaluation-metrics-and-iteration.md` — the metric you compute *on* these sets, and when to change both.
- `ch04-basic-error-analysis.md` — Eyeball vs Blackbox splitting of the dev set.
- `ch08-training-on-different-distributions.md` — deliberately training on a different distribution than dev/test.
- `ch09-diagnosing-data-mismatch.md` — the fourth set (Training dev) that mismatch forces you to add.
