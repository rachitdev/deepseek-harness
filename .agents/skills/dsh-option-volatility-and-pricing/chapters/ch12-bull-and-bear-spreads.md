# Chapter 12 — Bull and Bear Spreads

## Core Idea
Nothing forces a trader to stay delta neutral. But a directional option position must still respect volatility — otherwise the trader "may be no better off, and perhaps even worse, than if he had taken an outright position in the underlying contract." The chapter's central discovery: ratio spreads, butterflies, and calendar spreads can all be tilted bullish or bearish, but **their deltas invert** as conditions change. Only the **vertical spread** stays bullish or bearish under every possible market condition.

## Frameworks Introduced

**The universal vertical-spread rule.**
> Regardless of whether a spread consists of calls or puts, whenever a trader **buys the lower exercise price and sells the higher**, the position is **bullish**; whenever he **buys the higher and sells the lower**, the position is **bearish**.

This holds even though it seems counterintuitive for puts.

| Spread | Construction |
|---|---|
| Bull call spread | Buy June 100 call, sell June 105 call |
| Bull put spread | Buy June 100 put, sell June 105 put |
| Bear call spread | Sell June 100 call, buy June 105 call |
| Bear put spread | Sell June 100 put, buy June 105 put |

**The strike-selection rule** — the chapter's most quotable result:
> **If implied volatility is low, the choice of spreads should focus on purchasing the at-the-money option. If implied volatility is high, the choice should focus on selling the at-the-money option.**

Once you have done that, the other leg is forced: buy the ATM call and you must sell the OTM call to make a bull spread; sell the ATM call and you must buy the ITM call. The rule follows directly from Chapter 6: **in total points, the at-the-money option is the most sensitive to a change in volatility**, so it is the most overpriced when implied is high and the most underpriced when implied is low.

**Sizing a directional view.** Two levers set total directional exposure: (1) the **delta of the chosen spread** and (2) the **size in which it is executed**. 500 deltas long = a 50-delta spread done 10 times, or a 25-delta spread done 20 times.

**Wider strikes = more directional.** With options near the money and the same expiration, **the greater the amount between exercise prices, the greater the spread's delta** — 95/110 is more bullish than 95/105, which is more bullish than 95/100 — and the maximum profit and loss also grow. (Not true for very deep ITM or far OTM spreads, where both deltas are pinned near 100 or 0.)

## Key Concepts
- **Naked directional positions have no margin for error.** Buy options and you lose if the market moves the wrong way *or* fails to move fast enough to offset decay. Sell options and time helps but the risk is unlimited.
- **Ratio spreads can be tilted — and can invert.** With the underlying at 101, ten weeks to June, 30% volatility: the June 100 call has delta 56 and the June 110 call delta 28. The delta-neutral frontspread is **buy 1 June 100, sell 2 June 110**. Tilt bullish to **buy 2 June 100, sell 3 June 110** for a delta of +28. But this has a **negative gamma**: if the market races to 130 or 140 all deltas approach 100 and the position becomes **−100 deltas** — bearish, despite the original bullish view. "The volatility characteristics of the position will eventually outweigh any considerations of market direction."
- **Backspreads invert too, but through time and volatility, not price.** Buy 3 June 110 calls, sell 1 June 100 call = +28 deltas. As time passes or volatility declines, deltas move *away* from 50: if the 100 call's delta rises to 70 and the 110 call's falls to 15, the position becomes (3 × 15) − 70 = **−25**. Overestimate volatility and a bullish position turns bearish.
- **Bullish and bearish butterflies.** With the underlying at 100, buy the June **105/110/115** call butterfly to be bullish (you want the market at 110 at expiration). Below 110 it is delta positive; above 110 it is delta negative. A move to 120 inverts it. Buy a butterfly with the body **below** the market to be bearish, and a fall through the body inverts it the other way.
- **Bullish and bearish calendars.** A long calendar always wants the short-term option to expire exactly at the money. With the underlying at 100, the **June/April 110** calendar is bullish (you want the market to rise to 110 by April expiration); the **June/April 90** calendar is bearish. Both have negative gamma, so both invert once the underlying crosses the strike. (In futures markets, differing prices across futures months may force a **diagonal** rather than a plain calendar to get the intended sign of delta.)
- **Butterflies and calendars look alike in delta but not otherwise.** Figures 12-3 and 12-4 are nearly identical *with respect to changes in the underlying price* — but the two spreads "react quite differently to the passage of time and changes in implied volatility."
- **Why verticals can't invert.** Two options identical except for exercise price cannot have identical deltas. The June 100 call's delta always exceeds the June 105 call's; the December 105 put always has a greater negative delta than the December 95 put. Even deep in or far out of the money, where both tend to 100 or 0, the ordering survives.
- **Vertical spread bounds.** At expiration a vertical is worth **0** if both options are out of the money and the **full amount between exercise prices** if both are in the money. So a 100/105 call vertical trades between 0 and 5.00; a 95/105 put vertical between 0 and 10.00. With the market at 80 and no sign of rising, the 100/105 call spread is near 0 and the 95/105 put spread near 10.00; at 120 with no sign of falling, the reverse.
- **The ITM-containing spread always costs more** — because it profits in more scenarios. Compare a 95/100 and a 100/105 bull call spread with the market at 100: if the market rises to 110 both reach 5.00; if it falls to 90 both go to 0; but if it **sits at 100**, the 100/105 collapses to 0 while the 95/100 expands to its full 5.00. The 95/100 needs the market only *not to fall*; the 100/105 needs it to rise. Hence the 95/100 has **positive theta / negative gamma** and the 100/105 has **positive gamma / negative theta**.
- **Which to prefer depends on your volatility view.** If the market does move to 110, both profit, but the cheaper 100/105 profits more; if it falls to 90, both lose, but the 100/105 loses less. So **if your volatility estimate exceeds implied, prefer the 100/105** (greater likelihood of movement); if it is below implied, prefer the 95/100.
- **Focus on the at-the-forward option, strictly speaking.** For futures options the ATM option usually has the delta closest to 50, but for stock options with high rates and long time to expiration the at-the-forward strike can be considerably above the stock price. "For practical purposes, a trader will not go too far wrong if he focuses on the at-the-money option."
- **If no strike is exactly at the money, use the closest.** Underlying at 103 with 95, 100, and 105 strikes → focus on the 105 call.
- **Deep or far spreads for strong views.** With the underlying at 100, a strongly bullish trader can buy the 115/120 call spread — very cheap because it will probably expire worthless, but executable many times over, and worth the full 5.00 if the market clears 120.
- **You may leg in.** The spread has two options and either may be traded first for practical reasons — but the discipline stands: buy the near-the-money option when implied volatility is low, sell it when high.
- **Shape of the Greeks.** For vertical spreads, "the maximum gamma, vega, and theta tend to occur when the underlying price is either just below the lowest exercise price or just above the highest exercise price." Time passing and volatility declining have similar effects on value, delta, gamma, and vega; theta differs slightly between the two.

## Mental Models
- **Vertical spread vs. outright position.** To get 500 deltas long: buy 5 underlying contracts, or buy 25 vertical call spreads with delta 20 each. Twenty-five spreads sounds riskier "until we remember that a vertical spread has limited risk, whereas the position in the underlying has open-ended risk." The outright trader reaps huge rewards on a big favorable move; the spreader's profit is capped but "he will also be much less bloodied if the market makes an unexpected move in the wrong direction."
- **Options add a second dimension.** Trading the underlying, you can only profit by being right about direction. With options, "a bull spread can be profitable if the market fails to rise or in some cases even if it declines," provided the volatility estimate is right.
- **Pick the time horizon first.** Options expire. If it is May and you expect a move but not within two months, June or July options make no sense — you may need September or December, accepting the liquidity cost of going further out.

## Anti-patterns
- **Using a ratio spread, butterfly, or calendar as a pure directional vehicle.** In each, volatility is still the primary consideration, and the delta can invert against you.
- **Taking naked directional option positions** and accepting either the decay drag or the unlimited risk.
- **Choosing between two same-delta spreads on value alone.** "The value of a strategy is only important if we can compare it with the price of the strategy" — the choice depends on the market's implied volatility.
- **Assuming a put spread is the mirror of a call spread.** Bullishness is determined by which strike is bought, not by the option type.
- **Ignoring the forward** when picking the "at-the-money" strike in a high-rate, long-dated stock option market.

## Reference Tables

**Base evaluation** (8 weeks to expiration, 25% volatility, interest rate assumed 0)

| | 95 call | 100 call | 105 call |
|---|---|---|---|
| Theoretical value | 6.82 | 3.91 | 1.99 |
| Delta | 72 | 52 | 32 |

| Spread | Theoretical value | Delta |
|---|---|---|
| 95/100 | 6.82 − 3.91 = **2.91** | 72 − 52 = **20** |
| 100/105 | 3.91 − 1.99 = **1.92** | 52 − 32 = **20** |

**Call spread values across volatility** (deltas in parentheses)

| Volatility | 95 call | 95/100 spread | 100 call | 100/105 spread | 105 call |
|---|---|---|---|---|---|
| 20% | 6.18 | 3.06 | 3.12 | 1.82 | 1.30 |
| 25% | 6.82 (72) | **2.91** (20) | 3.91 (52) | **1.92** (20) | 1.99 (32) |
| 30% | 7.50 | 2.81 | 4.69 | 1.98 | 2.71 |

**Put spread values across volatility** (deltas in parentheses)

| Volatility | 95 put | 95/100 spread | 100 put | 100/105 spread | 105 put |
|---|---|---|---|---|---|
| 20% | 1.18 | 1.94 | 3.12 | 3.18 | 6.30 |
| 25% | 1.82 (−28) | **2.09** (20) | 3.91 (−48) | **3.08** (20) | 6.99 (−68) |
| 30% | 2.50 | 2.19 | 4.69 | 3.02 | 7.71 |

**Delta inversion of ratio spreads**

| Spread | Initial delta | Inverts because | Ends up |
|---|---|---|---|
| Buy 2 June 100 calls, sell 3 June 110 calls | +28 | Underlying rallies to 130–140; all deltas → 100 | −100 |
| Buy 3 June 110 calls, sell 1 June 100 call | +28 | Time passes / volatility falls; deltas move away from 50 (100 call → 70, 110 call → 15) | −25 |

## Worked Example
**Choosing between two identically-bullish bull call spreads.**

With 8 weeks to expiration and a volatility estimate of 25%, both the **95/100** and the **100/105** bull call spreads carry **+20 deltas**. On value alone the 100/105 looks cheaper (1.92 vs 2.91), but that is meaningless until compared with market prices.

*Scenario 1 — market implied volatility is 20%.* Prices are 3.06 (95/100) and 1.82 (100/105).
- 95/100: pay **3.06** for something worth 2.91 → theoretical edge **−0.15**
- 100/105: pay **1.82** for something worth 1.92 → theoretical edge **+0.10** ✅

*Scenario 2 — market implied volatility is 30%.* Prices are 2.81 (95/100) and 1.98 (100/105).
- 95/100: pay **2.81** for something worth 2.91 → theoretical edge **+0.10** ✅
- 100/105: pay **1.98** for something worth 1.92 → theoretical edge **−0.06**

The reversal is not arbitrary. At-the-money options move the most in total points when volatility changes, so at 20% implied the **100 call** is the most underpriced option on the board — buy it, and the bull spread forces you to sell the 105. At 30% implied the 100 call is the most overpriced — sell it, and the bull spread forces you to buy the 95.

**The same rule with puts.** Implied volatility is low (20%) and you want a **bear put spread** (buy the higher strike, sell the lower). Buy the at-the-money **100 put** and you must sell the **95 put**. Price of the 95/100 put spread at 20% implied = **1.94**, against a value at your 25% estimate of **2.09**. Result: delta **−20** and theoretical edge **+0.15**.

**And the economics behind the price gap.** Why does the 95/100 spread always cost more than the 100/105? Three scenarios with the market starting at 100:

| Market at expiration | 95/100 spread | 100/105 spread |
|---|---|---|
| 110 | widens to 5.00 (profit) | widens to 5.00 (larger profit — bought cheaper) |
| 100 | **widens to 5.00** | **collapses to 0** |
| 90 | collapses to 0 (loss) | collapses to 0 (smaller loss — bought cheaper) |

The 95/100 profits in more cases, so it costs more, and it carries **positive theta / negative gamma**. The 100/105 requires movement, so it carries **positive gamma / negative theta**, wins bigger when right, and loses less when wrong.

## Key Takeaways
- Buy the lower strike / sell the higher = bullish; buy the higher / sell the lower = bearish — calls or puts alike.
- Only vertical spreads keep their directional sign under all conditions; ratio spreads, butterflies, and calendars all invert.
- Buy the at-the-money option when implied volatility is low; sell it when implied volatility is high. The second leg follows automatically.
- Wider strikes make a vertical more directional and raise both its maximum profit and maximum loss.
- The ITM-containing vertical always costs more, carries positive theta, and profits if the market merely stands still; the OTM-containing vertical is cheaper, carries positive gamma, and needs movement.
- Total directional exposure = spread delta × size; either lever can be adjusted.
- A vertical spread has bounded risk where an outright position does not — the reason a directional trader might prefer 25 spreads to 5 futures.

## Connects To
- [Chapter 6 — Volatility](ch06-volatility.md): the "at-the-money option is most sensitive in total points" principle that drives strike selection.
- [Chapter 9 — Risk Measurement II](ch09-risk-measurement-ii.md): why deltas migrate toward or away from 50 and cause inversions.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): the delta-neutral versions of the same structures.
- [Chapter 13 — Risk Considerations](ch13-risk-considerations.md): the margin-for-error framework applied to these choices.
- [Chapter 17 — Hedging with Options](ch17-hedging-with-options.md): directional option positions used protectively.
