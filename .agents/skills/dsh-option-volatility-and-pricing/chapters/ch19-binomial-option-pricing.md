# Chapter 19 — Binomial Option Pricing

## Core Idea
In the late 1970s John Cox (MIT), Stephen Ross (Yale), and Mark Rubinstein (UC Berkeley) set out to teach option pricing without advanced mathematics. The result — **binomial option pricing** (Cox-Ross-Rubinstein, 1979) — is not only easier to understand than Black-Scholes but can price options Black-Scholes cannot: **American options**. The whole model rests on one question: what probabilities of up and down moves would make an investor **indifferent** between buying and selling the underlying? Those are the **risk-neutral probabilities**, and everything else follows.

## Frameworks Introduced

**Risk-neutral probabilities.** A security at 100 can go to 120 or 90. Instinct says buy — up 20, down only 10 — but that assumes 50/50. Solve instead for the p that makes expected value equal the current price:
```
p × 120 + (1 − p) × 90 = 100  →  30p = 10  →  p = ⅓
Check: ⅓ × 120 + ⅔ × 90 = 40 + 60 = 100 ✓
```
Generalizing with multipliers u and d on a one-period tree:
```
p·Su + (1 − p)·Sd = S    →    p = (1 − d)/(u − d)
```
For a non-dividend-paying stock the expected value must equal the **forward price** S(1 + r·t), giving:
```
p = [(1 + r·t) − d] / (u − d)      and over n periods:  p = [(1 + r·t/n) − d] / (u − d)
```

**Valuing an option on a one-period tree:**
```
Call = { p·max[Su − X, 0] + (1 − p)·max[Sd − X, 0] } / (1 + r·t)
Put  = { p·max[X − Su, 0] + (1 − p)·max[X − Sd, 0] } / (1 + r·t)
```

**Two periods**, with `d = 1/u` so that `ud = du = 1` (a **recombining**, driftless tree). Three terminal prices — Suu, Sud, Sdd — but **two paths** lead to the middle one:
```
Call = { p²·max[Suu − X,0] + 2p(1−p)·max[Sud − X,0] + (1−p)²·max[Sdd − X,0] } / (1 + r·t/2)²
```

**n periods.** Terminal prices are `S·u^j·d^(n−j)` for j = 0…n, and the number of paths to each is the **binomial expansion** `n! / [j!(n − j)!]`:
```
            1        n     n!
Call = ───────────── Σ  ───────────── · p^j(1−p)^(n−j) · max[S·u^j·d^(n−j) − X, 0]
       (1 + rt/n)^n j=0  j!(n−j)!

            1        n     n!
Put  = ───────────── Σ  ───────────── · p^j(1−p)^(n−j) · max[X − S·u^j·d^(n−j), 0]
       (1 + rt/n)^n j=0  j!(n−j)!
```

**Choosing u and d.** They must be (a) **recombining** — otherwise every node spawns a fresh tree and the calculations explode — and (b) **multiplicative inverses** so `u × d = 1` and there is no drift. (Choosing u = 1.25 and d = 0.75 would give ud = 0.9375, a downward drift, making theta uncomputable.) To make the terminal prices approximate a lognormal distribution, define u and d as a **one standard deviation price change per period**:
```
u = e^(σ√(t/n))        d = 1/u = e^(−σ√(t/n))
```

**Binomial notation.** Each node is `S_{i,j}` where i locates the node left to right and j bottom to top. Fill the tree with option values `C_{i,j}` or `P_{i,j}` instead, set the terminal values to intrinsic value, and **work backwards**:
```
C_{i,j} = [ p·C_{i+1,j+1} + (1 − p)·C_{i+1,j} ] / (1 + r·t/n)
P_{i,j} = [ p·P_{i+1,j+1} + (1 − p)·P_{i+1,j} ] / (1 + r·t/n)
```

**The Greeks from the tree:**
```
Δ_{i,j} = (C_{i+1,j+1} − C_{i+1,j}) / (S_{i+1,j+1} − S_{i+1,j})
Γ_{i,j} = (Δ_{i+1,j+1} − Δ_{i+1,j}) / (S_{i+1,j+1} − S_{i+1,j})
daily theta ≈ (C_{i,j} − C_{i+2,j+1}) / (t × 365/n)
annual theta θ_{i,j} = (C_{i+1,i+2} − C_{i,j}) / t
```
**Vega and rho have no simple tree arithmetic** — change the volatility or interest-rate input and rebuild.

**Pricing American options.** At every node, compare the European value with intrinsic value. **If intrinsic value is greater, replace the node value with intrinsic value**, then continue working backwards. This changes not only the price but the delta and gamma too.

**Model variants** (Figure 19-13), determined by r and b:

| Condition | Model |
|---|---|
| b = r > 0 | Options on **stock** |
| b = r = 0 | Options on **futures**, futures-type settlement |
| b = 0 and r > 0 | Options on **futures**, stock-type settlement |
| b = r − r_f | Options on **foreign currencies** |

with `p = [(1 + b·t/n) − d] / (u − d)`.

## Key Concepts
- **Trees are not symmetric when drawn to scale.** In the three-period example: 115.76 − 105.00 = 10.76, 105.00 − 95.24 = 9.76, 95.24 − 86.38 = 8.86. The branches **narrow** going down.
- **Why compute the intermediate values at all?** Because they yield the risk sensitivities *and* they are the only way to price an American option.
- **Dynamic hedging reappears inside the model.** Buy the 100 call at its theoretical value of 5.22, sell 62% of a stock contract at 100. Either branch produces a loss of about **0.56–0.57** on the hedge — but the trade created a **credit** of `−5.22 + 0.62 × 100 = +56.78`, which at 1% over the period earns `0.01 × 56.78 ≈ +0.57`. Including interest, the position **exactly breaks even**, as theory requires. Rehedge at every node and, whatever path the stock takes, you break even at expiration — so buying below theoretical value or selling above it produces a profit equal to the difference.
- **Gamma rent.** "Traders sometimes refer to volatility trading as **renting the gamma**, with the rental costs being equal to the theta." Since a delta-neutral position taken at theoretical value breaks even when the underlying moves by u or d, and u and d are one standard deviation moves, **the amount of movement needed to break even over any interval equals one standard deviation**. This is an approximation only because u and d stay constant while theta changes — accurate over short intervals or with plenty of time left, less so over long intervals or very near expiration.
- **Consistency checks on a completed tree.** At every node, **put-call parity holds**, **|call delta| + |put delta| = 100**, and **call and put gammas are identical**.
- **Pseudoprobabilities.** p and 1 − p need not lie between 0 and 1. With S = 100, t = 9 months, n = 3, u = 1.05, d = 0.9524 and r raised from 4% to **40%**: `p = (1 + 0.1 − 0.9524)/0.0976 = 1.51`, so 1 − p = **−0.51**. This means the potential movement in the stock is too small to offset the interest lost by buying it: a 5% up-move per period versus 30%/4 = **7.5%** available in the bank. To restore ordinary probabilities, u must exceed `1 + r·t/n` — at 30% interest, u must exceed **1.075**.
- **Dividends break the recombining property.** Reducing prices by the dividend at a *terminal* node is easy. But if the dividend falls in the **next-to-last** period, subsequent prices no longer recombine — each node starts a new binomial tree (six terminal prices instead of four in a three-period tree). With multiple dividends and many periods this becomes unwieldy. **There is no ideal solution.** The simplest approximation: build the tree without dividends and reduce every post-dividend node's stock price by the total dividend. This is only an approximation — it produces call values slightly **larger** than the exact tree.
- **Convergence.** Binomial values converge to Black-Scholes as n → ∞, with the **error oscillating between positive and negative** and shrinking in absolute value. Three-period values: call **5.22**, put **2.28**. Four-period: **4.79** and **1.84**. True Black-Scholes: **5.01** and **2.05**.
- **How many periods?** Accuracy rises with n but the number of calculations grows exponentially. "A common choice is often somewhere between **50 and 100 periods**."
- **Half-steps.** Averaging two adjacent period counts sharply improves accuracy. The 9-period tree overvalues the 100 call by about **0.07**; the 10-period tree undervalues it by about **0.09**; the average (a "9½-period value") is off by only **0.01**.

## Anti-patterns
- **Assuming a 50/50 up/down probability** because the tree looks balanced.
- **Choosing u and d that do not recombine or that are not multiplicative inverses** — the first explodes the computation, the second introduces drift and makes theta meaningless.
- **Treating p as a real-world probability.** It is a risk-neutral pseudoprobability that can exceed 1 or fall below 0.
- **Computing an American value without checking intrinsic value at every node** — and forgetting that the corrected values change the delta and gamma too.
- **Handling a mid-tree dividend by simply shifting all subsequent nodes down** and expecting exactness.
- **Using too few periods** and accepting the oscillating error instead of averaging half-steps.

## Reference Tables

**Figure 19-1 — The three-period binomial tree**

| Parameter | Value |
|---|---|
| Stock price S | 100.00 |
| Periods n | 3 |
| Time to expiration t | 9 months = 0.75 |
| Annual interest rate r | 4.00% |
| u | 1.05 |
| d = 1/u | ≈ 0.9524 |
| p = [1 + (0.04 × 0.75/3) − 0.9524] / (1.05 − 0.9524) = 0.0576/0.0976 | ≈ **0.59** |
| 1 − p | **0.41** |
| Implied volatility (backed out of u) | **9.76%** |

Node prices: **115.76**, 110.25, **105.00**, 100.00, **95.24**, 90.70, **86.38**.

**Terminal values and probabilities**

| Terminal price | 100 call value | 100 put value | Path probability | Number of paths | Total probability |
|---|---|---|---|---|---|
| 115.76 | 15.76 | 0 | 0.59³ = 0.2054 | 1 | 0.2054 |
| 105.00 | 5.00 | 0 | 0.59² × 0.41 = 0.1427 | 3 | 0.4281 |
| 95.24 | 0 | 4.76 | 0.59 × 0.41² = 0.0992 | 3 | 0.2976 |
| 86.38 | 0 | 13.62 | 0.41³ = 0.0689 | 1 | 0.0689 |

**Figure 19-4 — European 100 call across the tree**

| Node | Stock | Call value | Delta |
|---|---|---|---|
| C₀,₀ | 100.00 | **5.22** | **62** |
| C₁,₁ | 105.00 | 7.75 | 81 |
| C₁,₀ | 95.24 | 1.71 | 31 |
| C₂,₂ | 110.25 | 11.24 | 100 |
| C₂,₁ | 100.00 | 2.92 | 51 |
| C₂,₀ | 90.70 | 0 | 0 |
| C₃,₃ | 115.76 | 15.76 | — |
| C₃,₂ | 105.00 | 5.00 | — |
| C₃,₁ | 95.24 | 0 | — |
| C₃,₀ | 86.38 | 0 | — |

Initial gamma = (81 − 31)/(105.00 − 95.24) = 50/9.76 = **5.1**.

**Figure 19-5 / 19-6 — The 100 put, European vs. American**

| Node | Stock | European put | American put | European Δ | American Δ |
|---|---|---|---|---|---|
| P₀,₀ | 100.00 | **2.28** | **2.44** | −38 | **−42** |
| P₁,₁ | 105.00 | 0.78 | 0.78 | −19 | −19 |
| P₁,₀ | 95.24 | 4.50 | **4.90** | −69 | **−79** |
| P₂,₂ | 110.25 | 0 | 0 | 0 | 0 |
| P₂,₁ | 100.00 | 1.93 | 1.93 | −49 | −49 |
| P₂,₀ | 90.70 | **8.31** | **9.30** (intrinsic) | −100 | −100 |
| P₃,₁ | 95.24 | 4.76 | 4.76 | — | — |
| P₃,₀ | 86.38 | 13.62 | 13.62 | — | — |

European gamma 5.1 → American gamma **6.1**.

**Figures 19-8 / 19-9 — Call on a dividend-paying stock** (2.00 dividend in the last period; terminal prices reduced to 113.76 / 103.00 / 93.24 / 86.38)

| Node | Stock | European call | American call |
|---|---|---|---|
| C₀,₀ | 100.00 | **3.99** (Δ 52, Γ 5.5) | **4.33** (Δ 58, Γ 6.6) |
| C₁,₁ | 105.00 | 6.12 (Δ 73) | 6.70 (Δ 83) |
| C₁,₀ | 95.24 | 1.02 (Δ 19) | 1.02 (Δ 19) |
| C₂,₂ | 110.25 | **9.26** | **10.25** (intrinsic) |
| C₂,₁ | 100.00 | 1.75 (Δ 31) | 1.75 (Δ 31) |
| C₂,₀ | 90.70 | 0 | 0 |

**Figure 19-11 vs. 19-12 — Exact vs. approximate handling of an early dividend**

| Node | Exact (non-recombining) tree | Approximation (shift all post-dividend nodes down by 2.00) |
|---|---|---|
| C₀,₀ | **3.95** (Δ 52) | **4.02** (Δ 53) |
| C₁,₁ | 6.08 (Δ 74) | 6.18 (Δ 73) |
| C₁,₀ | 0.99 (Δ 18) | 1.02 (Δ 19) |
| C₂,₂ | 9.24 | 9.26 |

The approximation gives **slightly larger** values.

**Convergence to Black-Scholes** (strike 100, stock 100, volatility 9.76%, interest 4%)

| Periods | Call value | Put value |
|---|---|---|
| 3 | 5.22 | 2.28 |
| 4 | 4.79 | 1.84 |
| ∞ (Black-Scholes) | **5.01** | **2.05** |

## Worked Example

**A. Pricing a European call and put on the three-period tree.**

With p = 0.59 and the terminal probabilities above, the 100 call:
```
[0.2054 × (115.76 − 100) + 0.4281 × (105.00 − 100)] / (1 + 0.03/3)³
= [0.2054 × 15.76 + 0.4281 × 5.00] / 1.0303
= 5.22
```
The 100 put:
```
[0.2976 × (100 − 95.24) + 0.0689 × (100 − 86.38)] / (1 + 0.03/3)³
= [0.2976 × 4.76 + 0.0689 × 13.62] / 1.0303
= 2.28
```

*Verify with put-call parity.* The forward price, compounding over three periods:
```
F = 100 × (1 + 0.75 × 0.04/3)³ = 100 × 1.0303 = 103.03
(F − X)/(1 + r·t/n)^n = (103.03 − 100)/1.0303 = 2.94
C − P = 5.22 − 2.28 = 2.94 ✓
```

**B. Working backwards node by node.** At C₂,₂ (stock 110.25) there is a 59% chance of reaching 115.76, where the call is worth 15.76, and a 41% chance of reaching 105.00, where it is worth 5.00:
```
expected value    = (0.59 × 15.76) + (0.41 × 5.00) = 11.35
theoretical value = 11.35 / (1 + 0.03/3) = 11.35 / 1.01 = 11.24
```
At C₂,₁: `[(0.59 × 5.00) + (0.41 × 0)] / 1.01 = 2.92`. At C₂,₀ both branches are worthless, so the value is **0**. Continue backwards to C₀,₀ = **5.22**.

**C. Reading the Greeks off the tree.**
```
Δ₀,₀ = (7.75 − 1.71) / (105.00 − 95.24) = 6.04 / 9.76 = 0.62  →  delta 62
Γ₀,₀ = (81 − 31) / (105.00 − 95.24) = 50 / 9.76 = 5.1
```
Theta: from C₀,₀ = 5.22 to C₂,₁ = 2.92 the call loses **2.30** over two periods. Two periods span (0.75 × 365)/3 = **91.25 days**, so the daily theta is `−2.30 / 91.25 = −0.0252`.

**D. Proving the hedge breaks even.** Buy the 100 call at 5.22 and sell 62% of a stock contract at 100.

*Up branch (stock 105.00):* the call gains 7.75 − 5.22 = **+2.53**; the short stock loses 0.62 × (100 − 105) = **−3.10**. Net **−0.57**.
*Down branch (stock 95.24):* the call loses 1.71 − 5.22 = **−3.51**; the short stock gains 0.62 × (100 − 95.24) = **+2.95**. Net **−0.56**.

Identical to rounding (the true delta is 61.88). The missing piece: the trade generated a credit of `−5.22 + 62.00 = +56.78`, earning `0.01 × 56.78 ≈ +0.57` over the period. **Break even, exactly as theory says.**

**E. Backing volatility out of u.** In the example u = 1.05 with t/n = 0.75/3 = 0.25:
```
1.05 = e^(σ√0.25) = e^(0.5σ)
ln(1.05) = 0.5σ  →  0.0488 = 0.5σ  →  σ = 0.0976 = 9.76%
```

**F. Turning the European put into an American put.** Scan Figure 19-5 for nodes where intrinsic value exceeds the European value. At **P₂,₀** the stock is 90.70 and the European put is worth **8.31** — but intrinsic value is **9.30**. Anyone holding an American put there exercises. Replace 8.31 with 9.30 and re-work backwards:
```
P₁,₀ = [(0.59 × 1.93) + (0.41 × 9.30)] / 1.01 = 4.90    (European value was 4.50)
P₀,₀ = [(0.59 × 0.78) + (0.41 × 4.90)] / 1.01 = 2.44    (European value was 2.28)
```
And the Greeks change too:
```
Δ₀,₀ = (0.78 − 4.90) / (105.00 − 95.24) = −0.42   →  −42, versus −38 European
Γ₀,₀ = (−19 − (−79)) / (105.00 − 95.24) = 6.1     →  versus 5.1 European
```

**G. An American call on a dividend-paying stock.** Add a 2.00 dividend in the final period, reducing terminal prices to 113.76, 103.00, 93.24, 86.38. The European call values come out to C₀,₀ = **3.99** with delta 52. But at **C₂,₂** the stock is 110.25 and the European call is worth **9.26** against an intrinsic value of **10.25** — exercise. Replacing that node and reworking gives an American value of **4.33** with delta **58** and gamma **6.6**.

Note the contrast with Figure 19-4's no-dividend tree: **at no node is the call worth less than intrinsic value**, so European and American values are identical — confirming Chapter 16's rule that a call on a non-dividend-paying stock is never exercised early.

## Key Takeaways
- Risk-neutral probability p = [(1 + b·t/n) − d] / (u − d) makes the expected underlying value equal the forward price.
- Set u = e^(σ√(t/n)) and d = 1/u so the tree recombines, has no drift, and approximates a lognormal distribution.
- Value the terminal nodes at intrinsic value and work backwards, discounting one period at a time.
- Delta and gamma are simple differences across adjacent nodes; theta needs two periods; vega and rho require re-running the model.
- Price an American option by replacing any node value below intrinsic value with intrinsic value — this changes the Greeks as well as the price.
- Renting the gamma: the movement needed to break even over any interval is one standard deviation, and the rent is the theta.
- p is a pseudoprobability and can fall outside [0, 1] when interest rates are large relative to u.
- Dividends destroy recombination; the practical fix is approximate. Convergence to Black-Scholes oscillates; 50–100 periods, or averaging half-steps, is the usual compromise.

## Connects To
- [Chapter 5 — Theoretical Pricing Models](ch05-theoretical-pricing-models.md): the expected-value-then-discount recipe, made concrete.
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): the rehedging break-even demonstrated node by node.
- [Chapter 16 — Early Exercise of American Options](ch16-early-exercise-american-options.md): the early exercise decisions this model implements numerically.
- [Chapter 18 — The Black-Scholes Model](ch18-black-scholes-model.md): the continuous limit this converges to.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): put-call parity, used here as a consistency check.
- [Chapter 6 — Volatility](ch06-volatility.md): the one-standard-deviation move that defines u and d.
