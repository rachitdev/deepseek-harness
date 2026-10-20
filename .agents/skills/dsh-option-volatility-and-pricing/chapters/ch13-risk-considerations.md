# Chapter 13 — Risk Considerations

## Core Idea
Theoretical edge is one number; risk is not. "Theoretical edge is only an indication of what we expect to earn if we are right about market conditions." Since you can make the edge as large as you like simply by trading bigger, **edge cannot be the criterion**. The method: **equalize the theoretical edge across candidate spreads by adjusting size, then compare their risk profiles**. Natenberg's definition of a good spread:

> "A good spread is not necessarily the one that shows the greatest profit when things go well; it may be the one that shows the least loss when things go badly. Winning trades always take care of themselves. Losing trades that do not give back all the profits from the winning ones are just as important."

## Frameworks Introduced

**The five risks, named and characterized:**
- **Delta (directional) risk** — the market moves one way rather than another. Delta neutrality does not eliminate directional risk; it makes the position immune within a limited range.
- **Gamma (curvature) risk** — a large move regardless of direction. A *positive* gamma position has no gamma risk (movement helps); a *negative* gamma position can lose its edge quickly.
- **Theta (time decay) risk** — the other side of gamma. With a negative theta, ask: how much time can pass before the edge disappears — a day, a week, a month?
- **Vega (volatility) risk** — the volatility input may be wrong, or implied volatility may change. Every position has it, in one direction or the other. Ask both "how far can volatility move against me before the profit disappears?" and "will I be able to *maintain* the position through adverse implied-volatility moves?"
- **Rho (interest-rate) risk** — least important of the model inputs, hence usually the least important risk measure.

**The comparison procedure.** Scale every candidate spread to the same theoretical edge, then plot:
1. Theoretical P&L vs. **underlying price**
2. Theoretical P&L vs. **volatility** (from which read each spread's **breakeven / implied volatility**)
3. Theoretical P&L vs. **time to expiration**
4. For stock options with a dividend between expirations: P&L vs. **interest rate** and vs. **dividend**

**Efficiency = |gamma / theta|.** For a positive-gamma/negative-theta position the reward is the gamma and the risk the theta, so you want the ratio **large**. For a negative-gamma/positive-theta position the risk is the gamma and the reward the theta, so you want it **small**. Valid as a quick comparison only when all options expire at the same time and edges are similar; with multiple expirations, vega matters too and a fuller analysis is needed.

**Sizing to the margin for error.** Rather than asking "what is an adequate margin for error?" (which depends entirely on the market and the trader's experience — 5 percentage points may be enormous in one market and nothing in another), ask **what size is right given the margin this spread offers**. A 2×1 ratio spread with an implied volatility of 23% against a 25% forecast is only a 2-point margin → do it 10 times (20 × 10). The same spread at 18% implied is a 7-point margin, and if such a low volatility is extremely rare → 100 × 50.

**Volga is the shape of the volatility risk.** Zero volga → vega constant, symmetric tradeoff. **Negative** volga → vega becomes more negative as volatility rises and less negative as it falls, so changes work against you in both directions (accelerating losses on the way up, shrinking gains on the way down). **Positive** volga → changes work in your favor in both directions.

**The three adjustment choices, and their consequences.** Adjusting delta with the **underlying** is essentially **risk-neutral** — the underlying has zero gamma, theta, and vega, so nothing else changes. Adjusting with **options** necessarily changes gamma, theta, and vega too. "This is something that new traders sometimes forget."

**Trend style and gamma.**
> A trader with a **negative gamma** is always adjusting **with** the trend of the underlying market. A trader with a **positive gamma** is always adjusting **against** the trend.

| Preference | Route A | Route B |
|---|---|---|
| Trade **with** the trend | Positive gamma + less frequent adjustments | Negative gamma + more frequent adjustments |
| Trade **against** the trend | Negative gamma + less frequent adjustments | Positive gamma + more frequent adjustments |

The purely theoretical trader has no trend to worry about — but "old habits, such as trading with or against the trend, are hard to break."

## Key Concepts
- **Ruin risk trumps expected value.** A strategy that wins $7,000 half the time and loses $5,000 half the time averages +$1,000 — but a trader with $3,000 of capital who loses first is out of business. "No trader will initiate a strategy where short-term bad luck might end his trading career." Every trader is his own financial officer.
- **Straddles and strangles are the riskiest of all spreads** — bought or sold. New traders assume buying them is safe because the risk is limited, but "it can be just as painful to lose money day after day when one buys a straddle or strangle and the market fails to move as it is to lose the same amount of money all at once when one sells a straddle and the market makes a violent move." They offer the least margin for error.
- **Theoretically best is not always practically best.** The long butterfly (Spread 3) has the best gamma/vega risk profile, but it is a **three-sided spread**: harder to execute, wider effective bid-ask, leg risk if executed piecemeal, and it required a 100 × 200 × 100 size to match the others' edge — which the May 46/48/50 puts may not have the liquidity to absorb, and which raises retail transaction costs.
- **Adjusting to maximize edge inflates the position.** In the running example a trader sells 20 of the 95/105 strangle, then keeps adjusting by selling more overpriced options: first 11 more 105 calls, then 28 more 95 puts. The position he was willing to do 20 times is now **48 × 31**. "If the market makes a very swift move, the trader may not survive. For this reason, a new trader is usually well advised to avoid making adjustments that increase the size of a position."
- **When size becomes the binding constraint, there are only two choices:** decrease the size of the spread, or adjust in the underlying market. "A disciplined trader knows that sometimes, because of risk considerations, the best course is to reduce the size of the spread, even if it means giving up some theoretical edge… if a trader is unwilling to swallow his pride from time to time, his trading career is likely to be a short one."
- **Liquidity is a risk input.** The most liquid options are **short-term and at or slightly out of the money** — narrowest bid-ask, many willing counterparties. Bid-ask widens and interest thins for longer-term and deeper in-the-money options; "deeply in-the-money long-term options may not trade for weeks at a time." Before entering an illiquid spread, ask whether you are willing to hold it to expiration, because that may be the only time you can exit at a fair price. Long-dated positions should demand **greater theoretical edge** as compensation for committing capital longer — the same logic as the term structure of interest rates.
- **New vs. experienced traders and liquidity.** New traders should start in liquid markets so mistakes can be exited cheaply. Experienced market makers often prefer *less* liquid markets, where the wider bid-ask means more edge per trade — but any error is one they must live with for a long time.
- **The most dangerous markets are those where both the options and the underlying are inactively traded.** In an illiquid option market with a liquid underlying, at least adjustments remain feasible.
- **Interest-rate and dividend risk are usually small relative to gamma and vega risk** — but not for large positions or when a real change is likely. When all options expire together, both effects hit one forward price and largely wash out of a spread; calendar spreads use two forward prices and so carry real exposure.

## Anti-patterns
- **Judging spreads by theoretical edge.** Just scale up and any spread wins.
- **Reading a single gamma or vega number as the whole risk.** Both change as conditions change. Spread 2 has the smallest gamma (−165.5) and smallest vega (−0.875) at current conditions, yet at a big enough move or a big enough volatility change it deteriorates almost as badly as the short straddle.
- **Adjusting only to harvest more edge**, inflating the position past what you were originally willing to hold.
- **Forgetting that an option adjustment changes gamma, theta, and vega**, not just delta.
- **Entering illiquid long-dated spreads without accepting you may be "married to the position… for what may seem like an eternity."**
- **Chasing a spread that passes every risk test** — "it would probably have so little theoretical edge that it would not be worth doing."

## Reference Tables

**Figure 13-1 — Futures-style evaluation table** (underlying 48.40; volatility 18%; interest rate 0%)

*May, 56 days:*

| Strike | Call price | Call TV | Δ | Γ | Θ | Vega | Call implied | Put price | Put TV | Δ | Put implied |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 44 | 4.59 | 4.53 | 92 | 4.5 | −0.0046 | 0.029 | 19.83% | 0.20 | 0.13 | −8 | 20.12% |
| 46 | 2.99 | 2.86 | 78 | 8.8 | −0.0091 | 0.057 | 20.25% | 0.58 | 0.46 | −22 | 20.09% |
| 48 | 1.75 | 1.56 | 56 | 11.6 | −0.0121 | 0.075 | 20.48% | 1.35 | 1.16 | −44 | 20.48% |
| 50 | 0.93 | 0.73 | 33 | 10.7 | −0.0111 | 0.069 | 20.88% | 2.53 | 2.33 | −67 | 20.88% |
| 52 | 0.47 | 0.28 | 16 | 7.2 | −0.0075 | 0.047 | 21.63% | 4.06 | 3.88 | −84 | 21.45% |
| 54 | 0.23 | 0.09 | 6 | 3.7 | −0.0038 | 0.024 | 22.46% | 5.84 | 5.69 | −94 | 22.73% |

*July, 112 days:*

| Strike | Call price | Call TV | Δ | Γ | Θ | Vega | Put price | Put TV | Δ |
|---|---|---|---|---|---|---|---|---|---|
| 44 | 4.96 | 4.82 | 84 | 5.0 | −0.0052 | 0.064 | 0.56 | 0.42 | −16 |
| 46 | 3.52 | 3.32 | 71 | 7.1 | −0.0074 | 0.091 | 1.13 | 0.92 | −29 |
| 48 | 2.38 | 2.12 | 55 | 8.2 | −0.0085 | 0.106 | 1.98 | 1.72 | −45 |
| 50 | 1.55 | 1.24 | 39 | 8.0 | −0.0083 | 0.103 | 3.14 | 2.86 | −61 |
| 52 | 0.97 | 0.69 | 25 | 6.6 | −0.0069 | 0.085 | 4.58 | 4.29 | −75 |
| 54 | 0.60 | 0.35 | 15 | 4.8 | −0.0050 | 0.062 | 6.21 | 5.95 | −85 |

**Figures 13-2 / 13-3 — three negative-vega spreads, before and after equalizing edge**

| | Original size | Edge | Scaled size | Edge | Delta | Gamma | Theta | Vega | Efficiency \|Γ/Θ\| |
|---|---|---|---|---|---|---|---|---|---|
| **Spread 1** short straddle −15 May 48 calls / −20 May 48 puts | 15 × 20 | +6.65 | same | +6.65 | +40 | −406.0 | +0.4235 | −2.625 | 959 |
| **Spread 2** ratio call +10 May 50 / −20 May 52 | 10 × 20 | +1.80 | 35 × 70 | +6.30 | +70 | −165.5 | +0.1365 | −0.875 | 1,212 |
| **Spread 3** long put butterfly +10 May 46 / −20 May 48 / +10 May 50 | 10 × 20 × 10 | +0.60 | 100 × 200 × 100 | +6.00 | −100 | −370.0 | +0.4000 | −2.400 | **925** |

Breakeven volatilities (Figure 13-5): **Spread 1 ≈ 21%**, **Spread 2 ≈ 23%**, **Spread 3 ≈ 21.5%**. Volga: Spread 1 ≈ 0, Spread 2 **negative**, Spread 3 **positive**.

**Figure 13-6 — three more spreads, edge equalized**

| Spread | Edge | Delta | Gamma | Theta | Vega | Breakeven volatility |
|---|---|---|---|---|---|---|
| **4** short put calendar: +85 May 48 puts / −85 July 48 puts | +5.95 | +85 | +289.0 | −0.3060 | −2.635 | ≈20.5% |
| **5** diagonal call: +100 May 52 calls / −100 July 54 calls | +6.00 | +100 | +240.0 | −0.2500 | −1.500 | ≈22% |
| **6** put diagonal ratio: +30 May 48 puts / −80 July 44 puts | +5.50 | −40 | −52.0 | +0.0530 | −2.870 | ≈20% |

**Figure 13-10 — Stock-option evaluation table** (stock 99.25; dividend 0.50 in 4 weeks and every 13 weeks after; volatility 29%; interest rate 6%)

*March, 8 weeks:*

| Strike | Call price | Call TV | Δ | Γ | Θ | Vega | Put price | Put TV | Δ |
|---|---|---|---|---|---|---|---|---|---|
| 85 | 14.90 | 10.91 | 93 | 1.2 | −0.0264 | 0.053 | 0.37 | 0.38 | −7 |
| 90 | 10.51 | 10.65 | 83 | 2.3 | −0.0371 | 0.098 | 0.94 | 1.07 | −17 |
| 95 | 6.74 | 7.06 | 68 | 3.2 | −0.0457 | 0.138 | 2.11 | 2.44 | −32 |
| 100 | 3.85 | 4.32 | 51 | 3.6 | −0.0477 | 0.154 | 4.18 | 4.65 | −49 |
| 105 | 1.95 | 2.42 | 34 | 3.3 | −0.0421 | 0.142 | 7.23 | 7.71 | −66 |
| 110 | 0.88 | 1.25 | 21 | 2.6 | −0.0319 | 0.111 | 11.12 | 11.49 | −79 |

*June, 21 weeks:*

| Strike | Call price | Call TV | Δ | Γ | Θ | Vega | Put price | Put TV | Δ |
|---|---|---|---|---|---|---|---|---|---|
| 85 | 16.70 | 16.90 | 84 | 1.3 | −0.0256 | 0.149 | 1.40 | 1.61 | −16 |
| 90 | 12.85 | 13.25 | 76 | 1.7 | −0.0293 | 0.195 | 2.44 | 2.84 | −24 |
| 95 | 9.52 | 10.11 | 66 | 2.0 | −0.0316 | 0.229 | 3.99 | 4.58 | −34 |
| 100 | 6.78 | 7.51 | 55 | 2.2 | −0.0320 | 0.247 | 6.13 | 6.86 | −45 |
| 105 | 4.64 | 5.44 | 45 | 2.2 | −0.0306 | 0.246 | 8.87 | 9.67 | −55 |
| 110 | 3.06 | 3.84 | 35 | 2.0 | −0.0278 | 0.231 | 12.18 | 12.95 | −65 |

**Figures 13-11 / 13-14 — four positive-vega stock-option spreads**

| Spread | Edge | Delta | Gamma | Theta | Vega | Total rho | Total dividend sensitivity |
|---|---|---|---|---|---|---|---|
| **7** call calendar: +20 June 100 calls / −20 March 100 calls | +5.40 | +80 | −28.0 | +0.3140 | +1.860 | **+2.340** | **−11.600** |
| **8** put calendar: +20 June 95 puts / −20 March 95 puts | +5.20 | −40 | −20.0 | +0.3030 | +1.820 | **−2.040** | **+7.220** |
| **9** call diagonal: +17 June 110 calls / −17 March 105 calls | +5.27 | +17 | −22.1 | +0.2431 | +1.513 | **+1.258** | **−5.882** |
| **10** put diagonal: +65 June 85 puts / −65 March 90 puts | +5.20 | +65 | −65.0 | +0.6760 | +3.315 | **−2.665** | **+9.035** |

**Figure 13-14 — Per-option interest-rate and dividend sensitivity** (rho = point change per 1.00% rate change; dividend sensitivity = point change per point change in the dividend)

*March:*

| Strike | Call rho | Put rho | Call div. sens. | Put div. sens. |
|---|---|---|---|---|
| 85 | 0.118 | −0.012 | −0.924 | 0.072 |
| 90 | 0.109 | −0.027 | −0.826 | 0.169 |
| 95 | 0.093 | −0.052 | −0.681 | 0.315 |
| 100 | 0.071 | −0.081 | −0.509 | 0.487 |
| 105 | 0.048 | −0.111 | −0.342 | 0.653 |
| 110 | 0.030 | −0.138 | −0.208 | 0.788 |

*June:*

| Strike | Call rho | Put rho | Call div. sens. | Put div. sens. |
|---|---|---|---|---|
| 85 | 0.266 | −0.068 | −1.668 | 0.308 |
| 90 | 0.247 | −0.107 | −1.498 | 0.478 |
| 95 | 0.220 | −0.154 | −1.300 | 0.676 |
| 100 | 0.188 | −0.205 | −1.089 | 0.887 |
| 105 | 0.154 | −0.258 | −0.880 | 1.096 |
| 110 | 0.122 | −0.310 | −0.688 | 1.288 |

## Worked Example
**Choosing among three negative-vega spreads with equal edge.**

The evaluation table (Figure 13-1) shows every option's price above its theoretical value at 18% volatility — every implied volatility is roughly 20–22%. So look for **negative vega**: short straddles/strangles, ratio spreads selling more than buying, long butterflies, short calendars. Restricting to May options rules out calendars, so every candidate will be negative gamma and negative vega.

*Step 1 — three candidates.* Spread 1 (short 15 May 48 calls / short 20 May 48 puts, a 4:3 straddle ratioed toward neutral) has edge **+6.65**; Spread 2 (+10 May 50 calls / −20 May 52 calls) has **+1.80**; Spread 3 (+10 May 46 puts / −20 May 48 puts / +10 May 50 puts) has **+0.60**. On edge alone Spread 1 wins — but Spread 2 done 50 × 100 would yield 9.00, so the comparison is meaningless.

*Step 2 — equalize the edge.* Scale Spread 2 to **35 × 70** (+6.30) and Spread 3 to **100 × 200 × 100** (+6.00). Now compare risks.

*Step 3 — price risk.* Spread 2 has the smallest gamma (−165.5) so it *looks* safest, but gammas change. Plotting P&L against the underlying (Figure 13-4): Spread 1's losses grow without limit in both directions; Spread 2 has **unlimited upside risk** but flattens on the downside and eventually shows a small profit; Spread 3 **flattens in both directions** — limited risk regardless of direction. If you fear a large downward move, Spread 2. If you cannot accept unlimited loss either way, Spread 3.

*Step 4 — volatility risk.* Breakevens: Spread 1 ≈ **21%**, Spread 2 ≈ **23%**, Spread 3 ≈ **21.5%**. On that measure Spread 2 looks least risky — but push volatility to 30% or 40% and Spread 2 begins losing at almost the same rate as the short straddle, while Spread 3 flattens (a butterfly's loss is bounded). And if volatility *falls* well below 18%, Spread 2's profit **declines toward zero** while Spread 3's **accelerates**. The reason is volga: Spread 1's is near zero (constant ATM vega), Spread 2's is negative (volatility changes work against it in both directions), Spread 3's is positive (they work for it in both directions).

*Step 5 — efficiency check.* |Γ/Θ| = 959, 1,212, and **925**. All three are negative gamma / positive theta, so smaller is better — Spread 3 again, consistent with the graphical analysis.

*Step 6 — practical reality.* Spread 3 is three-sided, harder and costlier to execute, exposed to leg risk, and needs 100 × 200 × 100 of liquidity in the May 46/48/50 puts. If it is impractical, the choice narrows to Spreads 1 and 2, and **Spread 2 is the clear winner** — greater margin for error in both gamma and vega.

**A second worked comparison, with calendars.** Spreads 4, 5, and 6 all have negative vega with breakevens ≈20.5%, ≈22%, ≈20%. Spread 5 looks lowest-risk on rising volatility, but its **negative volga** means the benefit of falling volatility fades fast — its vega reaches 0 at about **10%** volatility. Spread 6's volga is larger still: its vega actually turns **positive** below **11%**. Spread 4, the short calendar, has volga ≈ 0 and offers a symmetric tradeoff. On gamma, Spreads 4 and 5 are positive (they profit from movement — Spread 4 roughly symmetrically, Spread 5 more on the upside, though capped because the May 52 / July 54 spread can never exceed 2.00), while Spread 6 is negative and, on a large *downside* move, faces potentially unlimited loss. On decay (Figure 13-9): Spread 4 loses value at an accelerating rate; Spread 5 decays for **five weeks** and then turns positive; Spread 6 gains slightly at first but loses its entire edge after **seven weeks**.

**The adjustment trap, in numbers.** Underlying 99.25, all options overpriced. Sell 20 of the 95/105 strangle (95 put delta −32, 105 call delta +34): position delta = (−20 × 34) + (−20 × −32) = **−40**.

A week later the market is 97.00; new deltas −39 and +25: delta = (−20 × −39) + (−20 × 25) = **+280**. Three ways back to neutral: sell underlying, sell calls, buy puts. **Buying puts** would also reduce gamma, theta, and vega risk — but with options still overpriced it reduces theoretical edge. **Selling 11 more 105 calls** increases edge: (−20 × −39) + (−31 × 25) = **−5**. ✓

Another week; market back to 101.00, deltas −24 and +37: delta = (−20 × −24) + (−31 × 37) = **−667**. Sell 28 more 95 puts: (−48 × −24) + (−31 × 37) = **+5**. ✓

Every adjustment maximized edge — and the 20-lot strangle is now **48 × 31**. A violent move in either direction is now magnified accordingly.

## Key Takeaways
- Equalize edge by size, then compare risk graphs against price, volatility, time, rates, and dividends.
- A single gamma or vega reading is not the risk; the risk is how it behaves at the extremes, which is what volga and the graphs reveal.
- Straddles and strangles carry the largest gamma and vega and the least margin for error — long or short.
- |Γ/Θ| is a quick efficiency check when all options expire together; want it large for positive gamma, small for negative.
- Choose size from the margin for error, not the other way round.
- Adjust with the underlying when you want to change only delta; adjust with options only if you accept the change to gamma, theta, and vega — and never let edge-seeking inflate a position beyond what you were willing to hold.
- Negative gamma means you adjust with the trend; positive gamma means you adjust against it. Match your adjustment frequency to your style.
- Liquidity is part of the risk: short-term at- or slightly out-of-the-money options are the most liquid; long-dated illiquid positions should demand more edge.

## Connects To
- [Chapter 9 — Risk Measurement II](ch09-risk-measurement-ii.md): volga, vanna, and the higher-order behavior driving these graphs.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): the spread types being compared, and the adjustment disciplines.
- [Chapter 12 — Bull and Bear Spreads](ch12-bull-and-bear-spreads.md): the directional analogue of the strike-selection reasoning.
- [Chapter 10 — Introduction to Spreading](ch10-introduction-to-spreading.md): margin for error as the reason to spread at all.
- [Chapter 21 — Position Analysis](ch21-position-analysis.md): the same analysis applied to a whole book.
