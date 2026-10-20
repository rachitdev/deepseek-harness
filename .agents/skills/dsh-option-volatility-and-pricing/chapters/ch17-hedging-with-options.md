# Chapter 17 — Hedging with Options

## Core Idea
A futures contract transfers **all** the risk; an option transfers **part** of it — which makes an option far closer to a traditional insurance policy. Every hedging decision is therefore a tradeoff: **what is the hedger willing to give up under one set of market conditions in return for protection under a different set?** "A hedger with a long position who wants to protect his downside will almost certainly have to give up something on the upside."

## Frameworks Introduced

**Who hedges, and why.**
- **Natural longs** — a commodity **producer** (profits when prices rise); a **lender** (profits when rates rise).
- **Natural shorts** — a commodity **user**; a **borrower**.
- **Voluntary positions** — a speculator wanting to temporarily reduce risk; a fund manager who expects a short-term decline and finds hedging cheaper than selling the portfolio and buying it back.

Note that hedgers are a **minority** of participants in most option markets — arbitrageurs, speculators, and spreaders typically outnumber them.

**The four basic hedges** (Figure 17-7):

| Position | Hedging strategy | Advantages | Disadvantages |
|---|---|---|---|
| **Long underlying** | Sell a future or forward | No downside risk | No upside profit potential |
| | **Buy a protective put** | Limited downside risk; unlimited upside profit | Cost of the option |
| | **Sell a covered call** | Partial downside protection equal to the price of the call | Unlimited downside risk; limited upside profit |
| | **Long collar** (long protective put / short covered call) | Limited downside risk | Limited upside profit potential |
| **Short underlying** | Buy a future or forward | No upside risk | No downside profit potential |
| | **Buy a protective call** | Limited upside risk; unlimited downside profit | Cost of the option |
| | **Sell a covered put** | Partial upside protection equal to the price of the put | Unlimited upside risk; limited downside profit |
| | **Short collar** (long protective call / short covered put) | Limited upside risk | Limited downside profit potential |

**The synthetic identity of every hedge:**
```
Long underlying  + long put   ≈ synthetic long call     (protective put)
Short underlying + long call  ≈ synthetic long put      (protective call)
Long underlying  + short call ≈ synthetic short put     (covered call)
Short underlying + short put  ≈ synthetic short call    (covered put)
Long collar  = a bull vertical spread
Short collar = a bear vertical spread
```

**The volatility rule for hedgers:**
> All other factors being equal, in a **high**-implied-volatility market, a hedger should buy **as few** options as possible and/or sell **as many** options as possible. Conversely, in a **low**-implied-volatility market, a hedger should buy **as many** options as possible and/or sell **as few** as possible.

**Three questions before choosing a strategy:**
1. Does the hedge need to offer protection against a worst-case scenario?
2. How much of the current directional risk should the hedge eliminate?
3. What additional risks is the hedger willing to accept?

**Portfolio insurance (option replication).** When no option market exists, replicate the option you want. Feed exercise price, expiration, underlying price, interest rate, and volatility into a model — the strike and expiration are of your own choosing since none are listed; volatility must be estimated from historical price changes — then hold the fraction of the underlying equal to the synthetic call's delta, rebalancing as the delta changes. This is the **dynamic hedging process of Chapter 8**, used not to capture a mispricing but to manufacture an option that does not exist.

## Key Concepts
- **Deductible and premium.** In a protective option, the distance between the exercise price and the current underlying price is the **deductible**; the option's price is the **premium**. Lower deductible (a strike closer to the money) = more protection at a higher price.
- **Caps and floors.** A variable-rate **borrower** (short interest rates) buys an interest-rate **call** to **cap** his cost. A variable-rate **lender** (long interest rates) buys an interest-rate **put** to set a **floor** on his return.
- **Covered write strike selection.** Selling an in-the-money call gives high protection but eliminates most upside; selling an out-of-the-money call gives less protection but leaves upside room. **Many covered writes use at-the-money options** — least protection relative to ITM, least profit relative to OTM, but the **greatest time premium**, so the position appreciates most if the market stays put.
- **Buy/write.** Buying stock and simultaneously selling a call, quoted and executed as a single price (stock price − call price). With stock at 100 and the December 105 call at 2.00, the December 105 buy/write trades at **98.00**; a market maker might quote **97.90 – 98.10**. The **CBOE BuyWrite Index (BXM)** tracks buying an S&P 500 portfolio and each month selling a slightly out-of-the-money one-month SPX call.
- **Covered writes as target-price orders.** Sell a call struck at your target *sale* price: either the stock is called away at your target, or you keep the premium. Sell a put struck at your target *purchase* price: either you are assigned and buy at your target, or you keep the premium. Companies use the latter for **stock buy-back programs**.
- **Securing the two sides differs.** A short call is secured by **owning the stock**; a short put must be **cash-secured** — cash equal to the exercise price on deposit, or, for a European put with no early exercise, the **present value X/(1 + r·t)**.
- **Choosing between protective and covered.** Theoretically, the same criterion a trader uses — price versus value, expressed as implied volatility against expected volatility. Buy protection when options look cheap; write covered options when they look rich.
- **But practical needs can override theory.** "If a hedger knows that a move in the underlying contract beyond a certain price will represent a threat to his business, then the purchase of a protective option at that exercise may be the most sensible strategy regardless of whether the option is theoretically overpriced."
- **The insurance objection, answered.** Hedgers often resist buying protection: "Why should I pay for an option when I will probably lose the premium?" True — most protective options expire out of the money. But most insurance policies also expire without a claim; that is how insurers profit. "Most people do not buy insurance to make a profit. They do so for the peace of mind that the insurance policy affords."
- **Collar synonyms**: fence, tunnel, cylinder, range forward, split-strike conversion. Long/short refers to the **underlying** position.
- **Collar Greeks.** A long collar always has a **positive delta**, a short collar a **negative delta**. If the underlying is closer to the **protective** option: positive gamma, negative theta, positive vega. If closer to the **covered** option: negative gamma, positive theta, negative vega. Unless one option is far further out of the money, the two largely cancel, leaving small gamma, theta, and vega — and strikes can be chosen to make any of them approximately neutral.
- **Zero-cost collar** — when the protective option and covered option cost the same.
- **Ratio writes and their danger.** In a high-implied-volatility market a hedger aiming to hedge 50% of a long position could sell several OTM calls totaling 50 deltas rather than one ATM call. But this adds **unlimited upside risk** on top of the covered call's unlimited downside risk. "Most hedgers want to restrict their unlimited risk to one direction, usually the direction of their natural position… A hedger who constructs a position with unlimited risk in either direction is presumably taking a volatility position."
- **Hedging with volatility spreads.** Buy or sell calendar spreads or butterflies with the delta you need. With the underlying at 100 and implied volatility **low**, protect a short position by **buying** the 110 call calendar (positive delta, and cheap); if its delta is +25, buy two per short underlying to hedge 50%. With implied volatility **high**, **sell** a calendar — but you must drop to a lower strike to get a positive delta, e.g. **sell the 90 call calendar**; at +25 deltas, sell it three times per underlying to hedge 75%.
- **Hedging with vertical spreads.** Work around the at-the-money option. To protect a long position with the underlying at 100, execute a bear vertical (sell the lower strike, buy the higher). If implied volatility is **high**, sell the at-the-money option and buy a higher strike. If **low**, buy an in-the-money option and sell a lower strike. Both have negative delta and positive edge because the ATM option is most volatility-sensitive.
- **Don't be paralyzed.** Faced with the multitude of strategies, "perhaps a better approach is to consider a limited number of strategies (perhaps four or five) that make sense and compare the various risk-reward characteristics."

## Mental Models
- **Volatility destroys compounded returns.** Average annual return is not the same as total return; the Sharpe ratio (`average return / standard deviation of returns`, suggested by William Sharpe in 1966) captures the tradeoff. High volatility is acceptable *only* if the average return is commensurably higher.
- **You are already synthetically long or short an option.** Every hedge is a synthetic option position, so the choice of hedge is really a choice of which option you want to own or be short.

## Anti-patterns
- **Selling more calls than you can cover** while calling it a hedge — that is a volatility position with unlimited risk in both directions.
- **Refusing to buy protection because the premium usually expires worthless.**
- **Buying protective options in a wildly overpriced market when you have no interest in upside profit** — hedge in the futures or forward market instead. (Though "if option prices are high, there is usually a valid reason.")
- **Buying many cheap out-of-the-money options to reach a delta target when implied volatility is high** — buy one option with the delta you need instead.
- **Trusting portfolio insurance to deliver put-like protection at model cost.** After the 1987 crash practitioners learned it works only if the inputs are right and the model's assumptions hold. "No one foresaw the dramatic increase in volatility resulting from the crash, so the volatility input that was being used was clearly incorrect," and the real cost of replication proved far higher than anticipated. Some studies suggest the dynamic hedging it required **exacerbated** the October 19, 1987 crash, as insurers had to sell ever more index futures in a cascade. The strategy fell out of favor. (The firm most closely associated with it was Leland, O'Brien, Rubinstein of Los Angeles.)

## Reference Tables

**Cost of protection at different strikes** (American firm hedging a €1,000,000 payable; euro at 1.35)

| Protective call strike | Price | Total cost of insurance | Protection begins above |
|---|---|---|---|
| 1.35 | 0.04 | 0.04 × 1,000,000 = **$40,000** | 1.35 |
| 1.40 | 0.02 | 0.02 × 1,000,000 = **$20,000** | 1.40 |
| 1.45 | 0.01 | 0.01 × 1,000,000 = **$10,000** | 1.45 |

**Covered write choices** (stock at 100)

| Call sold | Price | Break-even down to | Upside participation | Profit if called away |
|---|---|---|---|---|
| 95 call (ITM) | 6.50 | 93.50 | none | the 1.50 of time premium |
| 105 call (OTM) | 2.00 | 98.00 | up to 105 | up to 105 plus the premium |

**Figure 17-8 — Volatility versus total return**

| | Yr 1 | Yr 2 | Yr 3 | Yr 4 | Yr 5 | Average annual return | Total 5-year return | Standard deviation | Sharpe ratio |
|---|---|---|---|---|---|---|---|---|---|
| Manager 1 | +19% | −14% | +27% | −9% | +22% | +9% | **+44.29%** | 17.01% | **0.5291** |
| Manager 2 | +25% | −20% | −23% | +44% | +24% | +10% | +37.49% | 26.71% | 0.3744 |
| Manager 3 | +35% | +15% | −35% | +65% | −20% | **+12%** | +33.20% | 36.28% | 0.3308 |
| Manager 4 | +8% | +8% | +8% | +8% | +8% | +8% | **+46.93%** | 0 | — |

## Worked Example

**A. A currency hedge, priced.** An American firm expects to take delivery of €1 million of German goods in six months, payable in euros — so it is **short euros against dollars**. At 1.35 the goods cost **$1,350,000**; if the euro rises to 1.45 at delivery they cost **$1,450,000**.

Hedge by buying a **1.40 call on euros**, underlying €1 million, expiring on the payment date. Above 1.40 the firm exercises and buys euros at 1.40; below 1.40 it lets the call expire and buys in the open market. Cost: the 1.40 call at 0.02 → **$20,000** paid regardless of outcome.

*The upside remains open.* If the euro falls to 1.25, the call expires unexercised and the firm buys €1 million for **$1,250,000** — a windfall of **$100,000** against the original 1.35 expectation.

*The deductible tradeoff.* A **1.45** call at 0.01 costs only **$10,000** but leaves the firm exposed all the way up to 1.45. A **1.35** call at 0.04 costs **$40,000** and protects from 1.35 upward. Note the synthetic identity: short euros + long 1.40 call ≈ **owning a 1.40 put**.

**B. Why volatility of returns matters more than average returns.** Take the average annual returns at face value:
- Manager 1: (19 − 14 + 27 − 9 + 22)/5 = **+9%**
- Manager 2: (25 − 20 − 23 + 44 + 24)/5 = **+10%**
- Manager 3: (35 + 15 − 35 + 65 − 20)/5 = **+12%**

Manager 3 looks best. Now compound the actual returns:
- Manager 1: 1.19 × 0.86 × 1.27 × 0.91 × 1.22 = **1.4429** → **+44.29%**
- Manager 2: 1.25 × 0.80 × 0.77 × 1.44 × 1.24 = **1.3749** → **+37.49%**
- Manager 3: 1.35 × 1.15 × 0.65 × 1.65 × 0.80 = **1.3320** → **+33.20%**

**The best average return produced the worst portfolio.** Manager 1, with the lowest average return of the three, ended **11 percentage points ahead** of Manager 3 over five years. Manager 4, plodding at exactly 8% every year with zero standard deviation, beat them all at **+46.93%**. Sharpe ratios rank them correctly: 0.5291, 0.3744, 0.3308.

This is the second, less obvious benefit of hedging: it does not only cap disasters, it **raises compounded returns by damping the volatility of returns**.

**C. Replicating a put you cannot buy (portfolio insurance).** You are long 1,000 shares and want downside protection, but no option market exists for the stock.

`Long stock + long put ≈ long call`, so what you actually want is a **long call** at the strike and expiration of the put you cannot buy. Choose the strike and expiration freely (none are listed), read the stock price and interest rate off the market, and estimate volatility from a historical price database. Suppose the model gives the intended call a **delta of 75**.

- Replicate 75% of the underlying: sell 25% of the holding — **250 shares**, leaving **750 shares**.
- Later, market conditions change and the delta falls to **60**: sell another 15% of the *original* holding — **150 shares**, leaving **600 shares**.
- Continue rebalancing to match the call's delta; at the target expiration date buy back enough stock to return to the full 1,000 shares.

At institutional scale: a manager with a **$100 million** portfolio who wants to insure against a fall below **$90 million** either buys a $90 million put or replicates the **$90 million call**, adjusting continuously. Rather than trading the portfolio itself (expensive in transaction costs), managers mimicked the delta adjustments by **buying and selling index futures**. Portfolio insurance firms charged a fee for estimating the correct volatility and choosing the pricing model, and sometimes earned brokerage on the futures adjustments as well.

## Key Takeaways
- Options transfer part of the risk, not all of it — the insurance analogy is literal, deductible and premium included.
- Every hedge is a synthetic option: protective put = long call, protective call = long put, covered call = short put, covered put = short call, collar = vertical spread.
- Strike choice sets the deductible; a further-out strike costs less and protects less.
- Buy protection when implied volatility is low; write covered options when it is high — but a real business threat can justify buying overpriced protection.
- In a high-volatility market buy few options and sell many; in a low-volatility market do the reverse — never at the cost of creating unlimited risk in both directions.
- Cash-secure short puts at X (or X/(1+r·t) for European puts); short calls are secured by the stock.
- Hedging reduces the volatility of returns, and lower volatility of returns raises the compounded total return — the Sharpe-ratio argument.
- Portfolio insurance replicates an option through dynamic hedging when no option exists, but it depends entirely on the volatility input and the model's assumptions holding.

## Connects To
- [Chapter 14 — Synthetics](ch14-synthetics.md): the synthetic identity behind every hedge.
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): the replication process portfolio insurance uses.
- [Chapter 12 — Bull and Bear Spreads](ch12-bull-and-bear-spreads.md): collars are vertical spreads; the at-the-money strike-selection rule.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): calendar spreads and butterflies used as hedges.
- [Chapter 1 — Financial Contracts](ch01-financial-contracts.md): the original insurance framing of an option.
- [Chapter 23 — Models and the Real World](ch23-models-and-the-real-world.md): why the assumptions behind portfolio insurance failed in 1987.
