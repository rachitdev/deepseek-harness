# Chapter 10 — Introduction to Spreading

## Core Idea
"Most successful option traders are spread traders." Theoretical values are only reliable in the long run, but a trader must survive the short run to reach it. **Spreading maintains profit potential while reducing short-term risk** — it does not trade edge away, it distributes exposure. The casino analogy is the chapter's whole argument in miniature: a casino's 5% edge is the same whether one player bets $2,000 on one number or thirty-eight players bet $1,000 on all thirty-eight numbers, but only in the second case is the profit certain.

## Frameworks Introduced

**Definition.** A spread is "a strategy that involves taking opposing positions in different but related instruments." Most commonly the legs move in opposite directions as conditions change. **A profitable spread is predicated on the assumption that the values will change at different rates** — if both legs changed at the same rate, the spread's value would never change.

**Taxonomy of spreads.**
| Type | Legs | Depends on |
|---|---|---|
| **Cash-and-carry** | Physical + forward | Interest, storage, insurance |
| **Intramarket (calendar)** | Two futures months, same commodity | Carry differential between months |
| **Intermarket** | Two different commodities or instruments | An observed price relationship |
| **Ratio** | Unequal contract counts | A price multiple (gold vs silver, corn vs soybeans, S&P 500 vs Dow) |
| **Yield-curve (NOB)** | 10-year notes vs 30-year bonds | Relative movement of short and long rates |
| **Crack spread** | Crude oil vs gasoline and heating oil | Refining cost and product demand |
| **Crush spread** | Soybeans vs soybean oil and meal | Processing cost and product demand |

**Option spread types by Greek.** Take opposing positions in a Greek rather than in direction:
- **Delta spread** — opposing but *unequal* delta positions; value depends on directional movement.
- **Gamma spread** — long gamma in one option, short in another; value depends on the **realized volatility** of the underlying. The dynamic hedges of Chapter 8 are typical gamma spreads: the underlying has no gamma, so the whole position carries the option's gamma.
- **Vega spread** — value depends on changes in **implied volatility**.
- **Rho spread** — value depends on changes in interest rates.

**Dynamic vs. static option spreads.** Many require periodic adjustment. A spread can be **static** — carried to expiration without adjustment — but "this is usually done only when the risk characteristics of the spread are well defined and limited."

**Three reasons option markets are the natural home of spreading:**
1. **Relative mispricing.** You may not know either contract's exact value, but you can estimate their *relative* value. In option markets the mispricing is expressed in **volatility**, not points.
2. **Constructing a view.** Options combine in almost infinite ways to profit under a chosen set of conditions while limiting loss under others — but only if the execution prices make it worthwhile.
3. **Risk control.** "Spreading is the primary method by which traders limit the short-term effects of 'bad luck.'"

## Key Concepts
- **Why not scalp options.** Scalping — quoting a bid-ask around a perceived equilibrium and never holding a position — was once popular on futures floors and requires no theoretical value at all. But "option markets are rarely sufficiently liquid to support this type of trading."
- **Why not trade options directionally.** Volatility changes and time decay can defeat a correct directional call. "If a trader's sole consideration is direction, he is usually better advised to take the position in the underlying market."
- **Mispricing is best measured in volatility.** An option worth 7.00 trading at 8.00 (26% implied) is *less* overpriced than one worth 6.00 trading at 6.75 (28% implied), even though the point mispricing is larger — both valued at 23% volatility.
- **Spreading does not eliminate all risk.** A cash-and-carry removes directional risk but leaves interest-rate risk (unless borrowing is fixed) and storage/insurance-cost risk. Rising rates or storage costs widen the futures spread and can turn a profitable spread into a loss.
- **Reverse cash-and-carry is often impossible in commodities** — unlike financial instruments, physical commodities cannot be borrowed and sold short.
- **Less well-defined relationships carry more risk.** Gold/silver, corn/soybean, S&P/Dow ratio spreads rest on observed rather than arbitrage-enforced relationships, so they "carry greater uncertainty and therefore greater risk."
- **Execution: do the hard leg first.** If you must leg into a spread, execute the least liquid leg first. Doing the easy leg first can leave you with a naked position if the hard leg cannot be completed in time or at a reasonable price.
- **A spread quoted as one contract is cheaper than the sum of its legs.** A market maker taking the whole spread at once has less risk and will quote inside the implied bid-ask.
- **Reported leg prices don't matter arithmetically** — only that they sum to the agreed spread price. In practice exchanges want prices reflecting current market conditions, or the trade may look like unethical or illegal activity.
- **Access matters.** A trader could replicate a futures calendar spread with the physical and a forward, but physical commodity markets are dominated by large corporations — the futures spread is simpler and cheaper.

## Mental Models
- **The casino's betting limit.** A single $2,000 bet on one roulette number exposes the casino to a $70,000 loss ($72,000 payoff less the $2,000 taken in). Two $1,000 bets on *different* numbers cap the loss at $34,000 ($36,000 payoff less $2,000) because the outcomes are mutually exclusive — **with the identical 5% edge**. Thirty-eight $1,000 bets covering all 38 numbers is a perfect spread: one player collects $36,000 from $38,000 on the table, a **sure $2,000 profit**. Betting limits exist because "if the bet is large enough and the bettor happens to win, the short-term bad luck can overwhelm the casino." A $50,000-capitalized casino that loses $70,000 has no long run left.
- **From the player's side, the mirror.** If the odds are against you and you want the best chance of a profit, bet the maximum on one outcome and hope for short-run luck. Keep betting and probability catches up with you.
- **There is no risk-reward tradeoff in spreading.** "We tend to assume that there is a tradeoff between risk and reward. But the edge to the casino in both cases is still the same 5 percent."

## Anti-patterns
- **Scaling a profitable strategy without bounding the volatility error.** 4 × 1 for 2.00 of edge, 40 × 10 for 20.00, 400 × 100 for 200.00 — and if realized volatility is 45% rather than the estimated 35%, the calls sold at 4.00 are worth 4.50 and the hoped-for **+200.00 becomes −200.00**. Size to the margin for error, not to the liquidity.
- **Legging into a spread easy side first.**
- **Trading direction through options** when direction is the only view.
- **Comparing option mispricing in points rather than volatility.**
- **Assuming a spread eliminates every risk** just because the directional risk is gone.
- **Reporting arbitrary leg prices** that sum correctly but do not reflect the market.

## Reference Tables

**Cash-and-carry: two-month forward on a commodity** (cash $700, interest 6.00%, storage + insurance $5/month)

| Item | Calculation | Amount |
|---|---|---|
| Cash price | | $700 |
| Interest | $700 × 0.06 × 2/12 | +$7 |
| Storage and insurance | 2 × $5 | +$10 |
| **Fair two-month forward** | | **$717** |

If the forward trades at **$725**: buy the commodity, sell the forward, carry to maturity.

| Cash flow | Amount |
|---|---|
| Borrow $700 for two months at 6.00% | −$7 |
| Buy the commodity | −$700 |
| Store and insure for two months | −$10 |
| Receive the forward price at maturity | +$725 |
| **Profit** | **+$8** |

exactly the mispricing — and unaffected by whether the commodity goes to $800 or $600.

**Four-month forward and the calendar spread**

| Item | Calculation | Amount |
|---|---|---|
| Compounded interest | −(700 × 0.06/6) − (717 × 0.06/6) = −7.00 − 7.17 | −$14.17 |
| Commodity | | −$700.00 |
| Storage and insurance, four months | | −$20.00 |
| **Fair four-month forward** | 700.00 + 14.17 + 20.00 | **$734.17** |
| **Fair 2-month/4-month spread** | 734.17 − 717.00 | **$17.17** |

Market spread is **$20** → sell the spread (buy 2-month at $717, sell 4-month at $737); it is **$2.83 too expensive**.

**Carrying the calendar spread to maturity**

| Cash flow | Amount |
|---|---|
| Borrow $717 for two months at 6.00% | −$7.17 |
| Buy the commodity at the agreed two-month forward price | −$717.00 |
| Store and insure for two more months | −$10.00 |
| Deliver and receive the agreed four-month forward price | +$737.00 |
| **Total profit** | **+$2.83** |

**Spread quoted as one contract vs. legged**

| Contract | Bid | Ask |
|---|---|---|
| A | 128 | 131 |
| B | 47 | 49 |
| C | 68 | 70 |

Buy A, sell B and C. Legged: **bid = 128 − 49 − 70 = 9**, **ask = 131 − 47 − 68 = 16**. Quoted as a single spread, a market maker will typically show something tighter — perhaps **11 bid, 14 offer**.

**Mispricing measured two ways** (theoretical values computed at 23% volatility)

| Theoretical value | Option price | Point mispricing | Implied volatility |
|---|---|---|---|
| 7.00 | 8.00 | +1.00 | 26% |
| 6.00 | 6.75 | +0.75 | **28%** |

The second option is more overpriced in the terms that matter.

## Worked Example

**A. The intermarket ratio spread.** A trader observes over an extended period that Commodity B trades at three times Commodity A. Current prices: **A = 120, B = 390** — a multiple of **3.25**. Either A should be 130 (390/3) or B should be 360 (3 × 120).

Betting on reversion, buy three A and sell one B:

| Leg | Cash flow |
|---|---|
| +3 Commodity A at 120 | −360 |
| −1 Commodity B at 390 | +390 |
| **Total credit** | **+30** |

When the 3:1 relationship returns, the position closes at zero cost, leaving the **+30**. The profit is independent of the absolute price levels as long as the ratio holds.

**B. The NOB (notes over bonds) spread.** A 10-year Treasury note future at **116 14/32** and a 30-year bond future at **118 27/32** (Treasury prices are quoted in points and 32nds of par).

```
spread = 118 27/32 − 116 14/32 = 2 13/32
```
Treasury prices move opposite to rates. A trader who believes rates will rise, and that **long-term rates will rise faster than short-term**, sells the spread. If prices later reach 115 10/32 and 113 7/32:
```
new spread = 115 10/32 − 113 7/32 = 2 3/32
profit = 2 13/32 − 2 3/32 = 10/32
```

**C. The 3:2:1 crack spread.** Three gallons of crude oil yield two gallons of gasoline and one of heating oil.
```
price of the 3:2:1 crack spread = (2 × gasoline) + (1 × heating oil) − (3 × crude oil)
```
Because refined products are worth more than crude, a trader **buys** the spread by buying the products and selling crude. Rising refining costs or rising product demand **widen** the spread; falling costs or demand **narrow** it. Sell the spread if you expect refined-product demand to fall.

**D. The sizing decision — the chapter's central risk lesson.** A trader estimates 35% volatility for an underlying and finds a call priced at 4.00 with a theoretical value of 3.50 and a delta of 25. He sells 4 calls, buys 1 underlying, and dynamically hedges: total edge = 4 × 0.50 = **2.00**.

Scale it up: 40 × 10 gives **20.00**; 400 × 100 gives **200.00**. But 35% is an estimate. At 45% realized volatility those calls are worth **4.50**, and the 400 × 100 position turns a hoped-for +200.00 into a **−200.00 loss**.

The disciplined framing is not "how much liquidity is there" but "how much error can I survive." A trader who can tolerate volatility up to 40% (a 5-point margin) might do 40 × 10. If a spreading construction raises his breakeven volatility to 45% (a 10-point margin), 400 × 100 becomes defensible. "No trader will survive very long if his livelihood depends on estimating each input with 100 percent accuracy."

## Key Takeaways
- A spread takes opposing positions in related instruments and profits when their values change at different rates.
- Spreading preserves theoretical edge while cutting short-term variance — the casino's betting-limit logic.
- In option markets, spread the Greeks: delta (direction), gamma (realized volatility), vega (implied volatility), rho (rates).
- Express option mispricing in volatility points, not currency points.
- Size positions to the margin for error in your volatility estimate, not to the available edge.
- Execute the hardest leg first, and prefer a single quoted spread price to legging in.
- Relationship-based spreads (ratios, intermarket) are less reliable than arbitrage-enforced ones and deserve more caution.

## Connects To
- [Chapter 2 — Forward Pricing](ch02-forward-pricing.md): the carry arithmetic behind cash-and-carry and calendar spreads.
- [Chapter 5 — Theoretical Pricing Models](ch05-theoretical-pricing-models.md): the roulette-table edge this chapter re-uses.
- [Chapter 8 — Dynamic Hedging](ch08-dynamic-hedging.md): the archetypal gamma spread.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): the concrete option spread catalog.
- [Chapter 12 — Bull and Bear Spreads](ch12-bull-and-bear-spreads.md): directional option spreads.
- [Chapter 13 — Risk Considerations](ch13-risk-considerations.md): margin for error made explicit.
