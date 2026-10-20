# Chapter 25 — Volatility Contracts

## Core Idea
Before options existed there was no way to trade volatility at all. Options made it possible in theory — buy or sell and dynamically hedge — but in practice the model's weaknesses (path dependence, gaps, non-normal returns) and the **costs of rehedging** (bid-ask on every adjustment, plus brokerage and exchange fees) "will certainly reduce, and may even erase, any expected profit." **Volatility contracts** solve this: instruments whose expiration value depends solely on a straightforward volatility calculation, with no dynamic hedging required.

Two kinds of volatility, therefore two kinds of contract:
- **Realized volatility contracts** settle into the realized volatility of an underlying over a period.
- **Implied volatility contracts** settle into the implied volatility of options on an underlying on a date.

## Frameworks Introduced

**Realized volatility contract settlement** (252 trading days assumed):
```
                 ______________________
                / 252 · Σ [ln(xᵢ)]² / n
settlement =   √
```
where each xᵢ = pᵢ/pᵢ₋₁ (today's settlement price ÷ yesterday's) and n is the number of trading days.

Two deliberate conventions:
1. The expiration value is the **true** volatility over the period, not an estimate, so use the **population** standard deviation — divide by **n**, not n − 1.
2. The calculation is **independent of any trend**, so assume a **zero mean** — use ln(xᵢ), not ln(xᵢ) − μ.

**Variance, not volatility, is what settles.** `variance = volatility²`, `volatility = √variance`. Hence "variance contracts" or, more commonly, **variance swaps**. Two reasons:
- It is far easier to **replicate a variance position** than a volatility position.
- **Variance is proportional to time**, so contracts combine cleanly across consecutive, unequal periods:
```
combined variance = [(t₁ × σ₁²) + (t₂ × σ₂²)] / (t₁ + t₂)
```

**Converting notional vega into variance points** (the market convention):
```
value per variance point = vega notional / (2 × volatility price)
```

**Quoting convention.** A market maker quoting **19.50–20.50 for $10,000 notional vega** will buy at 19.50% or sell at 20.50%, each volatility point worth $10,000. A client can bid "$25,000 notional vega at 30" — willing to pay 30% volatility with each point worth $25,000.

**The original VIX methodology (1993, from OEX options).** Take the two exercise prices bracketing the index and the two nearest expirations:
1. Average the call and put implied volatility at each strike.
2. Interpolate between the two strikes, weighting by closeness to the index price.
3. Interpolate between the two months, weighting by closeness to 30 days.
4. Round to two decimals.

**Why it had to be replaced.** Two objections when the CBOE planned tradable products: (1) the calculation **required a theoretical pricing model** — which one? Black-Scholes? Binomial? And with what interest rate and dividend assumptions? (OEX options are American, complicating matters further.) Disagreement over an exchange-traded product's value "will certainly inhibit trading… and might, in some situations, lead to legal action against the exchange." (2) It used **only at-the-money options**, while traders increasingly wanted an index spanning the whole skew.

**The replacement (Goldman Sachs, 1999 — Demeterfi, Derman, Kamal, Zou).** Ask: can we build an option position that captures true volatility under **all** volatility scenarios?
- A single at-the-money option has maximum vega, but as the underlying moves, the option leaves the money and **vega declines** — exposure is not constant.
- Buying **one option at every strike** keeps one strike always at the money, but still fails: at-the-money options at **higher** exercise prices have **greater** vega, so total vega is lower at low underlying prices and higher at high ones.
- The fix: buy options **in proportion to 1/X²** — inversely proportional to the square of the exercise price. This gives **constant volatility exposure**.

**Five key features of the current VIX calculation:**
1. Derived from **volatility (time) value**, not intrinsic value — so only **out-of-the-money** options (relative to the forward) are used.
2. The implied **forward price** comes from put-call parity at the closest-to-the-money strike.
3. Each option is valued at the **average of bid and ask**.
4. When **two exercise prices with a zero bid** are encountered, no lower put strikes or higher call strikes are included.
5. Each option's contribution is weighted by the **distance between consecutive exercise prices** — wider gaps mean greater weight.

Crucially, **no theoretical pricing model is required**. The only non-price input is an interest rate — the CBOE uses the **risk-free rate**, the U.S. Treasury bill maturing nearest to option expiration — needed for the forward price under put-call parity and for the carrying cost of the options.

## Key Concepts
- **Off-exchange market.** Realized volatility contracts trade mostly over the counter, with banks and proprietary trading firms as market makers — which introduces **counterparty risk**, "an important consideration in the off-exchange market."
- **Variance settlement escalates fast.** Because variance is volatility squared, a one-time dramatic event can produce enormous payoffs. Hence many variance swaps carry a **cap**. Caps are **common on single stocks**, where one event can spike volatility, and **less common on broad indexes**, where a single component's news barely moves index volatility. Being an OTC product, buyer and seller can negotiate any specification.
- **VIX history.** The CBOE began disseminating VIX values in **1993**, computed from **OEX** (Options Exchange Index) options. In **2003** the calculation moved to **SPX** (S&P 500) options — the SPX had displaced the OEX as the exchange's most active index product and was far more widely followed. VIX **futures** began in **2004** (ticker UX or VX, $1,000 per volatility point); VIX **options** in **2006** (European, $100 per volatility point). Both settle into the VIX value at the **opening of trading on expiration Wednesday**.
- **VIX expiration timing.** Because the VIX is a theoretical **30-day** implied volatility, VIX contracts expire **30 days before** the SPX options used to compute them — usually the **third Wednesday of the previous month**. January VIX expires 30 days before February SPX; February VIX before March SPX; and so on.
- **The special opening rotation.** For settlement, the expiration VIX is computed from **actual opening trade prices** of SPX options, matched automatically from standing buy and sell orders. If an option does not trade, its bid-ask average is used. This "can sometimes cause unusual jumps in the VIX value at expiration": if all options print at the ask (a **buy print**) the settlement is higher than expected; at the bid (a **sell print**), lower. Normal methodology resumes immediately after.
- **The VIX as the "fear index."** Over 2003–2012 the VIX and the S&P 500 show a correlation of **−0.7444**, with the VIX's percent change roughly **5.7 times** the S&P's in the opposite direction. But a change in the VIX shows **essentially no predictive power** for the change in realized volatility over the following 30 days — correlation only **+0.1561**, "very small but probably insignificant." Natenberg's inference: "Perhaps what drives the VIX is not the expectation of future realized volatility, but the desire to buy protection in a falling stock market… They are driven by the fear of further declines."
- **Falling markets *are* more volatile — moderately.** Comparing 30-day S&P 500 price changes with 30-day realized volatility over 2003–2012 gives an inverse correlation of **−0.3895**: more high-volatility points to the left of zero, more low-volatility points to the right.
- **Why the VIX cannot be replicated.** Unlike a stock index, "as options go into and out of the money, the index components and their weights within the index are constantly changing." For most traders the only practical access is through **derivatives** — futures, options, or linked products — which is precisely why they do not track the index as expected.
- **VIX futures term structure.** Usually **contango** (upward sloping — long maturities dearer), occasionally **backwardation**. Figure 25-10 shows the structure flipping dramatically from contango to backwardation during the late-2008 financial crisis.
- **Futures spreads and curvature.** With a **straight-line** term structure of constant slope, a spread's value never changes — both legs gain or lose at exactly the same rate. With a **curved** structure (far more common), the **short-term contract changes faster**. So if the shape holds: in **contango**, buy long-term / sell short-term; in **backwardation**, buy short-term / sell long-term. And on a change of shape: expecting contango to flatten or invert → **sell the spread** (sell long-term, buy short-term); expecting backwardation to flatten or invert → **buy the spread**.
- **VIX options are hedged with VIX futures, not the index** — so their implied volatilities reflect **futures** volatility, which is lower. Over 2003–2012 the VIX's own 50-day volatility occasionally reached nearly **200%** and rarely fell below **50%**; but "rarely is the front-month futures contract as volatile as the index," and back months are progressively less volatile still. "For a theoretical trader who follows a dynamic hedging procedure, expectations about VIX volatility should focus on the volatility of the futures contract used to hedge the option position, not the volatility of the index."
- **The VIX skew is a "half frown."** In stock and commodity markets, strikes further from the underlying carry progressively higher implied volatilities — the volatility smile. For VIX options, **lower exercise prices' implied volatility drops off very quickly**, while higher strikes rise and then **flatten out**. The reason: unlike a stock, implied volatility has practical bounds ("an option trader would be surprised indeed to see implied volatility in a stock index market fall below 5 percent… [or] rise above 100 percent") and is **mean reverting** — low VIX is more likely to rise, high VIX more likely to fall.
- **The VIX implied distribution** (June options, March 19, 2012, June future at 23.95): compared with a lognormal distribution, the **left tail is much more restricted** — almost no chance of the VIX below 10.00 at expiration — and the **right tail is also more restricted**, though with a slightly **better chance of a very large upward move at the far end**.

## Replicating a volatility contract
- **A variance position can be replicated** by buying a strip of options across all exercise prices in the proportion **1/X²**, then dynamically hedging to stay delta neutral. The total value of the strategy then matches the realized variance exactly.
- **A volatility position cannot.** "The fact that volatility is the square root of variance means that if the variance exposure is constant, the volatility exposure cannot be constant." A trader hedging a *volatility* contract with the same 1/X² strip cannot be sure the strip will offset his position — "which is why such contracts are usually settled in variance points."
- **The VIX methodology is the strip's price expressed as a volatility.** "This is analogous to taking the price of an option and turning it into an implied volatility. The VIX methodology takes the prices of all the options in the strip and turns them into an implied volatility position, but one with constant-variance exposure."
- **Two strips, near-zero net gamma.** Prior to expiration no options expire in exactly 30 days, so the VIX needs two bracketing strips, weighted. Replication means **buying one strip and selling the other**, and their gammas approximately offset — so with total gamma near zero, **no delta-neutral rehedging is necessary**, and the position can be carried to VIX expiration.
- **Two problems remain, one with no good solution.** (1) **Expiration mismatch.** The long-term strip closes on VIX expiration Wednesday via the special opening rotation, but the short-term strip expires the Friday immediately **before or after**. If after, the trader must close it himself on Wednesday, **giving up the bid-ask spread on every option** — costly. If before, he must carry a **naked long-term strip for an additional five days** — also costly. "Unfortunately, there is no good solution to this problem, which is one reason why replicating the VIX is so difficult." (2) **Options going in the money.** The VIX uses only out-of-the-money options, but some will move in the money. By synthetics, an in-the-money option hedged with an underlying contract equals an out-of-the-money option of the opposite type, so buy or sell **one underlying contract per in-the-money option**. But SPX has no easily traded underlying — the basket of 500 stocks. Use **S&P 500 futures** expiring with the VIX as a proxy, or build one from **combos** (long call/short put or short call/long put at one strike) expiring with the short-term strip.

## Applications
- **Speculation.** A view on **realized** volatility → buy or sell a **variance swap**. A view on **implied** volatility → **VIX futures**. A view on the **volatility of the VIX itself** → **VIX options**.
- **Hedging an unintended volatility position.** Market makers and hedge fund managers accumulate volatility exposure as a byproduct of their activity. A **realized volatility position (gamma)** is hedged with **variance contracts**; an **implied volatility position (vega)** with **VIX contracts**.
- **Hedging a broad equity portfolio.** Because of the inverse VIX/S&P correlation, a manager long equities can take a **long VIX position** — buy VIX futures, buy VIX calls, or sell VIX puts. If stocks fall, implied volatility should rise, and the VIX gain offsets part of the equity loss.
- **Indirect volatility positions.** Three cases where volatility exposure is not obvious:
  1. **A market maker profits from volume**, and volume rises with volatility — an **indirect long volatility position**, hedgeable by going **short the VIX**. Caveat: "he is really hedging trading volume, and he should not take such a large VIX position that his attention is diverted from his primary market-making activities."
  2. **A portfolio manager facing a rebalancing date** pays more to rebalance in volatile markets, when bid-ask spreads widen — an **indirect short volatility position**, hedgeable by going **long the VIX**.
  3. **A covered-call writer** is short volatility whether he realizes it or not (see the worked example).

## Anti-patterns
- **Expecting a VIX futures contract to move point-for-point with the index.** "The price of a VIX futures contract will almost never change as quickly as the index price."
- **Ignoring contango decay.** "When the VIX term structure is contango, as time passes with no change in the index value, VIX futures prices will inevitably decline."
- **Evaluating VIX futures against the index.** Since replication is impractical, no arbitrage enforces convergence before expiration — "futures prices must often be evaluated independent of the index price."
- **Pricing VIX options off the index's volatility** rather than the hedging futures contract's.
- **Selling an uncapped variance swap on a single stock**, where one event can send volatility to a level that produces a catastrophic loss.
- **Reading a rise in the VIX as a forecast of higher realized volatility.** The correlation is +0.1561.
- **Slavish reliance on any model.** "A trader who slavishly uses a model to make every trading decision is heading for disaster. Only a trader who fully understands what a model can and cannot do will be able to make the model his servant rather than his master."

## Reference Tables

**Realized volatility contract P&L, settled in volatility points** ($1,000 notional vega, traded at 20%)

| Realized volatility | Calculation | P&L |
|---|---|---|
| 23.75% | $1,000 × (23.75 − 20.00) | **+$3,750** |
| 18.60% | $1,000 × (18.60 − 20.00) | **−$1,400** |

**The same contract settled in variance points** ($10,000 notional vega at 20 → $10,000/(2 × 20) = **$250 per variance point**)

| Realized volatility | Variance | Calculation | P&L |
|---|---|---|---|
| 19% | 361 | $250 × (361 − 400) | **−$9,750** |
| 23% | 529 | $250 × (529 − 400) | **+$32,250** |
| 50% | 2,500 | $250 × (2,500 − 400) | **+$525,000** |
| 40% (capped) | 1,600 | $250 × (1,600 − 400) | **+$300,000** max |

**Variance P&L vs. volatility P&L on the same $10,000-vega contract bought at 20**

| Realized variance | Variance P&L | Realized volatility | Volatility P&L |
|---|---|---|---|
| 250 | −$37,500 | 15.81 | −$41,900 |
| 300 | −$25,000 | 17.32 | −$26,800 |
| 350 | −$12,500 | 18.71 | −$12,900 |
| **400** | **0** | **20** | **0** |
| 450 | +$12,500 | 21.21 | +$12,100 |
| 500 | +$25,000 | 22.36 | +$23,600 |
| 550 | +$37,500 | 23.45 | +$34,500 |

The two agree exactly at the contract price and diverge further the more realized volatility departs from it.

**Original VIX methodology worked inputs** (index 863.40; Month 1 = 14 days, Month 2 = 42 days)

| | Month 1, 860 | Month 1, 870 | Month 2, 860 | Month 2, 870 |
|---|---|---|---|---|
| Call implied volatility | 22.16 | 21.48 | 20.13 | 19.93 |
| Put implied volatility | 22.21 | 21.44 | 20.17 | 19.94 |
| **Average** | **22.185** | **21.46** | **20.15** | **19.935** |

**Volatility contract characteristics compared**

| | Variance swap | VIX futures | VIX options |
|---|---|---|---|
| Settles into | Realized volatility (as variance) | VIX at Wednesday open | VIX at Wednesday open |
| Venue | Off-exchange (OTC) | CBOE, from 2004 | CBOE, from 2006 |
| Point value | Negotiated (notional vega) | $1,000 per volatility point | $100 per volatility point |
| Style | — | — | European |
| Replicable | Yes, in theory (1/X² strip) | Not practically | Not practically |
| Cap | Common on single stocks | — | — |

**Key correlations, S&P 500 and VIX, 2003–2012**

| Relationship | Correlation |
|---|---|
| Daily percent change in S&P 500 vs. daily percent change in VIX | **−0.7444** (VIX moves ≈ 5.7× as far, opposite direction) |
| Daily change in VIX vs. change in realized volatility over the next 30 days | **+0.1561** (essentially no predictive value) |
| 30-day S&P 500 price change vs. 30-day realized volatility | **−0.3895** (falling markets moderately more volatile) |

## Worked Example

**A. Combining variance across unequal periods.** Annualized volatility over two months is **25**; over the following one month it is **22**. What is the annualized variance over the whole three months?
```
[(2/12 × 25²) + (1/12 × 22²)] / (3/12)
= [(2/12 × 625) + (1/12 × 484)] / 0.25
= 578
```
Because variance — not volatility — is proportional to time, this addition is exact. It is precisely why contracts settle in variance points.

**B. Computing the original VIX, step by step.** Index at **863.40**; bracketing strikes **860** and **870**; Month 1 has **14** days, Month 2 has **42**.

*Step 1 — average call and put at each strike:*
```
Month 1, 860:  (22.16 + 22.21)/2 = 22.185
Month 1, 870:  (21.48 + 21.44)/2 = 21.46
Month 2, 860:  (20.13 + 20.17)/2 = 20.15
Month 2, 870:  (19.93 + 19.94)/2 = 19.935
```
*Step 2 — interpolate across strikes, weighting by closeness to the index:*
```
Month 1: 22.185 × (870 − 863.40)/10 + 21.46 × (863.40 − 860)/10
       = 22.185 × 0.66 + 21.46 × 0.34 = 14.6421 + 7.2964 = 21.9835

Month 2: 20.15 × 0.66 + 19.935 × 0.34 = 13.299 + 6.7779 = 20.0769
```
*Step 3 — interpolate across months, weighting by closeness to 30 days:*
```
20.0769 × (30 − 14)/28 + 22.2785 × (42 − 30)/28
= 20.0769 × 0.5714 + 22.2785 × 0.4286
= 11.4725 + 9.5479 = 21.0204
```
*Step 4 — round:* the disseminated value is **21.02**.

**C. Why the futures disappoint — two real episodes.**

*Contango, index rising.* Over four days in **July 2011** the VIX rose from about **19.4 to 23.7** — an increase of **4.4** index points. The front-month **August** future, with roughly three weeks to expiration, rose only **2.0**, from 19.3 to 21.3. Over the last two days, with the index climbing from 23.0 to 23.8, "futures prices hardly changed at all." A trader long the August future made money, but far less than the index move suggested.

*Backwardation, index falling.* Over four days in **December 2008** the VIX fell from about **52.4 to 44.9** — a decline of **7.5**. The front-month **January** future fell only **5.0**, from 52.4 to 47.4. A trader short January futures "would likewise be disappointed."

*Why no arbitrage corrects this.* In a stock index futures market, a divergence invites buying one side and selling the other, held to a guaranteed convergence at maturity. The VIX "is not easily tradable," so no such trade exists — VIX futures may move less than the index, or **not at all**. Only at expiration must they converge.

*The practical rules that follow:*
1. In contango, futures prices decline as time passes with no index change.
2. Futures almost never move as fast as the index.
3. Futures and index converge at expiration.
4. Since replication is impractical, evaluate futures **independently** of the index.

Therefore, buying VIX futures pays when the rise comes **close to expiration**, or when the rise is **large enough to invert the curve** from contango to backwardation; selling pays under the mirror conditions. Either way, "the trader must also temper his expectations."

**D. Constructing a constant-volatility strip.** To hold constant volatility exposure, buy `1/X²` of the option at each exercise price X. Buying **one** option at each strike (Figure 25-2) fails because at-the-money options at higher strikes carry more vega, tilting total vega up at higher underlying prices. Weighting by 1/X² buys **more** low-strike options and **fewer** high-strike options, flattening the total vega curve (Figure 25-3). Exact replication would need an infinite number of strikes; the CBOE approximates it with the finite listed set — that *is* the VIX methodology.

**E. An arbitrage between a volatility contract and its replication.** A trader is **short** a realized volatility contract at **20%** — variance **400** — with $250 per variance point. If he can construct the 1/X² strip that replicates a **long** variance position at a volatility of **19%** — variance **361** — the arbitrage is locked:
```
profit = (400 − 361) × $250 = 39 × $250 = $9,750
```
The question the VIX methodology answers is precisely *"Did he buy the strip at a volatility of 19 percent, or 20 percent, or 21 percent?"* — turning a basket of option prices into a single volatility number.

**F. The covered call as a hidden short volatility position.** A portfolio manager sells index calls against a broad stock portfolio. He has **two** goals: (1) the portfolio increases in value, and (2) it **outperforms a benchmark** such as the S&P 500.

| Market | Goal 1 (portfolio rises) | Goal 2 (beat the index) |
|---|---|---|
| Rises far | ✓ at first — but calls are exercised, capping upside | ✗ the index eventually outperforms |
| Falls far | ✗ covered calls are only a partial hedge | ✓ the premium received cushions the fall |
| **Sits still** | **✓** the premium increases portfolio value | **✓** the stock-only benchmark is beaten |

"If the portfolio manager wants the market to sit still, he has a **short volatility position**." He can hedge part of that risk by **going long the VIX**, usually by buying VIX futures.

## Key Takeaways
- Volatility contracts exist because dynamic hedging is expensive and imprecise; they settle on a formula, not on a hedging program.
- Realized volatility contracts use the population standard deviation and a zero-mean assumption, and settle in **variance** because variance is replicable and additive over time.
- Value per variance point = notional vega ÷ (2 × volatility price); caps are standard on single-stock variance swaps.
- The modern VIX is the price of a 1/X² strip of out-of-the-money SPX options, expressed as a volatility — requiring **no pricing model**, only option prices and a risk-free rate.
- The VIX is strongly inversely correlated with the S&P 500 (−0.7444) but has almost no power to forecast realized volatility (+0.1561) — it is a fear index, not a forecast.
- VIX futures move less than the index, decay down a contango curve, and cannot be arbitraged against the index; VIX options should be priced off futures volatility, and their skew is a "half frown" reflecting mean reversion and practical bounds on volatility.
- Replication is possible in theory (a 1/X² strip, two strips whose gammas offset) but blocked in practice by expiration mismatch and the absence of a tradable SPX underlying.
- Uses: speculating on realized or implied volatility, hedging accumulated gamma or vega, hedging equity portfolios, and hedging *indirect* volatility positions — market-maker volume, rebalancing costs, and covered-call writing.
- Closing counsel: have faith in a model, but not blind faith. "Trading is both an art and a science… Only a trader who fully understands what a model can and cannot do will be able to make the model his servant rather than his master."

## Connects To
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): the process volatility contracts exist to avoid.
- [Chapter 20 — Volatility Revisited](ch20-volatility-revisited.md): forward volatility and the variance-is-proportional-to-time result; the term structure VIX futures reflect.
- [Chapter 23 — Models and the Real World](ch23-models-and-the-real-world.md): the model weaknesses that make dynamic hedging unreliable.
- [Chapter 24 — Volatility Skews](ch24-volatility-skews.md): the butterfly method used to derive the VIX implied distribution, and the skew shapes contrasted with the VIX's "half frown."
- [Chapter 22 — Stock Index Futures and Options](ch22-stock-index-futures-and-options.md): index replication, which the VIX conspicuously resists.
- [Chapter 17 — Hedging with Options](ch17-hedging-with-options.md): the covered write, revealed here as a short volatility position.
- [Chapter 14 — Synthetics](ch14-synthetics.md): converting in-the-money to out-of-the-money options during replication.
