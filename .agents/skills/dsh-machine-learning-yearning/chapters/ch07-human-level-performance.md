# Chapter 7 — Comparing to Human-Level Performance (Book Chapters 33–35)

## Core Idea
When humans do a task well, human performance is a **triple-purpose instrument**: a source of high-quality labels, a source of intuition about *why* an example is hard, and an estimate of the optimal error rate that lets you compute avoidable bias and set a credible target. Define human-level performance as the **best** human performance achievable on the task, not the average — and once you surpass humans overall, keep the instrument alive by finding subsets where humans still win.

## Frameworks Introduced

**The three uses of human-level performance (Ch 33).**
1. **Ease of obtaining data from human labelers.** People recognize cat images well, so it is straightforward for them to provide high-accuracy labels.
2. **Error analysis can draw on human intuition.** If a speech recognizer transcribes an audio clip as "This recipe calls for a *pear* of apples," mistaking "pair" for "pear," you can ask what information a person uses to get it right and modify the algorithm accordingly.
3. **Estimate the optimal error rate and set a desired error rate.** "Suppose your algorithm achieves 10% error on a task, but a person achieves 2% error. Then we know that the optimal error rate is 2% or lower and the avoidable bias is at least 8%. Thus, you should try bias-reducing techniques."

Ng adds that #3 is more valuable than it sounds: "having a reasonable and achievable target error rate helps accelerate a team's progress. Knowing your algorithm has high avoidable bias is incredibly valuable and opens up a menu of options to try."

**Defining human-level performance = best achievable human performance (Ch 34).** Given a spread of human error rates, take the lowest. The reasoning is that all three uses point the same way: the best humans can label for you, can explain their reasoning to you, and prove the optimal rate is at least that low. "The optimal error rate could be even lower than 2%, but it cannot be higher, since it is possible for a team of doctors to achieve 2% error. In contrast, it is not reasonable to use 5% or 10% as an estimate of the optimal error rate, since we know these estimates are necessarily too high."

**Subset-level comparison after surpassing humans (Ch 35).** Even when your average performance beats humans, "if you can identify a subset of data in which humans significantly surpass your system, then you can still use those techniques to drive rapid progress." All three uses reapply *on that subset*.

## Key Concepts
- **Human-level performance as a proxy for the optimal error rate** — the practical route to the Bayes rate discussed in `ch05-bias-and-variance.md`.
- **Labeling cost tiering** — you need not send every example to the expensive top tier. "Perhaps you can have a single junior doctor label the vast majority of cases and bring only the harder cases to more experienced doctors or to the team of doctors."
- **Precision of the baseline scales with your maturity** — "If your system is currently at 40% error, then it doesn't matter much whether you use a junior doctor (10% error) or an experienced doctor (5% error)… But if your system is already at 10% error, then defining the human-level reference as 2% gives you better tools to keep improving."
- **Tasks humans are bad at** — picking a book to recommend, picking an ad to show, predicting the stock market. Computers already surpass most people at these.
- **Why progress slows past human level** — "Only a subset of techniques apply once humans have a hard time identifying examples that the algorithm is clearly getting wrong. Consequently, progress is usually slower on problems where machines already surpass human-level performance, while progress is faster when machines are still trying to catch up to humans."

## Mental Models
- **Human performance is scaffolding, not a finish line.** Its value is that it makes bias measurable and errors explainable. Losing it (by surpassing humans, or by working on a task humans can't do) costs you diagnostic power, not prestige.
- **"Best human," not "average human."** Any weaker definition overestimates the optimal error rate and makes you stop improving too early.
- **Existence proof logic.** A team of doctors achieving 2% *proves* the optimal rate ≤ 2%. Nothing about a junior doctor's 10% constrains the optimal rate at all.
- **Look for the slice where humans still win.** "So long as there are dev set examples where humans are right and your algorithm is wrong, then many of the techniques described earlier will apply. This is true even if, averaged over the entire dev/test set, your performance is already surpassing human-level performance."

## Anti-patterns
- **Using average or amateur human performance as the baseline.** Using 15% (untrained person) or 10% (junior doctor) as "human-level" for the x-ray task overstates unavoidable bias and hides avoidable bias you could still remove.
- **Declaring victory at parity.** Beating average human performance while a strong subset of humans still beats you on a definable slice means real avoidable bias remains.
- **Sending every training example to the most expensive labelers.** Tier the labeling instead.
- **Expecting the three techniques to work where humans are weak.** On stock prediction, "pretty much no one can predict the stock market. So if our stock prediction algorithm does no better than random guessing, it is hard to figure out how to improve it."

## Reference Tables

**Medical imaging x-ray diagnosis — candidate baselines (Ch 34)**

| Who | Error rate | Use as human-level performance? |
|---|---|---|
| Typical person, no medical background beyond basic training | 15% | No |
| Junior doctor | 10% | No — but fine for bulk labeling |
| Experienced doctor | 5% | No |
| **Small team of doctors discussing and debating each image** | **2%** | **Yes** — Ng's choice, used as both the optimal-error-rate proxy and the desired performance level |

**When human-level performance is (and isn't) available**

| Task type | Examples | Labels | Human intuition | Optimal-rate estimate |
|---|---|---|---|---|
| Humans do well | Image recognition, speech recognition, email spam classification | Easy, high accuracy | Available | Reliable |
| Humans do poorly | Book recommendation, ad selection, stock prediction | Hard to obtain — "it's hard for human labelers to annotate a database of users with the 'optimal' book recommendation" | "Human intuition is harder to count on" | "It is hard to know what the optimal error rate and reasonable desired error rate is" |

**Data-acquisition workaround for weak-human tasks:** if you operate a site that sells books, you can obtain data by showing books to users and observing what they buy. "If you do not operate such a site, you need to find more creative ways to get data."

## Worked Example
**The x-ray diagnosis baseline decision (Ch 34).** You are building a system that makes diagnoses automatically from x-ray images. Four measured human error rates are available: untrained person 15%, junior doctor 10%, experienced doctor 5%, team of doctors debating each image 2%.

Ng picks **2%**, and justifies it against all three uses:
- *Labels:* "You can get a team of doctors to provide labels to you with a 2% error rate."
- *Intuition:* "By discussing images with a team of doctors, you can draw on their intuitions."
- *Optimal error rate:* 2% is achievable, so the optimal rate is ≤ 2%; 5% and 10% "are necessarily too high" as estimates.

Then the cost-management move: doctors' time is expensive, so route the vast majority of cases to a single junior doctor and escalate only the harder cases to experienced doctors or the full team.

And the maturity caveat: at 40% system error, junior vs experienced doctor barely matters. At 10% system error, adopting 2% as the reference is what keeps you improving — with a 5% reference you would see 5% avoidable bias, with a 2% reference you see 8%.

**Surpassing humans on noisy speech (Ch 35).** Your speech dataset has many noisy clips; humans reach 10% error, and your system already achieves 8%. Averaged over the dev set you have surpassed humans, so the three techniques appear dead. But suppose your system is much better than people on *noisy* audio while humans remain better at *very rapidly spoken* speech. On that subset:
1. You can still obtain human transcripts higher in quality than your algorithm's output.
2. You can draw on human intuition to understand why a person correctly heard a rapidly spoken utterance when your system didn't.
3. You can use human performance on rapidly spoken speech as the desired performance target.

Applications where machines already beat humans overall: predicting movie ratings, how long a delivery car takes to drive somewhere, whether to approve loan applications.

## Key Takeaways
- Human-level performance buys you labels, intuition, and an optimal-error-rate estimate — use all three.
- Define human-level performance as the **best** achievable human performance (the team of doctors, not the junior doctor).
- The baseline's precision matters more the better your system gets.
- After surpassing humans on average, hunt for subsets where humans still win and keep applying the same techniques there.
- On tasks humans cannot do well, expect harder labeling, weaker intuition, and no trustworthy target — and expect slower progress.

## Connects To
- `ch05-bias-and-variance.md` — human-level performance is how you fill in the optimal error rate that defines avoidable bias.
- `ch06-learning-curves.md` — supplies the "desired error rate" line on the plot.
- `ch04-basic-error-analysis.md` — human intuition is what turns a misclassified example into an error category.
- `ch12-error-analysis-by-parts.md` — Chapters 56–57 compare each *pipeline component* to human-level performance.
