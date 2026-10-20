# Chapter 15 — Option Arbitrage

## Core Idea
A synthetic underlying position and a real underlying position do the same thing but are priced in different markets. **Put-call parity** pins the relationship between them, and every deviation is an arbitrage: buy the cheaper market, sell the dearer. The structures built on that relationship — conversions, reversals, three-ways, boxes, rolls, and time boxes — are the lowest-risk strategies in options and are traded in enormous size. Natenberg's warning is the counterweight: **"very few strategies are truly riskless."**

## Frameworks Introduced

**Put-call parity, derived from the credits and debits of a conversion.** In a December 100 conversion (sell the call, buy the put, buy the stock) held to expiration:

| Credits | Debits |
|---|---|
| Call price **C** | Put price **P** |
| Interest earned on the call **C × r × t** | Interest cost to buy the put **P × r × t** |
| Dividends **D** | Stock price **S** |
| Exercise price **X** received at expiration | Interest cost to buy the stock **S × r × t** |

Setting credits equal to debits and solving for the **combo** value C − P:

```
C + C·r·t + D + X = P + P·r·t + S + S·r·t
(C − P)(1 + r·t) = S(1 + r·t) − D − X
(C − P)(1 + r·t) = F − X                    [since F = S(1 + r·t) − D]

            F − X
   C − P = ─────────
            1 + r·t
```

> The difference between the call price and put price for European options with the same exercise price and expiration date must equal the **present value of the difference between the forward price and the exercise price**.

Also called the **combo value**, the **synthetic relationship**, or the **conversion market**.

**The three cases:**

| Market | Formula |
|---|---|
| Options on futures, **futures-type** settlement (most non-North American futures exchanges) | `C − P = F − X` |
| Options on futures, **stock-type** settlement (most North American futures exchanges) | `C − P = (F − X) / (1 + r·t)` |
| **Stock options** | `C − P = (F − X) / (1 + r·t)` where `F = S(1 + r·t) − D` |

**The stock-option shortcut.** Expand and approximate:
```
C − P = S − D/(1+r·t) − X/(1+r·t)
       ≈ S − (X − X·r·t) − D
C − P ≈ S − X + X·r·t − D
```
i.e. the stock price, minus the exercise price, plus interest on the exercise price, minus expected dividends. Errors grow with high rates, large dividends, or long-dated options, "but for short-term options our approximation often represents a reasonable tradeoff between speed and accuracy."

**Solving for the missing input.** With C, P, S, X, r, D and only one unknown, invert:
- Implied stock price: `S ≈ C − P + X − X·r·t + D`
- Implied interest rate: `r = (C − P − S + X + D)/X ÷ t`
- Implied dividend: `D = S − C + P − X + X·r·t`

**The arbitrage structures:**

| Structure | Construction |
|---|---|
| **Conversion** | long underlying + short call + long put (same strike/expiry) |
| **Reversal (reverse conversion)** | short underlying + long call + short put |
| **Three-way** | conversion or reversal with the underlying replaced by a deeply in-the-money option (long deep ITM call, or short deep ITM put) |
| **Box** | synthetic long at one strike + synthetic short at another, same expiration |
| **Roll** | synthetic long in one month + synthetic short in another, same strike |
| **Time box (diagonal roll)** | synthetic positions at different strikes *and* different months |

**Valuation formulas:**
- **Box**: at expiration worth exactly the **amount between exercise prices**; today worth that amount if futures-type settled, or its **present value** if stock-type settled.
- **Roll** (stock options): `X/(1 + r_s·t_s) − X/(1 + r_l·t_l) − D`. Shortcut: `X·r·t − D`, where t is the time *between* expirations.
- **Time box**: `X_s/(1 + r_s·t_s) − X_l/(1 + r_l·t_l) − D`.

**Decompositions:**
- A **box** = a bull vertical call spread + a bear vertical put spread. Their prices must **sum** to the box value.
- A **roll** = a long call calendar spread + a short put calendar spread. Their prices must **differ** by the roll value.
- A **time box** = two diagonal spreads; also = buy a long-term box and sell a lower-strike roll, *or* buy a short-term box and sell a higher-strike roll.

## Key Concepts
- **Only the difference matters.** Selling the December 100 call at 5.00 and buying the December 100 put at 3.00 with the stock at 102.00 is economically identical to selling the underlying at 102.00 — and would be for *any* pair of prices differing by exactly 2.00.
- **Arbitrage forces convergence through any of three prices.** If C − P = 3.75 but F − X = 4.00, everyone does the reversal — buying the call pushes it to 5.50, or selling the put pushes it to 1.25, or selling the future pushes it to 103.75. Any of the three restores `C − P = F − X`.
- **Exchange settlement prices honor put-call parity.** The Eurex Euro-bund settlement table (Figure 15-2) satisfies `call − put = future − strike` at every strike and month, even where the volatility values may be inconsistent.
- **Locked-limit escape hatch.** When a futures market is limit-up (closed the previous day at 126.75, now locked at 131.75 with a 5.00 limit) a trader can still buy or sell futures **synthetically** through the option market, at whatever price put-call parity implies.
- **Imbalances are short-lived.** "Price imbalances in the synthetic relationship are usually small and rarely last for more than a few seconds." When they do occur, traders execute conversions and reversals in very large size because of the low risk.
- **Rolls are practical only in stock options.** In futures options a June conversion and an August reversal leave a long June future against a short August future — different contracts that do not cancel, so it is "not a true roll."
- **Sign of a roll.** Since the interest component usually exceeds the dividend, a long roll is normally a **debit** and the call calendar is worth more than the put calendar. If dividends exceed interest, the roll can have a **negative** value and the relationship inverts.
- **Rolls at different strikes, same months, differ by interest on the strike difference** — because dividends apply equally to both.
- **Selling a box is borrowing.** With European, cash-settled options (no early exercise, no pin risk), buying or selling a box is identical to lending or borrowing over the option's life. Selling a three-month 90/100 box at **9.80** = borrowing at **8%**; selling it at **9.70** = borrowing at **12%**. A firm can raise short-term cash by selling boxes, at the cost of a below-value price plus margin and transaction costs.
- **Rolls only defer the underlying risk.** The risk of holding the underlying is eliminated only up to the *short-term* expiration, at which point the trader buys or sells stock at the exercise price. Rolls remain sensitive to interest rates and dividends, more so the longer the gap between expirations.

## The four risks of "riskless" arbitrage

**1. Execution risk.** "Because no one wants to give away money, a trader is unlikely to be offered a profitable conversion or reversal all at one time." Legging in — buying puts and underlying, hoping to sell calls later — leaves the trader exposed if call prices fall before he completes the spread.

**2. Pin risk.** If the underlying finishes **exactly at the exercise price**, the conversion holder does not know whether he will be assigned on the short call, and by the time he learns (the day after expiration) the put has expired. At-the-money options *are* frequently exercised despite having no theoretical value, because exercising is almost always cheaper than trading the underlying — the right of exercise is typically included in the original transaction cost. Guessing wrong leaves an unwanted long or short position, magnified by the large size in which conversions and reversals are done. **The practical solution: reduce the size of at-the-money conversions/reversals as expiration approaches**, or cross positions with a trader holding the opposite structure — which is why traders on option exchanges look for counterparties to trade conversions and reversals "at even money." *Pin risk does not exist for cash-settled options* (e.g., stock indexes), where no underlying position results.

**3. Settlement risk** (only when the options and the underlying settle differently). In a December 100 conversion on a December future with stock-type-settled options: a fall in the future to 98.00 produces a ~4.00 **unrealized** gain on the synthetic but an **immediate 4.00 cash debit** on the long future, costing interest that the unrealized gain does not offset. In the extreme the trader may be **forced to liquidate**. The reverse — a rise to 106.00 — gives an unrealized 4.00 loss against an immediate cash credit earning interest, *increasing* profit.

Consequently **a conversion is not exactly delta neutral**:

| Contract | Delta |
|---|---|
| Short December 100 call | −57 |
| Long December 100 put | −41 |
| Long December future | +100 |
| **Total** | **+2** |

Those two deltas reflect the preference for an upward move that credits cash. Done 300 times, that is **+600 deltas** — the equivalent of six extra long futures. In general the delta of a synthetic futures position under stock-type option settlement is **100 / (1 + r·t)**: at 10% with nine months to go it may be **93**; at 4% with one month, **99**. (The analogous hedging problem when futures hedge a physical position is called **tailing**.) There is **no settlement risk when both contracts settle the same way**.

**4. Interest and dividend risk** (stock options). A conversion is done for a debit ≈ the exercise price, so it carries a borrowing cost: **negative rho** (wants rates to fall) and **positive dividend risk** (wants dividends raised, since it is long stock). A reversal is the mirror: **positive rho**, **negative dividend risk**. In a conversion or reversal the call and put rhos share a sign because one option is bought and the other sold.

**Eliminating the underlying.** Replace the underlying leg with a deeply in-the-money option (a **three-way**) — but as the market approaches that option's strike it behaves less and less like an underlying contract. Or replace it with a synthetic at a different strike or month, giving a **box** or **roll**, which removes the underlying-carrying risk entirely.

## Anti-patterns
- **Believing conversions and reversals are riskless.** Execution, pin, settlement, interest, and dividend risks all remain.
- **Carrying at-the-money conversions or reversals into expiration** when the underlying may pin the strike.
- **Ignoring the two extra deltas on a futures conversion** when the position is large.
- **Assuming an apparent parity violation is an arbitrage** — it may just mean your interest rate or dividend estimate is wrong. Back out the implied rate or implied dividend first.
- **Treating a June/August futures "roll" as a true roll** — the underlying futures do not cancel.
- **Selling a three-way's deep ITM option and forgetting the strike risk.**

## Reference Tables

**Figure 15-2 — Euro-bund option settlement prices, 25 May 2010** (June future 129.38; September future 128.90; July and August are serial months whose underlying is the September future). Every row satisfies `call − put = future − strike`.

| Strike | June calls | June puts | July calls | July puts | Aug calls | Aug puts | Sep calls | Sep puts |
|---|---|---|---|---|---|---|---|---|
| 126.00 | 3.39 | 0.01 | 3.16 | 0.26 | 3.45 | 0.55 | 3.76 | 0.86 |
| 126.50 | 2.89 | 0.01 | 2.75 | 0.35 | 3.07 | 0.67 | 3.42 | 1.02 |
| 127.00 | 2.41 | 0.03 | 2.37 | 0.47 | 2.73 | 0.83 | 3.09 | 1.19 |
| 127.50 | 1.93 | 0.05 | 2.02 | 0.62 | 2.40 | 1.00 | 2.78 | 1.38 |
| 128.00 | 1.48 | 0.10 | 1.70 | 0.80 | 2.10 | 1.20 | 2.50 | 1.60 |
| 128.50 | 1.06 | 0.18 | 1.41 | 1.01 | 1.83 | 1.43 | 2.23 | 1.83 |
| 129.00 | 0.71 | 0.33 | 1.16 | 1.26 | 1.59 | 1.69 | 1.99 | 2.09 |
| 129.50 | 0.43 | 0.55 | 0.94 | 1.54 | 1.37 | 1.97 | 1.76 | 2.36 |
| 130.00 | 0.23 | 0.85 | 0.76 | 1.86 | 1.17 | 2.27 | 1.56 | 2.66 |
| 130.50 | 0.12 | 1.24 | 0.60 | 2.20 | 1.00 | 2.60 | 1.37 | 2.97 |
| 131.00 | 0.05 | 1.67 | 0.48 | 2.58 | 0.85 | 2.95 | 1.21 | 3.31 |
| 131.50 | 0.02 | 2.14 | 0.37 | 2.97 | 0.72 | 3.32 | 1.06 | 3.66 |
| 132.00 | 0.01 | 2.63 | 0.29 | 3.39 | 0.61 | 3.71 | 0.92 | 4.02 |

**Extracting a synthetic futures price in a locked market** (no interest considerations; F = C − P + X)

| Exercise price | Call price | Put price | Equivalent synthetic futures price |
|---|---|---|---|
| 120 | 13.60 | 0.35 | 133.25 |
| 125 | 9.35 | 1.05 | 133.30 |
| 130 | 5.75 | 2.55 | 133.20 |
| 135 | 3.15 | 4.95 | 133.20 |
| 140 | 1.55 | 8.30 | 133.25 |
| 145 | 0.70 | 12.40 | 133.30 |

The spread across strikes (133.20–133.30) reflects bid-ask width and non-contemporaneous quotes; a trader can expect to trade synthetically somewhere in that range.

**Rho and dividend signs**

| Structure | Rho | Dividend risk |
|---|---|---|
| Conversion (long stock) | Negative — wants rates to fall | **Positive** — helped by a dividend increase |
| Reversal (short stock) | Positive — wants rates to rise | **Negative** — hurt by a dividend increase |

## Worked Example

**A. Futures options, futures-type settlement.** December 100 call at **5.25**, December 100 put at **1.50**. Since `C − P = 3.75`, `F − X` must be 3.75, so the December future should be **103.75**.

Suppose it is actually **104.00**. Then 5.25 − 1.50 = 3.75 ≠ 4.00, and everyone does a **reversal**: buy the cheap synthetic (buy the call, sell the put), sell the dearer future — an arbitrage profit of **0.25**, the amount of the mispricing.

**B. Futures options, stock-type settlement.** Six months to expiration, 6.00% annual interest, December 100 call at **4.90**, December future at **97.25**.
```
C − P = (97.25 − 100) / (1 + 0.06 × 6/12) ≈ −2.67
P = 4.90 + 2.67 = 7.57
```
The put must trade at **7.57**.

**C. Stock options, exact and approximate.** Stock **68.50**, six months, 4.00% rate, dividends **0.45**, December 65 call at **8.00**.
```
F = 68.50 × (1 + 0.04 × 6/12) − 0.45 = 69.42
C − P = (69.42 − 65) / (1 + 0.04 × 6/12) ≈ 4.33
P = 8.00 − 4.33 = 3.67
```
By the shortcut: `C − P ≈ 68.50 − 65 + 65 × 0.04 × 6/12 − 0.45 = 4.35` — an error of **0.02**.

Sensitivity check on the same combo: raise the rate to 5.00% and it becomes `68.50 − 65 + 65 × 0.05 × 6/12 − 0.45 ≈ 4.68`; raise the dividend and the value falls.

**D. Backing out the market's assumptions.** 90 call **7.20**, 90 put **1.40**, three months to expiration, 8.00% rate, expected dividend **0.47**.
```
S ≈ C − P + X − X·r·t + D = 7.20 − 1.40 + 90 − 90 × 0.08 × 3/12 + 0.47 = 94.47
```
But the stock trades at **94.30**. Rather than assume an arbitrage, question the inputs:
- Implied interest rate: `r = [(7.20 − 1.40 − 94.30 + 90 + 0.47)/90] ÷ (3/12) = 0.0875` → **8.75%**
- Implied dividend: `D = 94.30 − 7.20 + 1.40 − 90 + 90 × 0.08 × 3/12 = 0.30`

The market seems to expect a dividend of only **0.30** — a hint that the company may **cut the dividend** before expiration.

**E. Building a box and pricing it against verticals.** Combine a June 100 conversion with a June 90 reversal:

```
+1 June 90 call    −1 June 100 call
−1 June 90 put     +1 June 100 put
−1 underlying      +1 underlying      ← these cancel
```

leaving a **long June 90/100 box**: synthetically long at 90, synthetically short at 100. At expiration it is worth exactly **10.00** (buy at 90, sell at 100). With three months to go at 8% and stock-type settlement:
```
10.00 / (1 + 3/12 × 0.08) ≈ 9.80
```
Split the box horizontally instead of vertically:
```
+1 June 90 call / −1 June 100 call   = bull call vertical
−1 June 90 put  / +1 June 100 put    = bear put vertical
```
Their prices must **sum** to 9.80. If the call spread trades at **6.00**, the put spread should be **3.80**. A market maker would quote around that — say **3.70 bid / 3.90 ask**. Buy the put spread at 3.70 and then buy the call spread at 6.00 → a **9.70** purchase of a **9.80** box. Sell the put spread at 3.90 and the call spread at 6.00 → a **9.90** sale of a 9.80 box.

**F. Valuing a roll.** June/August 90 roll: two months to June, four to August, 6% constant, dividend **0.40** between expirations.
```
90/(1 + 0.06 × 2/12) − 90/(1 + 0.06 × 4/12) − 0.40 = 89.11 − 88.24 − 0.40 = 0.47
```
Shortcut: `X·r·t − D = 90 × 0.06 × 2/12 − 0.40 = 0.90 − 0.40 = 0.50` — an error of **0.03**.

Decomposed:
```
−1 June 90 call / +1 August 90 call   = long call calendar
+1 June 90 put  / −1 August 90 put    = short put calendar
```
Their prices must **differ** by 0.47. Call calendar at 2.25 → put calendar should be **1.78**. Put calendar at 1.50 → call calendar should be **1.97**.

And the June/August **80** roll should differ from the 90 roll by the interest on 10 points: `0.47 − (90 − 80) × 0.06 × 2/12 = 0.47 − 0.10 = 0.37`.

**G. A time box, valued two ways.** June 90 / August 100 time box (`+1 June 90 call, −1 June 90 put, −1 August 100 call, +1 August 100 put`), same parameters:
```
90/1.01 − 100/1.02 − 0.40 = −9.33
```
The negative sign means a **9.33 debit** — logical, since you buy the underlying at 90 in June and sell it at 100 in August.

Cross-check by decomposition. Buy the June 90/100 box and sell the June/August 100 roll:
```
June 90/100 box    = 10/(1 + 0.06 × 2/12) = 9.90
August 90/100 box  = 10/(1 + 0.06 × 4/12) = 9.80
June/Aug 90 roll   = 0.47
June/Aug 100 roll  = 100/(1+0.06×2/12) − 100/(1+0.06×4/12) − 0.40 = 0.57
```
- Buy June 90/100 box, sell June/August 100 roll: −9.90 + 0.57 = **−9.33** ✓
- Buy August 90/100 box, sell June/August 90 roll: −9.80 + 0.47 = **−9.33** ✓

As two diagonal spreads, the put diagonal is unambiguously a debit (the August 100 put always beats the June 90 put), but the call diagonal could go either way. Whatever the split, the total must be 9.33: call diagonal at 3.50 → put diagonal at **5.83**; put diagonal at 7.75 → call diagonal at **1.58**.

## Key Takeaways
- `C − P = (F − X)/(1 + r·t)` — put-call parity, the anchor of all option arbitrage. With futures-type settlement it simplifies to `C − P = F − X`.
- Quick stock-option approximation: `C − P ≈ S − X + X·r·t − D`.
- Conversion = long underlying + short synthetic; reversal = the opposite. Boxes remove the underlying by pairing two strikes; rolls by pairing two months; time boxes do both.
- A box is worth the strike interval (present-valued under stock-type settlement) and decomposes into a bull and a bear vertical whose prices **sum** to it; a roll decomposes into two calendars whose prices **differ** by it.
- Selling a box is borrowing; the discount to fair value is the interest rate you are paying.
- An apparent parity violation is more often a wrong rate or dividend assumption than an arbitrage — solve for the implied value first.
- The four real risks: execution (legging), pin (at-the-money expiration), settlement (mismatched settlement procedures, giving a synthetic futures delta of 100/(1+r·t)), and interest/dividend.

## Connects To
- [Chapter 2 — Forward Pricing](ch02-forward-pricing.md): the forward price F that put-call parity is built on, and the implied-value technique.
- [Chapter 14 — Synthetics](ch14-synthetics.md): the six synthetics whose *pricing* this chapter supplies.
- [Chapter 1 — Financial Contracts](ch01-financial-contracts.md): stock-type vs futures-type settlement, the source of settlement risk.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): calendar and diagonal spreads, the components of rolls and time boxes.
- [Chapter 16 — Early Exercise of American Options](ch16-early-exercise-american-options.md): why parity is an inequality rather than an equality for American options.
- [Chapter 6 — Volatility](ch06-volatility.md): why same-strike calls and puts share an implied volatility.
