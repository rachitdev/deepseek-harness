# Chapter 23 — Models and the Real World

## Core Idea
A trader faces **two** kinds of model risk: wrong inputs, and a wrong model. Earlier chapters handled the first through the Greeks. This chapter attacks the second — the assumptions built into every traditional pricing model (Black-Scholes and its variations, or Cox-Ross-Rubinstein) — and shows how each fails, what it costs, and what an experienced trader does about it.

## Frameworks Introduced

**The six assumptions of a traditional pricing model:**
1. **Markets are frictionless.**
   - A. The underlying can be freely bought or sold, without restriction.
   - B. Unlimited money can be borrowed or lent, at one rate for all transactions.
   - C. There are no transaction costs.
   - D. There are no tax consequences.
2. **Interest rates are constant** over the life of an option.
3. **Volatility is constant** over the life of an option.
4. **Trading is continuous**, with no gaps in the underlying price.
5. **Volatility is independent** of the price of the underlying contract.
6. Over small periods, **percent price changes are normally distributed**, giving a **lognormal** distribution of prices at expiration.

**Three price processes:**
- **Diffusion process** — no gaps; if a contract trades at 46.05 and later 46.08, it must have traded at 46.06 and 46.07 in between. Analogy: temperature readings — 25° to 22° must pass through 24° and 23°. Graph it and "one would never lift the pen from the paper."
- **Jump process** — the price is fixed, then instantaneously jumps to a new fixed level. Analogy: a central bank's discount rate, fixed until the Fed announces a change.
- **Jump-diffusion process** — the real world: normal trading most of the time, punctuated by occasional instantaneous gaps.

**Describing a real distribution beyond mean and standard deviation:**
- **Skewness** — the lopsidedness, the extent to which one tail is longer. **Positive** skew = longer right tail (the lognormal distribution is positively skewed); **negative** skew = longer left tail. A normal distribution has skewness **0**.
- **Kurtosis** — how tall or flat the center is. **Positive** kurtosis = tall peak (**leptokurtic**); **negative** = flat peak (**platykurtic**); normal = **mesokurtic**, kurtosis **0**. (Mathematically a true normal distribution has kurtosis 3, but 3 is conventionally subtracted.)

**The "fat tail" signature.** Real exchange-traded markets consistently show, relative to a true normal distribution: **more days with small moves**, **more days with big moves**, and **fewer days with intermediate moves**. Picture a normal distribution whose midsections have been squeezed inward, forcing the peak up and the tails out. Distinguish this from a low-standard-deviation distribution, which also has a tall peak but **short** tails.

**Alternative models and their costs:**

| Model | Relaxes | Extra inputs required | Adoption |
|---|---|---|---|
| **Jump-diffusion** (Merton 1976) | Continuous trading | Average size of a jump; frequency of jumps | Rare — "unless the user can accurately estimate these new inputs, the values… may be no better—and might be worse" |
| **Constant-elasticity of variance (CEV)** (Cox-Ross 1976) | Volatility independent of price | A mathematical relationship between volatility and underlying price | "Has not found wide acceptance among option traders" |
| **Stochastic volatility models** | Constant volatility | A volatility process | "Add an additional dimension of complexity to a trader's life and for this reason are not widely used" |
| **Specialized interest-rate models** | All of the above, for bonds | — | Necessary, because bonds pull to par |

## Assumption 1 — Markets are frictionless
- **Locked futures markets.** A daily price limit halts trading until the market comes off its limit; if it does not, trading resumes only the next business day. **Workarounds:** trade the cash market; trade a **futures spread** where one leg is not locked (buy the June/March spread, then buy back the March future, leaving the intended long June); or trade **synthetic futures** in the option market if it is not locked.
- **Circuit breakers.** Stock exchanges halt trading when a designated index moves a predetermined percentage in a day; the circuit breakers specify how long.
- **Short-sale restrictions.** If a trader cannot freely sell stock, **put prices become inflated relative to calls**, and conversions and reversals appear mispriced. "Many stock option traders, as a matter of good trading practice, will try to carry some long stock so that they will always be in a position to sell stock if the need arises."
- **Unlimited borrowing is "a more serious weakness."** If money were free, margin would never be a problem — borrow it, deposit it, and collect the clearinghouse's interest at the same rate. In reality, a trader who cannot meet a margin call **may be forced to liquidate before expiration** — and every model, even those allowing early exercise, assumes you always have the option of holding to expiration. Assess risk not just as total possible loss but as **how much margin might be required to maintain the position over time**.
- **Borrowing and lending rates differ**, and "the greater the difference between borrowing and lending rates, the less reliable will be the values generated by the model."
- **Taxes are usually secondary** — "differences in tax consequences rarely make one strategy better than another" — though they matter in portfolio management and in dividend strategies taxed differently from gains.
- **Transaction costs are the serious flaw.** Brokerage fees, clearing fees, exchange membership — and they accrue not just on entry and exit but on **every adjustment**. "If a strategy will require many adjustments because it has a high gamma and the trader intends to remain approximately delta neutral, the transaction costs can have a significant impact on model-generated values."

## Assumption 2 — Interest rates are constant
- Almost no trader initiates one trade and holds to expiration; they borrow and lend continuously, and futures option traders face changing margin and variation. Most **finance through their clearing firm at a variable rate**, the firm acting as a bank and reporting the effective rate each day.
- Even with a fixed rate, which rate? A **borrowing rate**, a **lending rate**, or a **short stock rebate** (whose level depends on how hard the stock is to borrow)?
- **Rate risk is small for short-dated options** — most actively traded options expire within a year, so rates would have to move dramatically to matter except for deeply in-the-money options, and option values are far more sensitive to underlying price and volatility.
- **But long-term options are different.** For stock options, higher rates raise the forward, raising calls and lowering puts, and the largest rho values belong to **deeply in-the-money long-term options**. With exchanges now listing long-term options, this deserves attention.

## Assumption 3 — Volatility is constant
- A model assumes one, two, and three standard deviation moves are **evenly distributed** over the option's life. In reality, high-volatility and low-volatility periods alternate; the model "sees one volatility and evaluates options accordingly."
- **Option value is path dependent under dynamic hedging.** Held to expiration with no hedging, an option's value depends only on the terminal price. But "in a world where a trader dynamically hedges an option position, the value of the option is in fact **path dependent**."
- **Which path helps depends on where the gamma is.** An at-the-money option's gamma **rises** toward expiration, so late volatility is worth disproportionately more to a long straddle — and the accompanying high theta punishes late quiet. An out-of-the-money option's gamma is **largest early**, so early volatility helps it and it is worth *more* in a falling-volatility scenario.
- **This does not invalidate Black-Scholes.** "The Black-Scholes model is a probabilistic model. A given volatility will, on average, result in a given value." Generate many random price paths at the same 28% volatility and simulate dynamic hedging, and each strike averages very close to the Black-Scholes value.
- **Interest-rate products break the assumption structurally.** "As a bond approaches maturity, the price of the bond moves inexorably toward par" — at maturity it has a fixed, known value regardless of rates. Even with randomly moving rates at constant volatility, instruments of different maturities have different rate sensitivities, and rates themselves vary by maturity. Hence specialized interest-rate models.

## Assumption 4 — Trading is continuous
- A gap violates the diffusion assumption directly. Exchange-traded contracts cannot follow a pure diffusion process unless the exchange is open 24/7 — overnight gaps are structural — and intraday news can move a price almost instantaneously.
- **Gaps hurt negative gamma and help positive gamma**, because in both cases there is no opportunity to adjust as the market moves.
- **Gap damage concentrates in high-gamma options** — at the money, close to expiration, in a low-volatility market. Repeating Chapter 9's warning: **"at-the-money options close to expiration in a low-volatility market are among the riskiest of options."** With one year left, a gap from 100 to 105 on a short straddle is unpleasant but survivable — the market could come back. With one day left, "there is not enough time for the market to retrace its movement," the short 100 calls immediately become deeply in-the-money with deltas of 100, and the trader is **naked short**.
- **Portfolio insurance died here.** "The proponents of portfolio insurance suffered their greatest setbacks on **October 19 and 20, 1987**, when the market made several large-gap moves. Because of the gaps, the portfolio insurers were unable to make continuous delta adjustments to their positions" — and the cost of protection proved far greater than expected.
- **The gap explains why implied volatility exceeds historical volatility.** "Both a trader's experience and empirical evidence seem to indicate that a traditional model, with its built-in diffusion assumption, tends to **undervalue** options in the real world." Part of the gap between average implied and average historical volatility is hedger demand, "but the possibility of these gaps tends to indicate that perhaps the values of options are in fact greater in the real world than is predicted by a traditional theoretical pricing model."
- **Practical consequence:** "As expiration approaches, experienced traders will tend to rely less and less on model-generated values and more on their own experience and intuition." And: "It can be very dangerous to sell a large number of at-the-money options close to expiration… New traders in particular are advised to avoid such positions. No risk manager will appreciate even experienced traders being short large numbers of at-the-money options as expiration approaches."

**Expiration straddles — the contrarian trade this implies.** If selling at-the-money options near expiration is dangerous, buying them may be sensible. Conventional wisdom focuses on rapid time decay, but "because the model does not know about the possibility of a gap in the underlying market, the risk is often greater than the reward." Many traders are intent on selling time premium as expiration nears, so **cheap at-the-money options are often available**. Because put-call parity makes underpriced calls imply underpriced puts, the natural vehicle is the **at-the-money straddle** — profiting from a gap in either direction. Two practical notes: near expiration the model is unreliable for **deltas** as well as values, so "traders who buy expiration straddles often abandon any attempt to remain delta neutral and simply sit on the position to expiration"; and because most of the time no gap occurs, **size the position to what you can afford to lose repeatedly**. The roulette logic from Chapter 5 applies: a bet worth 95 cents bought for 50 cents still loses 37 times in 38, but the one win more than covers the losses.

## Assumption 5 — Volatility is independent of the underlying price
- Ask a **stock index trader** whether his market gets more volatile rising or falling, and he says **falling**. Ask a **commodity trader** the same question and he says **rising**. "The volatility of a market is not independent of the price of the underlying contract."
- The **CEV model** addresses this — price changes remain random but volatility varies with the underlying price — at the cost of mathematical complexity and an extra input relating the two.

## Assumption 6 — Lognormal distribution of prices
Studies "tend to indicate that price-change distributions for almost all exchange-traded underlying markets exhibit characteristics that are very similar to the S&P 500 distribution": more small moves, more big moves, fewer intermediate moves than a true normal distribution predicts.

## Anti-patterns
- **Assuming you can always hold a position to expiration.** A margin call you cannot meet forces liquidation, and every model assumes you had the choice.
- **Ignoring adjustment transaction costs on a high-gamma strategy.**
- **Selling large numbers of at-the-money options close to expiration**, especially in a low-volatility market.
- **Trusting model deltas and values in the final days.**
- **Assuming a stochastic-volatility, jump-diffusion, or CEV model is automatically better.** Each demands inputs the trader must estimate, and a bad estimate makes the fancier model worse. "Many traders take the view that whatever weaknesses are encountered in a traditional model can be best offset through intelligent decision making based on actual trading experience rather than through the use of a more complex jump-diffusion model."
- **Buying expiration straddles in large numbers** because they are cheap. "Such strategies are likely to result in a loss more often than a profit, so an intelligent trader should only invest an amount that he can afford to lose."
- **Reading a tall-peaked distribution as low-volatility.** Positive kurtosis means a tall peak *and* fat tails.

## Reference Tables

**Figure 23-5 — Option values under three volatility scenarios** (underlying 100; 80 days to expiration; interest rate 0; volatility 28% in all three cases. With zero interest, calls and puts at the same strike have identical time premium and differ only by intrinsic value.)

| Strike | 70 | 75 | 80 | 85 | 90 | 95 | 100 | 105 | 110 | 115 | 120 | 125 | 130 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **Constant volatility (Black-Scholes)** | | | | | | | | | | | | | |
| Calls | 30.01 | 25.06 | 20.21 | 15.62 | 11.48 | 7.98 | **5.23** | 3.22 | 1.87 | 1.03 | **0.54** | 0.27 | 0.13 |
| Puts | 0.01 | 0.06 | **0.21** | 0.62 | 1.48 | 2.98 | **5.23** | 8.22 | 11.87 | 16.03 | 20.54 | 25.27 | 30.13 |
| Straddle | 30.01 | 25.12 | 20.42 | 16.24 | 12.96 | 10.96 | **10.46** | 11.44 | 13.74 | 17.06 | 21.08 | 25.54 | 30.26 |
| **Falling volatility (Figure 23-3)** | | | | | | | | | | | | | |
| Calls | 30.04 | 25.15 | 20.44 | 16.00 | 11.79 | 7.59 | **2.97** | 2.69 | 2.10 | 1.43 | **0.89** | 0.51 | 0.27 |
| Puts | 0.04 | 0.15 | **0.44** | 1.00 | 1.79 | 2.59 | 2.97 | 7.69 | 12.10 | 16.43 | 20.89 | 25.51 | 30.27 |
| Straddle | 30.08 | 25.30 | 20.88 | 17.00 | 13.58 | 10.18 | **5.94** | 10.38 | 14.20 | 17.86 | 21.78 | 26.02 | 30.54 |
| **Rising volatility (Figure 23-4)** | | | | | | | | | | | | | |
| Calls | 30.00 | 25.01 | 20.05 | 15.21 | 10.75 | 8.97 | **6.41** | 3.36 | 1.29 | 0.41 | **0.14** | 0.05 | 0.02 |
| Puts | 0 | 0.01 | **0.05** | 0.21 | 0.75 | 3.97 | 6.41 | 8.36 | 11.29 | 15.41 | 20.14 | 25.05 | 30.02 |
| Straddle | 30.00 | 25.02 | 20.10 | 15.42 | 11.50 | 12.94 | **12.82** | 11.72 | 12.58 | 15.82 | 20.28 | 25.10 | 30.04 |

**Figure 23-7 — Effect of a gap from 100 to 105 on a 100 straddle**

*Implied volatility 15%:*

| Time to expiration | 1 day | 1 week | 1 month | 3 months | 6 months | 1 year |
|---|---|---|---|---|---|---|
| Initial straddle value | 0.63 | 1.66 | 3.45 | 5.98 | 8.46 | 11.96 |
| Straddle gamma | **101** | 38 | 18 | 11 | 8 | 5 |
| Value after the gap | 5.00 | 5.01 | 5.58 | 7.39 | 9.57 | 12.90 |
| **Increase in value** | **4.37** | 3.35 | 2.13 | 1.41 | 1.11 | 0.94 |

*Implied volatility 25%:*

| Time to expiration | 1 day | 1 week | 1 month | 3 months | 6 months | 1 year |
|---|---|---|---|---|---|---|
| Initial straddle value | 1.04 | 2.76 | 5.76 | 9.97 | 14.09 | 19.90 |
| Straddle gamma | **61** | 23 | 11 | 6 | 4 | 3 |
| Value after the gap | 5.00 | 5.25 | 7.20 | 10.98 | 14.98 | 20.78 |
| **Increase in value** | **3.96** | 2.49 | 1.44 | 1.01 | 0.89 | 0.88 |

Both effects visible: the damage grows sharply **near expiration** and is greater in the **low-volatility** market.

**Figure 23-8 — Daily price-change distributions, January 2003–December 2012**

| | S&P 500 | Crude oil | Euro (vs dollar) | Bund |
|---|---|---|---|---|
| Number of days | 2,535 | 2,514 | 2,609 | 2,551 |
| Biggest up move | **+11.58%** (Oct 13, 2008) | **+14.27%** (Jul 31, 2008) | +3.52% (Mar 18, 2009) | +1.98% (Nov 1, 2011) |
| Biggest down move | **−9.03%** (Oct 15, 2008) | **−10.80%** (Jan 7, 2009) | −2.40% (Oct 24, 2008) | −1.50% (Sep 2, 2003) |
| Mean | +0.296% | +0.384% | +0.110% | +0.0170% |
| Standard deviation | 1.31% | 2.25% | 0.651% | 0.364% |
| Volatility | 20.81% | 35.67% | 10.52% | 5.81% |
| Skewness | −0.536 | −0.188 | **+0.0669** | −0.0270 |
| Kurtosis | **+10.4150** | +2.6550 | +1.4113 | +1.4388 |

All four are positively kurtotic ("fat tails"); the euro is the only positively skewed one.

**Figure 23-11 — Probabilities of the biggest observed moves under a normal distribution**

| Product | One standard deviation | Biggest move | In standard deviations | Probability |
|---|---|---|---|---|
| S&P 500 | 1.31% | Up 11.58% | Up 8.84 s.d. | **Too unlikely to calculate** |
| | | Down 9.03% | Down 6.89 s.d. | 1 chance in 358,000,000,000 |
| Crude oil | 2.25% | Up 14.27% | Up 6.34 s.d. | 1 chance in 8,700,000,000 |
| | | Down 10.80% | Down 4.80 s.d. | 1 chance in 1,260,000 |
| Euro | 0.651% | Up 3.52% | Up 5.41 s.d. | 1 chance in 31,700,000 |
| | | Down 2.40% | Down 3.69 s.d. | 1 chance in 8,900 |
| Bund | 0.364% | Up 1.98% | Up 5.40 s.d. | 1 chance in 30,000,000 |
| | | Down 1.50% | Down 4.10 s.d. | 1 chance in 48,000 |

The samples cover only 2,500–2,600 days. Nassim Taleb calls such occurrences **"black swans."**

## Worked Example

**A. Two price paths, identical volatility, wildly different option values.** Figures 23-3 and 23-4 are daily high/low/close bar charts over 80 trading days. They are **mirror images of each other**, so both realize exactly **28%** close-to-close volatility, and both begin and end at **100**. Only the *order* differs: Figure 23-3 has large moves early and small moves late (**falling volatility**); Figure 23-4 the reverse (**rising volatility**).

Assume 80 calendar days to expiration, every day a trading day, interest rate 0. Black-Scholes at 28% gives the 100 call and put **5.23** each — a straddle worth **10.46**.

Now simulate the dynamic hedging process instead: each day, using the closing price, days remaining, and the known 28% volatility, compute the delta and trade the underlying to stay neutral.

| Scenario | 100 call | 100 put | 100 straddle |
|---|---|---|---|
| Black-Scholes (constant volatility) | 5.23 | 5.23 | **10.46** |
| Falling volatility (Figure 23-3) | 2.97 | 2.97 | **5.94** |
| Rising volatility (Figure 23-4) | 6.41 | 6.41 | **12.82** |

*Why.* The 100 straddle is at the money, and an at-the-money option's **gamma rises as expiration approaches**. So volatility arriving late is magnified — greater delta swings, greater rehedging profit — pushing the rising-volatility value well **above** Black-Scholes. Conversely, high gamma late goes with high theta late, so a quiet endgame decays the straddle fast, pushing the falling-volatility value well **below** Black-Scholes.

*And the out-of-the-money options invert it,* because their gamma is largest **early**:

| Option | Black-Scholes @ 28% | Falling volatility | Rising volatility |
|---|---|---|---|
| 80 put | 0.21 | **0.44** | 0.05 |
| 120 call | 0.54 | **0.89** | 0.14 |

Across the whole strike grid: with the underlying staying roughly between 95 and 105, the **95, 100, and 105** strikes are worth more than Black-Scholes in a rising-volatility market and less in a falling one; strikes **below 90 or above 110** are the opposite.

**B. Sizing the gap risk on a short straddle.** A trader is short a 100 straddle with the underlying at 100. The market gaps instantly to **105**. He cannot adjust — a gap gives no opportunity.

At **15% implied volatility**, a one-year straddle worth 11.96 becomes 12.90 — a loss of **0.94**, unpleasant but survivable, and the market has a year to come back. A **one-day** straddle worth 0.63 becomes 5.00 — a loss of **4.37**, nearly seven times as large in absolute terms and vastly larger in percentage terms. The gamma tells the story: **101** for the one-day straddle versus **5** for the one-year.

At **25% implied volatility** the one-day damage is **3.96** — still severe, but less than the 4.37 in the low-volatility market, because the one-day gamma is 61 rather than 101.

**C. How improbable are the observed extremes?** The S&P 500's largest up move over 2003–2012 was **+11.58%** against a daily standard deviation of **1.31%** — an **8.84 standard deviation** event, with probability roughly **1 in 2,000,000,000,000,000,000** (2 quintillion). The largest down move, **−9.03%**, is a **6.75–6.89 standard deviation** event at roughly **1 in 350–358 billion**. "Simply put, the likelihood of either of these occurrences is so small that they will essentially never occur" — under a normal distribution. Both occurred **within one 2,535-day sample**, days apart, in October 2008.

Even the tamest market in the sample, the Bund, produced a **5.40 standard deviation** up move whose normal-distribution probability is **1 in 30,000,000** — within 2,551 observations.

**D. When to buy an expiration straddle.** Three days to expiration; the Black-Scholes value of an at-the-money call is **0.75**. We do not know the true value, but "there is high likelihood that in the real world the call is worth **more** than 0.75 because the model doesn't know about the possibility of a gap in the market." If the call is also trading **below** its model value — say **0.65** — it is likely a good buy. Put-call parity makes the same-strike put equally underpriced, so buy the **straddle** and profit from a gap in either direction. Then, because the model's deltas are as unreliable as its values this close to expiration, most traders simply **hold to expiration without adjusting**.

## Key Takeaways
- Two model risks: wrong inputs and wrong assumptions. The Greeks handle the first; only judgment handles the second.
- Frictions that matter most: transaction costs (which scale with adjustment frequency) and the inability to borrow without limit (which can force liquidation before expiration).
- Interest-rate risk is minor for short-dated options but real for deeply in-the-money long-term ones.
- Under dynamic hedging, option value is **path dependent**: late volatility inflates at-the-money options, early volatility inflates out-of-the-money ones. Black-Scholes is right only on average, across many paths.
- Real prices follow a jump-diffusion process. Gaps hurt negative gamma, help positive gamma, and do the most damage to at-the-money options near expiration in a low-volatility market.
- The diffusion assumption makes traditional models **undervalue** options — one reason implied volatility persistently exceeds historical volatility, and the reason portfolio insurance failed in October 1987.
- Real distributions are fat-tailed: more small moves, more big moves, fewer intermediate moves. Observed extremes are astronomically improbable under a normal distribution and yet routinely occur.
- More sophisticated models (jump-diffusion, CEV, stochastic volatility) trade assumption error for input error; most traders prefer to correct a traditional model with experience.

## Connects To
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): the frictionless assumptions first raised there, and the path-dependence simulation.
- [Chapter 6 — Volatility](ch06-volatility.md): the lognormal assumption examined here empirically.
- [Chapter 9 — Risk Measurement II](ch09-risk-measurement-ii.md): why at-the-money gamma peaks near expiration in low volatility.
- [Chapter 17 — Hedging with Options](ch17-hedging-with-options.md): portfolio insurance and its 1987 failure.
- [Chapter 18 — The Black-Scholes Model](ch18-black-scholes-model.md): the model whose assumptions are audited here.
- [Chapter 20 — Volatility Revisited](ch20-volatility-revisited.md): the persistent gap between implied and realized volatility.
- [Chapter 24 — Volatility Skews](ch24-volatility-skews.md): how the market itself prices skewness and kurtosis into option prices.
