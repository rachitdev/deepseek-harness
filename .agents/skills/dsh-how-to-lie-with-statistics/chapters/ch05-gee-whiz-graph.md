# Chapter 5 — The Gee-Whiz Graph

## Core Idea

"There is terror in numbers." Because tabulated numbers repel readers and words often will not do the work, "there is one answer left: Draw a picture." The line graph is the simplest statistical picture — and it can be made to lie **while every plotted number remains correct**, because what the reader takes away is not the numbers but the visual slope.

Two operations do all the damage, and they compose:
1. **Truncation** — chop off the bottom of the chart, removing the zero baseline.
2. **Aspect distortion** — "change the proportion between the ordinate and the abscissa," so each vertical unit stands for less.

"The figures are the same and so is the curve. It is the same graph. Nothing has been falsified — except the impression that it gives."

## Frameworks Introduced

**The honest baseline chart.** Paper ruled into squares; months along the bottom; billions of dollars up the side; plot and draw. "He who runs may see and understand, because **the whole graph is in proportion and there is a zero line at the bottom for comparison**. Your ten per cent looks like ten per cent — an upward trend that is substantial but perhaps not overwhelming."

**Step 1 — Chop off the bottom.** "That is very well if all you want to do is convey information. But suppose you wish to win an argument, shock a reader, move him into action, sell him something. For that, this chart lacks schmaltz." Now "what the hasty reader sees is a national-income line that has climbed half-way up the paper in twelve months, all because most of the chart isn't there any more. Like the missing parts of speech in sentences that you met in grammar classes, it is 'understood'. Of course, **the eye doesn't 'understand' what isn't there**, and a small rise has become, visually, a big one."

Bonus defence, offered with a straight face: "You've saved paper too, something to point out if any carping fellow objects to your misleading graphics."

**Step 2 — Rescale the ordinate.** "Now that you have practised to deceive, why stop with truncating? You have a further trick available that's worth a dozen of that. It will make your modest rise of ten per cent look livelier than one hundred per cent is entitled to look... All you have to do is let each mark up the side stand for only one-tenth as many dollars as before." The example runs the axis from 20.0 to 21.4 billion. "Anyone looking at it can just feel prosperity throbbing in the arteries of the country."

**Why it beats prose.** "It is a subtler equivalent of editing 'National income rose ten per cent' into '...climbed a whopping ten per cent'. It is vastly more effective, however, because **it contains no adjectives or adverbs to spoil the illusion of objectivity. There's nothing anyone can pin on you.**"

## Key Concepts

- **The gee-whiz graph** — Huff's name (also "truncated" graph) for a chart whose baseline has been removed so a small change fills the frame.
- **Zero line** — the reference that makes proportional reading possible. Its absence is the single most reliable tell.
- **The undeclared break** — Columbia Gas "chopped off their chart at ninety per cent (**with no gap or other indication to warn you**)". Even an axis break honestly marked is a warning; an unmarked one is a trap.
- **Ordinate/abscissa ratio** — the aspect ratio is a free parameter with "no rule against it, and it does give your graph a prettier shape."
- **Respectable company.** "A news magazine has used this method to show the stock market hitting a new high, the graph being so truncated as to make the climb look far more dizzying than it was." Steel companies "have used similarly misleading graphic methods in attempts to line up public opinion against wage increases."
- **This is old news.** "Yet the method is far from new, and its impropriety was shown up long ago — not just in technical publications for statisticians either."

## Mental Models

- **The eye reads slope; the mind reads numbers.** Whichever the designer emphasises wins. A graph with big numbers and a suppressed baseline is engineered so the eye beats the mind.
- **Same data, two honest-looking headlines.** *Dun's Review* printed the fraudulent chart and its own version of the identical figures side by side, captioned **GOVERNMENT PAY ROLLS UP!** and **GOVERNMENT PAY ROLLS STABLE**. The rebuttal to a gee-whiz graph is to redraw it.
- **Read the little numbers and do the arithmetic.** The Columbia Gas case is only detectable by "if you read the little numbers and analysed them."
- **Two-step amplification.** Truncation alone exaggerates; truncation plus rescaling can make 10% out-shout 100%.

## Anti-patterns

- Removing the zero line on a chart of *levels* (as opposed to changes or rates) without saying so.
- Breaking an axis with no visible gap, jog, or note.
- Choosing an aspect ratio for drama and calling it "a prettier shape."
- Plotting a partial-period estimate against completed periods (the Collier's ad).
- Letting an exclamation point in the headline do work the figures do not support.

## Reference Tables

**Case file: what the numbers said vs what the chart said**

| Source | Actual figures | Visual impression | Mechanism |
|---|---|---|---|
| Huff's national income example | +10% over twelve months | line climbs half the page; then "prosperity throbbing" | truncation, then ordinate rescaled 10× |
| Columbia Gas System advertisement, "from our new Annual Report" | over ten years living costs **up about 60%**, cost of gas **down 4%** | "Living costs have more than tripled, and gas has gone down one-third!" | chart chopped at 90%, no gap or warning |
| Advertisement for advertising in Washington, D.C. (reproduced by *Dun's Review*, 1938), headline **GOVERNMENT PAY ROLLS UP!** | increase from about **$19,500,000 to $20,200,000** — under 4% | red line "shot from near the bottom of the graph clear to the top" — looks like **more than 400%** | truncation; *Dun's* redrew it honestly as a 4% rise, captioned **GOVERNMENT PAY ROLLS STABLE** |
| Collier's newspaper advertisement, 24 April 1953 | first nine months 1952: **3,104,729**; first quarter 1953: **3,205,000 est.** | steep climb | narrow axis band (3,150,000–3,200,000 gridlines); an *estimate* charted beside actuals; unequal time periods |
| News magazine stock-market chart | a new high | "far more dizzying than it was" | truncation |
| Iron and Steel Institute page (Ch. 6) | per-capita steelmaking capacity growth | doubled rate of climb | "a curve exaggerates... by getting along with the lower half of its graph missing. This saves paper and doubles the rate of climb." |

## Worked Example

**Building the gee-whiz graph in three drawings (Huff's own sequence).**

*Drawing 1 — the honest graph.* National income increased ten per cent in a year. Vertical axis: billions of dollars, gridlines from 0 up through 24. Months January to December along the bottom. The line rises modestly across the page. Verdict: "Now that's clear enough. It shows what happened during the year and it shows it month by month."

*Drawing 2 — truncated.* Keep the identical data and the identical curve. Delete everything below about 18–20 billion. The axis now starts near the level of the data. The line now occupies the full height of the frame. Verdict: "Now that's more like it." Nothing has been falsified — "except the impression that it gives."

*Drawing 3 — truncated and rescaled.* Keep the truncation. Now let each mark up the side stand for one-tenth as many dollars: gridlines at 20.0, 20.2, 20.4, 20.6, 20.8, 21.0, 21.2, 21.4. The same 10% rise now climbs at a near-vertical angle. Verdict: "That is impressive, isn't it?"

*Diagnosis procedure, run in reverse on any chart you meet:*
1. Find the vertical axis. Does it reach zero? If not, is the break marked?
2. Read the top and bottom values. Compute the actual percentage change from the numbers, not the picture.
3. Compare that number with the slope you perceived. The ratio between them is the size of the lie.
4. Redraw mentally with a zero baseline — the *Dun's Review* rebuttal.

## Key Takeaways

- A truthful chart of levels shows the zero line; the absence of it is the first thing to check.
- Truncation and axis rescaling are compoundable and leave every datum intact — there is no false statement to catch.
- Because the distortion carries "no adjectives or adverbs," it reads as objective. That is its advantage over prose.
- Always recompute the percentage change from the axis labels and compare it to the impression.
- The corrective is a redraw, not an argument.

## Connects To

- [Chapter 3 — The Little Figures That Are Not There](ch03-little-figures-that-are-not-there.md): charts with no numbers up the side at all (Grape-Nuts Flakes, the *Fortune* agency chart).
- [Chapter 6 — The One-Dimensional Picture](ch06-one-dimensional-picture.md): "A truncated bar chart has, and deserves, exactly the same reputation as the truncated line graph"; the Newsweek issue carried both a one-dimensional picture and a gee-whiz graph.
- [Chapter 9 — How to Statisticulate](ch09-how-to-statisticulate.md): the map as a distorting picture, and Time's honest dual chart as the counterexample.
- [Chapter 10 — How to Talk Back to a Statistic](ch10-how-to-talk-back-to-a-statistic.md): "What's missing?" applied to axes and bases.
