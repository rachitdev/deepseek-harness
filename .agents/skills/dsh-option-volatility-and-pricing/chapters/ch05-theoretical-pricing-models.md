# Chapter 5 — Theoretical Pricing Models

## Core Idea
A stock trader only needs **direction**. An option trader needs direction *and* **speed**. A trader who is sure a $100 stock reaches $115 in five months can still lose money buying a three-month 110 call at $2 (the stock arrives after expiration) or a six-month 110 call at $6 (the stock arrives but the call is only worth $5 of intrinsic value). "If predicting the correct market direction is difficult, correctly predicting direction and speed is probably beyond most traders' capabilities." A theoretical pricing model exists to quantify speed — as probability — so a trader can decide whether an option's market price is too high or too low.

## Frameworks Introduced

**Expected value → theoretical value.** *Expected value* is the average payback over infinitely many trials. *Theoretical value* is "the price one would be willing to pay now to just break even in the long run" — i.e., the **present value of the expected value**, adjusted for any other cash flows (dividends).

**The four-step model recipe** (Natenberg's explicit summary):
1. Propose a series of possible prices at expiration for the underlying contract.
2. Assign a probability to each price, **subject to the constraint that the market is arbitrage-free: the expected value of the underlying must equal the forward price**.
3. Compute the expected value of the option from those prices, probabilities, and the exercise price.
4. Take the present value, according to the option's settlement procedure.

**Expected value formulas:**
- Call: `Σᵢ pᵢ · max(Sᵢ − X, 0)`
- Put: `Σᵢ pᵢ · max(X − Sᵢ, 0)`

**The five required Black-Scholes inputs** (Figure 5-6): exercise price, time to expiration, current underlying price, applicable interest rate, **volatility**. Dividends are a sixth input for stock options that pay them.

**The riskless hedge and the four basic positions.**

| Option position | Corresponding market position | Appropriate hedge |
|---|---|---|
| Buy call(s) | Long | Sell underlying |
| Sell call(s) | Short | Buy underlying |
| Buy put(s) | Short | Buy underlying |
| Sell put(s) | Long | Sell underlying |

The mnemonic: **always do the opposite with calls and the underlying; always do the same with puts and the underlying.** Doing it backwards with puts (buy puts *and* sell the underlying) "is no hedge at all." The correct proportion is the option's **hedge ratio**.

## Key Concepts
- **Edge.** The casino sells a bet worth 95 cents for $1 — a 5-cent edge, or 5 cents kept per dollar wagered. A trader wants the casino's side, or to buy the 95-cent bet for 88 cents (a 7-cent edge).
- **"In the long run" is doing all the work.** Expected value only pays off over many trials. A casino that lets a player hit 20 numbers in a row may go out of business before reaching the long run — hence risk management matters at least as much as calculating theoretical value.
- **Probability drives strategy choice.** A high-probability-of-profit strategy justifies a small potential profit; a low-probability strategy demands a large one.
- **Model as candle in a dark room.** A model is a scaled-down representation, never an exact replica; garbage in, garbage out. The new trader with a model sees the layout dimly, with some distortion from the flickering flame — better than groping in the dark. The danger arrives with confidence and size, when a misread detail is magnified. Natenberg's footnote: financial crises seem to occur when many traders drop their candles at the same time.
- **At the money vs. at the forward.** "At the money" = exercise price equals the *current* underlying price; **"at the forward"** = exercise price equals the *forward* price at expiration. In many markets at-the-forward options are the most actively traded and serve as the benchmark for evaluating other options. For European options the current underlying price matters only insofar as it can be converted to a forward price.
- **Probabilities need not be symmetric** — only the expected value must equal the forward price.
- **An option is a substitute for an underlying position.** A call substitutes for long, a put for short. Whether it is a *better* substitute depends on theoretical value vs. market price: buy calls or sell puts below/above value to get long more profitably than buying the underlying; buy puts or sell calls to get short.

## Model lineage and history
- Charles Castelli, *The Theory of Options in Stocks and Shares*, London **1877** — describes the "call-of-more" and "call-and-put," today's **covered write** and **straddle**.
- Louis Bachelier, *The Theory of Speculation*, **1900** — first higher-mathematics treatment; no practical application because no organized option markets existed.
- Fischer Black and Myron Scholes, **1973**, concurrent with the opening of the CBOE — the first practical model. Robert Merton's parallel work makes it the **Black-Scholes-Merton** model; Scholes and Merton received the Nobel Prize in 1997, Black having died in 1995.
- **Black model (1976)** — options on futures. **Garman-Kohlhagen (1983)** — options on physical foreign currencies. All three "differ only in how they calculate the forward price of the underlying contract and the settlement procedure for the options."

## Notes on each input
- **Exercise price** — fixed by contract; a March 60 call cannot become a March 55 call (exchanges adjust only for stock splits or extraordinary dividends, an accounting change).
- **Time to expiration** — entered annualized (days ÷ 365). Only **business days** carry price-movement risk, but **every day** accrues interest; the model resolves this because volatility is measured on business days and then scaled to an annual number, so a trader can enter actual calendar days remaining. Finer increments (hours, minutes) theoretically improve accuracy, but near expiration discrete time increments poorly represent continuous time and **many traders stop using model values altogether**.
- **Underlying price** — not obvious. Given a market last traded at 75.25 quoting **75.20–75.40**, use the price at which you expect to execute the *hedge*: near the **bid** if the option trade creates a long market position (buying calls, selling puts, hedged by selling the underlying); near the **ask** if it creates a short market position (selling calls, buying puts, hedged by buying the underlying). In a liquid market with a narrow spread, the midpoint (75.30) is a reasonable shortcut; in an illiquid market with a wide spread it is not.
- **Interest rates** — play two roles: (1) they set the forward price (higher rates raise the forward under stock-type settlement, raising calls and lowering puts) and (2) they discount the option's value (higher rates reduce present value under stock-type settlement). Usually one rate suffices; foreign-currency options need two (Garman-Kohlhagen). Benchmarks: the risk-free rate (a government yield matched to the option's life — a 60-day T-bill for a 60-day option), or more realistically **LIBOR** or Eurocurrency rates; for dollar options, CME **Eurodollar futures**. Interest rates are **the least important input** for most traders — "using a rate that 'makes sense' is usually a reasonable solution" — except for very large positions or very long-dated options.
- **Dividends** — only matter for stock options. A trader focuses on the **ex-dividend date** rather than the payable date, because only ownership of the stock (not a deep in-the-money option) carries the right to the dividend. Assume continuation of past policy absent other information, but a company can raise, cut, or omit. **If an expected ex-dividend date just before expiration slips a few days past expiration, that is equivalent to eliminating the dividend entirely** — worth a special effort to pin down.
- **Volatility** — the hardest input to understand and often the most important in real trading decisions; deferred to Chapter 6.

## Mental Models
- **Roll the die.** Expected value of a fair six-sided die paying its face in dollars = 21/6 = **$3.50**. Pay less than $3.50 and probability is on your side even on a single roll.
- **Rehedge to track changing probabilities.** As the underlying moves, the probability of each outcome changes. Establishing a riskless hedge and *adjusting it* as conditions change is how a trader keeps up with those shifting probabilities.

## Anti-patterns
- **Buying options because the risk-reward looks favorable.** A buyer must be right about *both* direction and speed.
- **Trading options when your real skill is direction.** "If a trader is highly proficient at predicting directional moves in the underlying market, he is probably better advised to trade the underlying instrument."
- **Hedging puts backwards** — buying puts and selling the underlying, or selling puts and buying the underlying.
- **Trusting a model's output near expiration**, where discrete time increments and unreliable inputs degrade the values.
- **Using the midpoint of a wide bid-ask spread** as the underlying price in an illiquid market.
- **Assuming a declared-looking dividend.** Until officially declared, it is an estimate.

## Reference Tables

**Expected value with five equally likely outcomes** (Figure 5-1 / 5-2)

| Underlying price at expiration | 80 | 90 | 100 | 110 | 120 | Expected value |
|---|---|---|---|---|---|---|
| Probability | 20% | 20% | 20% | 20% | 20% | — |
| Underlying | 80 | 90 | 100 | 110 | 120 | **$100** |
| 100 call value | 0 | 0 | 0 | 10 | 20 | **$6.00** |

**Three probability distributions, one 100 call, two-month expiry, 12% annual interest**

| Figure | Prices | Probabilities | Expected value of underlying | Expected value of 100 call | Theoretical value (÷1.02) |
|---|---|---|---|---|---|
| 5-2 | 80/90/100/110/120 | 20/20/20/20/20 | $100 | $6.00 | **$5.88** |
| 5-3 | 80/90/100/110/120 | 10/20/40/20/10 | $100 | $4.00 | **$3.92** |
| 5-4 (centered on the forward) | 82/92/102/112/122 | 10/20/40/20/10 | $102 | $5.40 | **$5.29** |
| 5-5 (asymmetric) | 83/90/99/110/123 | 6/15/39/33/7 | $102 | $4.91 | **$4.81** |

## Worked Example
**Building an option value from probabilities, step by step.**

*Step 1 — the naive distribution.* Five equally likely outcomes: 80, 90, 100, 110, 120. Expected underlying value = (20% × 80) + (20% × 90) + (20% × 100) + (20% × 110) + (20% × 120) = **$100**. Overlay the 100 call's parity graph: it is worth 0, 0, 0, 10, 20. Expected value = (20% × 0) × 3 + (20% × 10) + (20% × 20) = **$6.00**. Under stock-type settlement with 12% annual interest (1% per month) and two months to expiration, theoretical value = 6.00 / (1 + 0.12 × 2/12) = 6.00 / 1.02 ≈ **$5.88**.

*Step 2 — concentrate the probabilities.* Extreme moves are less likely than small ones: given a $100 underlying, $110 is far more plausible than $250. Reweight to 10/20/40/20/10. Expected underlying value is still $100 (the distribution is symmetric), but the call's expected value falls to (20% × 10) + (10% × 20) = **$4.00**, theoretical value 4.00/1.02 ≈ **$3.92**.

*Step 3 — impose the arbitrage-free constraint.* The market's own view of the most likely future price is the forward price. For a non-dividend-paying $100 stock at 12% for two months, the forward is 100 × (1 + 0.12 × 2/12) = **$102**. Shift the distribution to be symmetric around 102: prices 82, 92, 102, 112, 122 with probabilities 10/20/40/20/10. The 100 call is now worth 0, 0, 2, 12, 22. Expected value = (40% × 2) + (20% × 12) + (10% × 22) = 0.80 + 2.40 + 2.20 = **$5.40**; theoretical value 5.40/1.02 ≈ **$5.29**.

*Step 4 — symmetry is not required, only the forward constraint.* Take prices 83, 90, 99, 110, 123 with probabilities 6%, 15%, 39%, 33%, 7%. Expected underlying value = 4.98 + 13.50 + 38.61 + 36.30 + 8.61 = **$102** — still arbitrage-free. The 100 call is worth 0, 0, 0, 10, 23, so its expected value = (33% × 10) + (7% × 23) = 3.30 + 1.61 = **$4.91**, theoretical value 4.91/1.02 ≈ **$4.81**.

*The remaining hurdle:* the real world has an infinite number of possible outcomes, not five. Approximating that continuous distribution is the job of Chapter 6.

**A parallel worked example on interest and dividends.** A roulette bet on a 38-slot wheel paying $36 has an expected value of 36/38 = **$0.9474** ≈ 95 cents; the casino charges $1, a 5-cent edge. If the casino pays winnings **two months later** at 12% annual interest, the bet's *theoretical value* is 95 / (1 + 0.12 × 2/12) ≈ **93 cents** — the player loses two cents of foregone interest even at "fair" expected value. Add a 1-cent loyalty bonus payable in one month (the analogue of a dividend) and theoretical value rises to **94 cents**.

## Key Takeaways
- Direction plus speed, not direction alone — that is the option trader's problem, and volatility is how speed enters the model.
- Theoretical value = present value of expected value, adjusted for dividends.
- The arbitrage-free constraint fixes the *mean* of the distribution at the forward price; the shape of the distribution is a separate question.
- Five inputs: exercise price, time, underlying price, interest rate, volatility (plus dividends for stock). Interest rates matter least; volatility matters most and is hardest.
- A model is a candle, not daylight. Calculating theoretical value is only half the job; managing short-run risk is the other half.
- To capture a theoretical edge you must establish and maintain a riskless hedge — the reason the hedge ratio (delta) is the next topic.

## Connects To
- [Chapter 2 — Forward Pricing](ch02-forward-pricing.md): the forward price that anchors the distribution.
- [Chapter 4 — Expiration Profit and Loss](ch04-expiration-profit-and-loss.md): the parity graphs overlaid on the distribution.
- [Chapter 6 — Volatility](ch06-volatility.md): replacing five discrete outcomes with a continuous distribution.
- [Chapter 7 — Risk Measurement I](ch07-risk-measurement-i.md): the hedge ratio becomes the delta.
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): adjusting the riskless hedge as probabilities change.
- [Chapter 18 — The Black-Scholes Model](ch18-black-scholes-model.md): the model's formal structure.
- [Chapter 23 — Models and the Real World](ch23-models-and-the-real-world.md): where the candle's flicker becomes dangerous.
