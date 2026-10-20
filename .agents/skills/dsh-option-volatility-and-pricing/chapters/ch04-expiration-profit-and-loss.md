# Chapter 4 — Expiration Profit and Loss

## Core Idea
Option prices are contentious at every moment except one: **at expiration an option is worth exactly its intrinsic value** — zero if out of the money, the distance between underlying and exercise price if in the money. That certainty makes expiration the anchor from which a trader can graph any position, no matter how many strikes and legs it has, using nothing but **slopes**.

## Frameworks Introduced

**The slope method.** `slope = change in position value / change in underlying price`. Every contract contributes a slope that changes only at its own exercise price.

| Position | Slope |
|---|---|
| Long or short **any out-of-the-money option** | 0 |
| Long an in-the-money **call** | +1 |
| Short an in-the-money **call** | −1 |
| Long an in-the-money **put** | −1 |
| Short an in-the-money **put** | +1 |
| Long the underlying (any price) | +1 |
| Short the underlying (any price) | −1 |

**Procedure for any position, however complex:**
1. Determine the slope below the lowest exercise price, above the highest, and in every interval between.
2. Sum the individual slopes interval by interval.
3. Compute the P&L at **one** convenient point — usually at an exercise price.
4. Walk the slopes outward from that point to get P&L everywhere; connect the line segments.

**Parity graph → P&L graph.** For a simple position the expiration P&L graph is the parity graph **shifted down by any net debit or up by any net credit**.

**Breakeven from a known point:** `breakeven = known price ± (|P&L at that price| / |slope of the interval|)`.

## Key Concepts
- **Parity graph** — the value of a position at expiration; parity is another name for intrinsic value. The four basic shapes (long/short call, long/short put) are the **hockey-stick diagrams**.
- **Buyers have limited risk (never more than the premium) and unlimited profit; sellers have limited profit (never more than the premium) and unlimited risk.** Natenberg immediately confronts the beginner's question — why would anyone sell? — with the answer that drives the whole book: *the best and worst outcomes matter less than their likelihood*. If the premium is large enough and the perceived risk low enough, selling is rational. Probability, not payoff shape, decides.
- **The put's "unlimited" caveat.** The underlying cannot fall below zero, so a put's profit is technically bounded; practically, traders treat both calls and puts as having unlimited potential value.
- **An option graph always bends at the exercise price**, because of its insurance feature; an underlying graph never bends. **The location of the underlying position is irrelevant to the parity graph** — its slope is +1 (long) or −1 (short) everywhere.
- **Multiple constructions of the same payoff.** Long call + long put at one strike (Figure 4-5) produces exactly the same parity graph as long two calls + short the underlying (Figure 4-7). Long call + short put at the same strike (Figure 4-8) has slope +1 everywhere — identical to a long underlying contract. This is the seed of synthetics (Ch 14).
- **Direction-neutral, magnitude-sensitive strategies.** The long call + long put position gains if the underlying moves in *either* direction away from the strike — "typical of many option strategies that may be sensitive to the magnitude of movement rather than the direction."
- **Value ordering.** Calls with **lower** exercise prices are worth more (buy lower); puts with **higher** exercise prices are worth more (sell higher).
- **No y-axis on complex graphs.** With options bought and sold at many strikes, it may not be possible to position the graph against a y-axis at all; the shape and slopes still carry the information (Figure 4-9 shows unlimited profit on the downside and unlimited loss on the upside).

## Mental Models
- **Options as piecewise-linear building blocks.** Any position is a sum of kinks; find the kinks, sum the slopes, anchor once.
- **Anchor at a strike.** P&L is easiest to compute exactly at an exercise price because every option there is worth either zero or a clean intrinsic amount.
- **Payoff shape is not an edge.** The apparently unbalanced risk-reward of buying vs. selling is resolved by probability, not by the diagram.

## Anti-patterns
- **Choosing to always buy options because the payoff picture looks favorable.** The picture ignores likelihood — the entire subject of Chapters 5 and 6.
- **Treating the parity graph as the P&L graph.** They differ by the net debit/credit; the breakeven only exists on the P&L graph.
- **Trying to find one formula for a multi-leg payoff.** Build it interval by interval instead.
- **Positioning a complex payoff against the y-axis** when the entry prices do not permit it.

## Reference Tables

**Expiration values of a 95 call and a 110 put**

| Underlying price | 95 call | 110 put |
|---|---|---|
| 80 | 0 | 30 |
| 85 | 0 | 25 |
| 90 | 0 | 20 |
| 95 | 0 | 15 |
| 100 | 5 | 10 |
| 105 | 10 | 5 |
| 110 | 15 | 0 |
| 115 | 20 | 0 |
| 120 | 25 | 0 |
| 125 | 30 | 0 |

**Option price grid used throughout the chapter** (underlying at 98.00)

| Strike | 85 | 90 | 95 | 100 | 105 | 110 | 115 |
|---|---|---|---|---|---|---|---|
| Calls | 14.25 | 9.75 | 6.25 | 3.50 | 1.75 | 0.75 | 0.25 |
| Puts | 0.25 | 1.00 | 2.25 | 4.50 | 7.75 | 11.75 | 16.25 |

**Simple positions and breakevens from that grid**

| Position | Debit/credit | Breakeven |
|---|---|---|
| Long 100 call at 3.50 | −3.50 | 103.50 |
| Short 95 put at 2.25 | +2.25 | 92.75 |
| Long 95 call at 6.25 | −6.25 | 101.25 |
| Long 105 call at 1.75 | −1.75 | 106.75 |
| Long 95 put at 2.25 | −2.25 | 92.75 |
| Long 100 put at 4.50 | −4.50 | 95.50 |
| Long 105 put at 7.75 | −7.75 | 97.25 |

**Figure 4-9 — slope table for a complex position** (−4 underlying; +3 65 calls, +2 65 puts; +2 70 calls, −4 70 puts; −6 75 calls, +3 75 puts; +4 80 calls, −2 80 puts)

| Contract | Below 65 | 65–70 | 70–75 | 75–80 | Above 80 |
|---|---|---|---|---|---|
| +3 65 calls | 0 | +3 | +3 | +3 | +3 |
| +2 65 puts | −2 | 0 | 0 | 0 | 0 |
| +2 70 calls | 0 | 0 | +2 | +2 | +2 |
| −4 70 puts | +4 | +4 | 0 | 0 | 0 |
| −6 75 calls | 0 | 0 | 0 | −6 | −6 |
| +3 75 puts | −3 | −3 | −3 | 0 | 0 |
| +4 80 calls | 0 | 0 | 0 | 0 | +4 |
| −2 80 puts | +2 | +2 | +2 | +2 | 0 |
| −4 underlying | −4 | −4 | −4 | −4 | −4 |
| **Total** | **−3** | **+2** | **0** | **−3** | **−1** |

## Worked Example
**A four-leg position priced, sloped, and broken even** (underlying 98.00; prices from the grid above).

Position: `+1 95 call @ 6.25`, `−1 105 call @ 1.75`, `−2 105 puts @ 7.75`, `−2 underlying @ 98.00`.

*Step 1 — slopes:*

| Contract | Below 95 | 95–105 | Above 105 |
|---|---|---|---|
| +1 95 call | 0 | +1 | +1 |
| −1 105 call | 0 | 0 | −1 |
| −2 105 puts | +2 | +2 | 0 |
| −2 underlying | −2 | −2 | −2 |
| **Total** | **0** | **+1** | **−2** |

*Step 2 — anchor the P&L at the 95 strike:*

| Position | Contract price | Value at 95 | Contract P&L | Total P&L |
|---|---|---|---|---|
| +1 95 call | 6.25 | 0 | −6.25 | −6.25 |
| −1 105 call | 1.75 | 0 | +1.75 | +1.75 |
| −2 105 puts | 7.75 | 10.00 | −2.25 | −4.50 |
| −2 underlying | 98.00 | 95.00 | +3.00 | +6.00 |
| **Total at 95.00** | | | | **−3.00** |

*Step 3 — walk the slopes:* below 95 the slope is 0, so the loss is a flat −3.00. From 95 to 105 the slope is +1, so at 105 the P&L is −3.00 + 10.00 = **+7.00**. Above 105 the slope is −2.

*Step 4 — breakevens:* first at 95.00 + (3.00 / 1) = **98.00**; second at 105.00 + (7.00 / 2) = **108.50**.

**Extending the method to Figure 4-9's eight-leg position.** Given only that the P&L at an underlying of 62.00 is +2.10, walk the slope table:

| Interval | Beginning P&L | Slope | Ending P&L |
|---|---|---|---|
| 62.00–65.00 | +2.10 | −3 | 2.10 − (3 × 3) = −6.90 |
| 65.00–70.00 | −6.90 | +2 | −6.90 + (2 × 5) = +3.10 |
| 70.00–75.00 | +3.10 | 0 | +3.10 |
| 75.00–80.00 | +3.10 | −3 | 3.10 − (3 × 5) = −11.90 |
| 80.00–81.50 | −11.90 | −1 | −11.90 − (1 × 1.50) = **−13.40** |

At 81.50 the position loses **13.40**. Its three breakevens: 62.00 + 2.10/3 = **62.70**; 65.00 + 6.90/2 = **68.45**; 75.00 + 3.10/3 ≈ **76.03**.

## Key Takeaways
- At expiration, value = intrinsic value. Everything about expiration risk follows from that.
- Sum slopes interval by interval, anchor P&L at one strike, and walk outward — this handles arbitrarily complex positions.
- The P&L graph is the parity graph shifted by the net debit or credit.
- Identical payoffs can be constructed from different combinations of calls, puts, and the underlying; long call + short put at one strike *is* a long underlying position.
- Limited-risk/unlimited-profit for buyers and its mirror for sellers is a shape, not an edge. Probability decides which side is worth taking.

## Connects To
- [Chapter 3 — Contract Specifications and Option Terminology](ch03-contract-specifications-and-terminology.md): intrinsic value defined.
- [Chapter 5 — Theoretical Pricing Models](ch05-theoretical-pricing-models.md): supplies the probability that makes selling rational.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): the magnitude-sensitive strategies previewed by Figure 4-5.
- [Chapter 14 — Synthetics](ch14-synthetics.md): the equivalence of long call + short put and a long underlying.
