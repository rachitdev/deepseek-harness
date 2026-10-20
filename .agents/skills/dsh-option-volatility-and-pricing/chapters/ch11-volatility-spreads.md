# Chapter 11 — Volatility Spreads

## Core Idea
An option position can be hedged with something other than the underlying. Buy 10 calls with delta 50 (+500 deltas) and you can neutralize by selling five underlying contracts, buying puts totaling −500 deltas, selling different calls totaling −500 deltas, or any combination. Whichever you choose, the result shares four properties — approximately **delta neutral**, sensitive to the **underlying price**, sensitive to **implied volatility**, and sensitive to the **passage of time**. Positions with those four properties are **volatility spreads**, and this chapter is their catalog.

## Frameworks Introduced

**The four-quadrant classification.** Because volatility spreads are delta neutral and gamma and theta always carry opposite signs, every spread falls into one of four boxes:

| Gamma | Vega | Conditions that help the position |
|---|---|---|
| + | + | More volatile underlying; rising implied volatility |
| − | − | Less volatile underlying; falling implied volatility |
| + | − | More volatile underlying; falling implied volatility |
| − | + | Less volatile underlying; **rising** implied volatility (the calendar-spread quadrant) |

**Vocabulary.** Positive gamma = **long premium** (hoping for a volatile market). Negative gamma = **short premium** (hoping for a quiet one). Long = executed at a **debit**; short = executed at a **credit**.

**Gamma is realized volatility; vega is implied volatility.** All spreads except calendar spreads want both to move the same way. Calendar spreads are the exception, and that is what makes them distinctive.

**The spread-selection principle.**
> If implied volatility is low, such that options generally appear underpriced, look for spreads with a **positive vega**. If implied volatility is high, such that options generally appear overpriced, look for spreads with a **negative vega**.

**Corollary for calendars:**
> Long calendar spreads are likely to be profitable when implied volatility is **low but expected to rise**; short calendar spreads when implied volatility is **high but expected to fall**.

**Four adjustment disciplines:**
1. **At regular intervals** — match the interval to the interval used to measure volatility (daily estimate → daily adjustment). Closest to the model's assumptions.
2. **At a predetermined delta threshold** — e.g., adjust whenever the position reaches ±500 deltas. Frequency is unpredictable. The tolerable number depends on typical position size, capitalization, and experience: a new independent trader may balk at 200 deltas; a large firm may treat several thousand as neutral.
3. **By feel** — "not made facetiously." With the market at 50.00, delta neutral, gamma −200, a fall to 48.00 leaves you roughly +400 deltas. If 48.00 is strong support you might decline to adjust. Legitimate if you are right more often than not.
4. **Not at all** — accepts directional risk equal to the position's maximum delta (five short straddles → up to ±500). Saves all transaction costs, but if directional considerations start to dominate a position taken for volatility reasons, the honest alternative is to **close the position**, not to convert it into a directional bet. Decide the exit conditions when you enter.

## The spread catalog

**Straddle** — a call and a put, same exercise price, same expiration. Usually one-to-one and at the money, so deltas near +50/−50 give approximate neutrality. Done at an in- or out-of-the-money strike it becomes directional: with the underlying at 100, a September 95 call (delta 75) plus a September 95 put (delta −25) is +50 — a **bull straddle**. To neutralize, buy **three puts for every call** — a **ratio straddle** (unequal numbers of long and short market contracts).

**Strangle** — a call and a put, same expiration, **different** exercise prices, usually chosen so the deltas are roughly equal. By convention a strangle means **out-of-the-money** options: with the market at 100, the "March 90/110 strangle" is the March 90 put and the March 110 call. The in-the-money version (March 90 call + March 110 put) has essentially the same P&L profile but is less actively traded and is called a **guts**.

**Butterfly** — three equally spaced exercise prices, all the same type, same expiration, **always in a 1 × 2 × 1 ratio**. Long butterfly = buy the outside strikes (**wings**), sell two of the inside strike (**body**). Its expiration value is **never below 0 and never above the amount between exercise prices**, and is maximal with the underlying exactly at the body. Long butterflies always require a cash outlay — otherwise there would be a riskless profit. When the body is at the money, a long butterfly behaves like a **short straddle** and a short butterfly like a **long straddle**, but with strictly bounded risk and reward. For European options, a call butterfly and a put butterfly with the same strikes and expiration **must trade at the same price** — otherwise buy the cheaper and sell the dearer for a sure profit. (Not guaranteed for American options, where early exercise means you cannot be certain of carrying to expiration.)

**Condor** — four options: two inside strikes (body), two outside (wings), always **1 × 1 × 1 × 1**, all the same type and expiration. The gap between the two inside strikes may vary, but the two lowest strikes must be the same distance apart as the two highest. Value at expiration is between 0 and the amount between the two higher (or two lower) strikes; maximal between the two intermediate strikes. Approximately delta neutral when the underlying sits midway between the inside strikes. Butterflies and condors together are **wingspreads**.

**Ratio spread** — unequal numbers of the same type of option, same expiration, typically delta neutral. **More bought than sold** = **backspread**: +gamma/−theta/+vega, and under a traditional model a delta-neutral version should always be executed for a **credit** (it must be, since the spread is worthless after a big enough adverse move). **More sold than bought** = **frontspread**: −gamma/+theta/−vega, behaving like a short straddle but with the loss limited on one side. Common ratios: 2:1 (most common), 3:1, 4:1, 3:2.

**Christmas tree (ladder)** — mimics a strangle with limited risk or reward on one side. A **call Christmas tree** buys (sells) a call at a lower strike and sells (buys) one call each at two higher strikes; a **put Christmas tree** buys (sells) a put at a higher strike and sells (buys) one put each at two lower strikes. **Long** (one bought, two sold) acts like a short strangle with limited risk one way; **short** (one sold, two bought) acts like a long strangle with limited profit one way.

**Calendar spread** (time spread, horizontal spread) — same type, same exercise price, **different expirations**. Long = buy the long-term, sell the short-term (a debit). Two defining characteristics for at-the-money options:
1. It **increases in value as time passes with no movement** — because at-the-money theta rises as expiration nears, so the short-term option decays faster.
2. It **increases in value as implied volatility rises** — because long-term options have larger vegas.

So a long calendar wants two apparently contradictory things: the underlying to sit still (−gamma) *and* everyone to expect movement (+vega). This happens often — e.g., a company announces the CEO will make an important statement in a week: the stock does not move, but implied volatility rises and calendars widen. If the statement turns out to be irrelevant ("the CEO wanted to announce that he and his wife just became grandparents"), implied volatility collapses and calendars narrow.

**Futures calendar caveat.** Stock options always share one underlying. Futures options may not: an April/June calendar shares a June future, but a **March/June calendar has a March future underlying the March options and a June future underlying the June options**, and those can move apart. Offset by trading a futures spread: buy 10 June calls (+500) and sell 10 March calls (−500) at delta 50, then **buy 5 March futures and sell 5 June futures**.

**Time butterfly (time fly)** — same exercise price, same type, **three different expirations** roughly evenly spaced. It is simultaneously buying a short-term calendar spread and selling a long-term one (or vice versa) sharing one month. Because a short-term calendar is worth more than a long-term one, **buying the body and selling the wings produces a debit** — so that is the *long* time butterfly, exactly the reverse of the debit/credit convention for a strike butterfly. A long time butterfly is −gamma / +theta / +vega, like a long calendar. (This assumes equal implied volatility across expirations; if implieds differ, a long time butterfly might actually be a credit.)

**Diagonal spread** — like a calendar but with different exercise prices; may be one-to-one or ratioed. Too varied to generalize about, **except**: a one-to-one diagonal of the same type with **approximately equal deltas** behaves very much like a conventional calendar spread.

## Key Concepts
- **Size and risk do not correlate.** "A trader may find that buying 300 butterflies (300 × 600 × 300) is actually less risky than selling 100 straddles." Risk depends on the strategy's characteristics, not just its size — which is why butterflies trade in much larger sizes than straddles.
- **The risk spectrum.** Straddles and strangles have the largest gamma and vega — biggest profit when right, biggest loss when wrong. Butterflies and condors are at the other end. Ratio spreads and Christmas trees fall in between.
- **Straddles and strangles are the obvious play when *everything* is mispriced** — you get positive edge on both legs — but Chapter 13 shows they are among the riskiest strategies, so traders often prefer ratio spreads and butterflies even at the cost of buying some overpriced options or selling some underpriced ones.
- **Interest-rate effects on stock-option calendars.** With stock at 100, 8% rates, three months to March and six to June: forwards are 102.00 and 104.00. Raise rates to 10% and they become 102.50 and **105.00** — the longer-dated forward moves more. So a **long call calendar has positive rho** (spread expands with rates) and a **long put calendar has negative rho** (the June put falls more than the March put). Magnitude depends on the gap between expirations: March/September ≫ March/April.
- **Dividend effects are the mirror image.** A dividend increase lowers the forward, so a call calendar **narrows** and a put calendar **widens**. A call calendar has **negative dividend risk**; a put calendar has **positive dividend risk**. Raise dividends enough and a call calendar can go negative; raise rates enough and a put calendar can.
- **Calendar floor and the short squeeze.** If a stock pays no dividend, a call calendar should always be worth at least the cost of carry on the stock between expirations — but only if a short stock position can be carried between months. If no stock can be borrowed, a trader long a call calendar may be forced to **exercise his long-term option**, forfeiting its time value.
- **Delta neutrality need not be exact.** "In practice, a volatility spread should have a delta that is small enough that the directional considerations are less important than the volatility considerations. This is often a subjective judgment." If the delta grows so large that direction dominates, it is no longer a volatility spread.
- **Spread execution.** A market maker quotes one bid and one ask for the whole spread, and neither party cares how the total is allocated across legs — a 6.25/6.75 straddle bought at 6.75 could be 3.75 call + 3.00 put, or 2.00 + 4.75. Electronic exchanges usually support simple call/put spreads, straddles, strangles, and calendars; butterflies, Christmas trees, and unusual ratios must be legged or handed to a broker on an open-outcry exchange. Contingency orders available: all or none; fill or kill; immediate or cancel; market if touched; market on close; not held; one cancels the other; stop limit; stop loss.
- **Prior trades are sunk.** "Once the spread has been established, the market conditions that will help or hurt the spread are determined by its characteristics, not by the initial prices… A trader's primary concern should not be what happened yesterday but what can be done today."

## Anti-patterns
- **Buying straddles and strangles because unlimited profit in both directions looks appealing.** "If the hoped-for movement fails to materialize, a trader will find that losing money, even a limited amount, can also be a painful experience." Consider the *likelihood* of the outcomes, which is why the model exists.
- **Assuming a strangle at named strikes is unambiguous** — 90/110 could be OTM or the ITM guts version.
- **Trading a March/June futures calendar without a futures spread** to neutralize the risk that the two futures months move apart.
- **Treating a long time butterfly like a long strike butterfly** — the debit/credit convention inverts.
- **Converting a volatility position into a directional one by refusing to adjust.**
- **Letting a market maker's leg allocation matter** — or reporting leg prices that do not reflect the market.

## Reference Tables

**Figure 11-9 / 11-18 / 11-30 — Volatility spread characteristics**

| Spread | Delta* | Gamma | Theta | Vega | Downside risk/reward | Upside risk/reward |
|---|---|---|---|---|---|---|
| Long straddle | 0 | + | − | + | Unlimited reward | Unlimited reward |
| Long strangle | 0 | + | − | + | Unlimited reward | Unlimited reward |
| Short butterfly | 0 | + | − | + | Limited reward | Limited reward |
| Short condor | 0 | + | − | + | Limited reward | Limited reward |
| Call ratio spread (buy more than sell) | 0 | + | − | + | Limited reward† | Unlimited reward† |
| Put ratio spread (buy more than sell) | 0 | + | − | + | Unlimited reward† | Limited reward† |
| Short call Christmas tree | 0 | + | − | + | — | — |
| Short put Christmas tree | 0 | + | − | + | — | — |
| Short straddle | 0 | − | + | − | Unlimited risk | Unlimited risk |
| Short strangle | 0 | − | + | − | Unlimited risk | Unlimited risk |
| Long butterfly | 0 | − | + | − | Limited risk | Limited risk |
| Long condor | 0 | − | + | − | Limited risk | Limited risk |
| Call ratio spread (sell more than buy) | 0 | − | + | − | Limited risk† | Unlimited risk† |
| Put ratio spread (sell more than buy) | 0 | − | + | − | Unlimited risk† | Limited risk† |
| Long call Christmas tree | 0 | − | + | − | — | — |
| Long put Christmas tree | 0 | − | + | − | — | — |
| **Long calendar spread** | 0 | **−** | **+** | **+** | Limited risk | Limited risk |
| **Short calendar spread** | 0 | **+** | **−** | **−** | Limited reward | Limited reward |

*All spreads assumed approximately delta neutral initially. †Applies to delta-neutral ratio spreads, which are a credit when buying more than selling and a debit when selling more than buying.

**Figure 11-19 — Calendar spread value as time passes** (underlying 100, volatility 20%)

| Time to June expiration | 4 months | 3 months | 2 months |
|---|---|---|---|
| Time to April expiration | 2 months | 1 month | 0 |
| June 100 call | 4.60 | 3.99 | 3.26 |
| April 100 call | 3.26 | 2.30 | 0 |
| **Spread value** | **1.34** | **1.69** | **3.26** |

**Figure 11-20 — Calendar spread value as the underlying price changes**

| Underlying price | 80 | 90 | 100 | 110 | 120 |
|---|---|---|---|---|---|
| June 100 call | 0.10 | 1.07 | 4.60 | 11.39 | 20.31 |
| April 100 call | 0.01 | 0.36 | 3.26 | 10.51 | 20.04 |
| **Spread value** | 0.09 | 0.71 | **1.34** | 0.88 | 0.27 |

**Figure 11-21 — Calendar spread value as volatility changes**

| Volatility | 10% | 15% | 20% | 25% | 30% |
|---|---|---|---|---|---|
| June 100 call | 2.30 | 3.45 | 4.60 | 5.75 | 6.90 |
| April 100 call | 1.63 | 2.44 | 3.26 | 4.07 | 4.88 |
| **Spread value** | 0.67 | 1.01 | **1.34** | 1.68 | 2.02 |

**Figure 11-28 — Interest rates and dividends on stock option calendars** (stock 100, volatility 20%, 3 months to March, 6 months to June)

*Varying the interest rate (dividend 0):*

| Interest rate | 0% | 3% | 6% | 9% | 12% |
|---|---|---|---|---|---|
| June 100 call | 5.64 | 6.37 | 7.16 | 7.99 | 8.87 |
| March 100 call | 3.99 | 4.36 | 4.75 | 5.16 | 5.58 |
| **Call spread value** | 1.65 | 2.01 | 2.41 | 2.83 | **3.29** |
| June 100 put | 5.64 | 4.88 | 4.20 | 3.59 | 3.05 |
| March 100 put | 3.99 | 3.61 | 3.26 | 2.93 | 2.63 |
| **Put spread value** | 1.65 | 1.27 | 0.94 | 0.66 | **0.42** |

*Varying the quarterly dividend (interest rate 6.00%):*

| Quarterly dividend | 0 | 0.50 | 1.00 | 1.50 | 2.00 |
|---|---|---|---|---|---|
| June 100 call | 7.16 | 6.57 | 6.00 | 5.47 | 4.96 |
| March 100 call | 4.75 | 4.46 | 4.19 | 3.93 | 3.68 |
| **Call spread value** | 2.41 | 2.11 | 1.81 | 1.54 | **1.28** |
| June 100 put | 4.20 | 4.60 | 5.02 | 5.47 | 5.95 |
| March 100 put | 3.26 | 3.47 | 3.70 | 3.93 | 4.17 |
| **Put spread value** | 0.94 | 1.13 | 1.32 | 1.54 | **1.78** |

**Figure 11-31 — Evaluation table** (stock 100; volatility 20%; interest rate 6.00%; dividend 0; European options, so some values are below intrinsic)

*April, 2 months to expiration:*

| Strike | Call TV | Δ | Γ | Θ | Vega | Rho | Put TV | Δ | Γ | Θ | Vega | Rho |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 90 | 11.17 | 93 | 1.7 | −0.023 | 0.06 | 0.136 | 0.27 | −7 | 1.7 | −0.008 | 0.06 | −0.013 |
| 95 | 6.98 | 79 | 3.6 | −0.031 | 0.12 | 0.119 | 1.04 | −21 | 3.6 | −0.016 | 0.12 | −0.037 |
| 100 | 3.76 | 56 | 4.8 | −0.035 | 0.16 | 0.088 | 2.77 | −44 | 4.8 | −0.019 | 0.16 | −0.077 |
| 105 | 1.71 | 33 | 4.4 | −0.030 | 0.15 | 0.052 | 5.67 | −67 | 4.4 | −0.013 | 0.15 | −0.121 |
| 110 | 0.65 | 16 | 3.0 | −0.019 | 0.10 | 0.025 | 9.56 | −84 | 3.0 | −0.001 | 0.10 | −0.156 |

*June, 4 months to expiration:*

| Strike | Call TV | Δ | Γ | Θ | Vega | Rho | Put TV | Δ | Γ | Θ | Vega | Rho |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 90 | 12.55 | 87 | 1.8 | −0.022 | 0.12 | 0.249 | 0.76 | −13 | 1.8 | −0.008 | 0.12 | −0.045 |
| 95 | 8.71 | 75 | 2.8 | −0.026 | 0.18 | 0.221 | 1.83 | −25 | 2.8 | −0.011 | 0.18 | −0.089 |
| 100 | 5.62 | 59 | 3.4 | −0.027 | 0.22 | 0.178 | 3.64 | −41 | 3.4 | −0.011 | 0.22 | −0.148 |
| 105 | 3.36 | 42 | 3.4 | −0.025 | 0.23 | 0.130 | 6.28 | −58 | 3.4 | −0.008 | 0.23 | −0.213 |
| 110 | 1.85 | 28 | 2.9 | −0.020 | 0.19 | 0.086 | 9.68 | −72 | 2.9 | −0.002 | 0.19 | −0.274 |

**Sample spread aggregates from Figure 11-33**

| Spread | Theoretical value (cash flow) | Delta | Gamma | Theta | Vega | Rho |
|---|---|---|---|---|---|---|
| Long straddle: +10 April 100 calls, +10 April 100 puts | −65.30 | +120 | +96.0 | −0.540 | +3.20 | +0.110 |
| Ratio straddle: +10 June 95 calls, +30 June 95 puts | −142.00 | 0 | +112.0 | −0.590 | +7.20 | −0.460 |
| Short straddle: −20 April 105 calls, −10 April 105 puts | +90.90 | +10 | −132.0 | +0.730 | −4.50 | −0.170 |
| Short straddle: −10 June 100 calls, −10 June 100 puts | +92.60 | −180 | −68.0 | +0.380 | −4.40 | −0.300 |

**Figure 11-29 — Diagonals that behave like calendars** (underlying 100, volatility 30%, 4 months to June, 2 months to April; deltas in parentheses)

| Spread | Value |
|---|---|
| +1 June 115 call (23) / −1 April 110 call (23) | 2.20 / 1.60 |
| +1 June 80 put (−8) / −1 April 85 put (−8) | 0.72 / 0.48 |

## Worked Example

**A. Valuing a long butterfly at expiration.** Take the March 90/100/110 call butterfly: +1 March 90 call, −2 March 100 calls, +1 March 110 call.

| Underlying at expiration | 80 | 100 | 120 |
|---|---|---|---|
| +1 March 90 call | 0 | +10 | +30 |
| −2 March 100 calls | 0 | 0 | −20 |
| +1 March 110 call | 0 | 0 | +10 |
| **Total** | **0** | **+10** | **0** |

Below 90 all calls expire worthless. Above 120 the 90 and 110 calls together exactly equal the two 100 calls. At the body, the 90 call is worth 10.00 and the others nothing — the **maximum, equal to the amount between exercise prices**. So the butterfly must trade somewhere between 0 and 10.00: perhaps **8.00** if the underlying is very likely to finish near 100, perhaps only **1.00 or 2.00** if it is likely to finish outside the wings.

**B. A delta-neutral call ratio backspread.** Underlying at 100 (deltas in parentheses):

| Position | Price |
|---|---|
| +3 October 105 calls (25) | 1.00 each |
| −1 October 95 call (75) | 6.00 |

Deltas: (3 × 25) − 75 = 0. Now the P&L at three expiration prices:

| Underlying | 95 call P&L | 105 call P&L | Total |
|---|---|---|---|
| 80 | +6.00 − 0 = **+6.00** | 3 × (−1.00 + 0) = **−3.00** | **+3.00** |
| 120 | +6.00 − 25.00 = **−19.00** | 3 × (−1.00 + 15.00) = **+42.00** | **+23.00** |
| 100 | +6.00 − 5.00 = **+1.00** | 3 × (−1.00 + 0) = **−3.00** | **−2.00** |

Big moves either way pay; the upside pays far more (unlimited); sitting at 100 loses. Note the spread was established for a credit — 6.00 received less 3.00 paid — which is what makes the downside profitable and is the general property of a delta-neutral backspread.

**C. Why a calendar spread grows with time.** The April/June 100 call calendar with the underlying constant at 100 and volatility constant at 20%. Initially, with four months to June and two to April, the spread is 4.60 − 3.26 = **1.34**. Over the first month the April call loses **0.96** while the June call loses only **0.61** (short-term at-the-money theta is larger), so the spread widens to 3.99 − 2.30 = **1.69**. Over the second month the April call, being at the money, must surrender its entire remaining 2.30 while the June call loses 0.73, leaving the spread at **3.26**.

*But the spread collapses if the underlying moves.* At 80 the spread is only **0.09**; at 120 only **0.27**. The calendar needs both options to stay at the money, where time value is maximal.

*And it widens with implied volatility.* At 10% volatility the spread is **0.67**; at 30% it is **2.02** — because the June call's vega exceeds the April call's.

**D. Rho on a stock-option calendar.** Buy 10 June 100 calls / sell 10 March 100 calls with the stock at 100, three months to March, six to June, no dividend. At 8% the forwards are 102.00 (March) and 104.00 (June). Raise rates to 10%: 102.50 and **105.00**. The June forward moved 1.00 while the March forward moved 0.50; with roughly equal deltas the June call gains more, so **the call calendar expands** — a positive rho. Puts have negative deltas, so the June put falls more than the March put and the **put calendar narrows** — a negative rho. Figure 11-28 confirms it numerically: the call spread runs 1.65 → 3.29 as rates go 0% → 12%, while the put spread runs 1.65 → 0.42.

## Key Takeaways
- Every volatility spread is delta neutral, gamma/theta opposed, and vega-exposed; classify it by the sign of gamma and vega.
- Straddle/strangle = maximum gamma and vega; butterfly/condor = the bounded versions; ratio spreads and Christmas trees sit in between with an asymmetric preference.
- Butterfly value at expiration is bounded by 0 and the strike interval; condor value by 0 and the wing interval.
- Calendar spreads are the only common spread wanting a quiet underlying *and* rising implied volatility (−gamma / +vega) — the pending-announcement configuration.
- Long call calendars have positive rho and negative dividend risk; long put calendars the reverse.
- Match the spread's vega sign to whether implied volatility looks high or low relative to your estimate.
- Decide your adjustment discipline — interval, delta threshold, feel, or none — before you enter.
- Size and risk are not the same thing.

## Connects To
- [Chapter 7 — Risk Measurement I](ch07-risk-measurement-i.md) and [Chapter 9 — Risk Measurement II](ch09-risk-measurement-ii.md): the Greeks that define each spread.
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): the archetypal gamma spread and the adjustment process.
- [Chapter 10 — Introduction to Spreading](ch10-introduction-to-spreading.md): why spread at all.
- [Chapter 12 — Bull and Bear Spreads](ch12-bull-and-bear-spreads.md): the directional counterparts.
- [Chapter 13 — Risk Considerations](ch13-risk-considerations.md): why straddles are riskier than butterflies with the same edge.
- [Chapter 14 — Synthetics](ch14-synthetics.md): iron butterflies and iron condors.
- [Chapter 16 — Early Exercise of American Options](ch16-early-exercise-american-options.md): why the European butterfly arbitrage is not guaranteed for American options.
