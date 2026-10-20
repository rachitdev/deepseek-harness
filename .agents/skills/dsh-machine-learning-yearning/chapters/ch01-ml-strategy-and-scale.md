# Chapter 1 — Why ML Strategy, How to Use the Book, Prerequisites, and Scale (Book Chapters 1–4)

## Core Idea
Machine learning progress is bottlenecked less by algorithms than by **choosing which direction to work on next**. Ng's premise: "Most machine learning problems leave clues that tell you what's useful to try, and what's not useful to try. Learning to read those clues will save you months or years of development time." The whole book is a manual for reading those clues. The second premise, from Chapter 4, is that the two reliable engines of modern progress are (i) training a bigger neural network and (ii) getting more data — but *how* to accomplish those two is "surprisingly complex," and that is what the remaining 54 chapters address.

## Frameworks Introduced

**The direction-choice problem (Ch 1).** The book opens with a cat-picture startup whose detector is not accurate enough. The team's candidate directions:
- Get more data: collect more pictures of cats.
- Collect a more *diverse* training set (cats in unusual positions, unusual coloration, varied camera settings).
- Train the algorithm longer, by running more gradient descent iterations.
- Try a bigger neural network (more layers / hidden units / parameters).
- Try a smaller neural network.
- Try adding regularization (such as L2).
- Change the architecture (activation function, number of hidden units, etc.).

"If you choose well among these possible directions, you'll build the leading cat picture platform… If you choose poorly, you might waste months." Every later framework in the book exists to convert this menu into a *diagnosis-driven* selection.

**The short-chapter distribution device (Ch 2).** Ng deliberately wrote 1–2 page chapters so a reader can print out just the pages a teammate needs to be convinced of one decision (e.g., adopting a single-number evaluation metric). The unit of persuasion is a chapter, not the book. "A few changes in prioritization can have a huge effect on your team's productivity."

**The scale thesis (Ch 4).** Two drivers of recent progress:
- **Data availability** — people spend more time on digital devices, generating huge amounts of data to feed learning algorithms.
- **Computational scale** — only in the last few years could we train networks big enough to exploit those datasets.

The accompanying performance-vs-data curves: older algorithms (e.g. logistic regression) **plateau** — the learning curve flattens and the algorithm stops improving even with more data. A Small NN does slightly better. Larger and larger NNs do better still. Best performance requires *both* conditions simultaneously: (i) train a very large neural network, so you are on the top ("green") curve, and (ii) have a huge amount of data.

## Key Concepts
- **Supervised learning** — the assumed prerequisite: learning a function mapping x to y from labeled examples (x,y). Includes linear regression, logistic regression, and neural networks. "The majority of Machine Learning's practical value today comes from supervised learning."
- **Deep learning / neural networks** — used throughout; only a basic understanding is required. Prerequisite pointer: the first three weeks of the Coursera Machine Learning course (http://ml-class.org).
- **Plateau** — the flattening of an older algorithm's learning curve; the algorithm "didn't know what to do with all the data we now have."
- **Small NN vs Large NN regime** — the small-data regime is *inconsistent*: with 20 training examples, hand-engineering of features matters more than logistic-regression-vs-NN. With 1 million examples, Ng favors the neural network.

## Mental Models
- **Clue-reading over guessing.** Treat a stuck model as a diagnostic problem with observable symptoms (training error, dev error, misclassified examples), not as a menu of tactics to try in random order.
- **The two levers first.** Before exotic architecture search, ask: can I make the model bigger, and can I get more data? "One of the more reliable ways to improve an algorithm's performance today is still to (i) train a bigger network and (ii) get more data." Architecture innovation matters but its results "are less predictable."
- **Strategy is a team artifact.** A correct technical direction that your teammates do not believe in does not get executed. Ng frames the reader as the person who sets technical direction *and* has to persuade.

## Anti-patterns
- **Picking the enthusiastic direction.** Choosing whichever idea the loudest or most excited engineer proposes, with no estimate of its ceiling.
- **Assuming more data always helps.** Chapter 4 says data is a driver; Chapter 20 will show that with 15% training error and a 5% target, more data "probably won't help much."
- **Assuming a small network is safer.** In the big-data regime, small networks leave performance on the table; the mitigation for a big model's overfitting is regularization, not shrinking the model (see ch05).
- **Optimizing for paper publication when you want a product.** Ng flags repeatedly that his advice targets people building AI applications, not those publishing academic papers.

## Reference Tables

| Regime | Data volume | What dominates performance |
|---|---|---|
| Small data (e.g. 20 examples) | tiny | Hand-engineering of features; algorithm choice barely matters |
| Modest data | thousands | Algorithm choice starts to matter; NNs may or may not win |
| Big data (e.g. 1M+ examples) | huge | Ng favors the neural network; larger NN → better performance |

| Driver | What changed | Consequence |
|---|---|---|
| Data availability | Digital-device usage generates huge datasets | Older algorithms plateau; NNs keep improving |
| Computational scale | Ability to train very large NNs | Big models can finally absorb big data |

## Worked Example
**The cat picture startup, framed as a decision.** You run a startup providing an endless stream of cat pictures. Your neural-network cat detector's accuracy "is not yet good enough" and you are under pressure to improve it. Seven concrete directions are on the table (listed above). The book's answer is *not* to rank them by intuition; it is to gather evidence first:

1. Later chapters will tell you to look at training error vs dev error to decide between "bigger model" (bias) and "more data" (variance) — see `ch05-bias-and-variance.md`.
2. They will tell you to hand-count 100 misclassified dev images to decide whether "more diverse data" or "3rd-party dog software" has a high enough ceiling to be worth a month — see `ch04-basic-error-analysis.md`.
3. They will tell you to plot dev error against training-set size to decide whether "get more data" can even reach your target — see `ch06-learning-curves.md`.

The chapter's payoff line: two hours of manual analysis can save a month of wasted effort. That trade ratio is the book's economic argument.

## Key Takeaways
- The scarce resource in an ML project is *correct prioritization*, not ideas.
- Modern progress comes from (i) bigger networks and (ii) more data; everything else in the book is about executing those two well and knowing when neither is the bottleneck.
- Older algorithms plateau with more data; large neural networks do not, which is why the two drivers compound.
- Chapters are short on purpose — hand a teammate the two pages that make the argument you need them to accept.
- Prerequisite floor: comfort with supervised learning and a basic idea of what a neural network is.

## Connects To
- `ch02-dev-and-test-set-design.md` — the first concrete strategy decision: what data you measure on.
- `ch05-bias-and-variance.md` — turns "bigger network or more data?" into a diagnosable question.
- `ch06-learning-curves.md` — tests the "more data will help" hypothesis before you buy the data.
- `ch12-error-analysis-by-parts.md` — the closing chapter (58) returns to the "superhero team" framing from Chapter 2.
