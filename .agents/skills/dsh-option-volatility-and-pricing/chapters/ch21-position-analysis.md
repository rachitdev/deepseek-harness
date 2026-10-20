# Chapter 21 — Position Analysis

## Core Idea
Simple spreads have legible risks. A market maker's book — hundreds of options across many strikes and expirations — does not. "If a trader cannot determine the risks of a position, he will be unprepared to take the necessary action to protect himself when market conditions move against him or to take advantage of his good fortune when market conditions move in his favor." Reading a today's snapshot of delta, gamma, theta, and vega is only the **first step**; the real work is asking how those numbers change as price, volatility, and time change.

## Frameworks Introduced

**Rewriting a position in one option type.** Before pricing models, traders used synthetics to make a complex position recognizable: convert every put to a synthetic call (or vice versa), cancel the offsetting underlying legs, and see what remains.

**The three questions to expand a snapshot into an analysis:**
- What will happen if the underlying contract moves up or down in price?
- What will happen if implied volatility rises or falls?
- What will happen as time passes?

**Graphic interpretation of the Greeks** (Figure 21-2):

| Shape of the P&L-vs-price graph | Meaning |
|---|---|
| Upper left to lower right | Negative delta |
| Lower left to upper right | Positive delta |
| Horizontal | Delta neutral |
| Curves upward — a **"smile"** | Positive gamma (movement either way helps) |
| Curves downward — a **"frown"** | Negative gamma (movement either way hurts) |
| Straight line at a point | An **inflection point** — gamma changing sign |

As time passes, a **positive gamma / negative theta** graph shifts **downward**; a **negative gamma / positive theta** graph shifts **upward**. A **positive vega** position shifts up as volatility rises; a **negative vega** position shifts up as volatility falls — and gamma and vega signs are independent, so all four combinations exist.

**Net contract position.** Ask what remains if the market moves so far that *all* options are deeply in or far out of the money — i.e., all puts become short underlying contracts or all calls become long ones.
- **Upside contract position** = sum of all calls + underlying contracts
- **Downside contract position** = sum of all puts + underlying contracts

"Traders have learned, sometimes through painful experience, that big moves occur more often in the real world than one might expect. Extraordinary and unpredictable events—political and economic upheavals, scientific breakthroughs, natural disasters, corporate takeovers—can sometimes cause markets to move dramatically."

**Two quick estimates from the aggregate Greeks:**
```
price at which a negative-gamma position becomes delta neutral
    ≈ current price − (delta / gamma)

breakeven (implied) volatility of the whole position
    ≈ current volatility + (theoretical edge / vega)
```
A **negative gamma position always wants to become delta neutral** — its profit is maximized where the delta reaches zero.

**Improving the margin for error.** From `edge / vega`, either **increase the theoretical edge without increasing the vega** or **reduce the vega without reducing the edge**.

**Delta migration under time and volatility.** Since deltas move toward 50 when volatility or time increases and away from 50 when they decrease, the *extreme* delta of a position can be computed by pushing every option to 50 (high volatility) or to 0/100 (low volatility or expiration).

**Stock split adjustments.** For a Y-for-X split:

| Quantity | New value |
|---|---|
| Stock price | old × X/Y |
| Exercise price | old × X/Y |
| Option position | old × Y/X |
| Underlying contract | **unchanged** (100 shares) — unless Y/X is not a whole number |
| Delta position | old × Y/X |
| Gamma position | old × (X/Y)² … in practice, the position gamma scales by Y/X per the worked example |
| Theta, vega, rho | **unchanged** |

Where Y is not a whole number (a 3-for-2 split), fractional option positions are not allowed, so the **clearinghouse adjusts the underlying contract instead**: 100 shares × 3/2 = **150 shares**.

## Key Concepts
- **A position can have all four Greeks at zero and still be dangerous.** The Greeks measure risk **under current conditions only**, "but today's market conditions may not be—in fact, cannot be—tomorrow's conditions. Even if the underlying price and volatility remain unchanged, time will pass."
- **Market maker's job, reduced to a loop:** "Get an edge…. Control the risk…. Get an edge…. Control the risk…." The ideal — a horizontal P&L line at a positive theoretical value — is "virtually impossible to achieve" with a large book.
- **Market maker mechanics.** A market maker continuously quotes a two-sided market as **buyer or seller of last resort**. Exchange rules cap the bid-ask width (a 2.00 maximum means a 63.00 bid implies an offer no higher than 65.00) and set a minimum size. Size is quoted alongside and need not be balanced — `63.50–64.50, 500 × 200` means willing to buy 500 but sell only 200. For very large orders (a customer asking **for size**) the market maker may legitimately widen — 62.00–66.00 for 1,000 contracts. In return the exchange grants low fees or **allocation priority** (perhaps 50% of an order against two other equal bidders' 25% each). Market makers do not set prices: "A market maker does not set prices any more than a thermometer sets the temperature."
- **The market maker's three questions:** (1) What does the marketplace think the option is worth? (2) What do *I* think it is worth? (3) What positions am I currently carrying? The first supports pure **scalping** — profit from the bid-ask spread by identifying the equilibrium price with equal buyers and sellers, no pricing theory required. The second adds theoretical edge, captured either by completing an arbitrage or by dynamic hedging. The third is risk management.
- **Skewing quotes to shape inventory.** A market maker near his maximum negative gamma still must quote both sides, but he can shift the whole quote up — from 63.00–65.00 to **63.50–65.50** around a 64.00 value — making a purchase more likely. He may still be hit on the offer, "but at least he has done so at a more advantageous price."
- **Diversify risk across strikes.** A gamma of `+110, +425, +300, +68, −2,388, +92, +244, +616, +338, +195` across ten strikes nets to **zero**, but the −2,388 concentrated at the 95 strike is alarming: as time passes and the underlying approaches 95, that risk grows. "In the same way that a wise investor will seek to diversify his risk, a market maker will strive for a similar goal." Any concentration — one strike, one expiration, or one large sensitivity — is a cause for concern.
- **Cabinet bids.** A trader short far out-of-the-money options still posts margin for each one and may want to buy them back; a trader long worthless options would happily sell at any price. When the willing price is below the exchange minimum (say $5.00), many exchanges permit a **cabinet bid** — typically **one currency unit ($1.00)** — to clear such positions off the books.
- **Analyzing a book requires assumptions beyond the Greeks** — a term structure of implied volatility, and (deferred to Ch 24) a distribution of implied volatility across exercise prices.
- **Dividend risk on a stock book** is easy to estimate without simulation: the position value change ≈ **change in dividend × number of shares held**. To eliminate it, replace the long stock with **synthetic long stock** (sell stock, buy calls, sell puts at one strike) — the same move that turns a conversion into a box.
- **Rho on a book:** a total rho of +12.70 means that for each full point (100 **basis points**, where one basis point = 1/100 of a percentage point) decline in interest rates, the position loses 12.70.
- **Clearing firm limits** are a real constraint: a firm may require enough capital to withstand a **20% move in the underlying in either direction**, or a **doubling of implied volatility**. Fall short and you deposit more or cut the position.
- **The three planning questions:** (1) What will I do if conditions move against me? (2) What will I do if they move in my favor? — "Risk analysis should focus not only on protecting against the bad things that might occur but also on taking advantage of the good things." (3) What can I do **now** to avoid the adverse effects of conditions moving against me later?
- **A stock split changes nothing economically.** "It simply results in an accounting change in such a way that equity is maintained" — theta, vega, and rho are untouched. But a split often signals a company doing well and is "not unusual" to be accompanied by a **dividend increase**, which does change option values.

## Anti-patterns
- **Reading a snapshot of zero Greeks as zero risk.**
- **Assuming a far out-of-the-money short option can be ignored** because it "couldn't possibly go into the money."
- **Estimating the delta-neutral price with a constant gamma.** In the worked example the estimate (88.93) missed the true value (~95.00) badly because gamma grew as the market fell.
- **Letting risk concentrate at a single strike or expiration** even when the aggregate nets to zero.
- **Refusing to trade at inflated prices when survival is the issue.** "If the decline is very swift, the primary objective may be survival. And in the long run, simply surviving, in order to be able to take advantage of those subsequent occasions when conditions work in our favor, can mean the difference between success and failure in option trading."
- **Assuming the analysis will hold.** "When conditions do change, we can never know for certain how the marketplace will react… We may have to accept the fact that our analysis was incorrect."

## Reference Tables

**Small position, all Greeks zero** (underlying 99.60; 9 weeks to September expiration; volatility 18%; interest rate 0)

| | Delta | Gamma | Theta | Vega |
|---|---|---|---|---|
| September 95 put | −25 | 4.3 | −0.019 | 0.132 |
| September 105 call | +25 | 4.3 | −0.019 | 0.132 |

Position: long 10 September 95 puts, short 10 September 105 calls, long 5 underlying contracts (a **risk reversal** or **split strike conversion**).

| Measure | Calculation | Total |
|---|---|---|
| Delta | (10 × −25) − (10 × 25) + (5 × 100) | **0** |
| Gamma | (10 × 4.3) − (10 × 4.3) | **0** |
| Theta | (10 × −0.019) − (10 × −0.019) | **0** |
| Vega | (10 × 0.132) − (10 × 0.132) | **0** |

**How that "riskless" position actually behaves**

| Change in market conditions | Resulting delta | Resulting gamma | Resulting theta | Resulting vega |
|---|---|---|---|---|
| Rising underlying price | Negative | **Negative** | Positive | Negative |
| Falling underlying price | Negative | **Positive** | Negative | Positive |
| Time passes | Positive | 0 | 0 | 0 |
| Volatility rises | Negative | 0 | 0 | 0 |
| Volatility falls | Positive | 0 | 0 | 0 |

**Figure 21-9 — A complex single-expiration position** (underlying 101.25; 6 weeks to expiration; volatility 27.00%; interest rate 0)

*Per-option values:*

| Strike | Call Δ | Γ | Θ | Vega | Put Δ |
|---|---|---|---|---|---|
| 90 | 90.9 | 1.77 | −0.0181 | 0.056 | −9.1 |
| 95 | 77.1 | 3.27 | −0.0335 | 0.104 | −22.9 |
| 100 | 57.2 | 4.23 | −0.0433 | 0.135 | −42.8 |
| 105 | 36.3 | 4.04 | −0.0414 | 0.129 | −63.7 |
| 110 | 19.5 | 2.97 | −0.0304 | 0.095 | −80.5 |

*Position aggregates:*

| Option | Position | Delta | Gamma | Theta | Vega |
|---|---|---|---|---|---|
| 90 C | −4 | −363.6 | −7.08 | +0.0724 | −0.224 |
| 95 C | −12 | −925.2 | −39.24 | +0.4020 | −1.248 |
| 100 C | +14 | +800.8 | +59.22 | −0.6062 | +1.890 |
| 105 C | −17 | −617.1 | −68.68 | +0.7038 | −2.193 |
| 110 C | +12 | +234.0 | +35.64 | −0.3648 | +1.140 |
| **Call totals** | **−7** | **−871.1** | **−20.14** | **+0.2072** | **−0.635** |
| 90 P | +13 | −118.3 | +23.01 | −0.2353 | +0.728 |
| 95 P | −20 | +458.0 | −65.40 | +0.6700 | −2.080 |
| 100 P | −8 | +342.4 | −33.84 | +0.3464 | −1.080 |
| 105 P | +12 | −764.4 | +48.48 | −0.4968 | +1.548 |
| 110 P | +8 | −644.0 | +23.76 | −0.2432 | +0.760 |
| **Put totals** | **+5** | **−726.3** | **−3.99** | **+0.0411** | **−0.124** |
| Underlying | +13 | +1,300 | 0 | 0 | 0 |
| **Totals** | | **−297.4** | **−24.13** | **+0.2483** | **−0.759** |

**Figure 21-14 — A market maker's stock option book** (share price 68.76; interest rate 4.95%; dividend 0.58 in 10 weeks; April 4 weeks at 34.27% implied, June 13 weeks at 33.20%, August 21 weeks at 32.14%)

| Strike | April calls | April puts | June calls | June puts | August calls | August puts |
|---|---|---|---|---|---|---|
| 55 | +77 | +47 | −103 | +32 | | |
| 60 | −162 | +111 | +3 | +81 | +24 | −46 |
| 65 | | | +13 | −77 | +92 | −25 |
| 70 | +106 | −19 | −110 | −49 | −26 | −20 |
| 75 | −8 | +122 | +86 | −2 | +8 | −25 |
| 80 | | | −31 | −18 | +21 | +30 |
| 85 | −135 | +46 | −25 | +7 | | −72 |

| | Calls | Puts | Delta | Gamma | Theta | Adjusted vega | Rho |
|---|---|---|---|---|---|---|---|
| April | −140 | +212 | −12,615 | +504 | −2.51 | +6.26 | −7.50 |
| June | +67 | −91 | +6,093 | +13 | −0.62 | +0.36 | +9.75 |
| August | −36 | −59 | +3,425 | −252 | +1.25 | −11.19 | +10.45 |
| Shares | | | +3,300 | | | | |
| **Totals** | | | **+203** | **+265** | **−1.88** | **−4.57** | **+12.70** |

**Term-structure assumption used:** April is the primary month, mean volatility 30%, June moves at **75%** of April's rate, August at **50%**. Verify: April is 34.27% − 30.00% = 4.27% above the mean; June's 3.20% ≈ 0.75 × 4.27%; August's 2.14% ≈ 0.50 × 4.27%. A **20% increase** from current levels gives April 41.12%, June 38.34%, August 35.56%; a 20% decrease gives April 27.42%, June 28.06%, August 28.71%.

**Figure 21-20 — Risk sensitivities across underlying prices**

| Stock price | 45 | 50 | 55 | 60 | 65 | 70 | 75 | 80 | 85 | 90 | 95 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Theoretical P&L | +179 | +32 | −25 | −1 | +4 | +5 | +74 | +182 | **+221** | +131 | −76 |
| Total delta | −3,206 | −2,306 | −14 | +533 | −232 | +571 | +2,083 | +1,805 | −439 | −3,092 | −5,046 |
| Total gamma | +2 | +391 | +378 | −134 | −46 | +325 | +179 | −292 | −544 | −479 | −301 |
| Total adjusted vega | −0.94 | +0.86 | +0.81 | −3.97 | −4.01 | +0.11 | −2.09 | −9.68 | −14.84 | −14.27 | −10.70 |
| Total theta | +0.61 | −1.06 | −1.53 | +0.99 | +0.51 | −2.50 | −1.74 | +2.91 | +6.50 | +6.77 | +5.18 |
| Total rho | +19 | +19 | +18 | +17 | +15 | +12 | +10 | +7 | +3 | −1 | −4 |
| Total vanna | −5 | +28 | −98 | −115 | +34 | +16 | −131 | −149 | −30 | +74 | +100 |
| Total charm | −15 | −41 | +33 | +39 | −55 | −38 | +68 | +98 | +38 | −21 | −36 |
| Total speed | +45 | +77 | −90 | −63 | +81 | +34 | −82 | −84 | −14 | +32 | +34 |
| Total color | −8.00 | −4.47 | −16.60 | −15.53 | −12.74 | +18.10 | +17.62 | −5.65 | −14.77 | −7.36 | +0.54 |
| Total volga | −0.0004 | −0.0008 | −0.0023 | −0.0005 | 0 | −0.0015 | −0.0015 | 0 | +0.0002 | −0.0009 | −0.0017 |
| Total vega decay | +0.037 | +0.065 | −0.030 | +0.092 | +0.134 | −0.046 | −0.0117 | −0.035 | −0.052 | −0.179 | −0.247 |
| Total zomma | +10 | −8 | −28 | +24 | +21 | −26 | −22 | +15 | +27 | +13 | −1 |

**Higher-order measures, restated**

| Measure | Sensitivity of the… | To a change in… |
|---|---|---|
| Vanna | delta | volatility |
| Charm | delta | passage of time |
| Speed | gamma | underlying price |
| Color | gamma | passage of time |
| Volga | vega | volatility |
| Vega decay | vega | passage of time |
| Zomma | gamma | volatility |

**Figure 21-21 — The same book after a 2-for-1 split** (share price 34.38; dividend 0.29 in 10 weeks; strikes halved, positions doubled)

| Strike | April calls | April puts | June calls | June puts | August calls | August puts |
|---|---|---|---|---|---|---|
| 27½ | +154 | +94 | −206 | +64 | | |
| 30 | −324 | +222 | +6 | +162 | +48 | −92 |
| 32½ | | | +26 | −154 | +184 | −50 |
| 35 | +212 | −38 | −220 | −98 | −52 | −40 |
| 37½ | −16 | +122 | +172 | −4 | +16 | −50 |
| 40 | | | −62 | −36 | +42 | +60 |
| 42½ | −270 | +92 | −50 | +14 | | −144 |

| | Delta | Gamma | Theta | Adjusted vega | Rho |
|---|---|---|---|---|---|
| April | −25,230 | +2,016 | −2.51 | +6.26 | −7.50 |
| June | +12,186 | +52 | −0.62 | +0.36 | +9.75 |
| August | +6,850 | −1,008 | +1.25 | −11.19 | +10.45 |
| Shares | +6,600 | | | | |
| **Totals** | **+406** | **+1,060** | **−1.88** | **−4.57** | **+12.70** |

Delta and gamma changed; **theta, vega, and rho did not**.

## Worked Example

**A. Making a monster legible with synthetics.** Underlying at 71.50, position:
```
+29 underlying contracts
−44 March 65 calls,  +44 March 65 puts
 −7 March 70 calls,  +49 March 70 puts
−33 March 75 calls,  −51 March 75 puts
+30 March 80 calls,  +12 March 80 puts
```
Rewrite every put as a synthetic call (long put = long call + short underlying; short put = short call + long underlying):

| Contract | Underlying | 65 calls | 70 calls | 75 calls | 80 calls |
|---|---|---|---|---|---|
| Original underlying | +29 | | | | |
| +19 March 65 puts (net) | −19 | +19 | | | |
| Original 65 calls | | −19 | | | |
| +49 March 70 puts | −49 | | +49 | | |
| Original 70 calls | | | −7 | | |
| −51 March 75 puts | +51 | | | −51 | |
| Original 75 calls | | | | −33 | |
| +12 March 80 puts | −12 | | | | +12 |
| Original 80 calls | | | | | +30 |
| **Total** | **0** | **0** | **+42** | **−84** | **+42** |

The position is **+42 March 70 calls, −84 March 75 calls, +42 March 80 calls** — a plain **long butterfly**, body at 75. With the underlying at 71.50, below the body, the position must be **delta positive**. (Rewriting entirely in puts gives the same answer, since call and put butterflies are equivalent.)

**B. Reasoning through a position whose Greeks are all zero.** Long 10 September 95 puts, short 10 September 105 calls, long 5 underlying. Delta, gamma, theta, and vega all sum to **0**.

*If the underlying falls* toward 95 and away from 105, the 95 put's gamma **rises** while the 105 call's gamma **falls**. Long the put and short the call, the net gamma turns **positive** — so as the market falls, the delta turns **negative**.

*If the underlying rises* toward 105 and away from 95, the gammas reverse and the net gamma turns **negative** — so as the market rises, the delta again turns **negative**.

Delta negative in both directions — odd, until you see the gamma is positive going down and negative going up.

*If volatility rises*, all deltas move toward ±50. Suppose the 95 put's delta goes to −30 and the 105 call's to +30:
```
(10 × −30) − (10 × 30) + (5 × 100) = −100
```
*If volatility falls*, deltas move away from 50 — say to −20 and +20:
```
(10 × −20) − (10 × 20) + (5 × 100) = +100
```
So: **if volatility rises, we want the market to fall; if volatility falls, we want the market to rise.**

*As time passes*, deltas move away from 50, both options drift further out of the money, and the **five long underlying contracts dominate** — the delta turns positive.

*Net contract positions.* Downside: 10 long puts acting as short underlying plus 5 long underlying = **short 5**. Upside: 10 short calls acting as short underlying plus 5 long underlying = **short 5**. Consistent with the delta approaching **−500** in both directions.

**C. Estimating and then correcting the delta-neutral price.** The Figure 21-9 position has delta **−297.4** and gamma **−24.13**. A negative-gamma position maximizes its profit where it becomes delta neutral, so:
```
101.25 − (297.4 / 24.13) = 101.25 − 12.32 = 88.93
```
A slow decline to about **89.00** looks ideal. But the estimate assumes constant gamma. Plotting the actual P&L shows the maximum downside profit at about **95.00**, because the gamma grows far more negative as the market falls — reaching about **−80** at an underlying price of **93** — so the negative delta is offset much faster than the linear estimate suggests.

*Upside.* A swift rally hurts (delta and gamma both against). But the **upside contract position** is net short 7 calls plus long 13 underlying = **+6**: a truly large rally leaves the position **long 6 underlying contracts** and unlimited profit. Consistent with the graph — gamma turns positive at **104**, delta turns positive at **112**, and the position returns to profit above **124**.

*Downside.* The **downside contract position** is net long 5 puts plus long 13 underlying = **+8** — a violent collapse leaves the position **long 8 underlying contracts**, "with potentially disastrous results."

**D. Volatility margin for error.** Assume the Figure 21-9 position was established with **+6.00** of theoretical edge at 27% volatility, against a vega of **−0.759**:
```
27.00 + (6.00 / 0.759) = 27.00 + 7.90 = 34.90%
```
The position breaks even at about **34.90%** volatility — a margin of **7.90 volatility points**.

Improving it: raise the edge to 8.00 without raising the vega → `27.00 + (8.00/0.759) = 37.54%`. Or cut the vega to −0.65 without cutting the edge → `27.00 + (6.00/0.65) = 36.23%`. Often neither is possible, and the trader must simply decide whether −0.759 of vega is worth 6.00 of edge.

*Why the graph says 36% rather than 34.90%:* the six-week curve has a slightly **positive volga** — it curves upward, so vega becomes less negative as volatility rises. As time passes the volga turns slightly negative: the four-week graph is nearly straight and the two-week graph curves slightly **downward**.

**E. Delta at the extremes.** The Figure 21-9 position is net short 7 calls and net long 5 puts, with 13 long underlying.

*Volatility rises dramatically* — all deltas approach ±50:
```
calls: −7 × 50   = −350
puts:  +5 × −50  = −250
total: −350 − 250 + 1,300 = +700
```
**In a high-volatility market we prefer the market to rise.**

*Volatility falls or time passes* — deltas move to 0 or 100. Net short 2 in-the-money calls (90, 95, 100) and net long 20 in-the-money puts (105, 110):
```
−200 − 2,000 + 1,300 = −900
```
**In a low-volatility market, or as expiration nears, we prefer the market to fall.**

**F. Planning actions for the market maker's book.** From Figure 21-14 the book is delta +203, gamma +265, theta −1.88, adjusted vega −4.57, rho +12.70.

*Greatest threat:* a violent **upward** move. Above 85 the delta turns negative and losses grow; the **upside contract position is −76**. If the company is a takeover target, buying higher-strike calls is warranted **even at inflated prices** — "if a takeover could result in the trader's demise, this may be a price that he will have to pay."

*Secondary threat:* a moderate decline toward 62 combined with **rising** implied volatility, since the vega grows more negative there.

*Best single adjustment:* selling ~200 deltas — sell stock, sell calls, or **buy puts**. Buying **April 60 or 65 puts** is best: it reduces the delta *and* reduces the negative vega in the 60–65 range, and because short-term at-the-money options carry the largest gamma, those puts will most effectively offset the negative gamma if the stock does fall into that range.

*Favorable outcomes:* below 55 the delta turns negative (downside contract position **−29**), so a collapse helps; a climb toward 85 with falling implied volatility helps; almost any decline in implied volatility helps.

*Time decay:* the theta is small but **concentrated in April**, largely in a big long position in April 70 calls. As those near-the-money options approach expiration their theta accelerates. Partly offsetting: a market that stays near 70 will likely see implied volatility fall, which helps a negative-vega book.

*Dividend risk:* long 3,300 shares means the value moves by roughly **change in dividend × 3,300**. To remove it, replace the long stock with **synthetic long stock**.

**G. Stock splits, worked from first principles.** Stock at 60, long 1 May 60 call with delta 50 and gamma 5. After a 2-for-1 split:

| | Before | After |
|---|---|---|
| Stock price | 60 | 30 |
| Position | +1 May 60 call | +2 May 30 calls |
| Underlying contract | 100 shares | 100 shares |
| Delta position | +50 | **+100** |

*Gamma.* If the stock rises to 31 (equivalent to 62 pre-split), the pre-split delta would have been 50 + (2 × 5) = 60; the split doubles it to **120**. Delta moving from 100 to 120 over a one-point move means the new gamma is **+20**.

*A 3-for-2 split* (Y/X = 1.5) on the same call:
```
new stock price   = 60 × 2/3 = 40
new exercise price = 60 × 2/3 = 40
new option position = +1 × 3/2 = +1½ May 40 calls   ← not allowed
new delta position  = +50 × 3/2 = 75
new gamma position  = +5 × (3/2)² = +11.25
```
Fractional positions are prohibited, so the clearinghouse keeps **one May 40 call** and instead adjusts the underlying contract to `100 × 3/2 = **150 shares**`. Check: the at-the-money option should be worth about 50% of 150 = **75 shares** ✓. At a stock price of 41 (61½ pre-split) the old delta would have been 50 + (1.5 × 5) = **57.5**, so the new option is worth 0.575 × 150 = **86.25 shares** — exactly delta (75) + gamma (11.25) ✓.

## Key Takeaways
- Rewriting a position entirely in calls or entirely in puts can expose a familiar structure hiding inside a mess of contracts.
- Zero delta, gamma, theta, and vega today says nothing about tomorrow; project each Greek forward across price, volatility, and time.
- Read the P&L graph directly: slope is delta, curvature is gamma (smile = positive, frown = negative), vertical shifts are theta and vega.
- Always compute the upside and downside **net contract positions** — what the book becomes after a violent move.
- A negative-gamma position maximizes profit where it becomes delta neutral; estimate that price as current price − delta/gamma, then check it against the graph because gamma is not constant.
- Position implied volatility ≈ current volatility + edge/vega; widen the margin by raising edge or cutting vega.
- Diversify risk across strikes and expirations — a net-zero gamma hiding −2,388 at one strike is not a hedged book.
- Market making is "get an edge, control the risk," repeated; quotes can be skewed to steer inventory.
- A stock split changes delta and gamma by accounting, leaves theta, vega, and rho alone — but may come with a dividend change that does matter.

## Connects To
- [Chapter 7 — Risk Measurement I](ch07-risk-measurement-i.md) and [Chapter 9 — Risk Measurement II](ch09-risk-measurement-ii.md): the Greeks, first- through third-order, aggregated here.
- [Chapter 13 — Risk Considerations](ch13-risk-considerations.md): the same analytical method applied to individual spreads.
- [Chapter 14 — Synthetics](ch14-synthetics.md): the rewriting technique and synthetic stock.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): turning a conversion into a box, the model for removing stock risk.
- [Chapter 20 — Volatility Revisited](ch20-volatility-revisited.md): the term-structure assumptions needed to compute an adjusted vega.
- [Chapter 24 — Volatility Skews](ch24-volatility-skews.md): the distribution of implied volatility across strikes, assumed uniform here.
