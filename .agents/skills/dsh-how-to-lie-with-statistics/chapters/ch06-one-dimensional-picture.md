# Chapter 6 — The One-Dimensional Picture

## Core Idea

The **pictograph** — a chart in which "a little man represents a million men, a moneybag or stack of coins a thousand pounds sterling or a million dollars, an outline of a steer your beef supply for next year" — is a useful device with "what I am afraid is known as eye-appeal. And it is capable of becoming a fluent, devious, and successful liar."

The lie is geometric. A quantity is **one-dimensional** (a length on a scale). A drawn object is **two-dimensional on the page** and depicts something **three-dimensional in reality**. Scale the picture's height by 2 to represent a doubling, and the reader receives 4× (area) or 8× (volume). "While saying 'twice', I have left the lasting impression of an overwhelming eight-to-one ratio."

## Frameworks Introduced

**The ancestry of the pictograph.** "The daddy of the pictorial chart, or pictograph, is the ordinary bar chart, a simple and popular method of representing quantities when two or more are to be compared."

**Bar-chart red flags.** "Look with suspicion on any version in which
- the bars change their **widths as well as their lengths** while representing a single factor, or in which
- they picture **three-dimensional objects** the volumes of which are not easy to compare."
And: "A truncated bar chart has, and deserves, exactly the same reputation as the truncated line graph." Habitat: "the geography book, the corporation statement, and the news magazine."

**The honest pictograph rule — repeat the unit, never resize it.** With England at £30 a week and Rotundia at £15: "One moneybag for the unfortunate Rotundian's pittance, two for the Englishman's wage. Or three for the Rotundian, six for the Englishman. Either way, the chart remains honest and clear, and it will not deceive your hasty glance. **That is the way an honest pictograph is made.**"

**The dimensional escalation.** Draw one moneybag for £15 and a second **twice as tall** for £30. "That's in proportion, isn't it?"
- Because the second bag is twice as high, **it is also twice as wide**. "It occupies not twice but four times as much area on the page."
- Because these are pictures of objects with three real dimensions, "the second must also be twice as thick as the first. As your geometry book put it, **the volumes of similar solids vary as the cube of any like dimension.** Two times two times two is eight. If one moneybag holds £15, the other, having eight times the volume, must hold not £30 but £120."

| Ratio claimed | Height ratio drawn | Area on page | Implied volume |
|---|---|---|---|
| 2 : 1 | 2 | 4 | **8** |
| 1.5 : 1 | 1.5 | 2.25 | 3.4 |
| 3 : 1 | 3 | 9 | 27 |

**The confession of intent.** "To tell the truth (which, of course, is what I am planning not to do), I want you to infer something, to come away with an exaggerated impression, but I don't want to be caught at my tricks."

## Key Concepts

- **Eye-appeal as the payload.** The honest bar chart is "a clear and honest picture. Twice as much money is twice as big on the chart and looks it." Huff's complaint is only that "the chart lacks that eye-appeal." Everything after that is the cost of buying appeal.
- **The visual impression dominates.** "The numbers still say two to one, but the visual impression, which is the dominating one most of the time, says the ratio is four to one. Or worse."
- **Deniability.** "You'll have trouble pinning any criminal intent on me, too. I am only doing what a great many other people do."
- **Sloppiness that always runs one way.** "Some of this may be no more than sloppy draughtsmanship. But it is rather like being short-changed: **When all the mistakes are in the cashier's favour, you can't help wondering.**"
- **The category error the reader falls into.** Vary the size of the *object* and the hasty scanner may conclude the object itself has changed. On the milk-cow chart: "He may easily come away with the idea that cows are bigger now than they used to be." Applied to declining rhinoceros numbers you get "the diminishing rhinoceros" — "Ogden Nash once rhymed rhinosterous with preposterous. That's the word for the method too."
- **Yellow journalism.** Of Newsweek's life-expectancy figures: "This picture sensationalized facts in order to make a better story. I would call it a form of yellow journalism."
- **The fabricated-number aside.** Huff invents £30 and £15 for illustration and remarks: "You see how easy it is to hide behind the most disreputable statistic if you don't include any other information with it? You probably guessed I just made this one up for purposes of illustration, but I'll bet you wouldn't have if I'd used £29.35 instead."

## Mental Models

- **Length is the only honest visual encoding of a scalar** — hence bars of constant width, or repeated identical icons.
- **Ask "what does one icon equal?"** If the answer is "it depends on how big it is drawn," the chart is not a chart.
- **Cube-law check.** Whenever a picture is scaled in both dimensions, cube the linear ratio to find what the reader is being told.

## Anti-patterns

- Scaling a picture's height *and* width to encode a single number.
- Drawing objects with implied depth (moneybags, blast furnaces, human figures, animals) at proportional heights.
- Adding a horizontal stretch on top of the vertical one — a second, undeclared multiplier.
- Truncating a bar chart.
- Putting a truncated line graph and a one-dimensional picture on the same page and calling it a report.

## Reference Tables

**Case file: the escalation from stated ratio to received impression**

| Case | Stated figures | How drawn | What the eye received |
|---|---|---|---|
| Huff's moneybags | Rotundia £15 vs England £30 (2:1) | second bag twice as tall (hence twice as wide, twice as thick) | **8:1** (£120 rather than £30) |
| **Iron and Steel Institute** blast furnaces, "steel capacity added" (adapted from *Steelways*) | 10 million tons one decade vs **14¼ million tons** the next (about 1.4:1) | smaller furnace drawn "just over two-thirds as tall" as the larger | "one of them close to three times as big as the other" — "to say 'almost one and one-half' and to be heard as 'three'" |
| Same page, the molten-iron bar | a **50 per cent** increase | "the second furnace had fattened out horizontally beyond the proportion of its neighbour," and the black bar became **2½ times as long** | "a 50 per cent increase given, then drawn as 150 per cent to give a visual impression of... **over 1,500 per cent**. Arithmetic becomes fantasy." |
| Same page, per-capita capacity curve | growth in capacity per head | line graph "getting along with the lower half of its graph missing" | "This saves paper and doubles the rate of climb." |
| **Newsweek**, "Old Folks Grow Older" | life expectancy **68.2 years** today vs **34 years** in 1879–89 | two male figures, one twice as tall | "eight times the bulk or weight" |
| US milk cows | **just over 8 million in 1860**, **more than 25 million** about a century later (about 3:1) | two cows, one three times the height | 27:1 by volume — *and* the impression "that cows are bigger now than they used to be" |

Note the Institute's underlying thesis too: the chart existed "to show how the industry's steelmaking capacity had boomed between one decade and the next and so indicate that the industry was doing such a job on its own that any governmental interference was uncalled for. However shaky the thesis, it has more merit than there is in the way it was presented."

## Worked Example

**Two ways to chart England £30 vs Rotundia £15.**

*The honest versions.*
- **Bar chart.** Pounds per week up the left side; two bars of equal width; England's bar exactly twice the height of Rotundia's. "It is a clear and honest picture. Twice as much money is twice as big on the chart and looks it." Its only defect, from a propagandist's view, is that it "lacks that eye-appeal."
- **Repeated-icon pictograph.** One moneybag for Rotundia, two for England — every bag the same size. Or three and six. Either scaling works because *the icon is the unit*; only the count varies.

*The dishonest version.*
- Draw one moneybag for £15. Draw a second moneybag **twice as tall** for £30. This is "in proportion" in exactly one dimension and in no other.
- Height ×2 forces width ×2 → **4× the area of ink on the page**.
- The drawing depicts a solid object, so depth ×2 as well → **8× the volume**.
- Reader's takeaway: the Englishman earns something like **£120** to the Rotundian's £15.

*The audit, in reverse.* Given any picture-chart:
1. Read the two stated numbers and compute the true ratio.
2. Measure the two drawn heights and compute the height ratio.
3. Square it (page area) and cube it (implied solid).
4. Check whether width was stretched independently — if so, multiply again.
5. The gap between step 1 and steps 3–4 is the size of the deception. Then ask whether the icon count could have carried the message instead.

## Key Takeaways

- Encode quantity with **one** dimension — bar length, or a count of identical icons.
- Scaling a drawn object's height by *k* delivers *k²* on the page and *k³* in the reader's mind.
- The Institute's furnaces show the three multipliers stacking: proportional height, extra horizontal fattening, and a bar length of its own — 50% presented as 1,500%.
- Distorted pictographs also corrupt the *subject*: bigger cows, a diminishing rhinoceros.
- Errors that are all in the sponsor's favour are not draughtsmanship.

## Connects To

- [Chapter 5 — The Gee-Whiz Graph](ch05-gee-whiz-graph.md): truncation, the sibling trick, appears on the same pages.
- [Chapter 3 — The Little Figures That Are Not There](ch03-little-figures-that-are-not-there.md): unlabelled axes make the pictograph unauditable; Huff's own "£29.35" aside.
- [Chapter 9 — How to Statisticulate](ch09-how-to-statisticulate.md): "The Darkening Shadow" map is the same crime committed with geography instead of geometry.
- [Chapter 10 — How to Talk Back to a Statistic](ch10-how-to-talk-back-to-a-statistic.md): "Does it make sense?" applied to a picture.
