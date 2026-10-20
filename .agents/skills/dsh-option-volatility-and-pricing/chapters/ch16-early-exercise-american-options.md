# Chapter 16 — Early Exercise of American Options

## Core Idea
Early exercise is worth something only when **holding the underlying beats holding the option** — which requires either a dividend or a positive cash flow earning interest. "If there are no dividend or interest considerations, there is no value to early exercise," and then `value of an American option = value of a European option`. That is the normal case for options on futures traded outside the United States under futures-type settlement.

## Frameworks Introduced

**Decomposing option value.**
```
Call value = intrinsic value + volatility value + interest value − dividend value
Put  value = intrinsic value + volatility value − interest value + dividend value
```

**The early exercise criteria.**

| Option | Exercise early when… |
|---|---|
| **American call on stock** | dividend value > volatility value + interest value |
| **American put on stock** | interest value > volatility value + dividend value |
| **American option on futures** (stock-type settled) | interest value > volatility value |

In every case the criterion says the same thing: the European value has fallen **below intrinsic value**, so you can collect intrinsic value now by exercising.

**Estimating the components without a model:**
- **Dividend value** = the total dividend expected over the option's life.
- **Interest value** = interest on the exercise price to expiration (`X × r × t`) for stock options; interest on the **intrinsic value** for futures options (`(F − X) × r × t` for calls, `(X − F) × r × t` for puts).
- **Volatility value** ≈ the price of the **companion out-of-the-money option** (same strike, same expiration, opposite type). Justified because put-call parity gives companion options identical vegas, so identical volatility values. (The companion's own interest and dividend value is small because an out-of-the-money option has a small delta and small rho.)
- **One day's volatility value** = the **theta of the companion out-of-the-money option** — the only quantity requiring a pricing model.

**Two-level test.** For early exercise to be worthwhile *at all*, the criteria must hold **over the entire life** of the option. For it to be worthwhile **today**, the criteria must also hold **over the next day**.

**Timing rules that follow:**
- **Stock calls**: exercising early always costs a day of volatility value *and* a day of interest, and you gain nothing until the dividend arrives. Therefore **the only day on which early exercise is ever optimal is the day before the stock goes ex-dividend**. If the stock pays no dividend over the option's life, **never** exercise a call early.
- **Stock puts**: may be exercised at **any time** prior to expiration. The most common day is the day the stock pays the dividend, but earlier can be optimal whenever accumulated interest exceeds the dividend plus volatility value.
- **Futures options** (stock-type settled): exercise when the option's **daily interest exceeds the companion option's theta**.

**The blackout period** for puts:
```
number of days in the blackout period = dividend / (exercise price × interest rate / 365)
```
> The period prior to the ex-dividend date during which it is impossible to earn enough interest through early exercise of a put to offset the loss of the dividend.

**Arbitrage boundaries.** An American option **should never trade below intrinsic value** — buy it, hedge with the underlying, exercise immediately, collect the difference. European lower boundaries come from put-call parity with the companion option set to zero.

## Key Concepts
- **Why an American boundary is "at least" intrinsic value.** You can always convert an American option into a European one by declining to exercise, so the American boundary is the **maximum** of intrinsic value and the European boundary.
- **A European option can trade below parity.** When the lower arbitrage boundary is below intrinsic value, the option's value rises toward intrinsic value as time passes — a **positive theta** (Figure 7-9).
- **Boundary drift.** For futures options under stock-type settlement, the lower boundary always **rises** toward intrinsic value. For stock options it can rise or fall: if F > S (interest exceeds dividends) the boundary rises for calls and falls for puts; if F < S the reverse.
- **Third choice: sell rather than exercise.** Both replace the option with an underlying position. If the option trades above intrinsic value and transaction costs are ignored, **selling is always better**. But in practice a deeply in-the-money option is illiquid with a wide bid-ask, so any sale is likely to be at no better than intrinsic value.
- **Exercising a call = selling the companion put.** Exercise the 90 call (short call + long stock synthetically = short put), and you regret it under exactly the same circumstances — the stock below 90 at expiration. And you can compute the *price* at which you sold it: if exercising gains 0.10 over a 90 put priced at 0.20, you have effectively sold the put at **0.30**.
- **Buying back the protection.** A trader who exercises early can restore the option's protective value by simultaneously buying the companion out-of-the-money option — in the example, exercise the 90 call and buy the 90 put at 0.20, giving the same protection **0.10 cheaper**. Whether to do so depends on whether implied volatility looks cheap or rich.
- **Short stock changes the calculus.** A short stock position lowers the effective interest rate (by borrowing costs), which makes **calls more likely** and **puts less likely** to be exercised early — reinforcing "whenever possible, a trader should avoid a short stock position."
- **Fugit** — the number of days remaining until an option becomes an immediate early exercise candidate.
- **Optimal exercise, formally.** Using a true American model, "an option is optimally exercised early when its theoretical value is exactly at parity and its delta is exactly 100."
- **American deltas can exceed 100 in a synthetic.** An in-the-money American option's delta reaches 100 faster than its European equivalent while the companion out-of-the-money option still retains delta, so `American call delta − American put delta > 100`. Higher volatility keeps the synthetic's delta closer to 100. Consequence: **conversions, reversals, boxes, and rolls that are delta neutral with European options may not be with American options** — and because these are traded in large size, the residual delta matters.
- **Model accuracy vs. input accuracy.** "If a trader evaluates an American option using an incorrect volatility, an incorrect interest rate, or an incorrect underlying price, the fact that he derives his values from an American rather than a European model is likely to make little difference." The American model helps most in **stock options** (large cash-outlay difference, plus dividends). For futures options under futures-type settlement a European model suffices; under stock-type settlement the extra value is small and matters only for very deeply in-the-money options.

## Model history
- The **Black-Scholes** model makes no attempt to value American options. When the CBOE opened in 1973 the first listed stock options were American, yet traders used Black-Scholes for years for lack of an equally simple alternative.
- **Pseudo-American call value** — take the greater of two Black-Scholes values: (1) the call expiring the day before the stock goes ex-dividend, and (2) the call expiring normally but valued on the current stock price *less* the expected dividend.
- For futures options and stock puts, traders used Black-Scholes values but **raised any value below parity to exactly parity**. Neither hack was truly accurate.
- **Cox-Ross-Rubinstein (binomial), 1979** — John Cox (MIT), Stephen Ross (Yale), Mark Rubinstein (UC Berkeley). Not closed form but an **algorithm or loop**; more passes → closer to true value. Easy to understand, the standard teaching model, but computationally expensive.
- **Barone-Adesi-Whaley (quadratic), 1987** — Giovanni Barone-Adesi (Alberta) and Robert Whaley (Duke). Mathematically more complex but converges far faster. Its limitation: it treats all cash flows as interest accumulating at a constant rate, whereas **dividends arrive in one lump sum** — which is why Cox-Ross-Rubinstein remains preferred for dividend-paying stocks.

## Anti-patterns
- **Exercising a stock call on any day other than the day before the ex-dividend date.** You surrender a day of volatility value and a day of interest and receive nothing in return.
- **Exercising a stock call at all when the stock pays no dividend before expiration.**
- **Exercising a put inside the blackout period** — the lost dividend always exceeds the interest earned.
- **Assuming a deep in-the-money American option should be sold rather than exercised** without checking the bid-ask.
- **Assuming conversions, boxes, and rolls are delta neutral when the options are American.**
- **Being surprised by an assignment.** "A trader need only ask himself, 'If I owned this option, would I logically exercise it now?' If the answer is yes then the trader ought to be prepared for assignment. If the answer is no and the trader is still assigned, it is probably good for the trader" — someone has abandoned the option's interest or volatility value.
- **Being short deeply in-the-money options without cash for an assignment squeeze.** "Forced liquidations are invariably losing propositions."

## Reference Tables

**Figure 16-5 — Summary of arbitrage boundaries**

*Lower arbitrage boundary:*

| | | American | European |
|---|---|---|---|
| Options on futures (stock-type settlement) | Call | max[0, F − X] | max[0, (F − X)/(1 + r·t)] |
| | Put | max[0, X − F] | max[0, (X − F)/(1 + r·t)] |
| Options on stock | Call | max[0, S − X, S − X/(1 + r·t) − D] | max[0, S − X/(1 + r·t) − D] |
| | Put | max[0, X − S, X/(1 + r·t) − S + D] | max[0, X/(1 + r·t) − S + D] |

*Upper arbitrage boundary:*

| | | American | European |
|---|---|---|---|
| Options on futures (stock-type settlement) | Call | F | F/(1 + r·t) |
| | Put | X | X/(1 + r·t) |
| Options on stock | Call | S | S − D |
| | Put | X | X/(1 + r·t) |

**Immediate-arbitrage examples** (American options trading below intrinsic value)

| Setup | Trade | P&L |
|---|---|---|
| June 90 call 9.90, underlying 100.00 | Buy call −9.90, sell underlying +100.00, exercise −90.00 | **+0.10** |
| June 70 put 4.80, underlying 65.00 | Buy put −4.80, buy underlying −65.00, exercise +70.00 | **+0.20** |

**Convergence of American−European differences**

| Case | Difference approaches |
|---|---|
| 90 call on stock (7 weeks, 6%, dividend 1.00 in 4 weeks) | 1.00 − (90 × 0.06 × 22/365) ≈ **0.67** |
| 110 put on stock, same conditions | 110 × 0.06 × 21/365 ≈ **0.38** |
| 90 call on futures at 110 (3 months, 8%, stock-type settlement) | (110 − 90) × 0.08 × 3/12 = **0.40** |

Higher volatility **shrinks** the American–European gap (early exercise less likely); lower volatility widens it. For stock options the gap has a **maximum**; for futures options it keeps growing with intrinsic value.

## Worked Example

**A. Is a stock call an early exercise candidate?**
```
Stock price       = 100
Time to expiration = 1 month
Interest rate      = 6.00%
Dividend           = 0.75, payable in 15 days
90 put price       = 0.20
```
- Dividend value = **0.75**
- Volatility value ≈ price of the companion 90 put = **0.20**
- Interest value = 90 × 0.06 × 1/12 = **0.45**

Criterion: 0.75 > 0.20 + 0.45 = **0.65** ✓ — the call is an early exercise candidate, worth **0.10** more exercised than carried to expiration. (Its European value has fallen below intrinsic value; the European boundary bends at X/(1 + r·t) + D = 90/(1 + 0.06/12) + 0.75 = **90.30**.)

But should we exercise **today**? The dividend is 15 days away. Exercising today instead of tomorrow costs a day of volatility value and a day of interest and gains nothing. **Wait until the day before the ex-dividend date.** If the dividend were payable tomorrow, the exercise would be immediate — and it would be economically equivalent to **selling the 90 put at 0.30** against its market price of 0.20.

**B. Is a stock put an early exercise candidate?**
```
Stock price       = 100
Time to expiration = 2 months
Interest rate      = 6.00%
Dividend           = 0.40
120 call price     = 0.55
```
- Interest value = 120 × 0.06 × 1/6 = **1.20**
- Volatility value ≈ **0.55**
- Dividend value = **0.40**

Criterion: 1.20 > 0.55 + 0.40 = **0.95** ✓ — worth **0.25** exercised now versus held. (European boundary bends at 120/(1 + 0.06/6) + 0.40 = **119.21**.)

*Timing.* One day's interest = 120 × 0.06/365 = **0.02**. If the dividend is payable tomorrow, exercising today gains 0.02 and loses the 0.40 dividend — **wait**. Two days away: gain 0.04, still lose 0.40 — **wait**. The **blackout period** is 0.40 / 0.02 = **20 days**: inside 20 days of the ex-dividend date, no knowledgeable trader exercises the put. But if the dividend is **30 days** away, exercising now earns 30 × 0.02 = **0.60** against a 0.40 dividend — and provided the volatility value over those 30 days is under **0.20**, immediate exercise is right.

**C. A futures option.**
```
Futures price      = 100
Time to expiration = 3 months
Interest rate      = 8.00%
80 put price       = 0.15
```
- Interest value = (100 − 80) × 0.08 × 3/12 = **0.40**
- Volatility value ≈ **0.15**

Criterion: 0.40 > 0.15 ✓ — an early exercise candidate, worth **0.25** more exercised.

*Timing.* One day's interest = (100 − 80) × 0.08/365 = **0.0044**. One day's volatility value = the theta of the 80 put. At an implied volatility of 24.68% (from Black-Scholes) the 80 put's theta is **−0.0046** — slightly larger than the daily interest, so exercising today loses 0.0002. **Not yet.** Using the model, the theta falls below 0.0044 in about **four days**, when it reaches **−0.0043** — that is the fugit. Holding the extra days is not irrational: the positive gamma pays if the future moves, and if it fell below 80 the call's protective value would be decisive. The theta of the companion put *is* the value of that one-day optionality.

**D. Valuing an American box without an American model.**
```
Time to expiration = 24 days
Interest rate      = 6.00%
Dividend           = 0.60, payable in 9 days
```
European 100/110 box = (110 − 100)/(1 + 0.06 × 24/365) ≈ **9.96**. Now enumerate the early-exercise cases:

| Case | Adjustment | Box value |
|---|---|---|
| 1 — both puts exercised (on dividend day) | + interest on 10.00 for 15 days = 0.025 | **9.985** |
| 2 — both calls exercised (day before dividend) | + interest on 10.00 for 16 days = 0.026 | **9.986** |
| 3 — only the 110 put exercised | + interest on 110 for 15 days = 0.271 | **10.231** |
| 4 — only the 100 call exercised | + dividend 0.60 − interest on 100 for 16 days (0.263) | **10.297** |
| 5 — both the 100 call and 110 put exercised | + 0.60 + 0.271 − 0.263 | **10.568** |

So: at very low stock prices (both puts exercisable) or very high ones (both calls), the box is worth about **9.99**. If exactly one option is exercisable, **10.23–10.30**. The **maximum ≈ 10.57** requires both the 100 call and the 110 put in the money — stock near **105** — *and* low volatility, "because in a high-volatility market, no one will want to give up an option's volatility value by exercising early."

**E. The dividend play.**
```
Stock price = 98.75
Time to expiration = 5 days
Dividend = 1.00, payable tomorrow
```
The 90 call ought to be exercised today and so should trade at parity, **8.75**. Sell the 90 call at 8.75 and buy 100 shares at 98.75.

*If assigned* (as should happen): sell the call +8.75, buy stock −98.75, sell stock at 90 on assignment +90.00 → **0**, break even.

*If not assigned*: the stock opens ex-dividend at 97.75, the call at ~7.75. Loss of 1.00 on the stock, gain of 1.00 on the call, **plus the 1.00 dividend** → profit ≈ **1.00**, the amount of the dividend.

The odds of escaping assignment depend on the **open interest** in the call (more outstanding calls → lower chance of being assigned) and on **market sophistication**. Dividend plays were common in the early, less sophisticated days of option trading; today only a professional with very low transaction costs would attempt one, and "he may find that he is assigned on the great majority of calls he has sold."

**Executing it as a spread.** If both the 85 and 90 calls ought to be exercised, the 85/90 call spread ought to be worth exactly **5.00** — and a market maker may quote **5.00 bid and 5.00 ask**, seemingly illogical. But buying the spread lets him exercise the 85 call (long stock, short 90 call = the dividend play); selling the spread lets him exercise the 90 call (long stock, short 85 call = the same play). "The market maker is willing to give up the edge on the bid-ask spread in return for the potential profit that will result if the short options go unexercised."

**F. The interest play.** Sell stock and simultaneously sell deeply in-the-money American puts that ought to be exercised. If they are exercised, break even; if not, earn interest on the exercise price (the combined proceeds) for as long as they stay unexercised. In futures options under stock-type settlement, the analogue is buying a future and selling a deep in-the-money call (or selling a future and selling a deep in-the-money put) — less profitable because the interest accrues only on the **intrinsic value**, not the exercise price.

## Key Takeaways
- Early exercise value exists only where dividends or interest make the underlying preferable to the option; with neither, American = European.
- Call: exercise when dividend > volatility + interest. Put: exercise when interest > volatility + dividend. Futures option: exercise when interest > volatility.
- Estimate volatility value with the companion out-of-the-money option's price, and one day's volatility value with its theta.
- A stock call is only ever exercised the day before the ex-dividend date; a stock put can be exercised any time outside the blackout period of `D / (X × r / 365)` days.
- An American option never trades below intrinsic value; a European one can, giving it positive theta.
- Cox-Ross-Rubinstein (binomial) and Barone-Adesi-Whaley (quadratic) price American options; the binomial is preferred for dividend-paying stocks because dividends are lump sums.
- American synthetic deltas exceed 100, so large arbitrage positions are not exactly delta neutral.
- Input accuracy beats model choice: an American model with a wrong volatility is still wrong.

## Connects To
- [Chapter 3 — Contract Specifications and Option Terminology](ch03-contract-specifications-and-terminology.md): exercise style, parity, and automatic exercise.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): put-call parity, the source of these boundaries, and the box/conversion structures re-priced here.
- [Chapter 7 — Risk Measurement I](ch07-risk-measurement-i.md): negative time value and positive theta; the rule against short stock positions.
- [Chapter 14 — Synthetics](ch14-synthetics.md): why exercising a call is equivalent to selling the companion put.
- [Chapter 19 — Binomial Option Pricing](ch19-binomial-option-pricing.md): the Cox-Ross-Rubinstein model in detail.
- [Chapter 18 — The Black-Scholes Model](ch18-black-scholes-model.md): the European model these values are compared against.
