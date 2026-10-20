# Chapter 9 — Risk Measurement II

## Core Idea
"Nothing remains constant." The Greeks are themselves sensitive to changes in the underlying price, volatility, and time — so today's small risk becomes tomorrow's big risk. This chapter maps how delta, gamma, theta, and vega each move, and names the **higher-order sensitivities** (vanna, charm, speed, color, volga, vega decay, zomma) that describe those movements.

## Frameworks Introduced

**The master principle:** *gamma, theta, and vega are all greatest when an option is at the money.* This is why at-the-money options are the most actively traded — they carry the characteristics traders come to an option market for.

**Time and volatility are interchangeable intuitions.** "If a trader cannot immediately determine the effect on an option's value or sensitivity of changing time, he might instead consider the effect of changing volatility. Conversely… Both effects are likely to be similar." More time, like higher volatility, increases the chance of large price changes.

**Delta convergence rule.** Raising volatility *or* time pushes all deltas **toward 50** (−50 for puts). Lowering volatility or letting time pass pushes deltas **away from 50** — toward 0 for OTM and 100 for ITM calls (0 and −100 for puts).

**Exercise-price proportionality.** For at-the-money options with otherwise identical terms:
- **Theta and vega are proportional to the exercise price.** A 1,000 call is worth exactly 100× a 10 call, and its theta is exactly 100× as large. An ATM option struck at 100 has twice the vega of one struck at 50.
- **Gamma is inversely proportional to the exercise price.** The gamma of a 50-strike option is twice that of a 100-strike option, because a one-point move at 50 is a 2% change while at 100 it is only 1%.

**Estimating the theta of an at-the-money option.** Theta is proportional to volatility, and volatility is proportional to √t, so the value one day on is `TV_{t−1} = TV_t × √((t−1)/t)` and therefore

```
theta ≈ TV_t × [1 − √((t−1)/t)]
```
The bracketed term grows as t shrinks, so at-the-money theta accelerates toward infinity at expiration.

**Lambda (Λ), a.k.a. elasticity, omega (Ω), or leverage value.**
```
Λ = Δ × (S / TV)
```
The percentage change in the option's value per percentage change in the underlying.

## Key Concepts
- **The implied delta.** Because the delta depends on the volatility input and the true future volatility is unknown, "no trader can be certain that a position is really delta neutral." Many traders use the delta produced by the **implied** volatility. That makes the delta shift with implied volatility even when nothing else moves: a trader long 40 calls at 32% implied with implied delta 25 each (1,000 deltas) hedges by selling 10 underlying contracts. If implied rises to 36% and the delta becomes 30, the position is (40 × 30) − (10 × 100) = **+200** — bullish, without a single market move.
- **The ATM call's delta is slightly above 50** even when exactly at the forward, because of the lognormal distribution; it rises slightly as volatility increases.
- **Delta neutrality decays.** A position delta neutral today may not be tomorrow even with nothing else changing — negligible with months to go, dramatic near expiration when one day is a large fraction of remaining life.
- **Vanna** — sensitivity of the delta to volatility (equivalently, of the vega to the underlying price; the two are mathematically identical). Greatest for call deltas near **20 and 80** and put deltas near **−20 and −80**; near zero at ±50. Vanna moves opposite to volatility (falls as volatility rises) and is **not** significantly affected by time.
- **Charm (delta decay)** — sensitivity of the delta to time. Same shape as vanna, greatest at deltas near ±20/±80, near zero at ±50. Rises as time to expiration shrinks; **not** significantly affected by volatility.
- **Theta shape.** Greatest at the money; declines as the option moves into or out of the money (deeply ITM and far OTM options have little time value to lose). At-the-money theta accelerates near expiration while ITM/OTM theta *slows*. Theta is directly proportional to volatility for ATM options — theta at 20% volatility is exactly double theta at 10% — but this linearity does not hold for ITM/OTM strikes, whose theta can reach 0 well before volatility does.
- **Higher exercise price decays faster.** With calls and puts equally out of the money, the lognormal distribution gives the **higher exercise price** (OTM call) more time premium than the lower (OTM put), so it must decay faster; the gap widens with volatility.
- **Vega shape.** Same shape as theta: greatest at the money, proportional to exercise price. The **vega of an at-the-money option is essentially constant across volatility levels** (its value line versus volatility is a straight line), while ITM and OTM vegas *rise* with volatility — because their deltas migrate toward 50 and they start behaving like at-the-money options. Vega rises with more time to expiration; long-term options are always more volatility-sensitive than short-term ones.
- **Volga (vomma)** — sensitivity of the vega to volatility; a contraction of "volatility gamma." Near zero at ±50 delta; maximal near call deltas **10 and 90**, put deltas **−10 and −90**. Long-term options have larger volga than short-term ones.
- **Vega decay (DvegaDtime)** — sensitivity of the vega to time. Greatest for deltas between 10 and 90 and increasing as expiration nears; short-term vegas change faster than long-term ones.
- **Gamma shape.** Greatest at the money. The **gamma of an at-the-money option rises as time passes or volatility falls**, and falls as volatility rises — the reverse of the ITM/OTM case.
- **Speed** — sensitivity of the gamma to the underlying price. Greatest for calls near delta 15 and 85 (puts −15, −85); smallest at ±50 and at the extremes. Speed rises with less time or lower volatility.
- **Color (gamma decay)** — sensitivity of the gamma to time. Large and **negative** for at-the-money options (gamma grows as time passes); large and **positive** for deltas near ±5/±95. Near zero for deltas near ±15/±85. Reducing time or volatility increases color.
- **Zomma** — sensitivity of the gamma to volatility. Same profile as color: large and **negative** at the money, large and **positive** near ±5/±95 deltas, near zero at ±15/±85. Reducing time or volatility increases zomma.
- **Lambda characteristics.** Calls have positive, puts negative lambda. Greatest for **out-of-the-money** options; falls for calls and rises (more negative) for puts as the underlying rises. Raising volatility lowers lambda for both; lowering volatility or letting time pass raises it.
- **Order of sensitivities.** Gamma, vanna, and charm are **second-order** (sensitivities of the delta). Speed, color, and zomma are **third-order**.

## Mental Models
- **"At-the-money options close to expiration in a low-volatility environment are among the riskiest options that one can trade."** They start with deltas near 50, but those deltas can swing to 0 or 100 on a small move — the gamma is enormous.
- **Maximum leverage recipe.** To maximize percentage return relative to the underlying: **out-of-the-money options, close to expiration, in a low-volatility environment**. True only in theory — the bid-ask spread and option-market liquidity may make a large lambda position impractical.

## Anti-patterns
- **Believing a delta-neutral position is genuinely neutral.** The delta rests on a volatility guess.
- **Ignoring that implied-volatility moves alone can put you directionally long or short.**
- **Assuming a small gamma position will stay small.** As the underlying approaches a strike, or as expiration nears, gamma balloons.
- **Extrapolating ATM linearity to other strikes.** Theta is linear in volatility only at the money; vega is constant in volatility only at the money.
- **Over-analysis.** The counterweight from Chapter 7 still applies — but Natenberg's stance here is that "it is almost impossible to overemphasize the importance of risk management in option trading."

## Reference Tables

**Figure 9-25 — Traditional risk measures**
(C = call theoretical value; P = put theoretical value; S = underlying/spot price; t = time to expiration; σ = annual volatility; r = domestic interest rate; r_f = foreign interest rate)

| Risk | Sensitivity of the… | To a change in… | Math | Maximized when |
|---|---|---|---|---|
| **Delta (Δ)** | Theoretical value (points) | Underlying price (points) | ∂C/∂S ≈ ∂P/∂S + 1 | Deeply in the money |
| **Lambda (Λ) / omega (Ω) / elasticity** | Theoretical value (percent) | Underlying price (percent) | Δ_c × (S/C); Δ_p × (S/P) | Out of the money; close to expiration; low volatility |
| **Gamma (Γ) / curvature** | Delta | Underlying price | ∂²C/∂S² = ∂²P/∂S² = ∂Δ/∂S | At the money; close to expiration; low volatility |
| **Theta (Θ) / time decay** | Theoretical value | Time to expiration | ∂C/∂t; ∂P/∂t | At the money; close to expiration; low volatility |
| **Vega** | Theoretical value | Volatility | ∂C/∂σ = ∂P/∂σ | At the money; long term |
| **Rho (Ρ)** | Theoretical value | Interest rate | ∂C/∂r; ∂P/∂r | Deeply in the money; long term |
| **Rho_f / phi (Φ)** | Theoretical value | Foreign interest rate or dividend yield | ∂C/∂r_f; ∂P/∂r_f | Deeply in the money; long term |

**Figure 9-26 — Nontraditional higher-order risk measures**

| Risk | Sensitivity of the… | To a change in… | Math | Maximized when |
|---|---|---|---|---|
| **Vanna** | Delta / Vega | Volatility / Underlying price | ∂²C/∂S∂σ; ∂²P/∂S∂σ | 15–20 and 80–85 delta; low volatility |
| **Charm (delta decay)** | Delta / Theta | Time / Underlying price | ∂²C/∂S∂t; ∂²P/∂S∂t | 15–20 and 80–85 delta; close to expiration |
| **Speed** | Gamma | Underlying price | ∂³C/∂S³ = ∂Γ/∂S | 15–20 and 80–85 delta; low volatility; close to expiration |
| **Color (gamma decay)** | Gamma / Charm | Time to expiration / Underlying price | ∂³C/∂S²∂t = ∂Γ/∂t | At the money; close to expiration; low volatility |
| **Volga (vomma)** | Vega | Volatility | ∂²C/∂σ² = ∂²P/∂σ² | 10 and 90 delta; long term; low volatility |
| **Vega decay (DvegaDtime)** | Vega | Time | ∂²C/∂σ∂t; ∂²P/∂σ∂t | 20 and 80 delta; close to expiration |
| **Zomma** | Gamma / Vanna | Volatility / Underlying price | ∂³C/∂S²∂σ = ∂Γ/∂σ | At the money; close to expiration; low volatility |

**How each Greek responds to rising volatility and to the passage of time**

| Measure | Rising volatility | Passage of time |
|---|---|---|
| Delta (OTM) | Rises toward 50 | Falls toward 0 |
| Delta (ITM) | Falls toward 50 | Rises toward 100 |
| Gamma (ATM) | Falls | Rises |
| Gamma (ITM/OTM) | Rises | Falls |
| Theta (ATM) | Rises proportionally | Accelerates (→ ∞ at expiration) |
| Theta (ITM/OTM) | Rises, may hit 0 before σ = 0 | Slows |
| Vega (ATM) | Roughly constant (declines very slightly) | Falls |
| Vega (ITM/OTM) | Rises | Falls |
| Lambda (calls and puts) | Falls | Rises |

## Worked Example

**A. Estimating and re-estimating an at-the-money theta.** An at-the-money option is worth **2.50** with **30 days** to expiration.

```
theta ≈ 2.50 × [1 − √(29/30)] = 2.50 × (1 − 0.9832) ≈ 0.042
```
One day later the option is worth 2.50 − 0.042 = 2.458 with 29 days left:
```
theta ≈ 2.458 × [1 − √(28/29)] = 2.458 × (1 − 0.9826) ≈ 0.043
```
The theta has grown even though the option has lost value — the √t relationship at work.

**B. Why gamma explodes at the money near expiration.** Take a **100 call** with the underlying at **97.50**, so the option is out of the money and its delta is below 50.

*Case 1 — close to expiration or very low volatility.* Deltas move **away** from 50, so the delta might be **25**. If the underlying rises 5 points to 102.50, the option becomes in the money and its delta might jump to **75**. Approximate gamma:
```
(75 − 25) / (102.50 − 97.50) = 50 / 5 = 10
```

*Case 2 — far from expiration or high volatility.* Deltas stay **near** 50. At 97.50 the delta might be **45**; at 102.50 it might be only **55**. Approximate gamma:
```
(55 − 45) / (102.50 − 97.50) = 10 / 5 = 2
```
The same option, same strike, same 5-point move: **gamma of 10 versus gamma of 2**. This is the arithmetic behind "at-the-money options close to expiration in a low-volatility environment are among the riskiest options that one can trade."

**C. Computing a lambda.** A call has theoretical value **4.00** and delta **20**, with the underlying at **100**. A one-point rise in the underlying (ignoring gamma) makes the option worth approximately **4.20**. In percentage terms the underlying moved 1/100 = **1%** while the option moved 0.20/4.00 = **5%**.
```
Λ = Δ × (S / TV) = 0.20 × (100 / 4.00) = 5
```
The option changes at five times the underlying's percentage rate.

**D. Implied delta drift.** Long 40 calls at 32% implied volatility, implied delta 25 → 40 × 25 = **1,000 deltas**, hedged by selling 10 underlying contracts (−1,000). Implied volatility rises to 36%; the delta migrates toward 50 and becomes **30**. Position delta = (40 × 30) − (10 × 100) = **+200**. A neutral position became bullish with no change in the underlying, no passage of time, and no trade.

## Key Takeaways
- Gamma, theta, and vega all peak at the money — the reason ATM options dominate volume.
- Volatility up or time up → deltas toward 50; volatility down or time passing → deltas toward 0/100.
- ATM theta and vega scale *with* the exercise price; ATM gamma scales *inversely* with it.
- ATM gamma and theta rise as expiration nears; ITM/OTM gamma and theta fall.
- ATM vega is nearly constant in volatility; ITM/OTM vega rises with volatility.
- The delta depends on an unknowable volatility, so delta neutrality is always provisional — many traders use the implied delta and accept that it moves with implied volatility.
- Second- and third-order Greeks (vanna, charm, speed, color, volga, vega decay, zomma) each peak at characteristic delta bands: ±20/±80 for vanna and charm, ±15/±85 for speed, at-the-money and ±5/±95 for color and zomma, ±10/±90 for volga.
- Maximum percentage leverage: out-of-the-money, near expiration, low volatility.

## Connects To
- [Chapter 7 — Risk Measurement I](ch07-risk-measurement-i.md): the first-order Greeks these are derivatives of.
- [Chapter 6 — Volatility](ch06-volatility.md): the √t scaling that drives the theta approximation and the ATM/OTM sensitivity principles.
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): why a delta computed from a guessed volatility makes hedging approximate.
- [Chapter 13 — Risk Considerations](ch13-risk-considerations.md): applying these behaviors to strategy selection.
- [Chapter 18 — The Black-Scholes Model](ch18-black-scholes-model.md): the analytic forms, including why ATM vega declines very slightly with volatility.
- [Chapter 21 — Position Analysis](ch21-position-analysis.md): reading these measures across a whole book of options.
