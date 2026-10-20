# Chapter 1 — The Sample with the Built-in Bias

## Core Idea

Every statistic that describes a large group is almost always built from a sample, and **the result of a sampling study is no better than the sample it is based on**. Huff's image: a river cannot rise above its source — "well, it can seem to if there is a pumping station concealed somewhere about." Statistical machinery downstream (decimal-pointed averages, projections, certifications) adds an "aura of conviction that a closer look at the sampling would deny." The chapter's job is to teach you to look at the sample *before* you look at the number.

Two ways a sample fails: it is **too small** (Chapter 3's subject) or it is **biased by the method of selection** (this chapter). Bias is usually not fraud — it is built in, and it runs in a predictable direction.

## Frameworks Introduced

**The barrel of beans.** To know exactly how many beans in the barrel are red, count them all. To know approximately, pull out a handful — "if your sample is large enough and selected properly, it will represent the whole well enough for most purposes. If, however, it fails in either respect it may be far less accurate than an intelligent guess."

**The test of the random sample.** *Does every name or thing in the whole group have an equal chance to be in the sample?* The "universe" is the statistician's word for the whole of which the sample is a part. Purely random samples are the only kind that "can be examined with entire confidence by means of statistical theory" — and are usually too expensive to obtain.

**Stratified random sampling** (the economical substitute used in opinion polling and market research). Divide the universe into groups in proportion to their known prevalence, then sample randomly within each. Its failure modes stack:
1. Your figures for the proportions may be wrong.
2. Interviewers misclassify — race usually right, income often wrong, occupation ambiguous ("how do you classify a man who farms part time and works in the city too?").
3. Age quotas get met by picking people obviously well under or well over the cutoff, so the late-thirties and early-forties vanish.
4. Randomness *within* a stratum is unaffordable, so you sample in the street (biases against stay-at-homes), by day (misses the employed), in the evening (misses movie-goers and night-clubbers). "You can't win."

**Self-selection.** The purest bias generator. Send out a questionnaire asking "Do you like to answer questionnaires?" and the No's throw it away — leaving "an overwhelming majority" who say yes.

**The three levels of sampling** (from the Kinsey critique). (1) The sample of the population; (2) the questionnaire is only a sample of the possible questions; (3) the answer given is only a sample of that person's attitudes and experiences on the question.

## Key Concepts

- **Built-in bias** — bias produced by the selection method, not by intent. "It is not necessary that a poll be rigged... The tendency of the sample to be biased in this consistent direction can rig it automatically."
- **The known direction of poll bias** — "towards the person with more money, more education, more information and alertness, better appearance, more conventional behaviour, and more settled habits than the average of the population he is chosen to represent." The interviewer with one interview to get approaches the neat man in patched overalls, not the dirty surly one — "and your colleagues all over the country are making similar decisions."
- **Non-response bias** — the people you *cannot reach* and the people who *will not answer* are systematically different from those who do.
- **Interviewer effect** — who asks changes the answer (see Worked Example).
- **The desire to give a pleasing answer** — "a tendency that must always be allowed for in reading poll results."
- **Reported behaviour ≠ behaviour** — a magazine-readership survey found *Harper's* beloved and *True Story* barely read, while circulation figures said the reverse. "About all the survey had uncovered was snobbery." Counting old magazines in people's houses works better — but even that "does not tell you what people read, only what they have been exposed to."
- **Invisible bias** — "even if you can't find a source of demonstrable bias, allow yourself some degree of scepticism about the result as long as there is a possibility of bias somewhere. There always is."

## Mental Models

- **"Sixty-seven per cent of *which* British people?"** No percentage of a population should be read without silently restating the universe it actually came from.
- **The Piccadilly caveat.** Every twentieth person met in Piccadilly is a random sample — of the people in Piccadilly at that moment. Not of London, not of England. (The interviewer who worked a railroad station because "all kinds of people can be found in a station" had to be told mothers of small children might be under-represented.)
- **The psychiatrist's patients.** A psychiatrist concluded practically everybody is neurotic. His sample was his patients. "If a man were normal, our psychiatrist would never meet him."
- **Running battle against bias.** Reputable pollers fight it constantly. "What the reader of the reports must remember is that the battle is never won."

## Anti-patterns

- Projecting from responders to the whole universe as if non-responders resembled them.
- Quoting an "allowance for error" on errors of unknown magnitude. Of Will Oursler's confident "even when allowances are made for error": "if he has discovered a method to compensate for errors of unknown magnitude, the world of statistics will be grateful."
- Treating a previously accurate sampling frame as permanently valid (the *Literary Digest*'s list had correctly called 1932).
- Trusting self-reported income, tooth-brushing, sexual history, or magazine reading. Some inflate from vanity, some minimise for the tax man; "it is possible that these two tendencies... cancel each other out, but it is unlikely. One tendency may be far stronger than the other, and we do not know which one."
- Spurious precision as a bias tell: a figure "surprisingly precise" and "quite improbably salubrious" is a double warning.

## Reference Tables

| Case | Sample as taken | Who is missing | Direction of error |
|---|---|---|---|
| *Christian Herald* conversions | 2,219 of 25,000 ministers replied | the ~90% with no conversions to report | Poling's projection: 4,144,366. Huff's rough re-projection using the same 181,000 total: ~370,000 — one eleventh as big |
| Yale Class of '24 income, $25,111 | addresses known + willing to reply + honest | "address unknown" (clerks, mechanics, tramps, unemployed alcoholics, barely surviving writers and artists) and wastebasket-flingers | inflated; true average "may well be nearer half that amount" |
| *Literary Digest* 1936 | 10 million telephone and *Digest* subscribers; predicted Landon 370, Roosevelt 161 | everyone who could not afford a telephone or a subscription in 1936 | loaded with Republican voters |
| Kinsey volumes | volunteers; 75% of women college-educated; prison residents | the taciturn who said no | leans toward sexual exhibitionists — confirmed by A. H. Maslow's Brooklyn College study, where later Kinsey volunteers were the more sexually unconventional and sophisticated students |
| Metric-system poll (Introduction) | Sunday paper's clipped-and-mailed coupons: 98% "knew" the metric system | readers uninterested enough not to clip | vs Gallup's cross-section, where 33% of *university-educated* respondents had never heard of it |

## Worked Example

**Deconstructing "the average Yale man, Class of '24, makes $25,111 a year."**

Two things stand out at first suspicious glance: *it is surprisingly precise* and *it is quite improbably salubrious.*

1. **Precision check.** "There is small likelihood that the average income of any far-flung group is ever going to be known down to the dollar." People in the $24,000 bracket have well-scattered investments; they do not know their own income to the dollar.
2. **Truthfulness check.** The figure is calculated from what the men *said* they earned, a quarter-century after graduation. Boasting and understating both occur; there is no reason to assume they cancel.
3. **Coverage check — the biggest error.** No one can reach all living members of the class. The lost addresses are not the rich: "the addresses of the rich will not be hard to come by," via *Who's Who in America* and other reference volumes. The lost names are "the men who, twenty-five years or so after becoming Yale bachelors of arts, have not fulfilled any shining promise... people of whom it would take half a dozen or more to add up to an income of $25,111." They also skip reunions, "if only because they cannot afford the trip."
4. **Response check.** Of those reached, many will not answer a personal questionnaire — and "it is at least a fair guess that many of them are just not making enough money to brag about."
5. **Verdict.** Two groups most likely to depress the average have been omitted. "If it is a true figure for anything it is one merely for that special group of the class of '24 whose addresses are known and who are willing to stand up and tell how much they earn."

## Key Takeaways

- Ask what the *universe* is, then ask who in it could not possibly have been sampled.
- Non-response is not neutral. Assume the missing differ from the present in the direction that flatters the sponsor.
- Precision is not accuracy; an implausibly exact figure is evidence of manipulation, not of care.
- Random sampling is the only fully analysable design and it is usually unaffordable; everything you actually read rests on a compromise.
- What people say is a different variable from what people do. Never let the report silently swap them.

## Connects To

- [Chapter 2 — The Well-Chosen Average](ch02-well-chosen-average.md): the Yale figure is also an unspecified average; Huff invites you to replay "what-kind-of-average-are-you" on it.
- [Chapter 3 — The Little Figures That Are Not There](ch03-little-figures-that-are-not-there.md): the other way a sample fails — being too small.
- [Chapter 4 — Much Ado about Practically Nothing](ch04-much-ado-about-practically-nothing.md): the 40% vs 35% readership survey is the same bias problem plus sampling error.
- [Chapter 10 — How to Talk Back to a Statistic](ch10-how-to-talk-back-to-a-statistic.md): "Who says so?" and "How does he know?" formalise this chapter.
