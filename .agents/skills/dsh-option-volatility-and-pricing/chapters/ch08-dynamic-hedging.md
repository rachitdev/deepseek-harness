# Chapter 8 — Dynamic Hedging

## Core Idea
A theoretical pricing model tells you the *long-run* value of a mispriced option, but a single trade is one bet, and one bet is at the mercy of luck. **Dynamic hedging turns the life of one option into a series of bets** at the same favorable odds. By re-establishing delta neutrality at intervals, the trader forces himself to sell as the market rises and buy as it falls, capturing the mismatch between the option's changing delta and the underlying's fixed delta of 100. Natenberg's closing principle:

> "In theory, we can replicate an option position through a dynamic hedging process. The cost of this replication is equal to the sum of all the cash flows resulting from the dynamic hedging process. The present value of this sum is equal to the option's theoretical value."

## Frameworks Introduced

**The dynamic hedging cycle.**
1. Buy the underpriced option (or sell the overpriced one).
2. Compute its delta; take the opposing position in the underlying to reach **delta neutral**.
3. At each interval, recompute the delta with the new underlying price and new time to expiration — **holding volatility and interest rate constant**, as the model assumes.
4. Trade the underlying to return to delta neutral. These trades are **adjustments**; because an underlying contract has no theoretical value to an option trader, adjustments have **zero theoretical edge** — they exist only to keep the hedge neutral.
5. At expiration: let OTM options expire worthless; sell ITM options at parity (or exercise and offset); liquidate any remaining underlying at market.

**Static hedge vs. dynamic hedge.** A protective strategy (Chapter 17) is a **static hedge** — opposing positions in different contracts carried to a fixed maturity. Capturing a mispricing requires a **dynamic hedge** — periodic re-evaluation and re-neutralization over the option's entire life.

**The five P&L components** of a stock-option dynamic hedge:
1. P&L on the original hedge (option leg + underlying leg)
2. P&L from all the adjustments
3. Carry (interest) on the option position
4. Carry (interest) on the underlying stock position
5. Interest on the adjustments (each adjustment's cash flow carried to expiration)

Then discount the total cash flow back to the present. For futures options, components 4 and 5 are replaced by **interest on the variation** credits and debits.

**Two rehedging disciplines** (absent transaction-cost considerations):
- **Rehedge at regular intervals** (weekly, daily, hourly). More frequent → results converge on the model's prediction.
- **Rehedge when the delta becomes unbalanced by a predetermined amount.** With a 500-delta tolerance, week 1 (+400) needs no trade. With a 1,000-delta tolerance, weeks 1 (+400), 3 (+800), and 8 (+1,000) need no trade. With a 1,500-delta tolerance, weeks 1, 3, 7 (−1,100), and 8 also need no trade. Wider tolerance = fewer trades = wider dispersion around the predicted result.

**The frictionless-market assumptions** the exercise depends on, and how each is violated:
1. Traders can freely buy or sell the underlying — violated by short-sale restrictions and by **locked limit** futures markets.
2. Traders can borrow and lend unlimited money at one constant rate — violated: rates differ by participant and by direction, with a possibly large borrow/lend spread. Fortunately rates matter least.
3. Transaction costs are zero — the most consequential violation. High costs can eat the entire hedge profit, and the cost scales with adjustment frequency.
4. No tax consequences — real, but idiosyncratic; the book assumes traders maximize pretax profit and worry about taxes afterward.

## Key Concepts
- **Adjustments force you to buy low and sell high** — mechanically, because a rising market makes the delta positive (sell) and a falling market makes it negative (buy).
- **Adjustments do not change the expected return.** "The adjustment process is simply a way of smoothing out the winning and losing hedges by forcing the trader to make more bets, always at the same favorable odds." A trader who never adjusts still has the odds on his side; he just has far higher variance.
- **Professional vs. retail.** An exchange member with low transaction costs adjusts often; a retail customer with high costs adjusts rarely. Both face the same favorable odds and should end with about the same long-run profit — the retail customer simply sees larger swings. (This ignores the professional's real advantage of buying at the bid and selling at the ask, which a retail customer "can never hope to match, nor should he try to do so.")
- **The race metaphor.** Think of a long-option hedge as a race between the loss in the option's time value and the cash flow from adjustments, with the pricing model as judge. Buy below theoretical value and the adjustments win; buy above it and time decay wins.
- **Breakeven volatility = implied volatility.** In the stock example, the June 100 call bought at 5.00 implies 32.40%. Above 32.40% realized volatility the hedge profits; below it, the hedge loses; at exactly 32.40% the race ends in a tie.
- **You may not need to hold to expiration.** If implied volatility re-evaluates from 32.40% up to 37.62% immediately, the call reprices from 5.00 to 5.89 and can be sold for the full 0.89 × 100 = **89.00** profit. The underlying is unaffected — "implied volatility is a characteristic associated with options, not with underlying contracts" — so the 50 short contracts are bought back at the original 97.70. Faster realization also removes the risk of the inputs drifting over a long holding period.
- **Adverse implied-volatility moves are normal.** If implied falls immediately from 32.40% to 30.35%, the call drops to 4.65 for an instant loss of 100 × −0.35 = **−35.00**. If the 37.62% forecast is right, hold and adjust; the 89.00 is still there at expiration. "Just as a speculator can rarely hope to pick the exact bottom or top… an option trader can rarely hope to pick the exact bottom or top in implied volatility."
- **Higher realized volatility → more and larger adjustments → more adjustment profit** for a long-option hedge. This is the mechanical reason higher volatility raises option value.
- **Continuous rehedging is the theoretical ideal.** Because volatility compounds continuously, the model assumes continuous adjustment; an infinite number of bets would replicate the option's value exactly. Discrete rehedging is the practical approximation, and "most traders have found through experience that using a dynamic hedging strategy, even if only at discrete intervals, is the best way to capture the difference between an option's price and its theoretical value."
- **Dividends** (omitted from the example) would add: dividends received on long stock, dividends paid away on short stock, plus interest on the dividend between payment date and expiration.

## Mental Models
- **The unhedged amount (Figure 8-3).** Between adjustments, the option's curved value line pulls away from the straight hedge line. That gap *is* the profit. Rehedging locks it in and starts a fresh, flat bet. Ignoring interest, the sum of all these gaps approximately equals the option's value.
- **Each rehedge ends one bet and begins another.**

## Anti-patterns
- **Buying a mispriced option and walking away.** Without the hedge, the position is a directional bet, not a volatility bet.
- **Adjusting with options instead of the underlying** when you want a clean, edge-neutral adjustment — the underlying's theoretical edge is always zero.
- **Adjusting so often that transaction costs consume the edge.** "The desirability of a strategy will depend not only on the trader's initial transaction costs but also on the subsequent costs of making adjustments."
- **Closing a hedge because implied volatility moved against you** while your realized-volatility forecast is still intact.
- **Changing the volatility input mid-hedge** in the model examples — the model assumes it constant. (Real traders do revise their views; the assumption is made here for consistency with option pricing theory.)

## Reference Tables

**Figure 8-1 — Stock option dynamic hedge** (stock 97.70; 10 weeks to June expiration; interest rate 6.00%; volatility 37.62%. June 100 call: price 5.00 = 32.40% implied; theoretical value 5.89; delta 50. Position: long 100 calls, short 50 underlying contracts.)

| Week | Share price | Delta of 100 call | Total delta position | Adjustment | Total adjustments | Adjustment cash flow | Interest on adjustment |
|---|---|---|---|---|---|---|---|
| 0 | 97.70 | 50 | 0 | — | — | — | — |
| 1 | 99.50 | 54 | +400 | Sell 4 | short 4 | +398.00 | +4.12 |
| 2 | 92.75 | 35 | −1,900 | Buy 19 | long 15 | −1,762.25 | −16.22 |
| 3 | 95.85 | 43 | +800 | Sell 8 | long 7 | +766.80 | +6.18 |
| 4 | 96.20 | 43 | 0 | None | long 7 | 0 | 0 |
| 5 | 102.45 | 62 | +1,900 | Sell 19 | short 12 | +1,946.55 | +11.20 |
| 6 | 93.30 | 28 | −3,400 | Buy 34 | long 22 | −3,172.20 | −14.60 |
| 7 | 91.15 | 17 | −1,100 | Buy 11 | long 33 | −1,002.65 | −3.46 |
| 8 | 95.20 | 27 | +1,000 | Sell 10 | long 23 | +952.00 | +2.19 |
| 9 | 102.80 | 72 | +4,500 | Sell 45 | short 22 | +4,626.00 | +5.32 |
| 10 | 103.85 | — | — | Buy 22 | — | −2,284.70 | — |

**Figure 8-2 — Stock hedge results**

| Component | Calculation | Amount |
|---|---|---|
| Original hedge, option leg | 100 × (3.85 − 5.00) | −115.00 |
| Original hedge, stock leg | 50 × (97.70 − 103.85) | −307.50 |
| **Original hedge P&L** | | **−422.50** |
| **Adjustment P&L** | sum of all adjustment cash flows | **+467.55** |
| Carry on the options | 100 × −5.00 × 6.00% × 70/365 | −5.75 |
| Carry on the stock | 50 × +97.70 × 6.00% × 70/365 | +56.21 |
| Interest on the adjustments | | −5.27 |
| **Total cash flow** | | **+90.24** |
| **Discounted** | 90.24 / (1 + 0.06 × 70/365) | **+89.21** |
| **Predicted P&L** | 100 × (5.89 − 5.00) | **+89.00** |

**Figure 8-4 — Futures option dynamic hedge** (futures 61.85; 10 weeks to March expiration; interest rate 8.00%; volatility 21.48%. March 60 put: price 1.70 = 23.92% implied; theoretical value 1.46; delta −35. Position: short 100 puts, short 35 futures.)

| Week | Futures price | Delta of 60 put | Total delta position | Adjustment | Total adjustments | Variation | Interest on variation |
|---|---|---|---|---|---|---|---|
| 0 | 61.85 | −35 | 0 | — | — | — | — |
| 1 | 60.83 | −42 | +700 | Sell 7 | short 7 | +35.70 | +0.49 |
| 2 | 62.78 | −28 | −1,400 | Buy 14 | long 7 | −81.90 | −1.01 |
| 3 | 63.16 | −24 | −400 | Buy 4 | long 11 | −10.64 | −0.11 |
| 4 | 61.68 | −34 | +1,000 | Sell 10 | long 1 | +35.52 | +0.33 |
| 5 | 59.86 | −50 | +1,600 | Sell 16 | short 15 | +61.88 | +0.47 |
| 6 | 62.88 | −21 | −2,900 | Buy 29 | long 14 | −151.00 | −0.93 |
| 7 | 61.50 | −31 | +1,000 | Sell 10 | long 4 | +29.98 | +0.13 |
| 8 | 62.60 | −15 | −1,600 | Buy 16 | long 20 | −34.10 | −0.10 |
| 9 | 60.18 | −45 | −3,000 | Sell 30 | short 10 | +36.30 | +0.06 |
| 10 | 58.61 | — | — | Buy 10 | — | — | — |

**Figure 8-5 — Futures hedge results**

| Component | Calculation | Amount |
|---|---|---|
| Original hedge, option leg | 100 × (1.70 − 1.39) | +31.00 |
| Original hedge, futures leg | 35 × (61.85 − 58.61) | +113.40 |
| **Original hedge P&L** | | **+144.40** |
| **Adjustment P&L** | | **−122.01** |
| Carry on the options | 100 × +1.70 × 8.00% × 70/365 | +2.61 |
| Interest on the variation | | −0.67 |
| **Total P&L** | | **+24.33** |
| **Discounted** | 24.33 / (1 + 0.08 × 70/365) | **+23.96** |
| **Predicted P&L** | 100 × (1.70 − 1.46) | **+24.00** |

## Worked Example
**Ten weeks of hedging an underpriced call, reconciled to the model.**

Conditions: stock 97.70, 10 weeks to June expiration, interest 6.00%, and (by crystal ball) a realized volatility over those 10 weeks of **37.62%**. The June 100 call is offered at **5.00** — an implied volatility of 32.40% — against a theoretical value of **5.89**, so it is underpriced by 0.89. Delta is 50.

*Establishing the hedge.* Buy 100 June 100 calls (+5,000 deltas), sell 50 underlying contracts (−5,000 deltas). Net delta **0**. (Since the stock underlying is 100 shares, this is equivalent to selling 5,000 shares.)

*First adjustment.* One week later the stock is 99.50 with 9 weeks left. Re-running the model at the *same* 6.00% and 37.62%, the call's delta is now **54** — so the position is +5,400 − 5,000 = **+400 deltas**. Sell 4 underlying contracts at 99.50 to return to short 54 / delta neutral. That is the end of one bet and the start of another.

*Continue weekly* through the table above: 19 bought at 92.75, 8 sold at 95.85, nothing at 96.20, 19 sold at 102.45, 34 bought at 93.30, 11 bought at 91.15, 10 sold at 95.20, 45 sold at 102.80, and 22 bought back at expiration at 103.85. Notice the pattern: every rise forces a sale, every fall forces a purchase.

*Settling up at expiration* (stock 103.85):
- Options: sell the 100 calls at parity, 3.85, against a 5.00 cost → 100 × (3.85 − 5.00) = **−115.00**.
- Original stock: bought back at 103.85 against the 97.70 sale → 50 × (97.70 − 103.85) = **−307.50**.
- **The original hedge lost 422.50.** Standing alone this looks like a failure.
- Adjustments: summing every cash flow in the adjustment column gives **+467.55**, more than offsetting it.
- Option carry: 100 options × 5.00 = 500.00 of cash outlay financed for 70 days at 6% → −500.00 × 6% × 70/365 = **−5.75**.
- Stock carry: 50 × 97.70 = 4,885.00 credit earning 6% for 70 days → **+56.21**.
- Interest on the adjustments, computed per adjustment to expiration — e.g., week 1's +398.00 credit earns 398.00 × 6% × 63/365 = **+4.12**; week 2's −1,762.25 debit costs −1,762.25 × 6% × 56/365 = **−16.22** — summing to **−5.27**.

*Total cash flow = +90.24.* Discounting back 70 days: 90.24 / (1 + 0.06 × 70/365) = **89.21** — against the model's prediction of 100 × (5.89 − 5.00) = **89.00**.

Two of the five components were positive (adjustments, stock interest) and three negative (original hedge, option carry, adjustment interest). Which components turn out positive is unknowable in advance because price movement is random; the point is that **in some combination they sum to the model's prediction**.

*The mirror case, an overpriced put on futures.* Futures 61.85, 10 weeks to March, 8.00% interest, realized volatility 21.48%. The March 60 put is worth **1.46** but trades at **1.70** (23.92% implied), delta −35. Sell 100 puts and sell 35 futures. Because futures are futures-type settled there is no initial cash outlay — only **variation**. After week 1 the future falls to 60.83, so the 35 short futures generate variation of 35 × (61.85 − 60.83) = **+35.70**, earning 35.70 × 8% × 63/365 = **+0.49**. After the week-1 adjustment (sell 7) the short is 42 futures; the rise to 62.78 produces 42 × (60.83 − 62.78) = **−81.90** of variation debit, costing −81.90 × 8% × 56/365 = **−1.01**. Total variation interest: **−0.67**. Here the original hedge *made* 144.40 and the adjustments *lost* 122.01 — the opposite composition from the stock example — yet the discounted total of **23.96** again lands on the predicted **24.00**.

## Key Takeaways
- Dynamic hedging converts one probabilistic bet into many, reducing variance without changing expected return.
- Adjustments mechanically buy low and sell high and carry zero theoretical edge themselves.
- Total P&L = original hedge + adjustments + option carry + underlying carry (or variation interest) + adjustment interest, discounted — and it lands on the model's prediction when the volatility input is right.
- Breakeven realized volatility for a hedge equals the implied volatility at the trade price.
- A favorable re-pricing of implied volatility lets you take the profit early; an adverse one is not a reason to abandon a sound realized-volatility view.
- Rehedge on a schedule or on a delta tolerance; the frequency is set by transaction costs and risk appetite, not by theory alone.
- The option's theoretical value equals the present value of all the cash flows generated by replicating it.

## Connects To
- [Chapter 5 — Theoretical Pricing Models](ch05-theoretical-pricing-models.md): the riskless hedge this chapter operationalizes.
- [Chapter 7 — Risk Measurement I](ch07-risk-measurement-i.md): delta and gamma, the source of the unhedged amount.
- [Chapter 9 — Risk Measurement II](ch09-risk-measurement-ii.md): gamma rent and how the Greeks evolve.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): the strategies whose profit comes from this same rehedging process.
- [Chapter 17 — Hedging with Options](ch17-hedging-with-options.md): static hedges, the contrast case.
- [Chapter 19 — Binomial Option Pricing](ch19-binomial-option-pricing.md): replication made discrete and explicit.
- [Chapter 23 — Models and the Real World](ch23-models-and-the-real-world.md): the frictionless assumptions examined at length.
