# Chapter 1 — Problem Solving Is Search

*Part I — See: Learning from Others. Maxim 1 of 12.*

> "Whenever one cannot go from the given situation to the desired situation simply by action, then there has to be recourse to thinking." — Karl Duncker

Chapter's opening questions: How do people solve hard problems? Are there generic methods that work on all kinds of problems? How do we solve problems that no one has ever solved before?

## Core Idea

Problem solving is **search through a problem space** — a start state, a goal state, and a set of legal moves that constrain the path between them (Herbert Simon and Allen Newell, *Human Problem Solving*, 1972). Because almost every real problem space is astronomically large, the difference between a solver who succeeds and one who flounders is almost never "general problem-solving ability." It is the **stock of strong methods** — domain-specific knowledge and heuristics — that prune the search. You get better at solving problems by acquiring other people's strong methods, not by exercising a generic problem-solving faculty.

## Frameworks Introduced

### The problem space
- **Start state** — where you are (a scrambled Rubik's Cube; a blank document; previously proven theorems).
- **Goal state** — recognizable when reached (one color per side; a finished chapter; the statement that aⁿ + bⁿ = cⁿ has no whole-number solutions for n > 2).
- **Legal moves / constraints** — the "walls of the maze." For Wiles, each move had to be a *valid deduction*.
- Problem spaces are usually abstract (spaces of configurations, hypotheses, experiments, designs), not physical.

### Strong methods vs. weak methods
- **Strong methods** — algorithms guaranteed to work, or domain-specific heuristics that drastically cut the search. Examples from math: **proof by induction**; **look for an invariant**.
- **Weak methods** — general fallbacks used when no strong method is available. Simon and Newell identified four:

| # | Weak method | How it works | Where it fails |
|---|---|---|---|
| 1 | **Generate-and-test** | Try something, see if it works (cycling old passwords; writing anything then editing) | Disastrous when the problem space is large; only viable when the space is already constrained or familiar |
| 2 | **Means-ends analysis** | Identify the gap between current and goal state, find a move that reduces it, recurse | Multiplies subgoals; heavy working-memory load (this becomes chapter 2's central point) |
| 3 | **Planning** | Reformulate in a simpler problem space, solve there, generalize back | Oversimplifies — solutions that work on paper fail in practice |
| 4 | **Hill-climbing** | Take any tentative solution, make the small adjustment that improves it most | Fails exactly when things must first get worse to get better |

### Satisficing
Simon's answer to intractable spaces: don't optimize, take the first *good enough* option. Two drawbacks:
1. You may never discover the better option — fine for one-off problems, corrosive for recurring ones (the hunt-and-peck typist never learns touch typing).
2. Even a good-enough answer can be hard to find, and some dimensions cannot be satisficed (Wiles could satisfice on elegance and length, never on rigor).

### The two types of problem difficulty (the knowledge frontier)
1. **Difficulty of learning from others** — the problem stumps you but is routine for someone else. Fix: acquire their strong methods.
2. **Difficulty at the frontier** — no one has solved it. Fix: explore the space; find promising problems; borrow every strong method that exists.

## Key Concepts

- **Domain-specific methods** — heuristics circumscribed to a range of problems. Induction is useless for painting a portrait or writing a marketing plan.
- **Heuristic** — a method with no guarantee that works decently often (turn the device off and on again).
- **Experience transforms problems into routines** — 5 + 7 requires no search because the answer is stored. Driving, laundry, and booking appointments were once genuine puzzles.
- **Representation precedes search** — the Gestalt tradition's contribution. If the correct answer is excluded from your initial representation, even exhaustive search misses it (the nine-dot puzzle).
- **Undecidability** — Hilbert's tenth problem (1900) asked for an algorithm deciding whether Diophantine equations like Fermat's have whole-number solutions. Seventy years later mathematicians proved **no such algorithm can exist**. Sudoku, chess and Tetris belong to the class where the only guaranteed method is barely better than trying everything.

## Mental Models

- **The maze**: you know where you are and whether you've arrived, but walls limit the route.
- **The dark mansion** (Wiles's own analogy): "One goes into the first room, and it's dark... One stumbles around bumping into the furniture, and gradually, you learn where each piece of furniture is, and finally, after six months or so, you find the light switch."
- **Standing on strong methods**: "Seeing further into the problem space depends on standing on the strong methods of the giants that came before us."
- **Puzzles are traps for weak methods**: what we call a puzzle is often exactly a problem where a tempting weak method conspicuously fails. Tower of Hanoi has only 27 states (trivially small for generate-and-test) yet is hard, because reaching the goal requires moving disks *away* from the destination (defeats hill-climbing) and nests many subgoals (defeats means-ends analysis).

## Anti-patterns

- **Trying to train "general problem-solving skill."** Tricot and Sweller argue there is little evidence weak methods are teachable — people apply means-ends analysis and hill-climbing instinctively. Effort spent on generic problem-solving training buys little.
- **Starting a project without learning how competent people frame it.** You search the wrong space.
- **Grinding on a problem with no evidence it is ripe.** Wiles waited until Frey (1984) connected Fermat's Last Theorem to Taniyama–Shimura before committing.
- **Habitual satisficing on recurring problems** — locks in an inferior method permanently.
- **Assuming classroom experience generalizes.** Most textbook problems have guaranteed methods; the vast majority of real-life problems do not.

## Reference Tables

### Practical takeaways
| # | Takeaway | Action |
|---|---|---|
| 1 | Start with the right representation | Find out how competent people think about the problem: what is the problem space, what are the major moves? |
| 2 | Seek out promising problems | Don't work on impossible problems. Work near people actively pushing the frontier — they give strong hints about what is ripe |
| 3 | Explore the problem space, one room at a time | When you have no one to learn from, explore rather than solve: try things, watch what happens, look for patterns that become new strong methods |

### Sizes of problem spaces cited
| Space | Size / cost |
|---|---|
| Rubik's Cube | > 43 quintillion configurations; one second each = ~5,000× the age of the universe |
| Tower of Hanoi (in-chapter version) | 27 states — small, yet still hard for humans |
| Proving Fermat's Last Theorem | Infinite; no algorithm can exist (Hilbert's 10th, refuted ~1970) |

## Worked Example — Andrew Wiles and Fermat's Last Theorem

The case history the chapter actually reproduces, as a search:

1. **The claim (1637).** Pierre de Fermat: "I have a truly marvelous proof of this, which this margin is too narrow to contain." The claim: aⁿ + bⁿ = cⁿ has no whole-number solutions for n > 2. Pythagorean triples exist infinitely for n = 2 (3,4,5 → 9 + 16 = 25; 5,12,13 → 25 + 144 = 169); the book's Figure 2 notes 6³ + 8³ = 9³ − 1, a near miss.
2. **Three centuries of failed search.** Euler was stumped (and begged a friend to search Fermat's old house for a scrap of proof). Cauchy and Lamé each briefly claimed a proof before a fatal flaw was found. Paul Wolfskehl offered 100,000 marks.
3. **Representation change (1984).** Gerhard Frey showed that if the Taniyama–Shimura conjecture (every modular form has a counterpart elliptic curve) were true, so was Fermat's Last Theorem. This converted an unattackable problem into one in Wiles's own specialty — elliptic curves. *The problem became ripe.*
4. **Equipping with strong methods.** Wiles's first eighteen months went to the library, learning every piece of mathematics related to modular forms and elliptic curves — "an adventurer wandering into an uncharted jungle... he needed to equip his toolkit for any possibility."
5. **Goal-free exploration.** He then explored the math on his own, looking for patterns suggesting a path.
6. **First breakthrough (after two years).** He linked the *first* element of each modular form to the first element of every elliptic curve — "Now he had only an infinity of elements left to go." **Proof by induction** (a strong method: show it holds for the first item, show it doesn't change moving to the next) was the heuristic that made the infinite chain tractable.
7. **Impasse, then borrowed methods.** Probing colleagues without revealing his project, his old advisor John Coates mentioned Matheus Flach's extension of Victor Kolyvagin's technique. "It seemed to be exactly what I needed... although I knew I would still have to further develop this Kolyvagin–Flach method." It required "a lot of hard algebra which required me to learn a lot of new mathematics."
8. **Breaking secrecy.** He confided in Nick Katz and got the pointers to finish. **Seven years** total. "It was the most important moment of my working life. Nothing I ever do again will mean as much."

**What the case demonstrates:** Wiles succeeded not through superior weak methods but through an enormous library of strong methods — most of them acquired from other people — plus the judgment to start only when a representational shift made the problem ripe. His costly self-imposed isolation was broken twice, and both times borrowed knowledge is what moved him forward.

**Companion worked example — the mutilated chessboard.** Can 31 dominos tile a chessboard with the top-left and bottom-right squares removed? Brute-force search over placements is enormous. The **invariant**: every domino, however placed, covers exactly one white and one black square. The two removed squares are both white. So 31 dominos would have to cover 30 white and 32 black squares — impossible. One heuristic collapses the entire search.

## Key Takeaways

1. Frame every hard task as a problem space: start, goal, legal moves. It tells you where the walls are.
2. Most problem spaces are too large to search; you cope by satisficing or by using knowledge to prune. Prefer pruning.
3. Weak methods (generate-and-test, means-ends analysis, planning, hill-climbing) are universal, probably untrainable, and routinely fail. Don't invest in them.
4. Improvement comes from accumulating **strong methods** — which almost always means learning from others.
5. Before searching, fix your representation. Ask a competent person how they see the problem space.
6. Choose ripe problems. The reason to sit near the frontier is that it tells you which regions are worth entering.
7. When there is no one to learn from, switch from *solving* to *exploring* — goal-free play that manufactures new strong methods.

## Connects To

- [Chapter 2 — Creativity Begins with Copying](ch02-creativity-begins-with-copying.md) — why means-ends analysis, the weak method, actively impedes *learning*; goal-free problems and worked examples as the fix.
- [Chapter 3 — Success Is the Best Teacher](ch03-success-is-the-best-teacher.md) — what happens when the strong methods (building blocks) are missing.
- [Chapter 4 — Knowledge Becomes Invisible with Experience](ch04-knowledge-becomes-invisible-with-experience.md) — why experts can't hand you their strong methods directly.
- [Chapter 5 — The Difficulty Sweet Spot](ch05-the-difficulty-sweet-spot.md) — tuning how much search a learner must do.
