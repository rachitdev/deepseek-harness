# Chapter 8 — Training and Testing on Different Distributions: Which Data to Use (Book Chapters 36–39)

## Core Idea
In the big-data era it is normal and often correct for your **training set to come from a different distribution than your dev/test sets**. Keep dev/test faithful to what you care about, then feed the learning algorithm every extra dataset that is **consistent** with the target task — provided your model has the capacity to absorb it. Discard data only when it is *inconsistent* (the same x maps to a different y) or when it teaches nothing relevant. If the extra data dwarfs the in-distribution data and you lack compute, **down-weight** it rather than dropping it.

## Frameworks Introduced

**Split by role, not by shuffle (Ch 36).** With 10,000 user-uploaded (target-distribution) images and 200,000 internet images:
- Do **not** shuffle all 210,000 into train/dev/test. That would make ~205,000/210,000 ≈ **97.6%** of your dev/test data internet images, which "does not reflect the actual distribution you want to do well on."
- Instead: dev/test come from user images; training gets the internet images. Ng's refinement: put only 5,000 user images into dev/test and move the other 5,000 *into the training set*, giving a 205,000-example training set that contains some data from the dev/test distribution. (Chapter 42 explains why this pays off — it lets you compare in-distribution performance across the train/dev boundary.)
- The governing rule is unchanged: "Choose dev and test sets to reflect data you expect to get in the future and want to do well on."
- Dev and test still come from the same distribution as each other.

**The consistency test (Ch 37–38).** Include extra data if there exists a single mapping x → y that works for both sources — i.e. "there exists some system that inputs either an internet image or a mobile app image and reliably predicts the label, even without knowing the source of the image." If the same x implies a different y depending on source, the data is **inconsistent** and will hurt.

**The two effects of adding out-of-distribution data (Ch 37).**
1. It gives the network more examples of what cats do and do not look like; internet and mobile images share similarities, so knowledge transfers.
2. It forces the network "to expend some of its capacity to learn about properties that are specific to internet images (such as higher resolution, different distributions of how the images are framed)." That capacity is then unavailable for the distribution you care about.

Effect 2 is the *Sherlock Holmes attic* argument (from *A Study in Scarlet*, Arthur Conan Doyle): the brain is an attic with finite space, and "for every addition of knowledge, you forget something that you knew before. It is of the highest importance, therefore, not to have useless facts elbowing out the useful ones." Ng's resolution: **build a big enough attic.** "If you have the computational capacity needed to build a big enough neural network… you have enough capacity to learn from both internet and from mobile app images, without the two types of data competing for capacity."

**Weighting data (Ch 39).** When the extra data outnumbers in-distribution data heavily (200,000 internet vs 5,000 mobile — a **40:1 ratio**), add a weight **β** to the extra data's term in the objective. With squared error (chosen only to simplify the explanation — "not a good choice for a classification task"), the objective is the sum over the 5,000 mobile images plus **β** times the sum over the 200,000 internet images. Setting **β = 1/40** gives the two sources equal total weight; β can also be tuned on the dev set.

## Key Concepts
- **Consistent vs inconsistent data** — internet vs mobile cat images are consistent (label depends only on the picture). New York City vs Detroit housing prices are inconsistent: "Given the same x (size of house), the price is very different depending on where the house is."
- **The city-feature workaround** — Ng's footnote: you *could* fix the Detroit inconsistency by adding a feature indicating the city, making y unambiguous given x. "However, in practice I do not see this done frequently."
- **Representational capacity competition** — the mechanism behind the risk of adding foreign data; mitigated by model size.
- **Zero-benefit data** — scanned historical documents when your dev/test are casual pictures of people, places, landmarks, animals. "There is no point including this data as negative examples, because the benefit from the first effect above is negligible… Including them would waste computation resources and representation capacity."
- **Historical caution, now weakened** — "When using earlier generations of learning algorithms (such as hand-designed computer vision features, followed by a simple linear classifier) there was a real risk that merging both types of data would cause you to perform worse… But in the modern era of powerful, flexible learning algorithms — such as large neural networks — this risk has greatly diminished."
- **Academic framing** — most ML literature assumes train, dev, and test share a distribution. Related research areas: **domain adaptation**, **transfer learning**, **multitask learning** — but "there is still a huge gap between theory and practice," partly because luck (hand-designed features, unmodeled factors) heavily affects cross-distribution results, making systematic academic study difficult.

## Mental Models
- **Dev/test define the goal; training data is just fuel.** They serve different jobs and need not match.
- **Capacity, not purity, is the binding constraint.** The old advice ("only train on matched data") was a workaround for small models. With a big enough network the mixed-data downside is mostly compute.
- **Ask the consistency question first, the size question second.** Is there one f(x) that serves both sources? If no → exclude (or add a disambiguating feature). If yes → include, subject to capacity and compute.
- **Weighting is the cheap substitute for capacity.** "By weighting the additional Internet images less, you don't have to build as massive a neural network to make sure the algorithm does well on both types of tasks."

## Anti-patterns
- **Shuffling everything together.** The 97.6% calculation is the book's clearest single argument against it.
- **Discarding useful extra data out of superstition.** "Some engineers will warn you against including the 20,000 internet images" — advice inherited from the pre-deep-learning era.
- **Adding data that teaches nothing** (scanned historical documents for a cat detector) — pure compute and capacity waste.
- **Merging inconsistent datasets** (NYC + Detroit housing) when you only care about one of them: "putting the two datasets together will hurt your performance."
- **Ignoring dataset-ratio skew.** A 40:1 imbalance without weighting may need 40× (or more) the compute to model both sources well.
- **Training a small network on mixed distributions.** "If you do not have a big enough neural network… then you should pay more attention to your training data matching your dev/test set distribution."

## Reference Tables

**The cat app data allocation (Ch 36)**

| Approach | Training set | Dev/test | Problem |
|---|---|---|---|
| Shuffle all 210,000 | ~205,000 mixed | ~97.6% internet images | Dev/test measure the wrong distribution |
| Naive split | 200,000 internet | 10,000 user | Works, but no user data in training |
| **Ng's recommendation** | **205,000 = 200,000 internet + 5,000 user** | **5,000 user** | Training contains some dev/test-distribution data, enabling later mismatch diagnosis |

**Speech recognition analogue (Ch 36).** Voice-controlled map/navigation app: 20,000 examples of users speaking street addresses plus 500,000 clips of users speaking about other topics. Allocation: 10,000 street-address examples to dev/test; the remaining 10,000 plus the 500,000 others to training.

**Include-or-exclude decision table**

| Extra data | Consistent with target task? | Any transferable signal? | Verdict |
|---|---|---|---|
| Internet cat images vs mobile cat images | Yes — one f(x) predicts both | Yes | Include (with a big enough network) |
| Detroit house prices vs NYC house prices | **No** — same x, different y | — | Exclude (or add a city feature) |
| Scanned historical documents vs casual photos | Technically yes (no cats) | ~None | Exclude for computational reasons |
| 200,000 internet vs 5,000 mobile | Yes | Yes | Include, but weight with β ≈ 1/40 if compute-limited |

**When re-weighting is needed (Ch 39)** — only when you suspect the additional data has a very different distribution from dev/test, **or** the additional data is much larger than the data drawn from the dev/test distribution.

## Worked Example
**The cat app, end to end.** Users have uploaded 10,000 images, which you have manually labeled. You also downloaded 200,000 internet images.

1. *Reject the shuffle.* 210,000 shuffled → dev/test would be ~97.6% internet images. Rejected.
2. *Assign roles.* User images represent the distribution you care about → dev/test. Internet images are plentiful fuel for a data-hungry deep network → training.
3. *Hold back half the user images for training.* 5,000 to dev/test, 5,000 into the training set. Training set = 205,000 examples spanning both distributions.
4. *Check consistency.* Given a picture, a human can say cat/no-cat without knowing whether it came from the web or a phone. There is one f(x). Consistent → keep the internet data.
5. *Check capacity.* If you can afford "a neural network with a large enough number of hidden units/layers, you can safely add the 20,000 images to your training set. Adding the images is more likely to increase your performance." If not, matching the training distribution to dev/test matters more.
6. *Prune worthless data.* You also hold a large collection of scanned historical documents. They contain "nothing resembling a cat" and look nothing like the dev/test distribution — exclude them for computational reasons.
7. *Handle the ratio.* With 200,000 internet and only 5,000 mobile images (40:1), and no huge compute budget, add weight β to the internet term of the objective. β = 1/40 equalizes the two sources; tune β on the dev set if you want something in between.

**The inconsistency counter-example.** You want to predict New York City housing prices from house size x. You also have Detroit, Michigan data, where prices are much lower. Given the same size, y differs sharply by city. If you only care about NYC, "it would be better to leave out the inconsistent Detroit data." The structural difference from the cat case: for cats there is a source-independent f(x); for housing there is not, unless you add the city as a feature.

## Key Takeaways
- Dev/test must reflect the target distribution; the training set does not have to.
- Include extra data when a single x → y mapping serves both sources; exclude it when the same x implies different y.
- Big networks make mixed-distribution training safe; small ones do not.
- Exclude data with negligible transferable signal purely for compute and capacity reasons.
- Put a slice of target-distribution data into the training set — it enables the mismatch diagnosis in the next chapter.
- Use a weight β on the over-represented source when the ratio is extreme and compute is limited (β = 1/40 for a 40:1 ratio).

## Connects To
- `ch02-dev-and-test-set-design.md` — the selection rule this chapter re-invokes.
- `ch09-diagnosing-data-mismatch.md` — the Training dev set and the three-way bias/variance/mismatch diagnosis that this setup makes possible.
- `ch05-bias-and-variance.md` — capacity and regularization, the reason mixed data is now safe.
- `ch11-end-to-end-deep-learning.md` — data availability as a design driver, applied to pipelines.
