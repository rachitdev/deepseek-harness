# Chapter 18 — The Black-Scholes Model

## Core Idea
Black and Scholes did not start by asking what an option is worth. They asked: **"if the stock price moves randomly over time, but in a manner that is consistent with a constant interest rate and volatility, what must be the option price after each moment in time such that an option position that is correctly hedged will just break even?"** The answer is a differential equation whose solution is the pricing formula. Read the equation and you find nothing but the Greeks you already know.

## Frameworks Introduced

**The Black-Scholes differential equation:**
```
       ∂C     1        ∂²C     ∂C
  rS ─────  +  ─ σ²S² ─────  + ────  =  rC
       ∂S     2        ∂S²      ∂t
```
Only **S and t** are variables; σ and r are inputs assumed constant over the option's life — exactly the assumption made in the Chapter 8 dynamic hedging examples.

| Term | What it is |
|---|---|
| ∂C/∂S | **delta (Δ)** |
| ∂²C/∂S² | **gamma (Γ)** |
| ∂C/∂t | **theta (Θ)** |
| rS | the **spot-to-forward** adjustment (carry on the underlying) |
| rC | the **expected-value-to-present-value** discount |
| ½σ²S²(∂²C/∂S²) = ½σ²S²Γ | the volatility component acting on the gamma |

Compare with the average-delta approximation from Chapter 7 — `(S₁ − S₂)Δ + ½(S₁ − S₂)²Γ` — versus `rS(∂C/∂S) + ½σ²S²(∂²C/∂S²)`. Same structure, with an interest component on S and a volatility component on gamma; the difference is that Black-Scholes assumes an **infinitesimally small (instantaneous)** price change rather than a discrete one.

**The solution:**
```
C = S·N(d₁) − X·e^(−rt)·N(d₂)

      ln(S/X) + [r + σ²/2]t
d₁ = ───────────────────────        d₂ = d₁ − σ√t
             σ√t
```

**The two questions the model asks** — Black-Scholes separates the underlying price and exercise price rather than combining them into (Sᵢ − X):
1. If held to expiration, **what is the average value of all the stock above the exercise price?** → `S·e^(rt)·N(d₁)`
2. If held to expiration, **what is the likelihood the owner of the option will end up paying the exercise price?** → `X·N(d₂)`

Expected value = `S·e^(rt)N(d₁) − X·N(d₂)`. Discount by e^(−rt) to get the familiar form:
```
C = [S·e^(rt)N(d₁) − X·N(d₂)] · e^(−rt) = S·N(d₁) − X·e^(−rt)·N(d₂)
```

**Anatomy of d₁** (Figure 18-5), reading numerator left to right:
- `ln(S/X)` — the relationship between underlying and exercise price in a *lognormal* world (positive = in the money for a call, negative = out of the money). In a normal distribution this would simply be S − X.
- `+ rt` — the **spot-to-forward adjustment** (carry on the underlying). Equivalently, drop rt and replace S with Se^(rt): `ln(S/X) + rt = ln(Se^(rt)/X)`.
- `+ σ²t/2` — the **adjustment for the mean of a lognormal distribution**. A lognormal distribution's elongated right tail puts the mean to the right of the mode; the higher the standard deviation, the longer the tail and the further the shift.
- Divided by `σ√t` — the **normalization factor**, one standard deviation, converting the whole thing into a number of standard deviations.

**d₂ uses the median instead of the mean.** In a lognormal distribution the median lies to the *left* of the mean by exactly σ√t, hence `d₂ = d₁ − σ√t`. `N(d₂)` is the true probability the option is exercised.

**The generalized model with the carry term b:**
```
C = S·e^((b−r)t)·N(d₁) − X·e^(−rt)·N(d₂)
P = X·e^(−rt)·N(−d₂) − S·e^((b−r)t)·N(−d₁)

      ln(S/X) + [b + σ²/2]t
d₁ = ───────────────────────        d₂ = d₁ − σ√t
             σ√t
```

| b | Model |
|---|---|
| **b = r** | Black-Scholes for options on **stock** |
| **b = r = 0** | Options on **futures** subject to **futures-type** settlement |
| **b = 0** | The **Black model** — options on futures subject to **stock-type** settlement |
| **b = r − r_f** | The **Garman-Kohlhagen model** — options on **foreign currencies** |

For a dividend-paying stock, approximate with **b = r − q** (q = annual dividend yield in percent), or more exactly replace S with `S − ΣDᵢe^(r·t_d)`, where t_d is the time from each dividend payment to expiration.

## The Greeks in closed form (Figure 18-6)

| | Call | Put |
|---|---|---|
| **Delta (Δ)** | e^((b−r)t)·N(d₁) | e^((b−r)t)·[N(d₁) − 1] |
| **Gamma (Γ)** | e^((b−r)t)·n(d₁) / (Sσ√t) | same as call |
| **Theta (Θ)*** | −Se^((b−r)t)n(d₁)σ / (2√t) + (b−r)Se^((b−r)t)N(d₁) − rXe^(−rt)N(d₂) | −Se^((b−r)t)n(d₁)σ / (2√t) + (b−r)Se^((b−r)t)N(d₁) + rXe^(−rt)N(−d₂) |
| **Vega**** | Se^((b−r)t)·n(d₁)·√t | same as call |
| **Rho (Ρ)**** | tXe^(−rt)N(d₂) if b ≠ 0; −tC if b = 0 | −tXe^(−rt)N(−d₂) if b ≠ 0; −tP if b = 0 |
| **Rho_f / Phi (Φ)** | −tSe^((b−r)t)N(d₁) | tSe^((b−r)t)N(−d₁) |
| **Elasticity (Λ)** | Δ_C(S/C) | Δ_P(S/P) |
| **Vanna** | −e^((b−r)t)n(d₁)·(d₂/σ) | same as call |
| **Charm** | −e^((b−r)t)n(d₁)·[b/(σ√t) − d₂/(2t)] + (b−r)N(d₁) | −e^((b−r)t)n(d₁)·[b/(σ√t) − d₂/(2t)] − (b−r)N(d₁) |
| **Speed** | −(Γ/S)·[1 + d₁/(σ√t)] | same as call |
| **Color** | Γ·{r − b + bd₁/(σ√t) + (1 − d₁d₂)/(2t)} | same as call |
| **Volga (vomma)** | vega·(d₁d₂/σ) | same as call |
| **Vega decay** | vega·{r − b + bd₁/(σ√t) − (1 − d₁d₂)/(2t)} | same as call |
| **Zomma** | Γ·[(d₁d₂ − 1)/σ] | same as call |

\* Theta gives sensitivity to the passage of **one year**; divide by **365** for daily decay.
\*\* Vega and rho give sensitivity to a **one full point (100%)** change; divide by **100** for a one-percentage-point change.

(A complete listing appears in Espen Gaarder Haug, *The Complete Guide to Option Pricing Formulas*, 2nd ed., 2007, McGraw-Hill.)

## Key Concepts
- **n(x) vs. N(x).** `n(x)` is the **standard normal distribution** — mean 0, standard deviation 1, total area under the curve exactly 1. `N(x)` is the **cumulative** function: the area between −∞ and x, i.e. the probability of an occurrence less than x. Therefore N(+∞) = 1, N(−∞) = 0, N(0) = 0.50, and by symmetry **N(x) = 1 − N(−x)**.
- **The model uses normal probabilities for a lognormal world** by adjusting the value of x — that is what the ln(S/X), rt, and σ²t/2 terms in d₁ accomplish.
- **Mode, mean, median.** *Mode* = the peak, where the most occurrences take place. *Mean* = the balance point, where half the **value** falls on each side. *Median* = where half the **occurrences** fall on each side. In a perfect normal distribution all three coincide. In the lognormal-ish distribution of Figure 18-4 the mode is ≈**9.3**, the median ≈**10.5**, and the mean ≈**12.7**.
- **The delta is N(d₁), not the probability of finishing in the money.** The true probability is **N(d₂)**. N(d₁) and N(d₂) are often close, especially for short-dated options, but **N(d₁) is always larger**.
- **An at-the-forward call has a delta greater than 50** — slightly. Since put delta = call delta − 100, the put's delta is less than −50 in absolute value, so **an at-the-forward straddle has a positive delta**.
- **The exactly delta-neutral straddle price.** d₁ = 0 requires `ln(S/X) + [r + σ²/2]t = 0`, so `S = X·e^(−[r + σ²/2]t)`. The delta-neutral forward price is **below** the exercise price, and falls further below it as time or volatility increase (the call goes further out of the money, the put further in). At a zero interest rate with a 100 strike, the neutral price is near 100 at very low volatilities but well below 100 at high volatility and long maturities.
- **The three components of theta.** (1) **Decay of volatility value** — as time passes the distribution of possible expiration prices narrows: `−Se^((b−r)t)n(d₁)σ / (2√t)`. (2) **Spot moving toward the forward**: `(b − r)Se^((b−r)t)N(d₁)`. (3) **Present value of the expected value changing**: `rXe^(−rt)N(d₂)`. Put-call parity forces component (1) to have the **same sign for calls and puts**; components (2) and (3) may be either sign. The volatility decay almost always dominates. When r = 0 or options on futures are futures-type settled, components (2) and (3) vanish, leaving the **driftless theta**.
- **The true locations of maximum gamma, theta, and vega** (they are *not* exactly at the money):

| Quantity | Critical underlying price |
|---|---|
| Delta of 50 | S = X·e^(−[b − σ²/2]t) |
| Maximum gamma | S = X·e^(−[b − 3σ²/2]t) |
| Maximum theta | S = X·e^([b + σ²/2]t) |
| Maximum vega | S = X·e^(−[b + σ²/2]t) |

  With **b = 0**, maximum gamma and theta occur **above** the exercise price (and at the same underlying price), while maximum vega occurs **below** it. Raising interest rates (b > 0) lowers the price at which maximum gamma and vega occur and raises the price for maximum theta. Equivalently: maximum gamma is where **speed = 0**, maximum theta where **charm = 0**, maximum vega where **vanna = 0**.
- **Vega does not always rise with time.** The rule "long-term options are more volatility-sensitive" holds only when the underlying price equals the forward price, as for futures options. For **stock** options the forward rises with both time and interest rates, moving the option away from at-the-forward — and since at-the-forward options have the highest vega, **a stock option's vega can decline as time to expiration increases**. With stock 100, strike 100, volatility 20%: at r = 0 vega always rises with time; at **r = 10%** it starts declining beyond **33 months**; at **r = 20%**, beyond **10 months**. The vega decay turns negative at exactly those points.

## Anti-patterns
- **Treating the delta as the probability of finishing in the money.** That is N(d₂); the delta is N(d₁), always larger.
- **Assuming an at-the-money straddle is delta neutral.** It has a positive delta; neutrality requires a forward price *below* the strike.
- **Assuming gamma, theta, and vega peak exactly at the money.** They peak at three different, computable prices.
- **Assuming vega always increases with time** — false for stock options at meaningful interest rates.
- **Applying the at-the-forward approximation to long-dated, high-volatility options** without expecting it to overstate value.

## Reference Tables

**Figure 18-1 — A discrete lognormal-like distribution** (153 total occurrences, troughs 0–27)

| Trough | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Occurrences | 0 | 1 | 1 | 2 | 4 | 6 | 8 | 10 | 12 | 13 | 13 | 12 | 11 | 11 |

| Trough | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Occurrences | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 2 | 1 | 1 | 1 | 0 | 0 |

**Value of occurrences above the 12½ strike**

| Trough | Occurrences | Stock value |
|---|---|---|
| 13 | 11 | 143 |
| 14 | 9 | 126 |
| 15 | 8 | 120 |
| 16 | 7 | 113 |
| 17 | 6 | 102 |
| 18 | 5 | 90 |
| 19 | 4 | 76 |
| 20 | 3 | 60 |
| 21 | 2 | 42 |
| 22 | 2 | 44 |
| 23 | 1 | 23 |
| 24 | 1 | 24 |
| 25 | 1 | 25 |
| 26 | 0 | 0 |
| 27 | 0 | 0 |
| **Total** | **60** | **987** |

**The direct expected-value check on the same distribution**

| Trough | Intrinsic value of the 12½ call | Occurrences | Probability | Option value |
|---|---|---|---|---|
| 13 | 0.5 | 11 | 0.0719 | 0.0359 |
| 14 | 1.5 | 9 | 0.0588 | 0.0882 |
| 15 | 2.5 | 8 | 0.0523 | 0.1307 |
| 16 | 3.5 | 7 | 0.0458 | 0.1601 |
| 17 | 4.5 | 6 | 0.0392 | 0.1765 |
| 18 | 5.5 | 5 | 0.0327 | 0.1797 |
| 19 | 6.5 | 4 | 0.0261 | 0.1699 |
| 20 | 7.5 | 3 | 0.0196 | 0.1471 |
| 21 | 8.5 | 2 | 0.0131 | 0.1111 |
| 22 | 9.5 | 2 | 0.0131 | 0.1242 |
| 23 | 10.5 | 1 | 0.0065 | 0.0686 |
| 24 | 11.5 | 1 | 0.0065 | 0.0752 |
| 25 | 12.5 | 1 | 0.0065 | 0.0817 |
| **Total** | | | | **1.5489** |

**Critical times at which a stock option's vega begins to decline** (stock 100, strike 100, volatility 20%)

| Interest rate | Vega decay turns negative beyond |
|---|---|
| 0% | never |
| 10% | 33 months |
| 20% | 10 months |

## Worked Example

**A. Valuing a call the Black-Scholes way, on a discrete distribution.** Using Figure 18-1's 153 occurrences and a 12½ strike:

*Question 1 — the average value of all stock above the exercise price.* Sum the value of troughs 13 through 27: **987** across **60** occurrences. But average over **all** 153 occurrences:
```
987 / 153 = 6.45
```

*Question 2 — the likelihood of paying the exercise price.* 60 of 153 occurrences are in the money:
```
60 / 153 = 0.392
```
so the average payout is `0.392 × 12½ = 4.90`.

*Expected value* = `S·e^(rt)·N(d₁) − X·N(d₂)` = 6.45 − 4.90 = **1.55**.

*Cross-check the old way* — sum intrinsic value × probability across every trough — and the answer is **1.5489**, the same to rounding. The only difference between this exercise and the real model is that Black-Scholes uses a **continuous lognormal** distribution rather than 153 discrete outcomes.

**B. Deriving the at-the-forward approximation (the "40% rule").** Set an option exactly at the money (X = S), one year to expiration (t = 1), zero interest (r = 0), and volatility of **1 percent** (σ = 0.01). Then ln(S/X) = 0 and σ√t = 0.01, so:
```
d₁ = (0.01²/2) / 0.01 = 0.005
d₂ = (−0.01²/2) / 0.01 = −0.005
N(d₁) = 0.501995      N(d₂) = 0.498005
```
With r = 0 the call is worth `(S × 0.501995) − (X × 0.498005)`, and since X = S:
```
X × (0.501995 − 0.498005) = X × 0.003990
```
**Each percentage point of volatility is worth 0.00399 × the exercise price**, per year. That value does not change as volatility rises because — from Chapter 9 — an at-the-money option's **vega is nearly constant in volatility**.

Applications:
- 100 call, one year, 20% volatility: 20 × 100 × 0.00399 = **7.98**
- 100 call, one year, 35% volatility: 35 × 100 × 0.00399 = **13.965**
- 50 call, one year, 20%: 20 × 50 × 0.00399 = **3.99** (value is proportional to the exercise price)
- 125 call, one year, 20%: 20 × 125 × 0.00399 = **9.975**
- 100 call, **six months**, 20%: since time value scales with √t, 7.98 × √0.5 = 7.98 × 0.707 = **5.64**

General form:
```
expected value   ≈ X × (σ × 100) × √t × 0.00399
theoretical value ≈ [X × (σ × 100) × √t × 0.00399] / (1 + r·t)
```
(applies to both calls and puts, since put-call parity makes an exactly at-the-forward call and put equal in value).

*Worked instance.* Volatility 18%, three months (t = ¼), exercise price 65:
```
65 × 18 × √(1/4) × 0.00399 = 65 × 18 × 0.5 × 0.00399 ≈ 2.33
```
At 4% interest, theoretical value ≈ 2.33 / (1 + 0.04/4) = 2.33 / 1.01 ≈ **2.31**.

*Where it breaks down.* The approximation **overstates** value as time and volatility grow, because at-the-money vega declines slightly with volatility and that decline is magnified with time. At 40% volatility and two years:
```
approximation: 65 × 40 × √2 × 0.00399 = 65 × 40 × 1.414 × 0.00399 ≈ 14.67
actual Black-Scholes expected value:                                   14.48
```

*Where 0.00399 comes from.* The peak of the standard normal distribution is **0.398942 ≈ 0.399**. A volatility of 1 percent is 1/100 of a standard deviation, so 0.399/100 = **0.00399**. Rounding it to **0.004** gives the **40% rule**: the expected value of an at-the-forward option is about **40% of one standard deviation**, where one standard deviation is `F × σ√t`.

## Key Takeaways
- The Black-Scholes equation is a statement about delta, gamma, and theta plus a carry term and a discount term — nothing more exotic.
- The model splits the payoff into "average value of stock above the strike" (Se^(rt)N(d₁)) minus "probability × exercise price" (XN(d₂)), then present-values the difference.
- d₁ is ln(S/X), shifted for carry (rt or bt) and for the lognormal mean (σ²t/2), then normalized by σ√t. d₂ = d₁ − σ√t uses the median instead of the mean.
- Delta = N(d₁); the probability of exercise = N(d₂); N(d₁) > N(d₂) always.
- One model, four variants, distinguished only by b: r (stock), 0 (Black, futures with stock-type settlement), r = 0 (futures-type settlement), r − r_f (Garman-Kohlhagen).
- At-the-forward value ≈ X × (σ × 100) × √t × 0.00399 — the 40% rule — accurate for short-dated, moderate-volatility options.
- An at-the-forward straddle is not delta neutral; neutrality requires S = X·e^(−[r + σ²/2]t).
- Maximum gamma, theta, and vega each occur at their own computable underlying price, not exactly at the money.
- A stock option's vega can *fall* with more time to expiration when interest rates are high enough.

## Connects To
- [Chapter 5 — Theoretical Pricing Models](ch05-theoretical-pricing-models.md): the expected-value framework the model formalizes, and the model's history.
- [Chapter 6 — Volatility](ch06-volatility.md): normal vs. lognormal distributions and the √t scaling.
- [Chapter 7 — Risk Measurement I](ch07-risk-measurement-i.md): the average-delta approximation that the differential equation generalizes.
- [Chapter 9 — Risk Measurement II](ch09-risk-measurement-ii.md): every higher-order Greek listed here in closed form, and the near-constancy of at-the-money vega.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): put-call parity, the starting point for interpreting the formula.
- [Chapter 16 — Early Exercise of American Options](ch16-early-exercise-american-options.md): what the European model cannot do.
- [Chapter 19 — Binomial Option Pricing](ch19-binomial-option-pricing.md): the discrete alternative.
- [Chapter 23 — Models and the Real World](ch23-models-and-the-real-world.md): where the assumptions fail.
