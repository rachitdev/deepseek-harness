# Chapter 14 — Synthetics

## Core Idea
Options can be combined with each other and with the underlying to build positions that are **almost identical** to some other contract. A long June 100 call plus a short June 100 put always ends up buying the underlying at 100 at expiration — by choice if the market is above 100 (exercise the call), by force if it is below (assigned on the put). The underlying price is irrelevant to the outcome. That single fact generates six synthetic contracts, explains why call and put versions of the same spread are the same strategy, and gives every trader multiple routes to any position.

## Frameworks Introduced

**The six synthetics** (all options same expiration, same exercise price):

```
synthetic long underlying   ≈ long call  + short put
synthetic short underlying  ≈ short call + long put
synthetic long call         ≈ long underlying  + long put
synthetic short call        ≈ short underlying + short put
synthetic long put          ≈ short underlying + long call
synthetic short put         ≈ long underlying  + short call
```

Written out for a specific strike:

| Synthetic | Construction |
|---|---|
| Long underlying | +June 100 call, −June 100 put |
| Short underlying | −June 100 call, +June 100 put |
| Long June 100 call | +underlying, +June 100 put |
| Short June 100 call | −underlying, −June 100 put |
| Long June 100 put | −underlying, +June 100 call |
| Short June 100 put | +underlying, −June 100 call |

**The memory rule:**
> If we trade a single option and hedge it with an underlying contract, we have the same position, synthetically, in the **companion option** (the opposite type at the same exercise price).

- Buy a call, sell the underlying → synthetically **bought a put**
- Sell a call, buy the underlying → synthetically **sold a put**
- Buy a put, buy the underlying → synthetically **bought a call**
- Sell a put, sell the underlying → synthetically **sold a call**

**Delta, gamma, vega relationships that fall out.**
- A synthetic underlying has delta ≈ **100**, so **|call delta| + |put delta| ≈ 100** for companion options. If the June 100 call's delta is 75, the June 100 put's is about −25; if the put's is −60, the call's is about 40. (Settlement procedure, interest rates, and early exercise can push this slightly off 100.)
- The underlying has **zero gamma and zero vega**, so a long call plus a short put must sum to zero on both — meaning **companion calls and puts have identical gamma and identical vega**. June 100 call gamma 5 → June 100 put gamma 5. June 105 put vega 0.20 → June 105 call vega 0.20.

**The consequence for volatility traders:**
> Option traders who focus on volatility make **no distinction between calls and puts** with the same exercise price and expiration date. "If a trader owns a call and would prefer instead to own a put he need only sell the underlying contract." The volatility risk of a position depends **not on whether the contracts are calls or puts, but on the exercise prices and expiration dates** which make up the position.

**Why theta is the exception.** Companion thetas need not sum to zero, because of **cost of carry**:
- **Stock**: buying stock that does not move loses money through borrowing cost, so the synthetic equivalent has a nonzero theta.
- **Futures options under stock-type settlement**: no carry on the future, but carry on the options; if companion options trade at different prices the carry differs, and the synthetic underlying has a nonzero theta.
- **Futures options under futures-type settlement**: no carry on either the underlying or the options — **companion calls and puts do have the same theta.**

## Key Concepts
- **A synthetic long underlying is really a synthetic forward.** It does not become an underlying contract until expiration, and its pricing depends on the value of a forward contract.
- **Any exercise price works.** A long June 110 call + short June 110 put is still a synthetic long underlying — it just buys the underlying at 110 at expiration.
- **Characteristics come from slopes, not prices.** In the parity-graph constructions the underlying position is taken at a price different from the exercise price, and the result is unchanged: "What gives the position its characteristics is not the prices of the contracts, but the slopes of the contracts."
- **Why a bull call spread equals a bull put spread.** Substitute the synthetic:

| Bull call spread | Synthetic equivalent |
|---|---|
| +1 June 100 call | +1 June 100 put / +1 underlying |
| −1 June 105 call | −1 June 105 put / −1 underlying |

The underlying legs cancel, leaving **+1 June 100 put / −1 June 105 put** — a bull put spread. The two differ only in cash flow: the call spread is a **debit**, the put spread a **credit**. With a maximum spread value of 5.00 and zero interest, the two must sum to **5.00**: call spread at 3.00 → put spread at 2.00. With nonzero rates and stock-type settlement, they sum to the **present value** of 5.00.
- **Three ways to buy a straddle:** (1) buy the call and buy the put, (2) buy the call and buy the put *synthetically*, (3) buy the put and buy the call *synthetically*. Which is best depends on the pricing of the synthetics against their real equivalents — the subject of Chapter 15.
- **Iron butterfly** — a strangle plus a straddle with the straddle centered exactly in the middle of the strangle. Same characteristics as a traditional butterfly, opposite cash-flow convention: a long butterfly (buy the wings, sell the body) is a **debit**, while the equivalent iron butterfly (buy the strangle, sell the straddle) is a **credit**, because the straddle sold is always worth more than the strangle bought. **Buying a traditional butterfly = selling an iron butterfly.**
- **Iron condor** — a long strangle plus a short strangle, one centered inside the other. A long condor is a debit; the iron condor equivalent (sell the inside strangle, buy the outside) is a **credit**, because the inside strangle sold is always worth more. **Buying a traditional condor = selling an iron condor.**
- **The additivity rule.** At expiration, butterfly + iron butterfly = the amount between exercise prices (present value of that amount, with interest). Same for condor + iron condor.
- **Synthetics reveal hidden structure.** A position that fits no named strategy often becomes recognizable when rewritten.

## Mental Models
- **Rewrite everything in one option type.** To identify an unfamiliar position, convert every put to a synthetic call (or every call to a synthetic put), cancel the offsetting underlying legs, and read what remains.
- **Substitution is free (in characteristics), not free (in price).** Every synthetic acts like its real equivalent, so the only question is which is cheaper.

## Anti-patterns
- **Treating a call spread and a put spread at the same strikes as different strategies.** They are the same position with a different cash flow.
- **Assuming companion calls and puts must have the same theta.** They only do when there is no cost of carry on either side.
- **Assuming a synthetic is exactly identical** — hence the ≈ sign throughout the chapter. Settlement, interest, and early exercise create small differences.

## Reference Tables

**Companion Greek relationships**

| Measure | Relationship between a call and its companion put |
|---|---|
| Delta | \|Δ_call\| + \|Δ_put\| ≈ 100 |
| Gamma | Identical |
| Vega | Identical |
| Theta | Identical **only** when there is no cost of carry on either the underlying or the options (futures options under futures-type settlement) |

**Debit/credit conventions**

| Position | Cash flow | Equivalent | Cash flow |
|---|---|---|---|
| Long butterfly (buy wings, sell body) | Debit | Short iron butterfly (buy strangle, sell straddle) | Credit |
| Long condor (buy outside, sell inside) | Debit | Short iron condor (sell inside strangle, buy outside strangle) | Credit |
| Bull call spread | Debit | Bull put spread | Credit |

**Value additivity at expiration** (zero interest; June 95/100/105 strikes)

| Structure | Price | Complement | Price |
|---|---|---|---|
| June 95/100/105 butterfly | 1.75 | June 95/100/105 iron butterfly | **3.25** |
| June 90/95/105/110 condor | 3.75 | June 90/95/105/110 iron condor | **1.25** |
| June 100/105 call spread | 3.00 | June 100/105 put spread | **2.00** |

## Worked Example

**A. Building an iron butterfly and proving it equals a long butterfly.** Combine two positions:
1. **+1 June 95 put / +1 June 105 call** — a long strangle
2. **−1 June 100 call / −1 June 100 put** — a short straddle

Rewrite each put as a synthetic call:

| Original position | Synthetic equivalent |
|---|---|
| +1 June 95 put | +1 June 95 call / −1 underlying |
| −1 June 100 call | −1 June 100 call |
| −1 June 100 put | −1 June 100 call / +1 underlying |
| +1 June 105 call | +1 June 105 call |

The long and short underlying contracts cancel, leaving:

```
+1 June 95 call
−2 June 100 calls
+1 June 105 call
```

— a **long butterfly**. (Expressing everything as puts instead gives a long put butterfly, confirming that call and put butterflies are the same position.)

Both want the market at 100 at expiration. Buy the butterfly for **1.75** or sell the iron butterfly for **3.25**: same profit and loss potential, and the two sum to the 5.00 strike interval (present value of 5.00 with nonzero rates).

**B. The same construction for a condor.** Combine a long June 90/110 strangle (+1 June 90 put, +1 June 110 call) with a short June 95/105 strangle (−1 June 95 put, −1 June 105 call). Rewrite the puts:

| Original | Synthetic |
|---|---|
| +1 June 90 put | +1 June 90 call / −1 underlying |
| −1 June 95 put | −1 June 95 call / +1 underlying |
| −1 June 105 call | −1 June 105 call |
| +1 June 110 call | +1 June 110 call |

Cancel the underlying legs:

```
+1 June 90 call
−1 June 95 call
−1 June 105 call
+1 June 110 call
```

— a **long condor**. Buy it for 3.75 or sell the iron condor for **1.25**; both want the market to finish inside the 95–105 range.

**C. Recognizing disguised straddles.** Take `+2 June 100 calls / −1 underlying contract`. Split the calls:

```
+1 June 100 call
+1 June 100 call  ┐
−1 underlying     ┘ = synthetic long June 100 put
```

The position is **+1 June 100 call / +1 June 100 put** — a **long straddle**.

Mirror case: `+2 June 100 puts / +1 underlying contract`. Split the puts; a long put plus a long underlying is a synthetic long call, so the position is again a **long straddle**.

**D. Decoding a long call Christmas tree.** The position `+1 June 95 call / −1 June 100 call / −1 June 105 call` has no obvious shape. Rewrite the June 95 and 100 calls as synthetic puts:

| Original | Synthetic |
|---|---|
| +1 June 95 call | +1 June 95 put / −1 underlying |
| −1 June 100 call | −1 June 100 put / +1 underlying |
| −1 June 105 call | −1 June 105 call |

Cancel the underlying legs:

```
+1 June 95 put
−1 June 100 put
−1 June 105 call
```

Read it two ways. Focusing on the June 100 put and June 105 call: a **short June 100/105 strangle plus a long put at a lower strike**. Focusing on the two puts: a **bull put spread plus a short call at a higher strike**. Either reading gives the same conclusion — **limited downside risk, unlimited upside risk**.

## Key Takeaways
- Six synthetics: long/short underlying, long/short call, long/short put — each built from the other two instruments at one strike and expiration.
- Hedge any option with the underlying and you have created its companion option synthetically.
- Companion calls and puts share gamma and vega exactly and split delta to about 100; theta differs whenever there is a cost of carry.
- Volatility risk lives in the exercise prices and expirations, not in whether the contracts are calls or puts.
- Call and put versions of the same spread (verticals, butterflies, condors) are the same strategy; they differ only in cash flow, and the two prices sum to the (present value of the) strike interval.
- Iron butterflies and iron condors are credit versions of long butterflies and condors; buying the traditional structure = selling the iron one.
- To identify an unfamiliar position, rewrite it entirely in calls or entirely in puts and cancel the underlying legs.

## Connects To
- [Chapter 4 — Expiration Profit and Loss](ch04-expiration-profit-and-loss.md): the parity graphs and slope arithmetic behind every synthetic.
- [Chapter 11 — Volatility Spreads](ch11-volatility-spreads.md): the butterflies, condors, straddles, and Christmas trees rebuilt here.
- [Chapter 12 — Bull and Bear Spreads](ch12-bull-and-bear-spreads.md): why buying the lower strike is bullish in calls and puts alike.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): pricing synthetics against their real equivalents — put-call parity, conversions, and reversals.
- [Chapter 7 — Risk Measurement I](ch07-risk-measurement-i.md): the evaluation tables where companion gammas and vegas can be checked.
