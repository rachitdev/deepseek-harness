# Chapter 6 — Volatility

## Core Idea
**Volatility is just a trader's term for standard deviation** (notation σ). Feed a mean and a standard deviation into a model and you have fully specified a normal distribution — and therefore the probabilities that price an option. The mean is the **forward price**; the standard deviation is the **volatility**. Everything else in the chapter is how to scale, measure, interpret, and distrust that number.

## Frameworks Introduced

**Working definition.** The volatility fed into a pricing model represents a **one standard deviation price change, in percent, over one year**.

**Scaling volatility for time.** Interest is proportional to time; **volatility is proportional to the square root of time**.

```
volatility_t = volatility_annual × √t          (t in years)
volatility_daily  ≈ volatility_annual / √256 = volatility_annual / 16
volatility_weekly ≈ volatility_annual / √52  = volatility_annual / 7.2
```
Traders assume **256 trading days** (250–260 in reality) purely because √256 = 16 is a whole number, and **52 trading weeks** because no complete week is closed.

**Standard-deviation probabilities** (the trader's working approximations):

| Range | Probability | Fractional approximation |
|---|---|---|
| ±1 standard deviation | 68.3% | about 2/3 |
| ±2 standard deviations | 95.4% | about 19/20 |
| ±3 standard deviations | 99.7% | about 369/370 |

Standard deviations are **additive**: if 1 s.d. = 3, then 2 s.d. = 6.

**Taxonomy of volatilities.**
- **Realized volatility** — annualized standard deviation of percent price changes of the *underlying*, over a stated period at a stated interval (a "50-day volatility" = annualized s.d. of daily changes over the last 50 trading days; a "52-week volatility" = annualized s.d. of weekly changes over 52 weeks).
  - **Future realized volatility** — what every trader wants to know; the volatility that actually describes the option's life and therefore determines its value.
  - **Historical realized volatility** — the past; the starting point for an estimate.
  - **Forecast volatility** — the output of a volatility-forecasting model.
- **Implied volatility** — derived from an *option's* price by running the model backwards; the marketplace's consensus estimate of future realized volatility over the option's life.
- **Yield volatility vs. price volatility** — for interest-rate instruments, volatility computed from the instrument's yield vs. from its price.

**The three principles of volatility sensitivity** (Natenberg numbers these explicitly):
1. **In total points**, a change in volatility affects an **at-the-money** option more than an equivalent ITM or OTM option.
2. **In percent terms**, a change in volatility affects an **out-of-the-money** option more than an equivalent ATM or ITM option.
3. A change in volatility affects a **long-term** option more than an equivalent short-term option.

Corollary: in-the-money options are least sensitive to volatility however you measure it; as an option goes deeply ITM it becomes more sensitive to underlying price and less sensitive to volatility. This is why **most option volume concentrates in at-the-money and out-of-the-money options** — those are the volatility instruments.

## Key Concepts
- **The pinball maze / quincunx (Galton board).** A ball dropped through a lattice of nails goes left or right with 50% probability at each — a **random walk**. Many balls produce a bell-shaped **normal distribution**. Sideways movement = price change; downward movement = passage of time. Restrict sideways movement (drop two levels between nails) and you get a **taller, narrower** distribution = low volatility; widen it (move two nails per level) and you get a **flatter, faster-spreading** one = high volatility.
- **Why volatility raises *all* option values.** The expected value of the *underlying* depends on all outcomes; the expected value of an *option* depends only on outcomes where it finishes in the money — everything else is zero. Widening a symmetric distribution puts more mass beyond the call's strike *and* more mass below the put's strike, so both gain.
- **Bachelier's flaw: normally distributed prices allow negative prices.** If a $50 contract can rise $75 to $125, symmetry requires it can fall $75 to −$25.
- **Lognormal distribution.** Volatility, unlike interest, is a rate of return with both positive and negative outcomes. Black-Scholes is a **continuous-time** model that compounds volatility continuously; continuously compounded *normally distributed percent changes* produce a **lognormal** distribution of prices at expiration — skewed to the upside, **bounded by zero on the downside**, and with the mean to the **right of the mode** because the right tail is open-ended while the left is bounded.
- **Continuous compounding, worked:** $1,000 × e^0.12 = **$1,127.50**; $1,000 × e^−0.12 = **$886.92**. A +12% continuously compounded return gains $127.50; a −12% one loses only **$113.08**. The same asymmetry applies when 12% is a volatility.
- **Settlement-to-settlement is the standard measurement.** "A one standard deviation daily price change of $1.04" means $1.04 from one settlement to the next, not the high/low or open/close range. (Alternatives exist: Parkinson 1980; Garman-Klass 1980; Beckers 1983.) Volatility is most often computed from **logarithmic** price changes — ln(current/previous) — to turn price changes into continuously compounded returns, though the practical difference from percent changes is small.
- **Eurocurrency interest-rate futures are indexed from 100.** LIBOR 7.00% → Eurodollar future at 93.00; Euribor 4.50% → Euribor future at 95.50. Volatility is computed on the **rate**, not the price. Exercise prices must also be indexed from 100, and **the option type reverses**: a listed 93.00 *call* is a 7.00% *put* in interest-rate terms, because the future rising above 93.00 requires rates to fall below 7.00%. A correctly configured model does this transformation automatically. Bonds and notes need no such transform.
- **Premium as slang for implied volatility.** Because implied volatility is derived from price, traders say "premium levels are high" when implied volatility is high by historical standards or relative to recent realized volatility.
- **Implied volatility is the trader's price unit.** A 105 call at 3.60 with 28.50% implied and a 100 call at 5.40 with 27.51% implied: the 100 call costs more in points but is nearly a full volatility point **cheaper**. Traders say "I bought the call at 27.51 percent."
- **Contemporaneity matters.** If the 3.60 print was two hours old and the stock was 99.25 at that time, the implied volatility was actually **26.95%**, not 28.50%.
- **Aggregated implied volatility** from vendors is a weighted average — by volume, by open interest, or most commonly by **weighting the at-the-money options most heavily**.
- **Interval choice barely matters.** Across the S&P 500's 250-day realized volatility 2003–2012 computed from daily, weekly, and four-weekly changes (Figure 6-8), "there is no clear evidence that using one interval rather than another results in consistently higher or lower volatility." A contract volatile day to day is usually equally volatile week to week.
- **Calls and puts at the same strike and expiration have very similar implied volatilities and change by approximately the same amount** when volatility changes — a consequence of **put-call parity** (Chapter 15).

## Mental Models
- **Volatility is the weather.** Historical volatility is climate ("you don't wear a winter coat in Chicago in July"); forecast volatility is the radio forecast; implied volatility is what the people in the street are wearing. When the forecast and the street disagree, you still must dress, weighing local knowledge and, crucially, **the consequences of being wrong** — missing an umbrella matters little if the bus stops at your door, a lot if you walk several blocks.
- **Unlikely is not impossible.** A contract with a forward of 100 and 20% volatility ending the year at 35 is a move of more than three standard deviations — about one chance in 1,500. Fifteen heads in fifteen coin flips is less than one chance in 32,000, and still possible. You cannot condemn a volatility estimate from a single observation; you need a representative sample.
- **Value vs. price.** Future realized volatility determines an option's **value**; implied volatility reflects its **price**. Low implied relative to expected future realized → prefer to buy options; high implied → prefer to sell.

## Anti-patterns
- **Assuming a normal distribution of prices** — it permits negative prices, and it makes a 10%-OTM call and a 10%-OTM put worth the same, which the lognormal assumption denies.
- **Drawing conclusions from a five-day sample.** Five heads in a row proves nothing; fifty does. Most traders want 20, 50, or 100 observations.
- **Continuing to use a volatility inconsistent with observed price changes.** "If we have the wrong volatility, we have the wrong probabilities. And if we have the wrong probabilities, we are generating incorrect theoretical values, thereby defeating the purpose of using a theoretical pricing model in the first place."
- **Computing implied volatility from stale inputs** — an old option price against a current underlying price gives a misleading number.
- **Choosing between two options on implied volatility alone.** If the 100 call implies 27.51% and the 105 call 28.50% and you sell one against a 25% forecast, both are overpriced — but if realized volatility turns out to be 32%, both lose. Judge the consequences of being wrong, not just the ranking.
- **Building strategies with no margin for error.** "No trader will survive very long pursuing strategies based on a future volatility estimate of 20 percent if such a strategy results in a significant loss when volatility actually turns out to be 18 or 22 percent. Given the shifts that occur in volatility, a 2 percentage point margin for error may be no margin for error at all."

## Reference Tables

**Compounding a +12% annual rate on $1,000**

| Rate of payment | Value after one year | Total yield |
|---|---|---|
| 12% once a year | $1,120.00 | 12.00% |
| 6% twice a year | $1,123.60 | 12.36% |
| 3% every three months | $1,125.51 | 12.55% |
| 1% every month | $1,126.83 | 12.68% |
| 12%/52 every week | $1,127.34 | 12.73% |
| 12%/365 every day | $1,127.47 | 12.75% |
| 12% compounded continuously | $1,127.50 | 12.75% |

**Compounding a −12% annual rate on $1,000**

| Rate of payment | Value after one year | Total yield |
|---|---|---|
| −12% once a year | $880.00 | −12.00% |
| −6% twice a year | $883.60 | −11.64% |
| −3% every three months | $885.29 | −11.47% |
| −1% every month | $886.38 | −11.36% |
| −12%/52 every week | $886.80 | −11.32% |
| −12%/365 every day | $886.90 | −11.31% |
| −12% compounded continuously | $886.92 | −11.31% |

**Normal vs. lognormal option values** (forward 100, six months, 30% volatility)

| Contract | Value if distribution is normal | Value if distribution is lognormal |
|---|---|---|
| 90 put | 4.37 | **4.00** |
| 110 call | 4.37 | **4.74** |

**Figure 6-10 — October gold options, July 31, 2012** (October gold futures 1612.4; 8 weeks / 56 days to expiration; interest rate 0.50%)

| | Settlement price | Implied volatility | TV at 14% | TV at 18% | TV at 22% |
|---|---|---|---|---|---|
| 1400 call | 215.2 | 22.36% | 212.37 | 213.13 | 214.98 |
| 1500 call | 122.5 | 19.01% | 116.05 | 121.01 | 127.34 |
| 1600 call | 50.8 | 17.68% | 41.65 | 51.60 | 61.57 |
| 1700 call | 16.1 | 18.42% | 8.08 | 15.28 | 23.49 |
| 1800 call | 5.3 | 20.46% | 0.78 | 3.05 | 7.00 |
| 1400 put | 2.9 | 22.26% | 0.13 | 0.89 | 2.74 |
| 1500 put | 10.2 | 19.02% | 3.74 | 8.70 | 15.02 |
| 1600 put | 38.4 | 17.68% | 29.26 | 39.21 | 49.18 |
| 1700 put | 103.7 | 18.45% | 95.61 | 102.82 | 111.03 |
| 1800 put | 192.8 | 20.50% | 188.23 | 190.51 | 194.46 |

**Figure 6-11 — effect of raising volatility from 14% to 18% (October gold)**

| Strike | TV @ 14% | TV @ 18% | Net change | Percent change |
|---|---|---|---|---|
| 1400 call | 212.37 | 213.13 | 0.76 | <1% |
| 1500 call | 116.05 | 121.01 | 4.96 | 4% |
| **1600 call (ATM)** | 41.65 | 51.60 | **9.95** | 24% |
| 1700 call | 8.08 | 15.28 | 7.20 | 89% |
| **1800 call (OTM)** | 0.78 | 3.05 | 2.27 | **291%** |
| **1400 put (OTM)** | 0.13 | 0.89 | 0.76 | **585%** |
| 1500 put | 3.74 | 8.70 | 4.96 | 133% |
| **1600 put (ATM)** | 29.26 | 39.21 | **9.95** | 34% |
| 1700 put | 95.61 | 102.82 | 7.21 | 8% |
| 1800 put | 188.23 | 190.51 | 2.28 | 1% |

**Figure 6-12 / 6-13 — December gold options** (December gold futures 1614.6; 17 weeks / 119 days; 0.50%)

| | Settlement | Implied vol | TV @ 14% | TV @ 18% | TV @ 22% | Net change 14→18 | Percent change |
|---|---|---|---|---|---|---|---|
| 1400 call | 226.3 | 22.00% | 216.03 | 220.06 | 226.30 | 4.03 | 2% |
| 1500 call | 142.7 | 20.17% | 126.41 | 136.59 | 148.05 | 10.18 | 8% |
| 1600 call | 78.8 | 19.51% | 58.78 | 73.31 | 87.86 | **14.53** | 25% |
| 1700 call | 40.0 | 19.93% | 20.71 | 33.52 | 47.10 | 12.81 | 62% |
| 1800 call | 20.4 | 21.07% | 5.44 | 13.03 | 22.84 | 7.59 | 140% |
| 1400 put | 11.9 | 21.92% | 1.78 | 5.81 | 12.04 | 4.03 | **226%** |
| 1500 put | 28.2 | 20.14% | 12.00 | 22.18 | 33.64 | 10.18 | 85% |
| 1600 put | 64.2 | 19.50% | 44.20 | 58.74 | 73.25 | **14.54** | 33% |
| 1700 put | 125.3 | 19.94% | 105.98 | 118.79 | 132.36 | 12.81 | 12% |
| 1800 put | 205.5 | 21.07% | 190.54 | 198.13 | 207.94 | 7.59 | 4% |

Note the confirmation of principle 3: for every strike, the December (long-term) change exceeds the October (short-term) change — 14.53 vs 9.95 at the 1600 call.

## Worked Example

**A. Reading a distribution.** In the quincunx of Figure 6-2 with 16 troughs (0–15), the mean is 7.50 and the standard deviation 3.00 (exactly 7.49 and 3.02, rounded). One standard deviation spans 7½ ± 3 = 4½ to 10½, i.e. troughs **5 through 10** — about two of every three balls. So the chance of landing below 5 or above 10 is about **1 in 3** (exactly 100% − 68.3% = 31.7%).

Now price a wager: 30-to-1 odds against landing specifically in trough 14 **or** 15. Two standard deviations = 7½ ± 6 = 1½ to 13½, so 14 and 15 lie outside 2 s.d. — a 1-in-20 event. But "outside 2 s.d." also includes troughs 0 and 1, and the distribution is symmetric, so the upper tail alone is **half of 1 in 20 = about 1 chance in 40**. At 30-to-1, the bet is a **bad** one.

**B. Turning an annual volatility into daily and weekly ranges.** A contract with a one-year forward of 100 and 20% volatility: in one year, 68% chance of trading between **80 and 120**, 95% between **60 and 140**, 99.7% between **40 and 160**.

For a *stock* at $100 with the same 20% volatility, first compute the mean — the forward price. At 8% with no dividends, the one-year forward is **$108**, so one standard deviation is 20% × $108 = **$21.60**: 68% between **$86.40 and $129.60**, 95% between **$64.80 and $151.20**, 99.7% between **$43.20 and $172.80**.

Daily and weekly, from the forward-100 contract at 20%: daily s.d. = 20%/16 = 1¼%, so **1.25** points — a move of 1.25 or less on two days in three, of 2.50 or less on 19 days in 20. Weekly s.d. = 20%/7.2 ≈ 2.78%, so **2.78** points — 2.78 or less two weeks in three, 5.56 or less 19 weeks in 20. (For short horizons traders center the distribution on the current price rather than computing a one-day forward.)

**C. Testing a volatility against observed prices.** A $45 stock with 37% annual volatility:
- One day: $45 × 37%/16 ≈ **$1.04**. One s.d. range ≈ **$43.96–$46.04**; two s.d. ≈ **$42.92–$47.08**.
- One week: $45 × 37%/7.2 ≈ **$2.31**. One s.d. ≈ **$42.69–$47.31**; two s.d. ≈ **$40.38–$49.62**.

Observed daily settlement changes over five days: **+$0.98, −$0.65, −$0.70, +$0.25, −$0.85**. At 37% volatility we expect a move greater than $1.04 about one day in three — so one and possibly two such days in five. **Not one** occurred. Conclusion: these changes are not consistent with 37% volatility. Before acting, check for a holiday week or other abnormality; absent one, use a lower number. (The volatility actually implied by those five changes is **27.8%**.) Five days is a small sample — prefer 20, 50, or 100.

**D. Solving for implied volatility (Figure 6-9).** A three-month European 105 call on a non-dividend stock: stock 98.50, three-month rate 6.00%, volatility estimate 25% → theoretical value **2.94**. The market trades it at **3.60**. Which input is wrong?
- Exercise price and time to expiration are fixed by contract.
- To get 3.60 from the stock price alone you would need the stock at **100.16** — far outside any realistic bid-ask spread.
- To get 3.60 from the interest rate alone you would need to raise it from 6.00% to **13.30%**.
- Therefore, volatility. Raising the volatility input until the model returns 3.60 gives **28.50%** — the implied volatility.

The option is 0.66 overpriced in points, or **3.50 volatility points** overpriced (28.50% vs. the 25% estimate) — which is usually the more useful statement.

**E. Interest-rate contract scaling.** A Eurodollar future at 93.00 (i.e. a 7.00% rate) with 26% **rate** volatility: daily s.d. = (100 − 93) × 26%/16 ≈ **0.11**; weekly s.d. = (100 − 93) × 26%/7.2 ≈ **0.25**.

## Key Takeaways
- Volatility = standard deviation, annualized, expressed as a percent; the forward price is the mean.
- Scale by √t: divide the annual number by 16 for a day, by 7.2 for a week.
- Use the 2/3, 19/20, 369/370 approximations to sanity-check whether observed price changes match your volatility assumption. A two-standard-deviation daily move should appear roughly once a month.
- Higher volatility raises the value of *all* options, calls and puts, because an option's expected value counts only in-the-money outcomes.
- The Black-Scholes lognormal assumption bounds prices at zero and makes an OTM call worth more than the equally-OTM put — a prediction the market frequently violates, sometimes because the market disagrees with the model.
- Compare *expected future realized* volatility against *implied* volatility: buy options when implied is low, sell when it is high — with an explicit margin for error, because a 2-point volatility error is common.
- Volatility changes hit ATM options hardest in points, OTM options hardest in percent, and long-dated options harder than short-dated.

## Connects To
- [Chapter 5 — Theoretical Pricing Models](ch05-theoretical-pricing-models.md): supplies the mean (forward price) and the probability framework this chapter fills in.
- [Chapter 7 — Risk Measurement I](ch07-risk-measurement-i.md): vega quantifies the sensitivity principles listed here.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): put-call parity, which explains why same-strike calls and puts share an implied volatility.
- [Chapter 18 — The Black-Scholes Model](ch18-black-scholes-model.md): the continuous-time, lognormal machinery.
- [Chapter 20 — Volatility Revisited](ch20-volatility-revisited.md): computing historical volatility, forecasting, and implied volatility as a predictor.
- [Chapter 23 — Models and the Real World](ch23-models-and-the-real-world.md): where the lognormal assumption breaks.
- [Chapter 24 — Volatility Skews](ch24-volatility-skews.md): why implied volatility varies across exercise prices, as Figures 6-10 and 6-12 show.
