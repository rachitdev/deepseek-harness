# Chapter 20 — Volatility Revisited

## Core Idea
> **The longer an option position is held, the more important is the realized volatility of the underlying contract and the less important is the implied volatility. If a position is held to expiration, realized volatility is the only consideration.**

Implied volatility still matters — it determines interim cash flows, capital requirements, and whether you can exit — but value ultimately comes from realized volatility. This chapter is about estimating it: how to calculate historical volatility, what volatility's own statistical character is, how to forecast it, how good implied volatility is as a predictor, and how to read the **term structure** of implied volatility for relative mispricings.

## Frameworks Introduced

**Calculating historical volatility.** Two standard deviation formulas differ only in the denominator:
```
population standard deviation:  √[ Σ(xᵢ − μ)² / n ]
sample standard deviation:      √[ Σ(xᵢ − μ)² / (n − 1) ]
```
A limited sample is likely to miss extreme observations from the wider population, so its standard deviation is biased low; dividing by **n − 1** corrects it. Since historical volatility is used to estimate a *future* volatility, **historical volatility is normally computed with the sample standard deviation**.

The data points xᵢ are price returns — either percent change `(pₙ − pₙ₋₁)/pₙ` or, more commonly, the **logarithmic change** `ln(pₙ/pₙ₋₁)`.

**The zero-mean assumption.** μ is the average of all returns, so a contract that rises exactly 1% every day for 10 days has a **volatility of zero** — the change never deviated from the average. "To most traders, this feels wrong." Hence **most historical volatility calculations assume μ = 0** regardless of the actual mean.

**Alternative estimators using intraday data.**

*Extreme-value method (Parkinson, 1980)* — uses the high and low of each 24-hour period:
```
                   √{ [1 / (2n·ln 2)] · Σᵢ [ln(hᵢ/lᵢ)]² }
annualized vol =  ───────────────────────────────────────
                                  √t
```
*Garman-Klass (1980)* — adds the open and close:
```
                   √{ [1/(2n)]·Σ[ln(hᵢ/lᵢ)]² − [(2·ln2 − 1)/n]·Σ[ln(cᵢ/oᵢ)]² }
annualized vol =  ──────────────────────────────────────────────────────────────
                                          √t
```
Both, like close-to-close, are annualized by dividing by √t.

**Weighted-average volatility forecasting.** Given historical volatilities over several windows, form a weighted average — with weights chosen so that **the window closest in length to the option's life gets the most weight** (the serial-correlation logic).

**Exponentially weighted moving average (EWMA).** Estimated variance over the next period:
```
σ² = α₁r₁² + α₂r₂² + … + αₙ₋₁rₙ₋₁² + αₙrₙ²
```
subject to `Σαᵢ = 1.00` and `αₙ > αₙ₋₁`. With a decay parameter λ between 0 and 1:
```
αᵢ = λ^(i−1)(1 − λ) / (1 − λⁿ)
```
Lowering λ discounts older returns more heavily; raising λ flattens the weights toward 1/n. **A common choice in risk-management programs is λ ≈ 0.94.** EWMA ignores two things: the correlation between successive returns, and mean reversion.

**ARCH / GARCH.** Robert Engle's **autoregressive conditional heteroskedasticity** model (1982; Nobel Prize 2003) was extended into the **GARCH** family. A GARCH model has three components: (1) a volatility estimate such as EWMA, (2) a **correlation component** — large returns tend to follow large returns, small follow small, and (3) a **mean-reversion component** specifying how fast volatility returns to its mean.

**Forward volatility.** Given a short-term implied σ₁ to t₁ and a long-term implied σ₂ to t₂, the volatility the market implies for the interval between them:
```
        √[ (σ₂²·t₂) − (σ₁²·t₁) ]
σ_f =  ─────────────────────────
              √(t₂ − t₁)
```
because **variance is directly proportional to time**: `σ_f² × (t₂ − t₁) = (σ₂² × t₂) − (σ₁² × t₁)`. Extended over n consecutive periods:
```
σ_{t_n − t_0} = √{ Σᵢ [σ_i² × (tᵢ − tᵢ₋₁)] / (tₙ − t₀) }
```

**Estimating a calendar spread's implied volatility without a model.** Because at-the-money vega is nearly constant in volatility:
```
spread implied volatility ≈ (O₂ − O₁) / (V₂ − V₁)
```
— the spread's price divided by the spread's vega, expressed as a whole number.

**Adjusting vega for the term structure.** A total vega of zero across expirations does **not** mean zero implied-volatility risk, because short-term implied volatility moves faster than long-term. Convert each month's vega into primary-month terms by multiplying by that month's relative rate of change, then sum.

## Key Concepts
- **Interval choice barely matters.** S&P 500 three-month volatility computed on trading days only (~252/year) versus all 365 days (assigning zero to non-trading days) produces graphs that are "almost indistinguishable." Gold volatility from daily versus weekly returns shows similar characteristics, with weekly slightly more jumpy because it has only 13 data points against 91. **Daily returns are used most often** to maximize the number of data points.
- **Parkinson and Garman-Klass can read low for part-day markets.** For the EuroStoxx 50, both intraday methods yield consistently lower volatility than close-to-close, because the index is calculated only from about 9:00 a.m. to 6:50 p.m. European time — barely 10 hours — and volatility during the rest of the day is unobservable. Garman and Klass recommend blending in the close-to-close estimate, weighted by the fraction of the day the market is open (usually giving close-to-close the greater weight, since most markets are closed longer than they are open). They give a precise weighting formula; **"a practical solution might be to simply weight the estimates equally."**
- **Method choice is not the point.** "It is far more important to be able to interpret historical volatility data rather than to worry about the exact method used."
- **Volatility is serial correlated.** Like tomorrow's high temperature, the best guess in the absence of other information is what happened last period.
- **Volatility is mean reverting.** Unlike price, which can trend in one direction for years, volatility gyrates back and forth through an equilibrium. Over 2001–2010 the price of gold rose from under $300 to over $1,400 per ounce and never revisited its 2001 lows — but gold volatility, ranging from **9% to over 40%**, "always seemed to return eventually to the **10 to 20 percent** range." The S&P 500's mean sits somewhere between **15 and 20 percent**; the far less volatile Bund's around **5 percent**.
- **The volatility cone / term structure of volatility.** Plotting minimum, maximum, and average realized volatility against the length of the measurement window produces a **conic shape** — wide variation over short periods, narrow over long. For the S&P 500 2001–2010, every two-week window ranged from about **5% to just over 100%** with an average near **18%**; every 300-week window ranged only from **14% to 24%** with an average near **19%**.
- **Long-term volatility is easier to predict than short-term** — counterintuitive, but volatility is an *average* variability, and over long periods large and small fluctuations offset. **But that does not make long-term options easier to value**, because long-term options have larger vegas: "the effect of a two or three percentage point volatility error on a long-term option may be greater than a five or six percentage point error on a short-term option."
- **Volatility trends, but is not price.** Gold volatility trended down from early 2004 to mid-2005, up sharply to mid-2006, and stepped up through 2007–2008 to over 40%. It is not unreasonable to apply technical-analysis ideas to volatility charts — but "if a trader tries to apply exactly the same rules of technical analysis to volatility analysis, he is likely to find that in some cases the rules have no relevance and that in other cases the rules must be modified."
- **Implied volatility lags realized volatility.** Comparing S&P 500 three-month and twelve-month *future* realized volatility against rolling at-the-money implied volatility 2002–2010, "the volatility of the S&P 500 Index tends to **lead** the implied volatility" — implied rose after the index became volatile in 2008 and fell after it calmed in 2009. The lag is even more pronounced over 12 months.
- **Under normal conditions implied volatility is too high — options tend to be overpriced.** Over much of 2002–2010 implied volatility predicted a future volatility too high by up to **10 percentage points**. Natenberg's explanations: (1) **insurance economics** — buyers accept an above-value price for the rare occasions when volatility explodes, just as insurance buyers accept a premium above fair value; (2) the market maker's **cost of replicating the option through dynamic hedging**, passed on to the customer; (3) **weaknesses in the pricing model** from which implied volatility is derived. "Taken together, these factors may in fact justify the seemingly inflated prices of options in the marketplace."
- **The exceptions are enormous.** On **September 8, 2008**, three-month implied volatility was **22%** against a realized outcome of **72%** — too low by 50 points. On **November 20, 2008**, implied was **65%** against a realized **45%** — too high by 20 points.
- **Term structure of implied volatility is driven by mean reversion.** If March implied rises from 25% to 30%, June and September will rise less — perhaps to 28% and 26% — because distant expirations stay closer to the mean. Falling volatility works the same way in reverse: March to 20% might leave June at 22% and September at 24%.
- **The front month often disconnects.** "Implied volatility in the front month can be unstable, especially as expiration approaches. It often changes in ways that are inconsistent with the term structure of other expiration months." Many traders evaluate the front month **separately** and apply the term-structure model to everything else, picking a different primary month.
- **A term-structure model needs at least three inputs:** a **primary month** against which others are compared, a **mean volatility** to which implied reverts, and a **"whippiness" factor** specifying how other expirations move relative to the primary month. Models are often "home grown." The primary month is usually the front month — except where activity concentrates elsewhere, as in agricultural markets around the planting or harvesting calendar.
- **Seasonal volatility.** Summer expirations in agricultural markets carry higher implied volatility year-round because of heat and drought risk. Energy markets price winter heating and summer cooling demand. In natural gas during 2009, **October options consistently traded at inflated implied volatility** — the highest average of any month — because the Atlantic hurricane season runs roughly early June to late November, peaking in August–September, and U.S. natural gas operations are concentrated on the northern Gulf of Mexico coast. October options, expiring toward the end of September, capture the height of that season.
- **Reading the calendar-spread implied volatility overlay.** When the term-structure graph is **downward sloping**, all calendar spread implied volatilities fall **below** it; when **upward sloping**, they fall **above** it. If every implied volatility lies exactly on the best-fit line, the resulting curve is smooth and **no calendar spread is obviously mispriced**. Kinks reveal the mispriced months — "the implied-volatility graph acts as a magnifying glass."
- **Forward volatility does the same job.** Its graph has the same general structure as the calendar-spread implied volatility graph; both highlight mispriced expiration months.
- **Caveat on settlement prices.** "Option settlement prices do not necessarily reflect actual trading activity" — a strategy built off settlement prices may not be executable at those prices.

## Anti-patterns
- **Assuming a rise in implied volatility means a profit.** Time decay can swamp it.
- **Using the true mean in a historical volatility calculation** rather than the zero-mean assumption, and reporting zero volatility for a steadily trending contract.
- **Applying Parkinson or Garman-Klass to a market that trades only part of the day** without blending in close-to-close.
- **Obsessing over which historical-volatility estimator to use.** They rarely differ enough to matter.
- **Treating a zero net vega as zero implied-volatility risk** across multiple expirations.
- **Assuming long-term options are easier to value because long-term volatility is more stable** — their vegas magnify any error.
- **Applying technical-analysis rules to volatility charts unmodified.**
- **Trading a calendar spread off settlement prices** that never traded.

## Reference Tables

**Volatility cones — realized volatility by window length, 2001–2010**

| Contract | 2-week min / max / avg | 300-week min / max / avg | Apparent mean |
|---|---|---|---|
| S&P 500 Index | ~5% / ~100% / ~18% | ~14% / ~24% / ~19% | 15–20% |
| Gold futures | — | — | 10–20% |
| Bund futures | — | — | ~5% |

**Weighted forecasting from four historical windows** (6-week 28%, 12-week 22%, 26-week 19%, 52-week 18%)

| Weighting scheme | Calculation | Forecast |
|---|---|---|
| Equal weights | (28 + 22 + 19 + 18)/4 | **21.75%** |
| Double weight to the 6-week | (40% × 28) + (20% × 22) + (20% × 19) + (20% × 18) | **23.00%** |
| Regressive (40/30/20/10) | (40% × 28) + (30% × 22) + (20% × 19) + (10% × 18) | **23.40%** |
| Centered on 26 weeks — for **5-month** options (15/25/35/25) | (15% × 28) + (25% × 22) + (35% × 19) + (25% × 18) | **20.85%** |
| Centered on 12 weeks — for **3-month** options (25/35/25/15) | (25% × 28) + (35% × 22) + (25% × 19) + (15% × 18) | **22.15%** |

**Implied volatility as a predictor — the extremes of 2008 (S&P 500)**

| Date | Three-month implied volatility | Three-month future realized volatility | Error |
|---|---|---|---|
| September 8, 2008 | 22% | **72%** | implied too low by ~50 points |
| November 20, 2008 | 65% | **45%** | implied too high by ~20 points |

**Term structure mean reversion — illustrative**

| | March | June | September |
|---|---|---|---|
| Starting implied volatility | 25% | 25% | 25% |
| After a rise in underlying volatility | **30%** | 28% | 26% |
| After a fall in underlying volatility | **20%** | 22% | 24% |

**Figure 20-19 — EuroStoxx 50 implied volatilities and calendar spreads, February 5, 2010** (underlying 100, no interest considerations)

| Expiration month | Days to expiration | Implied volatility | Price of the 100 call | Vega of the 100 call |
|---|---|---|---|---|
| February 2010 | 14 | 29.61% | 2.31 | 0.078 |
| March 2010 | 42 | 28.06% | 3.80 | 0.135 |
| April 2010 | 70 | 27.15% | 4.74 | 0.174 |
| June 2010 | 133 | **25.61%** | 6.15 | 0.240 |
| September 2010 | 224 | **25.08%** | 7.82 | 0.311 |
| December 2010 | 315 | 24.71% | 9.14 | 0.368 |
| June 2011 | 497 | 23.99% | 11.13 | 0.461 |
| December 2011 | 679 | 23.80% | 12.89 | 0.537 |

| Calendar spread | Spread price | Spread implied volatility | Spread vega |
|---|---|---|---|
| February / March 2010 | 1.49 | 25.94% | 0.057 |
| March / April 2010 | 0.94 | 24.02% | 0.039 |
| **April / June 2010** | 1.41 | **21.50%** | 0.066 |
| **June / September 2010** | 1.67 | **23.28%** | 0.071 |
| September / December 2010 | 1.32 | 22.70% | 0.071 |
| December 2010 / June 2011 | 1.99 | 21.13% | 0.093 |
| June 2011 / December 2011 | 1.76 | 22.67% | 0.076 |

## Worked Example

**A. Why implied volatility alone does not decide a straddle's fate.**
```
Underlying price   = 100.00
Time to expiration = 8 weeks
Interest rate      = 0
Implied volatility = 20%
```
Buy the 100 straddle at its implied value, **6.25** — approximately delta neutral.

| Scenario | Straddle value | P&L |
|---|---|---|
| Implied volatility jumps **immediately** to 22% | 6.87 | **+0.62** |
| Implied volatility drifts to 22% over **three weeks**, underlying unchanged | 5.43 | **−0.82** |
| Implied volatility falls **immediately** to 18% | 5.62 | **−0.63** |
| Implied falls to 18% **and** the underlying jumps immediately to 105.00 | 7.09 | **+0.84** |
| Implied falls to 18% **and** the underlying drops immediately to 95.00 | 6.87 | **+0.62** |

Rising implied volatility lost money once time decay was allowed to work; falling implied volatility made money once the positive gamma was allowed to work. **Realized volatility dominates the longer you hold.**

**B. Adjusting vega for the term structure.** A position across four expirations:

| | April | June | August | October |
|---|---|---|---|---|
| Time to expiration | 2 months | 4 months | 6 months | 8 months |
| Total vega | +15.00 | −36.00 | −21.00 | +42.00 |

Naive sum: `+15.00 − 36.00 − 21.00 + 42.00 = 0` — apparently no implied-volatility risk. But implied volatilities do not move together.

Suppose the mean volatility is 25% and April implied rises to **28%** (a 3-point move). Under the term-structure model, June rises to **27%** (2 points), August to **26.5%** (1.5 points), October to **26.1%** (1.1 points):
```
(3 × 15.00) − (2 × 36.00) − (1.5 × 21.00) + (1.1 × 42.00) = −12.30
```
A **loss of 12.30**, not zero — and a *fall* in April implied to 22% would produce a **profit of 12.30**. The position much prefers implied volatility to fall.

To express the risk per point of April implied volatility, scale each month's vega by its relative rate of change:
```
June   vega = −36.00 × (2/3)   = −24.12
August vega = −21.00 × (1.5/3) = −10.50
October vega = +42.00 × (1.1/3) = +15.54

adjusted total = +15.00 − 24.12 − 10.50 + 15.54 = −4.08
```
A genuinely **short vega** position of −4.08 per point of April implied volatility.

**C. Finding a mispriced expiration through calendar spread implied volatility.** From the February 5, 2010 EuroStoxx 50 data: February implied is **29.61%** and March **28.06%**; the at-the-money calls are worth **2.31** and **3.80**, so the calendar spread is **1.49**.

Now ask: what single volatility applied to *both* months makes the spread worth exactly 1.49? It must be **below 28.06%**, because at 28.06% March is fairly priced while February is too expensive, making the spread worth more than 1.49. Solving numerically gives **25.94%** — the spread's implied volatility.

Repeat for each successive spread and overlay the results on the term-structure graph. The picture is unambiguous: **the April/June spread implies only 21.50%** while the neighbouring spreads imply 24.02% and 23.28% — a pronounced dip — and **June/September implies 23.28%** against September/December's 22.70%. **June 2010 options are significantly underpriced relative to nearby expirations; September 2010 options are significantly overpriced.**

The trade: **buy the April/June 2010 calendar spread and sell the June/September 2010 calendar spread** — together, a **time butterfly**.

*Quick check without a model.* For February/March: `(3.80 − 2.31) / (0.135 − 0.078) = 1.49 / 0.057 ≈ 26.1` versus the true 25.94% — close enough for a fast judgment on whether a calendar spread is rich or cheap, with error coming from rounding and the slight change in vega with volatility.

**D. Forward volatility as an alternative lens.** Rather than pricing calendar spreads, compute the forward volatility implied between each pair of expirations:
```
σ_f = √[ (σ₂²·t₂ − σ₁²·t₁) / (t₂ − t₁) ]
```
Plotted against the term structure, the forward-volatility graph has **the same general structure as the calendar-spread graph** and serves the same purpose: highlighting the expiration months that are out of line.

## Key Takeaways
- Realized volatility determines value; implied volatility determines price, exit, and cash flow. Time held tilts the balance toward realized.
- Compute historical volatility with the sample standard deviation, log returns, and a zero-mean assumption; the estimator you choose matters far less than how you read the result.
- Volatility is serial correlated and mean reverting; the volatility cone shows short windows varying wildly and long windows converging on the mean.
- Weight historical windows by how closely their length matches the option's life; EWMA (λ ≈ 0.94) and GARCH formalize this.
- Implied volatility lags realized volatility and is typically too high by up to 10 points — the insurance premium, the cost of replication, and model weakness together.
- Short-term implied volatility moves faster than long-term implied volatility, so net vega across expirations must be rescaled before it means anything.
- Calendar spread implied volatilities and forward volatilities both magnify the term structure and expose mispriced expiration months.
- Seasonality (agricultural summers, natural gas Octobers) can permanently distort the term structure.

## Connects To
- [Chapter 6 — Volatility](ch06-volatility.md): the definitions, the weather analogy, and the taxonomy this chapter deepens.
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): why the adjustment process, not implied volatility, decides a held position's P&L.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): calendar spreads and time butterflies, the instruments for trading term-structure mispricings.
- [Chapter 9 — Risk Measurement II](ch09-risk-measurement-ii.md): why at-the-money vega is nearly constant, enabling the spread-implied-volatility shortcut.
- [Chapter 13 — Risk Considerations](ch13-risk-considerations.md): total vega as a risk measure, refined here.
- [Chapter 24 — Volatility Skews](ch24-volatility-skews.md): the other dimension of implied volatility, across strikes rather than expirations.
- [Chapter 25 — Volatility Contracts](ch25-volatility-contracts.md): trading volatility directly.
