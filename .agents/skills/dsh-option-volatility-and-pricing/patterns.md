# Patterns — Option Volatility and Pricing (Sheldon Natenberg)

The book's reusable moves, each as **When / How / Trade-offs**. Grouped by the book's natural parts.

---

## Part I — Pricing and Valuation

### Build a forward price from carry
**When** you need the mean of the pricing distribution for any underlying — the input every option model actually uses.
**How** `forward price = current cash price + costs of buying now − benefits of buying now`. Physical commodity: `F = C(1 + r·t) + s·t + i·t`. Stock: `F = S(1 + r·t) − D`. Bond: substitute coupons for dividends. Currency: `F = S × (1 + r_d·t)/(1 + r_f·t)`. Index: shortcut with `F = S[1 + (r − d)t]`.
**Trade-offs** The index shortcut treats dividends as a continuous negative interest rate; it is reasonable for long-dated forwards and can be grossly wrong for short ones, since dividends arrive in discrete, uneven bundles.

### Solve for the implied input
**When** market prices disagree with your model and you want to know *which* assumption the market is making differently.
**How** Invert the pricing identity for the unknown. From `F = S(1 + r·t) − D`: implied spot `= (F + D)/(1 + r·t)`; implied rate `= ([(F + D)/S] − 1)/t`; implied dividend `= S(1 + r·t) − F`. From an option price: implied volatility. From put-call parity: implied forward, implied rate, implied dividend.
**Trade-offs** The implied value is only the market's consensus *if* the contract is fairly priced. And it is only meaningful with **contemporaneous** inputs — a two-hour-old option print produced 26.95% implied volatility where a current one gave 28.50%.

### Price by expected value, then discount
**When** building any valuation from first principles.
**How** (1) Propose possible expiration prices. (2) Assign probabilities such that the expected underlying value equals the **forward price** (the arbitrage-free constraint). (3) Compute the option's expected value: `Σ pᵢ·max(Sᵢ − X, 0)` for a call. (4) Take the present value according to the settlement procedure.
**Trade-offs** Probabilities need not be symmetric — only the mean is constrained. The real world has infinitely many outcomes; discrete approximations under- or overstate depending on the shape assumed.

### Scale volatility by the square root of time
**When** converting an annual volatility into a horizon you can actually observe.
**How** `volatility_t = volatility_annual × √t`. Daily ≈ annual/16 (√256); weekly ≈ annual/7.2 (√52). Check observed moves against the 2/3, 19/20, 369/370 rule — and expect a two-standard-deviation daily move about once a month.
**Trade-offs** Assumes 256 trading days for arithmetic convenience. For short horizons the forward and spot prices are close enough to centre the distribution on spot. Electronic, globally integrated markets blur "how many days are in a year."

### Use the at-the-forward approximation instead of a computer
**When** you need a fast option value with no model at hand.
**How** `value ≈ X × (σ × 100) × √t × 0.00399`, discounted by `(1 + r·t)`. The **40% rule**: an at-the-forward option is worth about 40% of one standard deviation, where one standard deviation = `F × σ√t`.
**Trade-offs** Overstates value as time and volatility grow, because at-the-money vega declines slightly with volatility and the effect is magnified with time. At 40% volatility over two years the approximation gave 14.67 against a true 14.48.

---

## Part II — Risk Measurement

### Read the Greeks as a wish list
**When** you have any position and want to know what market conditions you are rooting for.
**How** Positive delta → want the price to rise. Positive gamma → want big or fast moves. Positive theta → time helps. Positive vega → want implied volatility to rise. Positive rho → want rates to rise. All risk measures are **additive**: multiply each by the signed contract count and sum.
**Trade-offs** These describe the position **only under current conditions**. Gamma and theta always carry opposite signs; you cannot have both movement and decay on your side.

### Distinguish gamma from vega
**When** a position looks internally consistent but the market moves against you in an unexpected way.
**How** Gamma is your preference for **realized** volatility; vega is your preference for **implied** volatility. They usually agree; calendar spreads are the standing exception (−gamma / +vega).
**Trade-offs** "In some cases the underlying contract can become more volatile while implied volatility is falling."

### Reprice a large move with the average delta
**When** the underlying has jumped far enough that the current delta is no longer a good approximation.
**How** `new value ≈ C + [(S₂ − S₁) × Δ] + [(S₂ − S₁)² × Γ/2]` — equivalently, use the delta averaged over the move.
**Trade-offs** Still approximate, because gamma itself changes across the move. Better than a constant delta.

### Compute a position's implied volatility
**When** you want a single number for how wrong your volatility estimate can be before the trade loses money.
**How** `breakeven volatility ≈ evaluation volatility + (total theoretical edge / total vega)`.
**Trade-offs** Linear, so it ignores volga — a position with positive volga breaks even at a *higher* volatility than the estimate suggests (34.90% estimated vs. ~36% actual in one worked case).

---

## Part III — Capturing Value

### Dynamically hedge to capture a mispricing
**When** you have identified an option trading away from theoretical value and want the model's predicted profit rather than a lottery ticket.
**How** Buy the underpriced option (or sell the overpriced one), neutralize the delta with the underlying, then recompute the delta at intervals — holding volatility and rate constant — and trade the underlying back to neutral. At expiration: let OTM options expire, sell ITM options at parity, liquidate the underlying.
**Trade-offs** Adjustments **do not change the expected return**; they reduce variance by turning one bet into many at the same odds. Every adjustment costs the bid-ask plus fees, so a professional adjusts often and a retail customer rarely — same expectation, wider swings for the retail trader.

### Choose an adjustment discipline before you enter
**When** initiating any volatility spread.
**How** Four options: (1) **regular intervals**, matched to the interval used to measure volatility; (2) a **delta threshold** — adjust whenever the position reaches ±N deltas; (3) **by feel**, declining an adjustment at a known support level; (4) **not at all**, accepting the position's maximum possible delta.
**Trade-offs** Frequency trades transaction costs against tracking error. Never let a volatility position drift into a directional one — "if the trader does not want to adjust the position but he also does not want directional considerations to dominate, the only choice left is to close out the position."

### Replicate an option you cannot buy
**When** you want option-like protection in a market with no listed options.
**How** Value the option you wish existed (choosing its strike and expiration freely), read off its delta, and hold that fraction of the underlying, rebalancing as the delta changes. Institutionally, mimic the delta with index futures rather than trading the portfolio.
**Trade-offs** This is **portfolio insurance**, and it works only if the volatility input is right and the model's assumptions hold. Gaps make continuous adjustment impossible; October 1987 showed the replication cost far exceeding expectations, and the required selling may have worsened the crash.

---

## Part IV — Choosing and Sizing a Strategy

### Match the vega sign to the implied volatility level
**When** your volatility estimate differs from the market's and you must pick among many spreads.
**How** If implied volatility is **low** (options underpriced), look for **positive vega** spreads: long straddles and strangles, short butterflies and condors, ratio spreads buying more than selling, long calendars. If implied volatility is **high**, look for **negative vega** spreads: the mirror set.
**Trade-offs** Straddles and strangles give positive edge on both legs but are the riskiest of all spreads; you may prefer a ratio spread or butterfly even at the cost of buying an overpriced leg.

### Work around the at-the-money option for vertical spreads
**When** you have a directional view and must choose which strikes.
**How** If implied volatility is **low**, **buy** the at-the-money option; if **high**, **sell** it. The second leg follows automatically from wanting a bull or bear spread. The at-the-money option is the most sensitive in total points to a volatility change, so it is the most mispriced in either direction.
**Trade-offs** Strictly this should be the **at-the-forward** option, which in a high-rate, long-dated stock market can be a materially higher strike.

### Buy the lower strike to be bullish — always
**When** constructing any vertical spread.
**How** Buy the lower exercise price and sell the higher for a bull spread; the reverse for a bear spread. This holds whether the spread is built from calls or puts.
**Trade-offs** Counterintuitive with puts, and the two versions differ in cash flow: the call version is a debit, the put version a credit, and the two prices sum to the strike interval (present-valued with interest).

### Equalize theoretical edge, then compare risk
**When** several candidate spreads all look profitable.
**How** Scale each spread's size until the theoretical edges match. Then plot P&L against the underlying price, against volatility (reading off each breakeven volatility), against time, and — for stock options across expirations — against rates and dividends.
**Trade-offs** A snapshot gamma or vega can mislead badly: the spread with the smallest gamma and vega at current conditions deteriorated almost as fast as a short straddle at the extremes, because of its negative volga.

### Size to the margin for error, not to the edge
**When** deciding how many times to do a profitable spread.
**How** Compute the position's breakeven volatility. A 2-point margin against your forecast → keep the size small (say 20 × 10). A 7-point margin against a volatility you consider historically rare → size up (100 × 50).
**Trade-offs** "No trader will survive very long if his livelihood depends on estimating each input with 100 percent accuracy" — but neither will one who mistakes a wide margin for no risk.

### Adjust with the underlying to change only delta
**When** you need to re-neutralize direction without disturbing anything else.
**How** Trade the underlying — its gamma, theta, and vega are all zero, so it is a **risk-neutral** adjustment.
**Trade-offs** Adjusting with options also changes gamma, theta, and vega — something new traders forget. And repeatedly adjusting by selling more overpriced options inflates a 20-lot into a 48 × 31 without anyone deciding to increase the size.

### Match adjustment frequency to your trend style
**When** you have a directional temperament you cannot suppress.
**How** A negative-gamma position adjusts **with** the trend; a positive-gamma position adjusts **against** it. To trade with the trend: positive gamma with infrequent adjustments, or negative gamma with frequent ones. To trade against it: the reverse.
**Trade-offs** A purely theoretical trader has no trend to consider — but "old habits, such as trading with or against the trend, are hard to break."

---

## Part V — Structural Relationships

### Rewrite a position in one option type
**When** a book of contracts fits no recognizable pattern.
**How** Convert every put to a synthetic call (or every call to a synthetic put), cancel the offsetting underlying legs, and read what remains.
**Trade-offs** Works cleanly in constructed examples; "in reality, the risk characteristics of a complex position rarely fall neatly into place." Real books still need a model.

### Hedge an option to create its companion
**When** you own the wrong option type for the position you want.
**How** Trade a single option and hedge it with the underlying and you hold the **companion option** synthetically. Buy a call and sell the underlying = bought a put. Buy a put and buy the underlying = bought a call.
**Trade-offs** Companion calls and puts share gamma and vega exactly and split delta to ~100, but their **thetas differ** wherever there is a cost of carry.

### Arbitrage the synthetic against the real
**When** put-call parity is violated.
**How** `C − P = (F − X)/(1 + r·t)`. If the combo is expensive relative to the underlying, do a **conversion** (buy underlying, sell call, buy put); if cheap, a **reversal**. Remove the underlying-carrying risk with a **box** (two strikes), a **roll** (two months), or a **three-way** (a deep in-the-money option).
**Trade-offs** Four real risks remain: execution (legging in), pin (an at-the-money expiration), settlement (mismatched settlement giving a synthetic future a delta of 100/(1+r·t)), and interest/dividend. Imbalances "rarely last for more than a few seconds," so these are professional trades done in size.

### Escape a locked market synthetically
**When** a futures contract is limit-up or limit-down and you must trade.
**How** Trade the cash market; or trade a futures spread whose other leg is unlocked and then unwind that leg; or build a synthetic future from options, reading the implied price from `F = C − P + X` across several strikes.
**Trade-offs** The implied futures prices will spread across a small range (133.20–133.30 in the book's table) because of bid-ask width and non-contemporaneous quotes.

---

## Part VI — Early Exercise

### Test an American option against three components
**When** deciding whether to exercise early or whether you may be assigned.
**How** Compare **dividend value** (the expected dividend), **interest value** (`X·r·t` for stock options, `(F − X)·r·t` for futures options), and **volatility value** (the price of the companion out-of-the-money option). Call: exercise when dividend > volatility + interest. Put: exercise when interest > volatility + dividend. Futures option: when interest > volatility.
**Trade-offs** The criteria must hold over the option's whole life **and** over the next day. One day's volatility value is the companion option's theta — the only figure requiring a model.

### Time the exercise
**When** the criteria are satisfied.
**How** A stock **call** is only ever exercised the day **before** the ex-dividend date. A stock **put** may be exercised any time outside the **blackout period** of `D / (X × r / 365)` days before the ex-dividend date. A futures option is exercised when its daily interest exceeds the companion option's theta.
**Trade-offs** Selling the option and buying the underlying is theoretically better whenever the option trades above parity — but a deep in-the-money option's bid-ask usually makes this impractical.

### Anticipate assignment
**When** you are short American options.
**How** Ask: "If I owned this option, would I logically exercise it now?" If yes, be prepared and hold cash for the squeeze. If no and you are assigned anyway, "the trader who is assigned will find that he is the recipient of an unexpected gift."
**Trade-offs** Forced liquidations from an unfunded assignment "are invariably losing propositions."

---

## Part VII — Hedging

### Choose the hedge by implied volatility, then by need
**When** protecting an existing underlying position.
**How** Options cheap (implied volatility low) → **buy** a protective put or call. Options rich → **sell** a covered call or put. Want both bounded risk and low cost → a **collar**. Want no risk and no upside → sell a future or forward.
**Trade-offs** Practical need can override theory: "If a hedger knows that a move in the underlying contract beyond a certain price will represent a threat to his business, then the purchase of a protective option at that exercise may be the most sensible strategy regardless of whether the option is theoretically overpriced."

### Buy few / sell many in a high-volatility market
**When** hedging a fixed fraction of a directional exposure.
**How** To hedge 50% of a long position with implied volatility **high**, sell one at-the-money call with a 50 delta rather than several out-of-the-money calls totalling 50 deltas — or buy one 50-delta put rather than several cheap ones. With implied volatility **low**, do the reverse.
**Trade-offs** Ratio writing to reach the delta target adds **unlimited risk on the second side**. "Most hedgers want to restrict their unlimited risk to one direction, usually the direction of their natural position."

### Set a target price with a covered option
**When** you already know the price at which you would buy or sell.
**How** Sell a call struck at your target sale price, or a **cash-secured put** struck at your target purchase price. Either you transact at your target or you keep the premium.
**Trade-offs** The short call is secured by the stock; the short put must be secured with cash equal to X (or X/(1 + r·t) for a European put). Companies use the put version for buy-backs.

---

## Part VIII — Volatility Analysis

### Forecast volatility by matching the window to the option's life
**When** choosing a volatility input.
**How** Weight the historical windows so that the one closest in length to the option's remaining life gets the most weight (serial correlation). Formalize with EWMA (λ ≈ 0.94) or GARCH, which adds correlation and mean-reversion components.
**Trade-offs** Long-term volatility is more stable and easier to predict, but long-dated options have larger vegas — "the effect of a two or three percentage point volatility error on a long-term option may be greater than a five or six percentage point error on a short-term option."

### Rescale vega for the term structure
**When** your book spans multiple expirations.
**How** Pick a primary month, estimate how fast each other month's implied volatility moves relative to it, multiply each month's vega by that ratio, and sum.
**Trade-offs** A net vega of zero across expirations is meaningless without this step — one book summing to zero actually carried −4.08 of vega per point of front-month implied volatility.

### Magnify the term structure with calendar-spread implied volatilities
**When** hunting for a mispriced expiration.
**How** For each adjacent pair of months, find the single volatility that prices the calendar spread at its market value. Shortcut: `spread price / spread vega`. Plot the results against the term structure; kinks reveal the mispriced months. Forward volatility, `√[(σ₂²t₂ − σ₁²t₁)/(t₂ − t₁)]`, does the same job.
**Trade-offs** Settlement prices do not always reflect executable prices. In the FTSE example the trade was buy April/June and sell June/September — a time butterfly.

---

## Part IX — Living with the Model's Flaws

### Treat the skew as an input, not an error
**When** implied volatilities differ across exercise prices, as they always do.
**How** Fit `y = a + bx + cx²`, where a is the at-the-money volatility, b the skewness, and c the kurtosis. Normalize the x axis to standard deviations `ln(X/S)/(σ√t)` and the y axis to percent of at-the-money implied volatility so skews are comparable across expirations. Combine with the term structure to build a **volatility surface**.
**Trade-offs** A steep skew model can produce absurd values — reducing volatility can *raise* an out-of-the-money put's value as it "climbs the skew." Many traders use the skew model for theoretical values but a traditional model for the Greeks.

### Trade the shape of the skew
**When** you have a view on skewness or kurtosis rather than on level.
**How** For **skewness**, a delta-hedged **risk reversal** using ±25 delta options chosen for **equal vegas**. For **kurtosis**, buy or sell strangles offset by at-the-money straddles (a **dragonfly** at 2 × 1), since at-the-money options are kurtosis-neutral. Across expirations, take opposing skew positions in two months with matched vegas.
**Trade-offs** Skew and term-structure views can reinforce or cancel — in the book's example, buy the put calendars (both effects favourable) and avoid the call calendars (effects offsetting).

### Extract the market's implied distribution from butterflies
**When** you want to compare your own view of the distribution with the market's.
**How** Butterflies across a full strike ladder always sum to the strike interval. So `probability of a strike = that butterfly's price ÷ the strike interval`. Narrow the increment to approach a continuous distribution, then overlay a constant-volatility lognormal distribution.
**Trade-offs** Requires many strikes to be accurate, and each result is a snapshot. "If there is a clear disagreement, it may point the way to a potentially profitable strategy."

### Distrust the model as expiration approaches
**When** the remaining life is days rather than months.
**How** Rely less on model values and more on experience. Consider **buying** cheap at-the-money straddles rather than selling time premium, and hold them unadjusted, since the model's deltas are as unreliable as its values.
**Trade-offs** Most of the time no gap occurs and the straddle loses. Size to what you can afford to lose repeatedly; the payoff logic is the discounted roulette bet — one win in many must cover the losses.

### Trade volatility directly instead of hedging it
**When** the cost and imprecision of dynamic hedging outweigh the edge.
**How** A view on **realized** volatility → a variance swap. A view on **implied** volatility → VIX futures. A view on the volatility **of** implied volatility → VIX options. To hedge an accumulated gamma → variance contracts; an accumulated vega → VIX contracts.
**Trade-offs** VIX futures move far less than the index and decay down a contango curve; the index cannot be replicated, so no arbitrage forces convergence before expiration. Uncapped single-stock variance swaps can produce catastrophic losses from one event.

### Hedge the volatility exposure you did not know you had
**When** your business or strategy is quietly long or short volatility.
**How** A market maker profits from volume, which rises with volatility → indirectly **long** volatility → hedge by selling the VIX. A manager facing a costly rebalancing → indirectly **short** volatility → hedge by buying the VIX. A covered-call writer wants the market to sit still → **short** volatility → hedge by buying VIX futures.
**Trade-offs** The market maker is really hedging trading volume and "should not take such a large VIX position that his attention is diverted from his primary market-making activities."
