# Chapter 3 — The Little Figures That Are Not There

## Core Idea

The most effective statistical deception is **omission**. A figure is published; the figures that would let you evaluate it — sample size, significance level, range, deviation, the axis scale, the base — are quietly left out. "The deceptive thing about the little figure that is not there is that its absence so often goes unnoticed. That, of course, is the secret of its success."

Two families of missing figure dominate:
1. **Missing significance** — how likely is it that this result is real rather than chance?
2. **Missing dispersion** — what is the range or deviation around this average?

A statistician's opening question to any survey result: **"How many juries did you poll before you found this one?"**

## Frameworks Introduced

**The Doakes' toothpaste machine (how to manufacture a true headline from nothing).**
"Users report 23 per cent fewer cavities with Doakes' toothpaste."
1. Take a *small* group — the printed small type reveals a dozen persons.
2. Have them count cavities for six months, then switch to Doakes'.
3. Three outcomes are possible: distinctly more cavities, distinctly fewer, or about the same.
4. If it is the first or last, "Doakes & Company files the figures (well out of sight somewhere) and tries again."
5. "Sooner or later, by the operation of chance, a test group is going to show a big improvement worthy of a headline and perhaps a whole advertising campaign. This will happen whether they adopt Doakes' or baking soda or just keep on using their same old dentifrice."

**Why the group must be small.** "With a large group any difference produced by chance is likely to be a small one and unworthy of big type. A two-per-cent-improvement claim is not going to sell much toothpaste."

**The penny test.** Toss ten pennies; Huff got heads eight times — "which proves that pennies come up heads eighty per cent of the time. Well, by toothpaste statistics they do." A thousand tosses gets you close to half. "Only when there is a substantial number of trials involved is the law of averages a useful description or prediction."

**Test of significance.** "It is simply a way of reporting how likely it is that a test figure represents a real result rather than something produced by chance," most simply expressed as a probability. Huff's thresholds:

| Level | Meaning | Verdict |
|---|---|---|
| 19 chances in 20 (5%) | Census-grade precision | "For most purposes nothing poorer than this five per cent level of significance is good enough" |
| 99 chances in 100 (1%) | the demanded level for some purposes | "sometimes described as 'practically certain'" |

Why it is missing: "on the assumption that you, the lay reader, wouldn't understand it. Or that, where there's an axe to grind, you would."

**The other missing figure: range and deviation.** "Often an average — whether mean or median, specified or unspecified — is such an oversimplification that it is worse than useless. Knowing nothing about a subject is frequently healthier than knowing what is not so."

## Key Concepts

- **Adequate n depends on effect size and base rate**, not on looking impressive. A polio vaccine trial vaccinated **440 children** with **680 unvaccinated controls**; an epidemic followed; not one vaccinated child got recognisable polio — *and neither did any control.* At the usual incidence "only two cases would have been expected in a group of this size, and so the test was doomed from the start to have no meaning. Something like fifteen to twenty-five times this many children would have been needed."
- **Preliminary work sold as proof.** Dr Cornish's Tooth Powder claimed "considerable success in correction of... dental caries" on laboratory work done "on precisely six cases."
- **Self-curing conditions defeat naive trials.** Cold vaccines and antihistamines drew their popularity "from the unreliable nature of the ailment and from a defect of logic. Given time, a cold will cure itself." (Introduction: "proper treatment will cure a cold in seven days, but left to itself a cold will hang on for a week.") Physician's motto quoted: "Make haste to use a new remedy before it is too late."
- **The average family that does not exist.** Housing planned around the statistically average family of **3.6 persons** — i.e. two bedrooms. But three- and four-person families are "only 45 per cent of the total. Thirty-five per cent are one-person and two-person; 20 per cent have more than four persons." Result: areas overbuilt with two-bedroom houses. "Common sense has somehow failed in the face of the convincingly precise and authoritative 3.6."
- **"Normal" ≠ "desirable."** Gesell's norms, stripped of their range, make half of all parents conclude their child is "retarded" — "since half the children are bound to fail to sit by the time mentioned." The remedy is to publish the *range*: "Parents seeing that their youngsters fall within the normal range will quit worrying about small and meaningless differences. Hardly anybody is exactly normal in any way, just as one hundred tossed pennies will rarely come up exactly fifty heads and fifty tails." Kinsey met the same confusion: he said practices were **usual**, "which is what normal means," and was accused of endorsing them.
- **Meaningless comparatives.** *Fortnight* reported "a new cold temper bath which triples the hardness of steel, from Westinghouse." Three times *what*? "It becomes as elusive as a ball of quicksilver."
- **Weasel words.** "Today, electric power is available to more than three-quarters of U.S. farms." The gimmick is *available* — which "could mean that the power lines go past their farms or merely within ten or a hundred miles of them." Paraphrase honestly: "Almost one-quarter of U.S. farms do not have electric power available today."
- **Graphs with no numbers.** Grape-Nuts Flakes wrappers show a steeply climbing red "energy release" line — one edition starts one minute after eating, the other two minutes; one climbs twice as fast as the other; one has numbers up the side, one has none, "which is just as well, since there is no hint of what the numbers mean." An advertising agency's growth chart in *Fortune* had no numbers at all: "with equal honesty this chart could have represented a tremendous growth, with business doubling or increasing by millions of dollars a year, or the snail-like progress of a static concern adding only a dollar or two to its annual billings."

## Mental Models

- **The armchair correspondent.** Unenterprising journalism passes numbers along without asking what they mean — "a process by which the contents of the textbook of the instructor are transferred to the notebook of the student without passing through the heads of either party."
- **Averages hide individuals.** A magazine chart claimed "You Can Tell Now HOW TALL YOUR CHILD WILL GROW" from percentage-of-adult-height by age — while the article itself explains that children grow on different schedules. "For the total, or average, heights of a hundred youngsters taken at random it is no doubt accurate enough, but a parent is interested in only one height at a time, a purpose for which such a chart is virtually worthless." Huff's own test: the chart predicted five feet eight; he is five feet eleven. "A three-inch error in human height comes down to a poor grade of guess."
- **The camp-site chooser.** Picking a camp from mean temperature alone leaves you "as blind as a man choosing a camp site from a report of mean temperature alone."

## Anti-patterns

- Running the trial repeatedly and publishing only the run that worked (and filing the rest "well out of sight somewhere").
- Reporting a percentage improvement with no n, no control, and no significance figure.
- Designing a study whose expected event count is near zero (the polio trial).
- Planning for the mean of a distribution that has no mass at the mean (the 3.6-person family).
- Reading "normal" as "good," "right," "desirable."
- Publishing a trend line with an unlabelled or absent vertical axis.

## Reference Tables

**Same mean, different world — the range that the average conceals**

| Location | Mean annual temperature | Range |
|---|---|---|
| San Nicolas Island, off the south California coast | 61°F | 47° to 87°F |
| Inland California desert | 61°F | 15° to 104°F |
| Oklahoma City (60 years, 1890–1952) | 60.2°F | record low −17°F to record high 113°F — **a range of 130 degrees** |

"You might take 61 degrees as a comfortable annual mean... But you can freeze or roast if you ignore the range."

## Worked Example

**Auditing "Users report 23 per cent fewer cavities with Doakes' toothpaste."**

What is supplied: a big-type percentage; a "reassuringly *independent*" laboratory; certification by a chartered accountant.

What is missing, and what each absence permits:

| Missing figure | Where it actually was | What it lets Doakes do |
|---|---|---|
| Sample size | small type: **a dozen persons** | make chance variation large enough to headline |
| Number of trials run | never disclosed | keep testing until a run comes out big; file the rest |
| Control group | none described | attribute to Doakes' what baking soda or the old dentifrice would have produced |
| Significance level | absent | prevent you from asking whether 23% is distinguishable from noise |

Huff's concession: "You have to hand it to Doakes, at that, for giving you a sporting chance. Some advertisers would omit this information and leave even the statistically sophisticated only a guess as to what species of chicanery was afoot."

Cross-check with experience — the outside view: "if you are not outstandingly gullible or optimistic, you will recall from experience that one toothpaste is seldom much better than any other."

## Key Takeaways

- The absent figure is the deception. Make a habit of listing what *should* be there before evaluating what is.
- Demand four things with any comparative claim: **n**, **control**, **significance level**, **range**.
- 5% (19 in 20) is Huff's floor for "good enough"; 1% (99 in 100) is "practically certain."
- Small samples do not merely add noise — they are *selected for* precisely because they produce large chance differences.
- An average without a range can be "worse than useless"; the Oklahoma City mean of 60.2° hides 130 degrees.
- "Normal" is a statement of frequency, never of merit.

## Connects To

- [Chapter 1 — The Sample with the Built-in Bias](ch01-sample-with-built-in-bias.md): the other sampling failure — selection rather than size.
- [Chapter 2 — The Well-Chosen Average](ch02-well-chosen-average.md): the range is the figure Chapter 2 promised to supply here.
- [Chapter 4 — Much Ado about Practically Nothing](ch04-much-ado-about-practically-nothing.md): what happens when you ignore the probable error you were never given.
- [Chapter 5 — The Gee-Whiz Graph](ch05-gee-whiz-graph.md): the missing zero line and the missing axis numbers.
- [Chapter 10 — How to Talk Back to a Statistic](ch10-how-to-talk-back-to-a-statistic.md): the third question, "What's missing?"
