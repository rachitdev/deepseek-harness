# Glossary — Machine Learning Yearning (Andrew Ng)

Chapter references use this skill's grouped chapter files (Ch 1–12); the book's own chapter numbers (1–58) are given in parentheses where helpful.

**A → B → C error attribution** — General procedure for blaming a pipeline error on one component: perfect A's output and re-run; if fixed, blame A; else perfect B's output; else blame C (Ch 12, book ch 55).

**Artificial data synthesis** — Manufacturing training data that matches the dev/test distribution, e.g. adding car-noise audio to clean speech or simulated motion blur to sharp images (Ch 9, book ch 43).

**Autorotation** — Helicopter maneuver of landing with the engine off; Ng's reinforcement-learning example for the Optimization Verification test (Ch 10, book ch 46).

**Avoidable bias** — Training error minus the optimal error rate; the portion of bias you can actually remove. Negative values mean you have over-memorized the training set (Ch 5, book ch 22).

**Bayes error rate (Bayes rate)** — The statistics term for the optimal error rate (Ch 5, book ch 22).

**Beam search** — Approximate search that keeps only the K top candidates; the canonical "search algorithm" suspect in the Optimization Verification test. Widening the beam is the standard fix (Ch 10, book ch 44).

**Bias (informal)** — The algorithm's error rate on the training set (Ch 5, book ch 20).

**Bias vs. Variance tradeoff** — The classical trade in which reducing one raises the other; substantially weakened in the deep-learning era by big models plus regularization plus big data (Ch 5, book ch 24).

**Blackbox dev set** — The dev set portion you never look at, used for automatic evaluation, model selection, and hyperparameter tuning; 1,000–10,000 examples is typical (Ch 4, book ch 17–18).

**Blameless component** — A pipeline component that produced a reasonable output given the (bad) input it received; the error belongs upstream (Ch 12, book ch 53).

**Ceiling** — The maximum error reduction available from fixing one error category, estimated as that category's share of misclassified examples (Ch 4, book ch 14).

**Computational scale** — One of the two drivers of recent ML progress: the ability to train networks large enough to exploit modern datasets (Ch 1, book ch 4).

**Consistent data** — Extra training data for which a single mapping x → y also serves the target task, so the source of x need not be known. Internet vs mobile cat images are consistent (Ch 8, book ch 37–38).

**Data availability** — (i) One of the two drivers of ML progress (Ch 1, book ch 4). (ii) The first criterion for choosing pipeline components (Ch 11, book ch 50).

**Data mismatch** — Third error source: the algorithm generalizes to unseen data from the training distribution but not to the dev/test distribution (Ch 9, book ch 40).

**Desired error rate** — The target performance line drawn on a learning curve; sourced from human-level performance, product intuition, or experience (Ch 6, book ch 28).

**Dev (development) set** — Data used to tune parameters, select features, and make algorithm decisions. Also called the hold-out cross validation set (Ch 2, book ch 5).

**Directed Acyclic Graph (DAG) ordering** — Requirement that pipeline components be computable left-to-right with later components depending only on earlier outputs; needed for valid error attribution (Ch 12, book ch 55).

**Domain adaptation** — Research area on training on one distribution and generalizing to another; "typically applicable only in special types of problems" (Ch 8–9, book ch 36, 42).

**Early stopping** — Halting gradient descent based on dev set error; reduces variance, increases bias, behaves like regularization (Ch 5, book ch 27).

**Ear dev set** — Ng's suggested name for an Eyeball dev set on an audio task (Ch 4, book ch 17).

**End-to-end learning algorithm** — One learning algorithm mapping raw input directly to the desired output, connecting the input end to the output end (Ch 11, book ch 47).

**Error analysis** — Examining dev set examples the algorithm misclassified to understand the underlying causes and prioritize projects; typically ~100 examples (Ch 4, book ch 14).

**Error analysis by parts** — Attributing each pipeline mistake to a specific component (Ch 12, book ch 53).

**Error analysis on the training set** — The same ~100-example tally run on poorly handled training examples when bias is high (Ch 5, book ch 26).

**Eyeball dev set** — The dev set portion you manually examine; size it to contain enough *mistakes* (~50–100) (Ch 4, book ch 17–18).

**F1 score** — Harmonic mean of precision and recall, 2/((1/Precision)+(1/Recall)); a standard way to collapse two numbers into a single-number metric (Ch 3, book ch 8).

**Flawed pipeline** — A pipeline whose components are each at human level but whose overall performance is not, because a component's inputs lack necessary information (Ch 12, book ch 57).

**Hold-out cross validation set** — Alternative name for the dev set (Ch 2, book ch 5).

**Human-level performance** — The best achievable human performance on the task; used as a labeling source, an intuition source, and a proxy for the optimal error rate (Ch 7, book ch 33–34).

**Inconsistent data** — Data where the same x implies a different y depending on source; e.g. NYC vs Detroit house prices. Usually excluded (Ch 8, book ch 38).

**Instagram category** — Ng's example of an error category that emerges only once you start looking at misclassified examples (Ch 4, book ch 15).

**Kolmogorov Complexity** — Information-theory notion of a function's complexity as the shortest program producing it; noted as having "found few practical applications in AI" (Ch 11, book ch 51).

**Learning curve** — Plot of dev error (and training error) against training set size, with a desired-error line (Ch 6, book ch 28–29).

**Mislabeled example** — An example whose human-assigned label y was already wrong before the algorithm saw it (Ch 4, book ch 16).

**MFCC (Mel-frequency cepstrum coefficients)** — Hand-designed audio features that capture utterance content while discarding properties like speaker pitch (Ch 11, book ch 48–49).

**Optimal error rate** — The error achievable by an optimal classifier; also called unavoidable bias or the Bayes error rate (Ch 5, book ch 22).

**Optimization Verification test** — Test comparing Score(correct output) against Score(produced output) to decide whether a search algorithm or the scoring function is at fault (Ch 10, book ch 44–45).

**Optimizing metric** — The single metric you maximize, subject to the satisficing thresholds (Ch 3, book ch 9).

**Overfitting** — Low training error with much higher dev error; high variance (Ch 5, book ch 21).

**Overfitting the dev set** — Degradation caused by repeatedly evaluating ideas on the dev set; detected when dev performance far exceeds test performance (Ch 3, book ch 11).

**Overfitting the Eyeball dev set** — Detected when Eyeball performance improves much faster than Blackbox performance (Ch 4, book ch 17).

**Parser** — Pipeline component annotating text with structural information (e.g. labeling adjectives and nouns) before sentiment classification (Ch 11, book ch 47).

**Phoneme** — Linguist-invented basic unit of sound (the "k" in "keep" and "c" in "cake"); an imperfect intermediate representation in speech pipelines (Ch 11, book ch 48–49).

**Pipeline** — A system of multiple components, learned or not, where outputs feed into subsequent stages (Ch 11, book ch 47–48).

**Plateau** — A flattened learning curve; for older algorithms, the point where more data stops helping (Ch 1 and Ch 6, book ch 4, 28).

**Precision** — Fraction of examples the classifier labeled as cats that really are cats (Ch 3, book ch 8).

**Recall** — Percentage of all cat images that the classifier correctly labeled as cats (Ch 3, book ch 8).

**Regularization (L1, L2, dropout)** — Techniques that reduce variance and increase bias; what makes safely increasing model size possible (Ch 5, book ch 23, 25, 27).

**Representational capacity** — The finite modeling budget of a network, which foreign training data can consume; the "Sherlock Holmes attic" concern (Ch 8, book ch 37).

**Reward function R(·)** — The hand-designed scoring function in reinforcement learning; a genuine suspect under the Optimization Verification test (Ch 10, book ch 46).

**Sampling with replacement** — Drawing each learning-curve subset independently, so an example can appear in more than one subset (Ch 6, book ch 32).

**Satisficing metric** — A criterion your system merely has to be good enough on, expressed as a threshold (e.g. ≤100ms, ≤1 false wake per 24h) (Ch 3, book ch 9).

**Scenario** — Self-driving term for a test case where the car behaves worse than a skilled driver (Ch 12, book ch 55).

**Score_x(y)** — A learned or hand-designed function rating how good response y is for input x; the object maximized by an approximate search (Ch 10, book ch 44–45).

**Sentiment classification** — Recognizing positive vs negative opinions in text; Ng's opening end-to-end example (Ch 11, book ch 47).

**Single-number evaluation metric** — One number per model on the dev set, enabling a total ranking of candidates (Ch 3, book ch 8).

**Supervised learning** — Learning a function mapping x to y from labeled examples (x,y); the book's assumed prerequisite (Ch 1, book ch 3).

**Task simplicity** — Second criterion for choosing pipeline components: prefer components whose individual tasks are easy to learn from modest data (Ch 11, book ch 51).

**Test set** — Data used to evaluate final performance, never to make algorithm decisions (Ch 2, book ch 5).

**Training dev set** — Data drawn from the same distribution as the training set but never trained on; required to separate variance from data mismatch (Ch 9, book ch 40).

**Training set** — The data the algorithm learns from; need not share the dev/test distribution (Ch 2 and Ch 8, book ch 5, 36).

**Transfer learning** — Research area related to training and testing on different distributions (Ch 8, book ch 36).

**Unavoidable bias** — Another name for the optimal error rate — the part of bias no system can remove (Ch 5, book ch 22).

**Underfitting** — High training error with dev error barely higher; high bias, low variance (Ch 5, book ch 21).

**Variance (informal)** — How much worse the algorithm does on the dev set than on the training set (Ch 5, book ch 20).

**Wakeword** — A word that wakes a device ("Alexa", "Hey Siri", "Okay Google", "Hello Baidu"); Ng's optimizing/satisficing example (Ch 3, book ch 9).

**Weighting data (β)** — Multiplying the loss term of an over-represented data source by a weight; β = 1/40 equalizes 200,000 internet against 5,000 mobile images (Ch 8, book ch 39).

**70%/30% split** — The pre-big-data heuristic for train/test splitting; still fine for 100–10,000 examples, obsolete for very large datasets (Ch 2, book ch 5, 7).
