# Cheatsheet — Option Volatility and Pricing (Sheldon Natenberg)

---

## Core formulas

| Quantity | Formula |
|---|---|
| Forward price, general | cash price + costs of buying now − benefits of buying now |
| Forward, commodity | `F = C(1 + r·t) + s·t + i·t` |
| Forward, stock (exact) | `F = S(1 + r·t) − Σ[dₙ(1 + rₙ·tₙ)]` |
| Forward, stock (simple) | `F = S(1 + r·t) − D` |
| Forward, bond | `F = B(1 + r·t) − Σ[cₙ(1 + rₙ·tₙ)]` |
| Forward, currency | `F = S × (1 + r_d·t)/(1 + r_f·t)` |
| Forward, stock index | `F = S[1 + (r − d)t]` |
| Implied spot | `S = (F + D)/(1 + r·t)` |
| Implied rate | `r = ([(F + D)/S] − 1)/t` |
| Implied dividend | `D = S(1 + r·t) − F` |
| Call intrinsic value | `max(0, S − X)` |
| Put intrinsic value | `max(0, X − S)` |
| Premium | intrinsic value + time value |
| Expected value, call | `Σ pᵢ · max(Sᵢ − X, 0)` |
| Volatility scaling | `σ_t = σ_annual × √t`; daily ≈ σ/16; weekly ≈ σ/7.2 |
| Black-Scholes | `C = S·N(d₁) − X·e^(−rt)·N(d₂)` |
| d₁ | `[ln(S/X) + (b + σ²/2)t] / (σ√t)` |
| d₂ | `d₁ − σ√t` |
| At-the-forward value | `X × (σ × 100) × √t × 0.00399 / (1 + r·t)` — the **40% rule** |
| Put-call parity | `C − P = (F − X)/(1 + r·t)` |
| Put-call parity, stock shortcut | `C − P ≈ S − X + X·r·t − D` |
| Delta of a synthetic future (stock-type settled options) | `100 / (1 + r·t)` |
| Average-delta repricing | `C + [(S₂−S₁)Δ] + [(S₂−S₁)²Γ/2]` |
| ATM theta estimate | `TV_t × [1 − √((t−1)/t)]` |
| Lambda (elasticity) | `Λ = Δ × (S/TV)` |
| Position implied volatility | evaluation volatility + (total edge / total vega) |
| Efficiency | \|gamma / theta\| |
| Delta-neutral price of a −gamma position | current price − (delta / gamma) |
| Roll value | `X/(1 + r_s·t_s) − X/(1 + r_l·t_l) − D` ≈ `X·r·t − D` |
| Box value | strike interval, present-valued under stock-type settlement |
| Forward volatility | `√[(σ₂²t₂ − σ₁²t₁)/(t₂ − t₁)]` |
| Combined variance | `[(t₁σ₁²) + (t₂σ₂²)]/(t₁ + t₂)` |
| Variance point value | vega notional / (2 × volatility price) |
| Binomial risk-neutral p | `[(1 + b·t/n) − d] / (u − d)` |
| Binomial u, d | `u = e^(σ√(t/n))`, `d = 1/u` |
| Put blackout period | `dividend / (exercise price × interest rate / 365)` days |
| Index impact | percent change in stock × its index weighting |
| Skew normalization, x axis | `ln(X/S) / (σ√t)` |
| Skew fit | `y = a + bx + cx²` (a = level, b = skewness, c = kurtosis) |
| Butterfly-implied probability | butterfly price ÷ strike interval |

---

## Standard deviation reference

| Range | Probability | Fraction |
|---|---|---|
| ±1 s.d. | 68.3% | ~2/3 |
| ±2 s.d. | 95.4% | ~19/20 |
| ±3 s.d. | 99.7% | ~369/370 |

A two-standard-deviation **daily** move should appear roughly **once a month** (20 trading days).

---

## Greek signs by position

| If you are… | Delta | Gamma | Theta | Vega | Rho |
|---|---|---|---|---|---|
| Long the underlying | + | 0 | 0 | 0 | 0 |
| Short the underlying | − | 0 | 0 | 0 | 0 |
| Long calls | + | + | − | + | + stock / − futures* |
| Short calls | − | − | + | − | − stock / + futures* |
| Long puts | − | + | − | + | − stock / − futures* |
| Short puts | + | − | + | − | + stock / + futures* |

*Under stock-type settlement of futures options. Under futures-type settlement, rho = 0.

**What each sign wants:** delta + → price rises; gamma + → big or fast moves; theta + → time passing helps; vega + → implied volatility rises; rho + → rates rise.

---

## Where each Greek is maximized

| Measure | Maximized when | Critical underlying price |
|---|---|---|
| Delta | Deeply in the money | `S = X·e^(−[b − σ²/2]t)` gives delta 50 |
| Lambda | Out of the money; near expiration; low volatility | — |
| Gamma | At the money; near expiration; low volatility | `S = X·e^(−[b − 3σ²/2]t)` |
| Theta | At the money; near expiration; low volatility | `S = X·e^([b + σ²/2]t)` |
| Vega | At the money; long term | `S = X·e^(−[b + σ²/2]t)` |
| Rho | Deeply in the money; long term | — |
| Vanna | ±20 delta; low volatility | speed = 0 marks max gamma |
| Charm | ±20 delta; near expiration | charm = 0 marks max theta |
| Speed | ±15 delta; low volatility; near expiration | vanna = 0 marks max vega |
| Color | At the money; near expiration; low volatility | — |
| Volga | ±10 delta; long term; low volatility | — |
| Vega decay | ±20 delta; near expiration | — |
| Zomma | At the money; near expiration; low volatility | — |

---

## How the Greeks respond to changing conditions

| Measure | Volatility rises | Time passes |
|---|---|---|
| Delta (OTM) | Rises toward 50 | Falls toward 0 |
| Delta (ITM) | Falls toward 50 | Rises toward 100 |
| Gamma (ATM) | **Falls** | **Rises** |
| Gamma (ITM/OTM) | Rises | Falls |
| Theta (ATM) | Rises proportionally | Accelerates (→ ∞) |
| Theta (ITM/OTM) | Rises, may hit 0 first | Slows |
| Vega (ATM) | Roughly constant | Falls |
| Vega (ITM/OTM) | Rises | Falls |
| Lambda | Falls | Rises |

**Exercise-price proportionality (ATM options):** theta and vega scale **with** the exercise price; gamma scales **inversely**.

---

## Effect of changing market conditions on option values

| If… | Calls | Puts |
|---|---|---|
| Underlying price rises | rise | fall |
| Underlying price falls | fall | rise |
| Volatility rises | rise | rise |
| Volatility falls | fall | fall |
| Time passes | fall* | fall* |
| Dividend raised | fall | rise |
| Dividend cut | rise | fall |

*Except a deeply in-the-money, European, stock-type-settled option, which has negative time value and positive theta.

**Interest rates:**

| | Domestic up | Domestic down | Foreign up | Foreign down |
|---|---|---|---|---|
| Stock calls | rise | fall | — | — |
| Stock puts | fall | rise | — | — |
| Futures calls/puts (stock-type settled) | fall | fall | — | — |
| Futures calls/puts (futures-type settled) | no effect | no effect | — | — |
| Currency calls | rise | fall | fall | rise |
| Currency puts | fall | rise | rise | fall |

---

## Volatility spread selection matrix

| Spread | Delta | Gamma | Theta | Vega | Downside | Upside |
|---|---|---|---|---|---|---|
| Long straddle | 0 | + | − | + | Unlimited reward | Unlimited reward |
| Long strangle | 0 | + | − | + | Unlimited reward | Unlimited reward |
| Short butterfly | 0 | + | − | + | Limited reward | Limited reward |
| Short condor | 0 | + | − | + | Limited reward | Limited reward |
| Call ratio, buy > sell | 0 | + | − | + | Limited reward | Unlimited reward |
| Put ratio, buy > sell | 0 | + | − | + | Unlimited reward | Limited reward |
| Short straddle | 0 | − | + | − | Unlimited risk | Unlimited risk |
| Short strangle | 0 | − | + | − | Unlimited risk | Unlimited risk |
| Long butterfly | 0 | − | + | − | Limited risk | Limited risk |
| Long condor | 0 | − | + | − | Limited risk | Limited risk |
| Call ratio, sell > buy | 0 | − | + | − | Limited risk | Unlimited risk |
| Put ratio, sell > buy | 0 | − | + | − | Unlimited risk | Limited risk |
| **Long calendar** | 0 | **−** | **+** | **+** | Limited risk | Limited risk |
| **Short calendar** | 0 | **+** | **−** | **−** | Limited reward | Limited reward |

**Four quadrants:**

| Gamma | Vega | What helps |
|---|---|---|
| + | + | More volatile underlying; rising implied volatility |
| − | − | Less volatile underlying; falling implied volatility |
| + | − | More volatile underlying; falling implied volatility |
| − | + | Quiet underlying; rising implied volatility (the calendar quadrant) |

**Risk spectrum:** straddles and strangles (largest gamma and vega) → ratio spreads and Christmas trees → butterflies and condors (smallest).

---

## Strategy selection by implied volatility

| Implied volatility vs. your estimate | Choose spreads with | Examples |
|---|---|---|
| **Low** (options underpriced) | **Positive vega** | Long straddles/strangles, short butterflies/condors, ratio spreads buying more than selling, long calendars |
| **High** (options overpriced) | **Negative vega** | Short straddles/strangles, long butterflies/condors, ratio spreads selling more than buying, short calendars |
| **Low but expected to rise** | Long calendar spreads | |
| **High but expected to fall** | Short calendar spreads | |

**Vertical spreads:** implied volatility low → **buy** the at-the-money option; high → **sell** it. The other leg follows.

**Hedgers:** high implied volatility → **buy as few options as possible, sell as many as possible**; low implied volatility → the reverse.

---

## Synthetics

| Synthetic | Construction |
|---|---|
| Long underlying | +call, −put |
| Short underlying | −call, +put |
| Long call | +underlying, +put |
| Short call | −underlying, −put |
| Long put | −underlying, +call |
| Short put | +underlying, −call |

**Mnemonic:** opposite with calls and the underlying; same with puts and the underlying.

**Companion relationships:** |Δ_call| + |Δ_put| ≈ 100; gammas identical; vegas identical; **thetas identical only when there is no cost of carry**.

**Hedge = synthetic option:** protective put = long call · protective call = long put · covered call = short put · covered put = short call · long collar = bull vertical · short collar = bear vertical.

**Additivity:** butterfly + iron butterfly = strike interval. Condor + iron condor = wing interval. Call vertical + put vertical (same strikes) = strike interval. Box = bull vertical **+** bear vertical. Roll = call calendar **−** put calendar.

---

## Arbitrage boundaries

*Lower:*

| | | American | European |
|---|---|---|---|
| Futures options (stock-type) | Call | max[0, F − X] | max[0, (F − X)/(1 + r·t)] |
| | Put | max[0, X − F] | max[0, (X − F)/(1 + r·t)] |
| Stock options | Call | max[0, S − X, S − X/(1 + r·t) − D] | max[0, S − X/(1 + r·t) − D] |
| | Put | max[0, X − S, X/(1 + r·t) − S + D] | max[0, X/(1 + r·t) − S + D] |

*Upper:*

| | | American | European |
|---|---|---|---|
| Futures options (stock-type) | Call | F | F/(1 + r·t) |
| | Put | X | X/(1 + r·t) |
| Stock options | Call | S | S − D |
| | Put | X | X/(1 + r·t) |

---

## Early exercise decision table

| Option | Exercise early when | Optimal day |
|---|---|---|
| American stock **call** | dividend > volatility + interest | **The day before the ex-dividend date.** Never, if no dividend before expiration |
| American stock **put** | interest > volatility + dividend | Any day **outside** the blackout period; most commonly the dividend day |
| Futures option (stock-type settled) | interest > volatility | When daily interest > the companion option's theta |
| Futures option (futures-type settled) | Never — American = European | — |

Component estimates: **dividend value** = expected dividend; **interest value** = `X·r·t` (stock) or `(F − X)·r·t` (futures); **volatility value** ≈ price of the companion out-of-the-money option; **one day's volatility value** = that option's theta.

Formally optimal: theoretical value exactly at parity **and** delta exactly 100.

---

## Hedging strategy summary

| Position | Strategy | Advantage | Disadvantage |
|---|---|---|---|
| **Long underlying** | Sell a future/forward | No downside risk | No upside |
| | Buy a protective put | Limited downside; unlimited upside | Cost of the option |
| | Sell a covered call | Partial protection = the call price | Unlimited downside; limited upside |
| | Long collar | Limited downside | Limited upside |
| **Short underlying** | Buy a future/forward | No upside risk | No downside profit |
| | Buy a protective call | Limited upside risk; unlimited downside profit | Cost of the option |
| | Sell a covered put | Partial protection = the put price | Unlimited upside; limited downside profit |
| | Short collar | Limited upside risk | Limited downside profit |

---

## Volatility skews

| Skew | Inflated strikes | Cause | Markets | Price/volatility relationship |
|---|---|---|---|---|
| **Investment** | Lower | Protective puts bought low, covered calls sold high | Stocks, stock indexes | Inverse |
| **Demand (commodity)** | Higher | End users buy protective calls, sell covered puts | Agriculturals, energy | Direct |
| **Balanced** | Both | Two-sided hedging | Currencies | Neither |

**Trading the shape:**

| View | Investment skew | Demand skew |
|---|---|---|
| Expect flatter | Sell lower strikes, buy higher | Sell higher strikes, buy lower |
| Expect steeper | Buy lower strikes, sell higher | Buy higher strikes, sell lower |
| Expect more curvature | Buy strangles (offset vega with straddles) | Same |
| Expect less curvature | Sell strangles (offset vega with straddles) | Same |

**Benchmarks:** skewness sensitivity peaks at **±25 delta** (the risk-reversal strikes); kurtosis sensitivity at **±5 delta**. At-the-money options are the pivot with **zero** sensitivity to both.

---

## Tells and smells — symptom → diagnosis → fix

| Symptom | Diagnosis | Fix |
|---|---|---|
| An option's price implies a value you cannot reproduce | One of your inputs, not the price, is wrong | Solve for the implied rate, dividend, or volatility before assuming an arbitrage |
| Observed daily moves are consistently smaller than your volatility predicts | Your volatility input is too high | Check for a holiday week; otherwise lower the input — wrong volatility means wrong probabilities means wrong values |
| A delta-neutral position keeps needing large adjustments | Large gamma, probably at the money and near expiration | Reduce the position; consider a butterfly instead of a straddle |
| Implied volatility rose but your long straddle lost money | Time decay outweighed the vega gain | Hold only if your **realized** volatility view is intact; realized dominates the longer you hold |
| Total vega across expirations is zero yet the book moves with volatility | Short-term implied volatility moves faster than long-term | Rescale each month's vega to the primary month before summing |
| A "delta-neutral" straddle keeps losing on directional moves | Volatility moves with price in this market | In a stock index the position is really delta negative; in a commodity, delta positive |
| Conversions and reversals look mispriced | Short-sale restrictions inflating puts, or a wrong rate/dividend assumption | Check borrowability and back out the implied rate |
| A conversion in a futures market is not delta neutral | Settlement risk; the synthetic delta is 100/(1 + r·t) | At 300 conversions of +2 deltas each you are long 600 deltas — hedge it |
| Puts trade above their theoretical relationship to calls | Short-sale restriction on the underlying stock | Carry some long stock so you can always sell long |
| A position's profit keeps shrinking whether volatility rises **or** falls | Negative volga | Prefer a structure with positive volga — typically the butterfly |
| Your position gets bigger every time you adjust | You are adjusting to maximize edge, not to control risk | Reduce the size, or adjust in the underlying |
| A gamma of zero hides −2,388 at one strike | Concentration risk | Spread the risk across strikes and expirations |
| A calendar spread collapses despite a quiet market | The underlying moved away from the strike | Calendars need the underlying to stay at the strike; check the price sensitivity before entering |
| A short at-the-money straddle loses catastrophically overnight | A gap — the model assumed continuous trading | Never carry large short at-the-money positions into expiration |
| The market is limit-locked and you must trade | No underlying access | Trade the cash market, an unlocked futures spread leg, or synthetic futures |
| An at-the-money option might pin the strike at expiration | Pin risk on a conversion or reversal | Reduce the size early, or cross with a trader holding the opposite structure at even money |
| VIX rose but your VIX future barely moved | Futures reflect expected VIX at *their* expiration, and the index is unreplicable | Evaluate VIX futures independent of the index; buy only when the move is expected near expiration or large enough to invert the curve |

---

## Thresholds and defaults

| Item | Value |
|---|---|
| Trading days per year (volatility) | 256 assumed (√256 = 16); really 250–260 |
| Trading weeks per year | 52 (√52 ≈ 7.2) |
| Peak of the standard normal distribution | 0.398942 → 0.00399 per volatility point |
| Auto-exercise threshold | ~0.05 retail, ~0.02 professional |
| U.S. stock settlement | T + 3 |
| Ex-dividend date | 2 business days before the record date |
| Stock option underlying | 100 shares (a round lot) |
| Futures option underlying | 1 futures contract |
| Stock/index option expiration | Third Friday |
| Binomial periods for accuracy | 50–100, or average adjacent counts (half-steps) |
| EWMA decay λ | ≈ 0.94 |
| S&P 500 mean volatility (2001–2010) | 15–20% |
| Gold mean volatility (2001–2010) | 10–20% |
| Bund mean volatility (2001–2010) | ~5% |
| Typical implied-over-realized premium | up to ~10 percentage points |
| VIX / S&P 500 correlation (2003–2012) | −0.7444 (VIX moves ≈ 5.7× as far) |
| VIX predictive correlation with future realized volatility | +0.1561 (negligible) |
| 30-day S&P change vs. 30-day realized volatility | −0.3895 |
| VIX futures point value | $1,000 |
| VIX options point value | $100 |
| VIX contract expiration | 30 days before the SPX options used, third Wednesday |
| Typical clearing-firm stress test | 20% underlying move; doubling of implied volatility |

---

## Model assumptions and how each fails

| Assumption | Reality | Consequence |
|---|---|---|
| Free trading in the underlying | Limit locks, circuit breakers, short-sale restrictions | Puts inflate; conversions look mispriced; hedging becomes impossible when it matters most |
| Unlimited borrowing at one rate | Borrow/lend spread; margin calls | You may be forced to liquidate — every model assumes you can hold to expiration |
| No transaction costs | Brokerage, clearing, membership, bid-ask on every adjustment | High-gamma strategies can lose their edge to adjustment costs |
| No taxes | Real but usually secondary | Rarely decides between strategies |
| Constant interest rates | Variable clearing-firm rates | Minor except for deeply in-the-money long-term options |
| Constant volatility | Volatility clusters | Option value becomes **path dependent** under dynamic hedging |
| Continuous trading | Jump-diffusion | Gaps hurt negative gamma; models **undervalue** options |
| Volatility independent of price | Stock indexes get volatile falling; commodities rising | Produces the skew; "delta neutral" is not neutral |
| Lognormal prices | Fat tails, nonzero skewness | Extremes occur that a normal distribution says are impossible |

---

## Model variants (the b parameter)

| b | Model |
|---|---|
| b = r | Black-Scholes, options on stock |
| b = r = 0 | Options on futures, futures-type settlement |
| b = 0 | Black model, options on futures, stock-type settlement |
| b = r − r_f | Garman-Kohlhagen, options on foreign currencies |
| b = r − q | Stock paying a continuous dividend yield q |

---

## Process templates

**Analyze any expiration P&L**
1. List the slope of every contract in every interval between strikes (0 for OTM; ±1 for ITM options; ±1 for the underlying everywhere).
2. Sum the slopes interval by interval.
3. Compute P&L exactly at one strike.
4. Walk the slopes outward; find breakevens as `known price ± |P&L| / |slope|`.

**Capture a mispricing by dynamic hedging**
1. Trade the mispriced option.
2. Neutralize the delta with the underlying.
3. At each interval, recompute the delta (holding volatility and rate constant) and trade back to neutral.
4. At expiration: expire the OTM options, sell the ITM options at parity, liquidate the underlying.
5. Reconcile: original hedge + adjustments + option carry + underlying carry (or variation interest) + adjustment interest, discounted.

**Choose among candidate spreads**
1. Verify a positive theoretical edge.
2. Scale each spread so the edges match.
3. Plot P&L vs. underlying price, vs. volatility, vs. time (and vs. rates and dividends for stock options across expirations).
4. Read off each breakeven volatility; check |Γ/Θ| if all options expire together.
5. Weigh liquidity, execution difficulty, and leg count.
6. Size to the margin for error.

**Analyze a complex book**
1. Sum the Greeks; note the theoretical edge.
2. Compute the **upside** and **downside net contract positions**.
3. Estimate the delta-neutral price as current price − delta/gamma, then verify graphically.
4. Estimate the position's implied volatility as edge/vega.
5. Push volatility and time to their extremes to find the limiting delta.
6. Check for concentration at any strike or expiration.
7. Decide in advance: what to do if conditions go against you, what to do if they go your way, and what to do **now** to limit later damage.

**Read a volatility term structure for mispricings**
1. Plot at-the-money implied volatility against time to expiration; fit a best-fit line.
2. For each adjacent pair of months, compute the calendar spread's implied volatility (or use `spread price / spread vega`).
3. Overlay those on the term structure; kinks below the line mark underpriced months, kinks above mark overpriced ones.
4. Cross-check with forward volatilities.
5. Trade the mispriced months against their neighbours — often a time butterfly.

**Extract the market's implied distribution**
1. Take butterfly prices across the full strike ladder.
2. Divide each by the strike interval to get the probability of that strike.
3. Verify the butterflies sum to the strike interval and the probabilities to 1.00.
4. Narrow the increment to approach continuity.
5. Overlay a constant-volatility lognormal distribution and note where they differ.
