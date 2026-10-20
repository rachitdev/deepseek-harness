# Chapter 12 — Error Analysis by Parts, Flawed Pipelines, and Conclusion (Book Chapters 53–58)

## Core Idea
In a multi-component pipeline, the question "which part should we improve?" is answered by **attributing each dev-set mistake to exactly one component**. The informal method is to look at each stage's output and judge who erred; the formal method is to **replace an upstream component's output with a hand-labeled "perfect" output and re-run the rest of the pipeline**. Then benchmark each component against human-level performance — and if every component is at human level while the overall system is not, the **pipeline itself is flawed** and must be redesigned.

## Frameworks Introduced

**Error analysis by parts (Ch 53).** Using the Siamese cat pipeline — part 1 a cat detector that crops cats out of the image, part 2 a cat breed classifier that decides if it is a Siamese cat — "you can try to attribute each mistake the algorithm makes to one (or sometimes both) of the two parts of the pipeline." Informal procedure: inspect what each stage produced and decide who is to blame.

**The formal attribution test (Ch 54).** For an ambiguous case:
1. Replace the cat detector's output with a hand-labeled bounding box.
2. Run the corresponding cropped image through the cat breed classifier.
   - **Case 1:** even with a perfect bounding box the breed classifier still outputs y=0 → **the breed classifier is at fault**.
   - **Case 2:** with a perfect bounding box the breed classifier now outputs y=1 → **the cat detector is at fault**.
"In other words, run an experiment in which you give the cat breed classifier a 'perfect' input." This "allows you to estimate the fraction of errors due to each component of the pipeline, and therefore decide where to focus your attention."

**General case, A → B → C (Ch 55).** For each dev-set mistake:
1. Manually modify **A**'s output to be a "perfect" output and run B, C on it. If the algorithm now gives a correct output, attribute the error to **A**. Otherwise go to step 2.
2. Manually modify **B**'s output to be perfect. If the output is now correct, attribute the error to **B**. Otherwise go to step 3.
3. Attribute the error to **C**.

**DAG ordering requirement (Ch 55).** "The components of an ML pipeline should be ordered according to a Directed Acyclic Graph (DAG), meaning that you should be able to compute them in some fixed left-to-right order, and later components should depend only on earlier components' outputs. So long as the mapping of the components to the A→B→C order follows the DAG ordering, then the error analysis will be fine." Swapping A and B (detect pedestrians first, then detect cars) yields "slightly different results" but the analysis "would still be valid and give good guidance."

**Human-level comparison per component (Ch 56).** A more informal alternative to the rigorous procedure — ask:
1. How far is the Detect cars component from human-level performance at detecting cars?
2. How far is the Detect pedestrians component from human-level performance?
3. How far is the overall system from human-level performance — where the human "has to plan a path for the car given only the outputs from the previous two pipeline components (rather than access to the camera images)"?

**Spotting a flawed pipeline (Ch 57).** If each component is at or near human level *given its own inputs*, but the overall system falls far short of human level, "the only possible conclusion is that the ML pipeline is flawed." A component "is doing as well as it can given its inputs, but the inputs do not contain enough information." Ask what other information a skilled human driver needs — e.g. the location of lane markings — and redesign the pipeline to supply it (add a **Detect lane markings** component).

## Key Concepts
- **Blameless component** — in the Chapter 53 example the detector crops a pile of rocks; the breed classifier reasonably outputs y=0, and "a human classifying the cropped image above would also have predicted y=0. Thus, you can clearly attribute this error to the cat detector."
- **Ambiguous attribution** — a poorly cropped image that "a highly skilled human could arguably still recognize the Siamese cat from." Ng: "If the number of ambiguous cases like these is small, you can make whatever decision you want and get a similar result" — otherwise use the formal test.
- **Scenario** — in the self-driving world, a case where "the car chooses a more jarring steering direction than a skilled driver would."
- **Deeper second-level analysis** — attributing 90 of 100 errors to the cat detector also hands you "90 examples where the cat detector outputted incorrect bounding boxes. You can use these 90 examples to carry out a deeper level of error analysis on the cat detector."
- **Error analysis as data science** — "Carrying out error analysis on a learning algorithm is like using data science to analyze an ML system's mistakes… There is no one 'right' way to carry out error analysis." Ng invites experimentation beyond the documented design patterns.
- **The human-solvability advantage** — "Many error analysis processes work best when we are trying to automate something humans can do and can thus benchmark against human-level performance… If you are building an ML system where the final output or some of the intermediate components are doing things that even humans cannot do well, then some of these procedures will not apply."
- **Conclusion (Ch 58)** — Ng closes by returning to Chapter 2's framing: "The only thing better than being a superhero is being part of a superhero team. I hope you'll give copies of this book to your friends and teammates and help create other superheroes!"

## Mental Models
- **Give a component a perfect input and see if the system recovers.** That single experiment converts a judgment call into a measurement.
- **Attribution is a counting exercise, like error analysis by category.** 90 vs 10 out of 100 misclassified images is the same arithmetic as 43% Great cat vs 8% Dog in `ch04-basic-error-analysis.md`.
- **Compare each component to a human given the same inputs.** The path planner must be judged against a human who also sees only the car and pedestrian detections — not against a human looking at the road.
- **Component excellence does not imply system excellence.** The flawed-pipeline diagnosis exists precisely for the case where every part passes and the whole fails.

## Anti-patterns
- **Improving whichever component your team knows best**, without attribution.
- **Comparing a component to a human who has more information than the component does.** Chapter 56's question 3 is carefully worded to avoid exactly this.
- **Concluding "our models need more work" when the pipeline is starving a component of information** — the lane-marking case.
- **Fixing a flawed pipeline by dumping raw inputs into a downstream module.** The Chapter 57 footnote: feeding the raw camera image into the planning component "would violate the design principle of 'Task simplicity' described in Chapter 51, because the path planning module now needs to input a raw image and has a very complex task to solve. That's why adding a Detect lane markings component is a better choice."
- **Ignoring DAG order** when mapping real components onto A → B → C.
- **Expecting these procedures to work where humans cannot do the task** — they largely will not.

## Reference Tables

**Attribution outcomes for a two-part pipeline (Ch 53–54)**

| Detector output | Breed classifier output given that crop | Breed classifier output given a perfect box | Attribution |
|---|---|---|---|
| Crops a pile of rocks | y=0 (reasonable) | — | Cat detector |
| Good bounding box | Wrong | Wrong | Cat breed classifier |
| Poor crop (ambiguous) | y=0 | still y=0 | Cat breed classifier (Case 1) |
| Poor crop (ambiguous) | y=0 | y=1 | Cat detector (Case 2) |

**Self-driving pipeline mapped to A → B → C (Ch 55)**

| Label | Component |
|---|---|
| A | Detect cars |
| B | Detect pedestrians |
| C | Plan path for car |

Swapping A and B (pedestrians first) still satisfies the DAG and still yields valid guidance.

**Flawed-pipeline diagnostic (Ch 57)**

| Detect cars at human level? | Detect pedestrians at human level? | Plan path at human level *given only the earlier components' outputs*? | Overall system at human level? | Conclusion |
|---|---|---|---|---|
| Yes | Yes | Yes | **No** | The pipeline is flawed — inputs lack necessary information; redesign (e.g. add Detect lane markings) |
| No | — | — | No | Improve the Detect cars component |
| Yes | Yes | No | No | Improve the Plan path component |

## Worked Example
**The Siamese cat pipeline, 100 errors attributed (Ch 53).** The system misclassifies an image as *not* containing a Siamese cat (y=0) when the correct label is y=1. Manually examine what each step did:

- The cat detector produced a bounding box containing a pile of rocks. The breed classifier, given that crop, correctly output y=0. "Thus, the cat breed classifier is blameless: It was given of a pile of rocks and outputted a very reasonable label y=0." Error → **cat detector**.
- Had the detector produced a good bounding box and the breed classifier still failed, the error would go to the **breed classifier**.

Go through 100 misclassified dev set images: **90 errors attributable to the cat detector, 10 to the cat breed classifier.** "You can safely conclude that you should focus more attention on improving the cat detector." Bonus: those 90 bad-bounding-box examples become the input to a second, deeper error analysis of the detector itself.

**Resolving an ambiguous case formally (Ch 54).** The detector crops badly; the breed classifier outputs y=0 (no cat). But "a highly skilled human could arguably still recognize the Siamese cat from the poorly cropped image." Ambiguous. Run the test: hand-label the perfect bounding box, crop, and re-run the breed classifier.
- Still y=0 → the breed classifier is at fault even with a perfect input.
- Now y=1 → "if only the cat detector had given a more perfect bounding box, then the overall system's output would have been correct" → attribute to the cat detector.

**The self-driving scenario (Ch 55).** You test the car on a closed track and find a scenario where it chooses a more jarring steering direction than a skilled driver would.
1. Manually give A (Detect cars) a perfect output — go in and tell it where the other cars actually are. Run B and C, letting C use A's now-perfect output. If the car now plans a much better path, attribute the error to A. Otherwise:
2. Manually give B (Detect pedestrians) a perfect output. If the output is now correct, attribute to B. Otherwise:
3. Attribute the error to C (Plan path).

**The flawed pipeline (Ch 57).** Suppose all three human-level questions from Chapter 56 answer *yes*: car detection is at human level, pedestrian detection is at human level, and the path planner matches a human given only those two components' outputs. Yet the overall car performs significantly below a human who can see the camera images. The only conclusion: the pipeline is flawed. The planner is doing as well as it can given its inputs, but those inputs lack information a human driver uses — for instance, the location of lane markings. Redesign the pipeline with an added **Detect lane markings** component feeding the planner. Ng's stated principle: "if you don't think your pipeline as a whole will achieve human-level performance, even if every individual component has human-level performance (remember that you are comparing to a human who is given the same input as the component), then the pipeline is flawed and should be redesigned."

## Key Takeaways
- Attribute each dev-set error to exactly one pipeline component; the counts tell you where to invest.
- The formal test: hand-label a "perfect" output for an upstream component and re-run the rest of the pipeline.
- Generalize with the A → B → C ladder, respecting DAG ordering.
- Benchmark each component against a human *given the same inputs as that component*.
- All components at human level + system below human level ⇒ the pipeline is flawed; add the missing information as a new component rather than overloading an existing one.
- Attribution also produces a curated set of failing examples for a deeper analysis of the guilty component.
- Closing message (Ch 58): distribute the relevant chapters to teammates — "the only thing better than being a superhero is being part of a superhero team."

## Connects To
- `ch11-end-to-end-deep-learning.md` — how the pipeline was designed, and the task-simplicity principle invoked when redesigning it.
- `ch04-basic-error-analysis.md` — the same count-100-errors discipline, applied to components instead of categories.
- `ch07-human-level-performance.md` — the per-component human benchmark this chapter depends on.
- `ch10-optimization-verification-test.md` — the analogous blame-assignment test for score-and-search systems.
- `ch01-ml-strategy-and-scale.md` — Chapter 2's "superhero of your team" framing that Chapter 58 closes on.
