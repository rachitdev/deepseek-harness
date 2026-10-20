# Chapter 7 — Risk Measurement I

## Core Idea
A stock or futures trader has exactly one risk: direction. An option trader has as many risks as the model has inputs, because any of them can be estimated wrongly or can change. The **Greeks** — delta, gamma, theta, vega, rho — are the model's own output, giving both the *direction* and the *magnitude* of the change in an option's value when each input moves. "The delta, gamma, theta, vega, and rho enable him to identify risk; they do not eliminate risk."

## Frameworks Introduced

**The five risk measures.**

| Greek | Measures | Usual units |
|---|---|---|
| **Delta (Δ)** | Sensitivity to the underlying price | Deltas (0–100 for calls, −100 to 0 for puts); underlying = 100 |
| **Gamma (Γ)** | Rate of change of the delta ("curvature") | Deltas gained/lost per one-point move |
| **Theta (Θ)** | Time decay | Value lost per day (conventionally negative) |
| **Vega (Κ, sometimes ν)** | Sensitivity to volatility | Value change per one percentage point of volatility |
| **Rho (Ρ)** | Sensitivity to interest rates | Value change per one percentage point of rate |

Vega is not a Greek letter; academic literature prefers **kappa (Κ)**. Traders keep "vega" because it starts with a *v*, like volatility.

**Four interpretations of the delta.**
1. **Rate of change** — a call with delta 25 changes at 25% of the underlying's rate; underlying up 1.00 → option up 0.25. Delta 75, underlying up 0.60 → option up 0.45.
2. **Hedge ratio** — divide 100 by the option's delta. Delta 50 → 100/50 = 2/1: sell one underlying per two calls bought. Delta 40 → 100/40 = 5/2. Put with delta −75 → 100/−75 = 4/−3: buy three underlying per four puts bought.
3. **Theoretical / equivalent underlying position** — every 100 deltas ≈ one underlying contract. Ten calls at delta 50 = +500 deltas = five underlying contracts long (or 500 shares). Selling 20 puts at delta −25 also gives −20 × −25 = **+500**.
4. **Probability** — ignoring sign, the delta approximates the probability of finishing in the money. Delta 25 ≈ 25% chance. This is why at-the-money options have deltas near 50 — and more precisely why **at-the-forward** options do: with a stock at 100, one year, 10% rates, the forward is 110, so the **110 call** has a delta near 50 and the seemingly out-of-the-money **105 call** has a delta *greater* than 50.

**Delta neutrality.** A position is delta neutral when all deltas sum to zero. `+2 × 50` calls with `−1 × 100` underlying = 0. `+4 × (−75)` puts with `−3 × 100` underlying = 0.

**Better price estimate using gamma.** A delta alone is an *instantaneous* measure. For a move from S₁ to S₂, use the **average delta**:

```
new delta       = Δ + (S₂ − S₁) × Γ
average delta   = Δ + (S₂ − S₁) × Γ/2
new value  ≈  C + [(S₂ − S₁) × Δ] + [(S₂ − S₁)² × Γ/2]
```
(remembering that the delta is really a value between 0 and 1.00). Works for puts too, with a negative delta.

**Implied volatility of a whole position.** A rough estimate: `position implied volatility ≈ evaluation volatility + (total theoretical edge / total vega)`. It is the volatility at which the position just breaks even.

## Key Concepts
- **Delta bounds.** A call can never gain or lose value faster than the underlying nor move opposite to it, so call delta ∈ [0, 100] and put delta ∈ [−100, 0]. The underlying contract always has a delta of **100** (whole-number convention originating in U.S. stock options, where one delta = one share of a 100-share contract).
- **The underlying contract's other Greeks are all zero** — gamma 0 (its delta never changes), theta 0 (it does not decay), vega 0, rho 0. Delta is the only risk measure an underlying position carries.
- **Gamma is positive for both calls and puts.** Always *add* gamma to the delta when the underlying rises and *subtract* it when the underlying falls. An ATM call at delta 50 with gamma 5 goes to 55 on a one-point rise; the ATM put at −50 goes to **−45** on the same rise (moving out of the money). On a one-point fall: call 45, put −55.
- **Long options = long gamma; short options = short gamma.** New traders are advised to avoid large gamma positions, especially negative ones, because directional risk changes fast.
- **Gamma and theta are almost always of opposite sign, and their magnitudes correlate.** "An option trader cannot have it both ways. Either market movement will help the position (positive gamma) or the passage of time (positive theta) will—but not both." (Interest considerations can occasionally produce same-sign gamma and theta, but the magnitudes will be tiny.)
- **Theta accelerates for at-the-money options.** Three months out, an ATM option may have theta −0.03; at three weeks, −0.06; at three days, **−0.16**.
- **Positive theta is possible.** A 60 call on an underlying at 100, European, stock-type settled, will be worth 40 at expiration but only the present value of 40 — perhaps **39** — today. It has *negative time value* and must rise toward intrinsic value: a positive theta. Requires all three conditions: stock-type settlement, deeply in the money, and European (if American, everyone would exercise today to earn interest on the intrinsic value).
- **Gamma = realized volatility preference; vega = implied volatility preference.** They usually agree but need not: the underlying can become more volatile while implied volatility falls, or less volatile while implied volatility rises.
- **Delta is direction risk; gamma is magnitude (or speed) risk.** A negative gamma position wants the market to sit still or move slowly; a positive gamma position wants big, fast moves.
- **All risk measures are additive.** Multiply each by the signed contract count and sum.
- **Theoretical edge of an underlying trade is always zero** — from an option trader's viewpoint the underlying's theoretical value is simply the price at which it traded.
- **Rho detail.** Foreign-currency options delivering the actual currency have two rate sensitivities: **rho₁** (domestic) and **rho₂** (foreign, sometimes denoted **phi, Φ**). If both the option and the underlying are futures-type settled, rho = 0. Futures options under stock-type settlement have **negative rho for both calls and puts** (higher rates raise the carrying cost). Stock options: **calls positive rho, puts negative rho**.
- **Why futures options barely care about rates.** Under stock-type settlement, raising rates leaves the forward price unchanged but reduces the option's present value, so both calls and puts fall — but the effect is small because the option's value is small relative to the underlying's, unless very deeply in the money.
- **Why stock calls rise with rates.** Higher rates raise the forward (raising calls, lowering puts) *and* reduce present value (lowering both). "Because the price of a stock is always greater than the price of an option, the increase in the forward price will always have a greater effect than the reduced present value."

## Mental Models
- **"Whenever possible a trader should avoid a short stock position."** A short stock hedge effectively reduces the interest rate by the borrowing cost, lowering the forward, cheapening calls and enriching puts — so option values would depend on how you hedge. The corollary: many active traders carry some long stock so that any hedging sale is a *long* sale at the ordinary rate, avoiding both the rate complication and short-sale regulation.
- **Paralysis through analysis.** "A trader who is so concerned with risk that he is afraid to make a trade cannot profit, no matter how well he understands options." The Greeks are for deciding *beforehand* which risks are acceptable.

## Anti-patterns
- **Treating the delta as an equivalent underlying position without qualification.** An option is not a surrogate for the underlying; it is sensitive to time, volatility, and rates as well. "The delta represents an equivalent underlying position only under very narrowly defined market conditions."
- **Trading the probability interpretation without regard to magnitude.** Selling a 10-delta option that expires worthless nine times in ten is still a losing trade if the tenth loss exceeds the nine premiums. "No experienced trader will want to pursue a strategy that leads to multiple small profits but occasionally results in a disastrous loss."
- **Estimating a large price move with a constant delta.** Use the average-delta/gamma correction.
- **Conflating gamma with vega.** One is realized volatility, the other implied.
- **Holding a position with negative theoretical edge.** Position 2 in Figure 7-14 has an edge of −2.10: if the inputs are right, it loses money in the long run. "The longer the trader holds the position, the more likely it is that it will result in a loss." (A trader may deliberately accept negative edge on one leg to make the rest of a larger position more secure — that is what hedging is.)

## Reference Tables

**Figure 7-1 — Effect of changing market conditions on option values**

| If… | Call values will… | Put values will… |
|---|---|---|
| Underlying price rises | rise | fall |
| Underlying price falls | fall | rise |
| Volatility rises | rise | rise |
| Volatility falls | fall | fall |
| Time passes | fall* | fall* |

*In unusual cases (deeply ITM, European, stock-type settled) an option's value can rise as time passes.

**Figure 7-2 — Effect of changing interest rates**

| | Domestic rates rise | Domestic rates fall | Foreign rates rise | Foreign rates fall |
|---|---|---|---|---|
| Stock option calls | rise | fall | n/a | n/a |
| Stock option puts | fall | rise | n/a | n/a |
| Futures option calls (stock-type settlement) | fall | fall | n/a | n/a |
| Futures option puts (stock-type settlement) | fall | fall | n/a | n/a |
| Futures option calls/puts (futures-type settlement) | no effect | no effect | n/a | n/a |
| Foreign currency option calls | rise | fall | fall | rise |
| Foreign currency option puts | fall | rise | rise | fall |

**Figure 7-3 — Effect of changing dividends on stock option values**

| | Dividend raised | Dividend cut |
|---|---|---|
| Stock option calls | fall | rise |
| Stock option puts | rise | fall |

**Figure 7-12 — Signs of the risk measures**

| If you are… | Delta | Gamma | Theta | Vega | Rho |
|---|---|---|---|---|---|
| Long the underlying | + | 0 | 0 | 0 | 0 |
| Short the underlying | − | 0 | 0 | 0 | 0 |
| Long calls | + | + | − | + | + (stock) / − (futures*) |
| Short calls | − | − | + | − | − (stock) / + (futures*) |
| Long puts | − | + | − | + | − (stock) / − (futures*) |
| Short puts | + | − | + | − | + (stock) / + (futures*) |

*When options on futures are subject to stock-type settlement. Under futures-type settlement the effective rho is zero.

**Figure 7-15 — What each position sign wants**

| Position | You want… |
|---|---|
| Delta positive | The underlying price to rise |
| Delta negative | The underlying price to fall |
| Gamma positive | Big or very fast moves |
| Gamma negative | The market to sit still or move very slowly |
| Theta positive | The passage of time increases your value |
| Theta negative | The passage of time reduces your value |
| Vega positive | Implied volatility to rise |
| Vega negative | Implied volatility to fall |
| Rho positive | Interest rates to rise |
| Rho negative | Interest rates to fall |

**Figure 7-13 — Theoretical evaluation** (stock 99.50; 91 days to June expiration; volatility 25%; dividend 0; interest rate 6.00%)

| June calls | Price | Theoretical value | Delta | Gamma | Theta | Vega | Rho |
|---|---|---|---|---|---|---|---|
| 90 | 12.30 | 11.96 | 84 | 2.0 | −0.029 | 0.122 | 0.178 |
| 95 | 8.55 | 8.33 | 71 | 2.8 | −0.034 | 0.170 | 0.155 |
| 100 | 5.35 | 5.44 | 56 | 3.2 | −0.035 | 0.196 | 0.124 |
| 105 | 3.15 | 3.32 | 40 | 3.1 | −0.032 | 0.192 | 0.091 |
| 110 | 1.80 | 1.90 | 27 | 2.6 | −0.027 | 0.163 | 0.062 |

| June puts | Price | Theoretical value | Delta | Gamma | Theta | Vega | Rho |
|---|---|---|---|---|---|---|---|
| 90 | 1.45 | 1.12 | −16 | 2.0 | −0.014 | 0.122 | −0.043 |
| 95 | 2.63 | 2.42 | −29 | 2.8 | −0.019 | 0.170 | −0.078 |
| 100 | 4.35 | 4.45 | −44 | 3.2 | −0.019 | 0.196 | −0.121 |
| 105 | 7.10 | 7.26 | −60 | 3.1 | −0.015 | 0.195 | −0.167 |
| 110 | 10.70 | 10.77 | −73 | 2.6 | −0.009 | 0.163 | −0.209 |

**Figure 7-14 — Five positions and their aggregate risk**

| Position | Theoretical edge | Delta | Gamma | Theta | Vega | Rho |
|---|---|---|---|---|---|---|
| **1.** −10 June 95 calls, +7 underlying | +2.20 | −10 | −28 | +0.34 | −1.70 | −1.55 |
| **2.** +10 June 100 calls, +10 June 100 puts | **−2.10** | +120 | +64 | −0.54 | +3.92 | +0.30 |
| **3.** +10 June 105 puts, −20 June 95 puts | +5.80 | −10 | −25 | +0.23 | −1.48 | +0.11 |
| **4.** +10 June 100 calls, −10 June 90 calls | +2.30 | −280 | −12 | −0.06 | +0.74 | −0.54 |
| **5.** −10 June 100 puts, +20 June 105 puts, −10 June 110 puts | +3.50 | −30 | +4 | −0.02 | +0.25 | −0.04 |

## Worked Example

**A. Using gamma to reprice a large move.** At an underlying price of 97.50 a call has theoretical value **3.65**, delta **40**, gamma **2.5**. The underlying rises to 101.50 (four points).
- New delta = 40 + (4 × 2.5) = **50**
- Average delta over the move = (40 + 50)/2 = **45**
- New option value ≈ 3.65 + (4.00 × 0.45) = **5.45**

Using the constant original delta would have given 3.65 + (4.00 × 0.40) = 5.25 — understating the gain by 0.20.

**B. Reading Position 1 end to end** (from Figures 7-13/7-14): sell 10 June 95 calls at 8.55 against a theoretical value of 8.33, and buy 7 underlying contracts.

*Theoretical edge.* 10 × 0.22 = **+2.20** from the options; the underlying contributes zero by definition.

*Delta.* (−10 × 71) + (7 × 100) = −710 + 700 = **−10** — a trivial downward bias; for practical purposes delta neutral.

*Gamma.* −10 × 2.8 = **−28**. Walk it: if the stock rises to 100.50, delta = −10 + (−28) = **−38**; at 101.50, −38 + (−28) = **−66**. Each point up makes the position want the market lower still. If the stock falls to 98.50, delta = −10 − (−28) = **+18**; at 97.50, +18 − (−28) = **+46**. Each point down makes the position want the market higher. Wanting neither a rise nor a fall leaves one wish: **the market should sit still**. Worst case: a swift upward move — wrong on both direction and speed.

*Theta.* −10 × −0.034 = **+0.34**. That is exactly where the profit comes from if the market obliges and stays near 99.50: about 0.34 per quiet day. Note the sign relationship — negative gamma, positive theta.

*Vega and the position's implied volatility.* −10 × 0.170 = **−1.70**. Raise volatility from 25% to 26% and the edge becomes 2.20 + (−1.70) = **+0.50**; to 27% and it becomes 0.50 + (−1.70) = **−1.20**. The breakeven volatility is therefore

```
25% + (2.20 / 1.70)% = 25% + 1.29% = 27.29%
```
which is the **implied volatility of the position** — the volatility that must be realized over its life for it to just break even.

*Rho.* −10 × 0.155 = **−1.55**. Unsurprising: the seven long underlying contracts dominate the cash flow and create a debit, so a falling rate makes that debit cheaper to carry (+1.55 per point of decline) and a rising rate more expensive.

## Key Takeaways
- Delta = direction, gamma = magnitude/speed, theta = time, vega = implied volatility, rho = rates. Rho matters least for most traders.
- The underlying carries delta 100 and nothing else.
- Gamma is always positive for long options, calls and puts alike; add it on the way up, subtract it on the way down.
- Gamma and theta trade off against each other, in sign and in size.
- Delta approximates the probability of finishing in the money, but the *size* of wins and losses, not just their frequency, decides whether a strategy pays.
- Estimate a big move with the average delta, not the current one.
- A position's implied volatility ≈ evaluation volatility + total edge / total vega.
- Prefer long stock to short stock as a hedge, so the ordinary long rate always applies.

## Connects To
- [Chapter 5 — Theoretical Pricing Models](ch05-theoretical-pricing-models.md): the hedge ratio introduced there is the delta.
- [Chapter 6 — Volatility](ch06-volatility.md): vega quantifies the three volatility-sensitivity principles.
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): maintaining delta neutrality over an option's life.
- [Chapter 9 — Risk Measurement II](ch09-risk-measurement-ii.md): how each Greek behaves across strike, time, and volatility.
- [Chapter 13 — Risk Considerations](ch13-risk-considerations.md): choosing which risks to accept.
- [Chapter 18 — The Black-Scholes Model](ch18-black-scholes-model.md): the formal derivatives behind these numbers.
