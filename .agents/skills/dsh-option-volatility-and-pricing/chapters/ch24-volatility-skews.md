# Chapter 24 — Volatility Skews

## Core Idea
In a perfect Black-Scholes world every option on the same underlying with the same expiration would carry the **same implied volatility** — the underlying can only have one realized volatility over the period. Yet "this almost never happens in any market." Rather than abandon the model, treat the **volatility skew as an additional input**: it is the market telling you what weaknesses *it* believes the model has. "Traders have found that using a model, even a flawed one, is usually better than using no model at all."

## Frameworks Introduced

**The three skew shapes** (Figure 24-2):

| Skew | Shape | Cause | Typical market |
|---|---|---|---|
| **Investment skew** | Lower exercise prices inflated | Buying pressure on low strikes (protective puts), selling pressure on high strikes (covered calls) | Stock and stock index markets |
| **Demand skew (commodity skew)** | Higher exercise prices inflated | End users buy protective calls at higher strikes and sell covered puts at lower strikes | Agricultural, energy |
| **Balanced skew** | Both tails inflated, roughly symmetric around the underlying | Longs and shorts equally worried | Currencies |

**Why hedging produces the investment skew.** An investor long stock buying a protective put chooses an **out-of-the-money** strike — "if the investor is so worried about a downward move that he needs the protection afforded by an in-the-money put, he ought to simply sell the stock." An investor selling a covered call chooses a **higher** strike, because an in-the-money call would have the stock called away immediately, eliminating the upside he holds the stock for. Result: buying pressure at low strikes, selling pressure at high strikes.

**Traders say "the skew is to the puts,"** but put-call parity means an inflated put implies an inflated call at the same strike — "so perhaps it is more accurate to say that the **skew is to the downside**."

**Why the model's own weakness reinforces it.** Stock markets get more volatile falling and less volatile rising, and vega is highest at the money. With stock at 100, a fall makes the **95 put** more at the money *and* the market more volatile — a double benefit. A rise makes the **105 call** more at the money but the market **less** volatile. So the 95 put naturally carries a higher implied volatility and the 105 call a lower one — exactly an investment skew.

**Modeling the skew as a function.** Fit `f(x) = y`, where y is implied volatility at exercise price x. Many traders use a polynomial `a + bx + cx² + dx³ + …`. In a simple second-degree model:
```
y = a + bx + cx²
```
- **a** = the **base volatility**, usually the at-the-money implied volatility
- **b** = the **skewness** — positive or negative depending on which side is inflated
- **c** = the **kurtosis** — "almost always positive" for exchange-traded markets, whose distributions always have fat tails

To make units workable, insert a multiplier: `y = a + 0.001bx + cx²` lets a one-unit change in b mean 0.001.

**Two ways the skew can move with the market:**
- **Sticky-strike** — the skew's location and shape are fixed; each strike keeps its implied volatility whatever happens. "Not consistent with the observed dynamics of the marketplace."
- **Floating skew** — the whole skew shifts **horizontally** with the underlying price (up five points → shift right five points) and **vertically** with implied volatility (down two points → shift down two points).

**Recalibrating the axes so skews are comparable.** All moves are relative to price and time — a 10.00 move is 10% at an underlying of 100 but 5% at 200; a 10% move over a week is bigger than the same move over a month.
- **x axis, step 1 — moneyness:** the 90 strike with the underlying at 100 has moneyness 0.90, the same as the 180 strike with the underlying at 200.
- **x axis, step 2 — logarithmic:** use `ln(X/S)`, consistent with lognormal prices.
- **x axis, step 3 — standard deviations** (the **sticky-delta** skew, since delta approximates moneyness):
```
ln(X/S) / (σ√t)
```
  with an exactly at-the-money option at 0. (More correctly, use the **forward price F** instead of S, putting the at-the-forward option at 0.)
- **y axis, step 1 — difference from at-the-money implied volatility.** ATM 20%, strike at 25% → y = 20 − 25 = −5; strike at 18% → y = 20 − 18 = 2.
- **y axis, step 2 — percent of at-the-money implied volatility**, which survives a doubling of overall volatility. 25/20 = **125%**; 18/20 = **90%**; ATM itself = **100%**.

**Skewness and kurtosis as tradable quantities:**
- **Skewness = the tilt** — how much low-strike implied volatilities differ from high-strike ones. A distribution with a longer **left** tail (negative skewness) means greater chance of large down moves → greater demand for lower strikes. Longer **right** tail (positive skewness) → greater demand for higher strikes.
- **Kurtosis = the curvature** — how much *both* wings are inflated relative to the at-the-money. Fat tails mean greater chance of large moves either way → greater demand for out-of-the-money options.

**Skew sensitivities.** If raising b by one unit lowers the option by 0.15, its **skewness sensitivity is −0.15**. If raising c by one unit raises the option by 0.08, its **kurtosis sensitivity is 0.08**. In most skew models the at-the-money strike is the **pivot point**, so an exactly at-the-money option has **zero** skewness and kurtosis sensitivity. Raising skewness lifts high-strike volatilities and lowers low-strike ones, so **higher strikes have positive skewness sensitivity, lower strikes negative**. Raising kurtosis lifts both wings, so **any option not exactly at the money has positive kurtosis sensitivity**.

**Benchmarks.** In many skew models, **−25 delta puts and +25 delta calls** have the greatest **skewness** sensitivity — hence the common measure of skewness is the difference between their implied volatilities. There is no equivalent benchmark for kurtosis, but **±5 delta** options tend to have the greatest **kurtosis** sensitivity.

**The volatility surface.** Combine the **term structure** across expirations with the **skew** across exercise prices. "The more exercise prices and expiration dates that are available, the more accurate will be the volatility surface."

## Skew trading strategies

**Risk reversal (skewness trade).** Buy one wing and sell the other, then neutralize the delta with the underlying. Most commonly done with **25 delta calls and −25 delta puts**, the options most sensitive to the slope.

| Skew type | Expect flatter | Expect steeper |
|---|---|---|
| **Investment skew** | Sell lower exercise prices, buy higher | Buy lower exercise prices, sell higher |
| **Demand skew** | Sell higher exercise prices, buy lower | Buy higher exercise prices, sell lower |

Typical constructions (underlying near 100; deltas in parentheses):
```
+10 June 95 puts (−25)          −30 December 90 puts (−15)
−10 June 105 calls (+25)        +30 December 110 calls (+15)
 +5 underlying contracts         −9 underlying contracts
```
Equal deltas are not required — **more commonly the calls and puts are chosen with equal vegas**, making the position vega neutral at inception and therefore sensitive to the *slope* of the skew rather than the level of implied volatility. (A vega-neutral risk reversal tends to be gamma neutral too, though not always; the trader must decide which matters more.)

**Dragonfly (kurtosis trade).** Expect kurtosis to **rise** → **buy strangles** (both wings). Expect it to **fall** → **sell strangles**. But a strangle carries a pronounced vega, so to isolate kurtosis, offset it with **at-the-money straddles**, which are kurtosis-neutral. Typical positions (vegas in parentheses):
```
Long strangles:  +35 June 90 puts (0.11)      Short strangles: −26 December 80 puts (0.08)
                 +35 June 110 calls (0.12)                     −26 December 120 calls (0.09)
Short straddles: −20 June 100 calls (0.20)    Long straddles:  +10 December 100 calls (0.22)
                 −20 June 100 puts (0.20)                      +10 December 100 puts (0.22)
```
When the strangle has exactly half the vega of the straddle, vega neutrality means **two strangles for each straddle** — a **2 × 1 ratio**, called a **dragonfly**.

**Trading skew across expirations.** If two expirations' skews are mispriced against each other, take opposing skew positions: **buy out-of-the-money June puts / sell out-of-the-money March puts**, and **sell out-of-the-money June calls / buy out-of-the-money March calls** — effectively buying put calendars and selling call calendars. Choose calendars with roughly equal vega to stay vega neutral, and hedge residual deltas in the underlying.

**Combining skew with a term-structure view.** If June implied volatility also looks **low** versus March, buy **put** calendar spreads only — they capture both the skew edge and the volatility edge. **Avoid the call calendars**, where the two effects cancel: "The June calls are too expensive with respect to skew, but the March calls are too expensive with respect to implied volatility." If June implied volatility looks **high** versus March, **sell call calendar spreads**, since June calls are then expensive on both counts. The same logic applies to kurtosis across months: buy June strangles and sell March strangles, choosing equal vegas — and if June implied volatility is also low, the June options are cheap on **both** volatility and kurtosis.

## Key Concepts
- **The skew as a model input** (Figure 24-11) sits alongside time, exercise price, underlying price, interest rate, and volatility — but is entered as a **formula**, not a single number.
- **The passage of time steepens an investment skew.** A 90 put with the underlying at 100 moves relatively further out of the money as time passes, "climbing the skew" to a higher implied volatility. Higher strikes may rise, fall, or stay flat depending on the shape and position along the skew.
- **Financial vs. commodity skew stability.** FTSE 100 skews are similar across expirations; **wheat** skews show more curvature, mildly inflated higher strikes, and "less consistency across different expiration months," probably from **seasonal volatility** or short-term supply and demand imbalances.
- **Skew models can generate illogical values.** Reducing volatility pushes an out-of-the-money put further out in standard-deviation terms, which in an investment skew *raises* its implied volatility. "If the skew is sufficiently steep, the increase in volatility may in fact cause the theoretical value of the put to rise" — absurd, since lowering volatility should lower every option's value. Adjustments are required.
- **Skewed (adjusted) risk measures.** With a floating skew, an underlying move changes each strike's implied volatility as well, so all the Greeks shift. Practical advice: "For many traders, it may be best to keep things simple, perhaps using a skew model to generate theoretical values while using a traditional model to calculate the delta, gamma, theta, and vega." Active traders with large positions need accurate skewed sensitivities, which is what financial engineers at professional firms build — "but even the most sophisticated model is unlikely to generate values that exactly model option prices under all market conditions."
- **Shifting volatility breaks delta neutrality in the real world.** An at-the-money straddle is delta neutral in theory (call +50, put −50). But in a **stock index** market, where falling prices bring higher volatility, the trader actually **prefers a decline** — the position is **delta negative** in reality. In a **commodity** market, where rising prices bring higher volatility, the same position is **delta positive**. "In neither case is the position truly delta neutral," though the true delta depends on how fast volatility responds.
- **Direct vs. inverse volatility relationships.** Demand-skew markets (agricultural, energy) show a **direct** relationship — underlying up, implied volatility up. Investment-skew markets (stock, stock index) show an **inverse** one.

## Implied distributions from butterfly prices
- At expiration a butterfly is worth **0** at or outside its wings and the **strike interval** at its body. Buy **every** five-point butterfly across a full strike ladder from 0 to infinity and the total is always exactly **5.00**, whatever the underlying does — so the summed prices must equal 5.00 (or its present value, under stock-type settlement with nonzero rates).
- Therefore, assuming the only possible expiration prices are the exercise prices, **the probability of any price = that butterfly's price ÷ the strike interval**. A 75/80/85 butterfly at 0.15 implies a 3% chance of 80; a 90/95/100 butterfly at 0.50 implies a 10% chance of 95.
- Narrowing the increment (5.00 → 2.00 → 1.00 → 0.50 → 0.10) makes the distribution approach continuous.
- **Comparing implied to lognormal.** Deriving butterfly prices from the FTSE 100 skew and from a constant volatility, the market implies, relative to a lognormal distribution: (1) a **greater** probability of a small-to-intermediate **upward** move, (2) a **greater** probability of a **large downward** move, (3) a **smaller** probability of a small-to-intermediate **downward** move, and (4) a **smaller** probability of a **large upward** move. Typical of stock index markets, and largely consistent with the S&P 500 histogram of Chapter 23 — except the histogram also shows more big *upward* moves, which the implied distribution does not.
- The **wheat** implied distribution (January 27, 2012, futures at 661.75) conforms far more closely to lognormal, though still with more small moves, slightly fewer intermediate upward moves, and slightly more large upward moves.
- "It can often be useful for a trader to compare his opinions about a probability distribution with that implied by prices in the marketplace. If there is a clear disagreement, it may point the way to a potentially profitable strategy."

## Anti-patterns
- **Chasing a "better" model.** "A model might be better in the sense that it gives slightly more accurate theoretical values. But if the model is extremely complex and difficult to use, or if it requires additional inputs of which a trader cannot always be certain, then the model may merely substitute one set of problems for another."
- **Assuming a sticky-strike skew.** Real skews float with price and volatility.
- **Comparing skews across expirations on a raw exercise-price axis.** Recalibrate to standard deviations and to percent of at-the-money volatility first.
- **Trusting a steep skew model at low volatilities** without guarding against the option value rising as volatility falls.
- **Trading skew unhedged.** A long-call/short-put skew position is a big directional bet; neutralize the delta with the underlying.
- **Trading kurtosis with naked strangles.** A correct kurtosis view can still lose to an overall implied volatility move; offset the vega with at-the-money straddles.
- **Calling an at-the-money straddle delta neutral** in a market where volatility moves with price.

## Reference Tables

**The three skews, summarized**

| Feature | Investment skew | Demand skew | Balanced skew |
|---|---|---|---|
| Inflated strikes | Lower | Higher | Both |
| Hedging driver | Protective puts bought low, covered calls sold high | Protective calls bought high, covered puts sold low | Two-sided currency hedging |
| Underlying/volatility relationship | Inverse (price down, volatility up) | Direct (price up, volatility up) | Neither dominates |
| Example markets | Stocks, stock indexes | Agriculturals, energy | Currencies |

**The skew polynomial, decoded**

| Parameter | Meaning | Sign for exchange-traded markets |
|---|---|---|
| a | Base (at-the-money) volatility | Positive |
| b | Skewness — tilt | Either; negative for an investment skew |
| c | Kurtosis — curvature | **Almost always positive** (fat tails) |

**Skewness and kurtosis sensitivities**

| Option | Skewness sensitivity | Kurtosis sensitivity |
|---|---|---|
| Exactly at the money (the pivot) | **0** | **0** |
| Higher exercise prices | Positive | Positive |
| Lower exercise prices | Negative | Positive |
| Maximum sensitivity at | ±25 delta | ±5 delta |

**Figure 24-18 — Butterfly values and implied probabilities** (approximating Black-Scholes at underlying 100, three months, 20% volatility, 0% interest)

| Exercise price | 75 | 80 | 85 | 90 | 95 | 100 | 105 | 110 | 115 | 120 | 125 | 130 | 135 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Call value | 25.00 | 20.04 | 15.20 | 10.71 | 6.88 | 3.98 | 2.06 | 0.95 | 0.39 | 0.15 | 0.05 | 0.01 | 0 |
| Butterfly value | 0.04 | 0.12 | 0.35 | 0.66 | 0.93 | **0.98** | 0.81 | 0.55 | 0.32 | 0.14 | 0.06 | 0.03 | 0.01 |
| Price probability | 0.008 | 0.024 | 0.070 | 0.132 | 0.186 | **0.196** | 0.162 | 0.110 | 0.064 | 0.028 | 0.012 | 0.006 | 0.002 |

Butterfly values sum to **5.00**; probabilities sum to **1.00**. The resulting distribution is **skewed to the right** — as expected, since the inputs came from a lognormal Black-Scholes model.

**Market data referenced**

| Chart | Market | Date | Underlying level |
|---|---|---|---|
| Figures 24-1, 24-3, 24-5, 24-6, 24-7, 24-12, 24-21 | FTSE 100 Index options (LIFFE) | March 16, 2012 | 5,965.58 |
| Figures 24-8, 24-13, 24-22 | Wheat options | January 27, 2012 | Front-month futures 647; three-month futures 661.75 |

FTSE 100 expirations charted: April (1 month), May (2 months), June (3 months), September (6 months), December (9 months). Wheat expirations: March (1 month), May (3 months), July (5 months), September (7 months), December (10 months).

## Worked Example

**A. Reading and normalizing a skew.** Figure 24-1 plots June 2012 FTSE 100 implied volatilities across exercise prices from 3,000 to 7,000 on March 16, 2012, computed from bid-ask midpoints with Black-Scholes. Implied volatility runs from roughly 45% at the lowest strikes down toward 15% at the highest — a pronounced **investment skew**. Since the FTSE can have only one realized volatility over the 13 weeks to June expiration, and everyone can see the same exercise price, time, underlying, and rate, this dispersion is a statement about the model, not about the index.

To compare it with the April, May, September, and December skews, rescale:
- x axis: replace each exercise price with `ln(X/S) / (σ√t)` — its distance from at the money in **standard deviations** (Figure 24-6). Left unadjusted (Figure 24-5), the one-month April skew looks far steeper than the nine-month December skew purely because the same strike is many more standard deviations away with less time.
- y axis: replace each implied volatility with its **percent of the at-the-money implied volatility** (Figure 24-7), so that a doubling of overall volatility does not distort the comparison.

After both adjustments the five FTSE skews are broadly similar in shape. Doing the same for wheat (Figure 24-8) produces skews with more **curvature**, mildly inflated **higher** strikes, and noticeably **less consistency** across expirations.

**B. Why time steepens the skew.** A 90 put with the underlying at 100, in an investment skew. As time passes, in relative (standard-deviation) terms the put moves **further out of the money**, and in an investment skew a further-out put carries a **higher** implied volatility — it "climbs the skew." So without any change in market prices, the observed skew appears more severe as expiration approaches.

**C. Computing a skewed delta.** An out-of-the-money put has a raw delta of **−20** and a vega of **0.10**. The underlying rises 1.00. Ignoring gamma, the naive expectation is a 0.20 decline in value. But under a floating investment skew the put has moved further out of the money, so its implied volatility rises — say by **0.5 percentage points**:
```
value gain from the volatility rise = 0.5 × 0.10 = 0.05
net change = −0.20 + 0.05 = −0.15
```
The put's **skewed (adjusted) delta is −15**, not −20.

**D. Deriving the market's implied distribution from butterflies.** Take the strike ladder 75, 80, 85, …, 135 and buy every five-point butterfly. Whatever the underlying does at expiration, the portfolio is worth exactly **5.00**, so the butterfly prices must sum to 5.00.

Using the call values in Figure 24-18: the 75/80/85 butterfly costs `25.00 − 2(20.04) + 15.20 = 0.12`, implying a probability of the underlying finishing at 80 of `0.12 / 5.00 = **0.024**`. The 95/100/105 butterfly costs `6.88 − 2(3.98) + 2.06 = 0.98`, implying `0.98 / 5.00 = **0.196**` — a 19.6% chance of finishing at 100.

Reduce the increment from 5.00 to **0.10** and the discrete histogram becomes a smooth curve (Figure 24-20).

**E. Comparing the market's distribution to lognormal.** Apply the same butterfly method twice to FTSE 100 June options on March 16, 2012 — once with prices generated by the fitted **skew**, once with a **constant volatility** across all strikes. Overlaying the two (Figure 24-21) shows what the market believes that Black-Scholes does not:

| Region | Market's implied probability vs. lognormal |
|---|---|
| Small-to-intermediate **up** move | **Greater** |
| Large **down** move | **Greater** |
| Small-to-intermediate **down** move | **Smaller** |
| Large **up** move | **Smaller** |

Three of the four are corroborated by the S&P 500 histogram in Chapter 23 — more small moves, more large down moves, fewer intermediate down moves. The one discrepancy: the histogram *also* shows more large **up** moves, which the option market is **not** pricing in.

**F. Constructing a pure kurtosis position.** Buy the June 90/110 strangle in 35 lots — vegas 0.11 and 0.12, total `35 × 0.23 = **8.05**`. That is a large positive vega, so a correct kurtosis view could still lose to a general drop in implied volatility. Offset with at-the-money straddles, which are kurtosis-neutral: sell 20 June 100 straddles at vegas 0.20 and 0.20, total `20 × 0.40 = **8.00**`. The vega now nets to approximately zero and the position responds almost purely to **curvature**.

Note the ratio: each strangle carries 0.23 of vega and each straddle 0.40 — close to the 2 × 1 case where the strangle is exactly half the straddle's vega, giving the classic **dragonfly**.

## Key Takeaways
- Different implied volatilities across strikes prove the market does not believe Black-Scholes; the skew is the market's own correction, usable as a model input.
- Investment skews (stock, index) inflate low strikes; demand skews (commodities) inflate high strikes; currencies are roughly balanced.
- Say "the skew is to the downside," not "to the puts" — put-call parity ties same-strike calls and puts together.
- Fit the skew with a polynomial: a = level, b = skewness (tilt), c = kurtosis (curvature, essentially always positive).
- Skews float rather than stick; to compare them across expirations, rescale x to standard deviations and y to percent of at-the-money implied volatility.
- Skewness sensitivity peaks at ±25 delta (the risk-reversal benchmark); kurtosis sensitivity at ±5 delta; at-the-money options are the pivot with zero sensitivity to both.
- Trade skewness with a delta-hedged, vega-neutral risk reversal; trade kurtosis with a strangle offset by at-the-money straddles (a dragonfly at 2 × 1).
- Combine skew views with term-structure views across expirations, and choose the calendar spread where both effects push the same way.
- Butterfly prices across a full strike ladder sum to the strike interval and therefore reveal the market's implied probability distribution — compare it with your own.

## Connects To
- [Chapter 23 — Models and the Real World](ch23-models-and-the-real-world.md): the model weaknesses the skew is compensating for, and the definitions of skewness and kurtosis.
- [Chapter 17 — Hedging with Options](ch17-hedging-with-options.md): the protective put and covered call behavior that creates the investment skew.
- [Chapter 20 — Volatility Revisited](ch20-volatility-revisited.md): the term structure that combines with the skew to form a volatility surface.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): straddles, strangles, butterflies, and calendar spreads, the instruments used here.
- [Chapter 21 — Position Analysis](ch21-position-analysis.md): the risk reversal, whose risk characteristics are analyzed there.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): put-call parity, which links call and put implied volatilities at a strike.
- [Chapter 6 — Volatility](ch06-volatility.md): the gold option tables where implied volatility already varied across strikes.
