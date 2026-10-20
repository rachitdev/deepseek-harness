# Chapter 5 — Bias and Variance: The Two Big Sources of Error (Book Chapters 20–27)

## Core Idea
Almost every "what should we do next?" question reduces to a two-number reading: **training error** and the **gap between dev error and training error**. The first is (informally) bias; the second is variance. Once you also estimate the **optimal error rate**, bias splits into *unavoidable* and *avoidable* parts, and the avoidable part is the only bias worth chasing. High avoidable bias → make the model bigger. High variance → add data. In the deep-learning era, with regularization available, these two levers barely trade off against each other any more.

## Frameworks Introduced

**Informal bias/variance definitions (Ch 20).** Given 15% training error and 16% dev error:
- **Bias** = the algorithm's error rate on the training set (15%).
- **Variance** = how much worse it does on dev than on training (1%).

Ng notes statistics has more formal definitions ("Roughly, the bias is the error rate of your algorithm on your training set when you have a very large training set… When your error metric is mean squared error… you can prove that Total Error = Bias + Variance") but the informal version is what guides action.

**Avoidable / unavoidable bias decomposition (Ch 22).** With a nonzero **optimal error rate** (a.k.a. **Bayes error rate** or Bayes rate):
- **Optimal error rate ("unavoidable bias")** — the error even the best possible system would suffer.
- **Avoidable bias** = training error − optimal error rate.
- **Variance** = dev error − training error.
- Identity: `Bias = Optimal error rate (unavoidable bias) + Avoidable bias`.

Ng's caveat: "In theory, we can always reduce variance to nearly zero by training on a massive training set. Thus, all variance is 'avoidable'… there is no such thing as 'unavoidable variance.'" And a footnote: if avoidable bias comes out **negative**, you are doing better on the training set than the optimal rate — you have over-memorized the training set, so switch to variance reduction.

**The simplest formula (Ch 23).**
- If you have high avoidable bias → **increase the size of your model** (add layers/neurons).
- If you have high variance → **add data to your training set**.
"If you are able to increase the neural network size and increase training data without limit, it is possible to do very well on many learning problems."

**The modern (weakened) bias–variance tradeoff (Ch 24).** Classically, bigger model → less bias, more variance; more regularization → more bias, less variance. But "in the modern era, we often have access to plentiful data and can use very large neural networks… therefore, there is less of a tradeoff." Increase network size *and* tune regularization to cut bias without noticeably raising variance; add training data to cut variance without affecting bias.

**Error analysis on the training set (Ch 26).** When bias is high, run the same ~100-example tally you'd run on the Eyeball dev set — but on training examples the algorithm handles poorly. "Your algorithm must perform well on the training set before you can expect it to perform well on the dev/test sets."

## Key Concepts
- **Overfitting** — low training error, much higher dev error (1% / 11% → bias 1%, variance 10%).
- **Underfitting** — high training error, dev error barely higher (15% / 16% → bias 15%, variance 1%).
- **Simultaneously overfitting and underfitting** — 15% / 30% → bias 15%, variance 15%. "The overfitting/underfitting terminology is hard to apply here."
- **Why more data can't fix bias** — "adding more examples to your training set only makes it harder for your algorithm to do well on the training set." And: at 85% training accuracy with a 95% target, "there's no way you're getting 95% accuracy on examples your algorithm hasn't even seen."
- **Estimating the optimal error rate** — for tasks humans do well (recognizing pictures, transcribing audio), ask a human to label and measure human accuracy against your labels. Hard for tasks humans are bad at (movie recommendation, ad selection).
- **Regularization as the enabler of big models** — "If you include a well-designed regularization method, then you can usually safely increase the size of the model without increasing overfitting." With L2 or dropout tuned on the dev set, increasing model size "usually… will stay the same or improve; it is unlikely to worsen significantly. The only reason to avoid using a bigger model is the increased computational cost."
- **Architecture as a joint lever** — a well-suited architecture can reduce bias and variance simultaneously, but "selecting such an architecture can be difficult" and results are "less predictable than the simple formula of increasing the model size and adding data."

## Mental Models
- **Read three numbers, not one.** Optimal error rate, training error, dev error. The two gaps between them name your problem.
- **The same training error means different things at different Bayes rates.** 15% training error with a ~0% optimal rate is a bias emergency; with a 14% optimal rate it is nearly optimal.
- **The attic is not full any more.** The classical tradeoff assumed a fixed capacity budget. Big networks + regularization + big data broke that assumption for many problems.
- **Bias first, then variance.** If training error is far from the target, dev-set work is premature.

## Anti-patterns
- **Collecting more data to fix underfitting.** Ch 25 lists "Add more training data" explicitly under *not helpful* for bias: "This technique helps with variance problems, but it usually has no significant effect on bias."
- **Decreasing model size to fix variance.** "Use with caution… I don't recommend this technique for addressing variance. Adding regularization usually gives better classification performance." Shrink only when you want cheaper/faster training.
- **Chasing bias when training error is already at the Bayes rate** — e.g. 15% training error against a 14% optimal rate leaves only 1% avoidable bias.
- **Aggressive feature selection in the modern data-rich regime.** Going 1,000 → 900 features is unlikely to matter; 1,000 → 100 may raise bias significantly. "In modern deep learning, when data is plentiful, there has been a shift away from feature selection." It remains very useful when the training set is small.
- **Expecting an algorithm to transcribe audio a human cannot.** Ch 26: double-check whether a person could transcribe the clip given the same input; if not, it is unreasonable to expect the algorithm to.

## Reference Tables

**Diagnosing from training/dev error, optimal rate ≈ 0% (Ch 21)**

| Training error | Dev error | Bias | Variance | Diagnosis |
|---|---|---|---|---|
| 1% | 11% | 1% | 10% | High variance (overfitting) |
| 15% | 16% | 15% | 1% | High bias (underfitting) |
| 15% | 30% | 15% | 15% | High bias **and** high variance |
| 0.5% | 1% | 0.5% | 0.5% | Low bias, low variance — done |

**Same numbers, optimal rate = 14% (Ch 22, speech recognition with unintelligible clips)**

| Training error | Dev error | Unavoidable bias | Avoidable bias | Variance | Action |
|---|---|---|---|---|---|
| 15% | 30% | 14% | 1% | 15% | Variance reduction only |
| 15% | 16% | 14% | 1% | ~1% | Already near-optimal; ~2% from the best possible |

**Techniques for reducing avoidable bias (Ch 25)**

| Technique | Effect on bias | Effect on variance | Note |
|---|---|---|---|
| Increase model size (neurons/layers) | ↓ | may ↑ | Pair with regularization to cancel the variance increase |
| Modify input features from error-analysis insight | ↓ | may ↓ or ↑ | Can help both; regularize if variance rises |
| Reduce or eliminate regularization (L1/L2/dropout) | ↓ | ↑ | Direct trade |
| Modify model architecture | ↓ | ↕ | Affects both; less predictable |
| *Add more training data* | **no significant effect** | ↓ | Listed as "one method that is not helpful" for bias |

**Techniques for reducing variance (Ch 27)**

| Technique | Effect on variance | Effect on bias | Note |
|---|---|---|---|
| Add more training data | ↓ | — | "The simplest and most reliable way," given data and compute |
| Add regularization (L1/L2/dropout) | ↓ | ↑ | |
| Add early stopping (stop gradient descent early on dev error) | ↓ | ↑ | "Behaves a lot like regularization" |
| Feature selection (fewer input features) | ↓ | may ↑ | 1,000→900 negligible; 1,000→100 significant. Most useful with small training sets |
| Decrease model size | ↓ | ↑ | Use with caution; prefer regularization. Real benefit is compute cost |
| Modify input features from error analysis | ↕ | ↕ | Repeated from Ch 25 |
| Modify model architecture | ↕ | ↕ | Repeated from Ch 25 |

## Worked Example
**Speech recognition with a 14% Bayes rate.** You are building a speech recognizer and discover that 14% of audio clips have so much background noise, or are so unintelligible, that "even a human cannot recognize what was said." The optimal error rate is therefore ~14%.

Your system reports:
- Training error = 15%
- Dev error = 30%

Decomposition of the 30% total dev error:
- **Optimal error rate (unavoidable bias): 14%** — even the best possible speech system in the world would suffer this.
- **Avoidable bias: 1%** = 15% − 14%.
- **Variance: 15%** = 30% − 15%.

Verdict: "The training set performance is already close to the optimal error rate of 14%. Thus, there is not much room for improvement in terms of bias… however, this algorithm is not generalizing well to the dev set; thus there is ample room for improvement in the errors due to variance." So: add training data, add regularization, consider early stopping — and do **not** spend the quarter making the model bigger.

Contrast with the *identical* training/dev numbers when the optimal rate is ~0% (Ch 21's third example): there, 15% training error "leaves much room for improvement" and bias-reducing changes are fruitful. Same measurements, opposite prescriptions — which is why estimating the Bayes rate is worth the effort.

**Training-set error analysis for a speech app (Ch 26).** Your system does poorly on its own training set of volunteer audio clips. Listen to ~100 poorly handled training examples and tally:

| Audio clip | Loud background noise | User spoke quickly | Far from microphone | Comments |
|---|---|---|---|---|
| 1 | ✔ | | | Car noise |
| 2 | ✔ | | ✔ | Restaurant noise |
| 3 | | ✔ | ✔ | User shouting across living room? |
| 4 | ✔ | | | Coffeeshop |
| **% of total** | **75%** | **25%** | **50%** | |

Conclusion: the algorithm struggles most with background noise (75%), so focus on techniques that let it fit noisy training examples better. Also sanity-check that a human could transcribe those clips at all.

## Key Takeaways
- Bias ≈ training error; variance ≈ dev error − training error; avoidable bias = training error − optimal error rate.
- Estimate the optimal error rate (via human performance where possible) — it changes the diagnosis for identical error numbers.
- High avoidable bias → bigger model (+ regularization). High variance → more data (then regularization, early stopping).
- More data does not fix bias; shrinking the model is the wrong way to fix variance.
- Modern deep learning weakens the classical tradeoff: bigger model + tuned regularization ≈ lower bias at constant variance.
- When bias is high, run error analysis on the *training* set, and check that a human could even solve those examples.

## Connects To
- `ch04-basic-error-analysis.md` — the ~100-example tally protocol reused on training data.
- `ch06-learning-curves.md` — the graphical version of this diagnosis, and how to tell whether more data can reach the target.
- `ch07-human-level-performance.md` — how to actually estimate the optimal error rate.
- `ch09-diagnosing-data-mismatch.md` — adds a third error source when training and dev come from different distributions.
