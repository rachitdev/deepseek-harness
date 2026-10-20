# Chapter 3 — Contract Specifications and Option Terminology

## Core Idea
"In option trading, all rights lie with the buyer and all obligations with the seller." A futures contract obliges both parties; an option gives the buyer a *choice*. Everything else in the chapter is the vocabulary and contract plumbing needed to state precisely what that choice is: which underlying, at what price, until when, exercisable how, and settling into what.

## Frameworks Introduced

**The five contract specifications.** Any exchange-traded option is fully identified by:
1. **Underlying** — what gets bought or sold
2. **Expiration date** — when the decision must be made
3. **Exercise (strike) price** — the price at which delivery occurs
4. **Type** — call (right to buy / go long) or put (right to sell / go short)
5. **Exercise style** — European (expiration only) or American (any business day)

Natenberg's Figure 3-1 naming convention reads left to right: `Crude Oil | October | 90 | Call` = underlying, expiration, exercise price, type. Likewise `GE | March | 30 | Put`.

**Premium decomposition.** `Premium = intrinsic value + time value`, always, exactly.
- Call intrinsic value = max(0, S − X)
- Put intrinsic value = max(0, X − S)
- Time value (a.k.a. time premium, extrinsic value) = premium − intrinsic value

Intrinsic value is **independent of the expiration date**: with the underlying at $83, a March 70 call and a September 70 call both have intrinsic value $13.

**Exercise/assignment direction table** (Natenberg's own summary):

| Action | Consequence |
|---|---|
| Exercise a call | You **choose to buy** at the exercise price |
| Are assigned on a call | You are **required to sell** at the exercise price |
| Exercise a put | You **choose to sell** at the exercise price |
| Are assigned on a put | You are **required to buy** at the exercise price |

**Three settlement destinations on exercise:** the physical underlying, a futures position, or cash.

## Key Concepts
- **Underlying size.** Stock options: typically **100 shares** (a *round lot*; fewer is an *odd lot*), adjustable by the exchange for very high or very low priced stock. Futures options: uniformly **one futures contract**.
- **Serial options** — futures option expirations with no corresponding futures month; the underlying is the **nearest futures contract beyond option expiration**. On a March/June/September/December cycle: January and February options → March future; April and May → June; July and August → September; October and November → December.
- **Midcurve options** — short-term options on long-term futures (Eurodollars at CME; Short Sterling and Euribor at LIFFE). One-year, two-year, and five-year midcurve = short-term option on a future with at least that much time to maturity.
- **Flex options** — buyer and seller negotiate quantity, expiration, exercise price, and exercise style.
- **Expiration vs. last trading day.** U.S. stock and stock index options expire the **third Friday** of the month; for most stock options the last trading day is the same day, but when Good Friday falls on the third Friday of April the last trading day is the preceding Thursday.
- **AM vs. PM expiration.** Large expiration-day stock orders from hedged derivative positions distorted prices, so index contracts moved to **AM expiration** — the expiration value is set from the underlying's **opening** price. Individual stock options remain **PM expiration** (closing price on the last trading day).
- **Futures-option expiration timing.** Physical-commodity futures options often expire days or weeks before futures maturity, commonly in the **prior month**: a March futures option expires in February, a July in June, a November in October. Check the exchange calendar.
- **Exercise prices** are set by the exchange at equal intervals bracketing the underlying. Underlying at 62 → strikes 50, 55, 60, 65, 70, 75; if the underlying rises to 70 the exchange may add 80, 85, 90, plus intermediate strikes 52½, 57½, 62½, 67½.
- **Assignment is random.** The assignee must be someone short the option who has not offset; beyond that no trader has a greater or lesser chance.
- **Parity** — an option trading at exactly intrinsic value (zero time value). A **European** option can have *negative* time value and trade **below parity** (see Ch 16).
- **Moneyness.** In the money = positive intrinsic value. Out of the money = no intrinsic value, price is all time value. At the money = exercise price equals the underlying price; technically out of the money, but distinguished because ATM options have specific desirable characteristics and are the most actively traded. In practice, ATM means the strike **closest** to the underlying: stock at $74 with $5 strike intervals → the **75 call and 75 put** are the at-the-money options.
- **Reciprocity of moneyness.** If a call is in the money, the put at the same strike must be out of the money, and vice versa. Stock at $44: the 40 call is ITM by $4 and the 40 put OTM by $4; the 55 put is ITM by $11 and the 55 call OTM by $11.
- **Automatic exercise.** Exchanges auto-exercise ITM options at expiration to prevent value being lost through a missed exercise notice. Thresholds differ by exchange and by trader class — e.g., **0.05 for retail, 0.02 for professionals**. Below the threshold, a trader may still exercise manually; above it, a trader who does not want exercise must submit a **do not exercise** notice.
- **Option margining.** Limited-risk positions never require margin above the maximum risk (a buyer's margin never exceeds the premium paid). Unlimited-risk positions require **risk-based** margining across an array of underlying prices and speeds of price change: the **OCC**'s system for stock and index options, **SPAN** (Standard Portfolio Analysis of Risk, developed by the CME) on futures exchanges.
- **European vs. American has nothing to do with geography.** Options on futures and on individual stocks tend to be American; index options tend to be European.

## Mental Models
- **Rights vs. obligations asymmetry** is the whole product. Price the asymmetry and you price the option.
- **Cash flow on exercise depends only on the exercise price.** Exercise a 110 call on stock and you pay $11,000 whether the stock is $10 or $1,000. P&L depends on the stock price and the premium paid; the *cash flow at exercise* does not.

## Anti-patterns
- **Assuming "American" and "European" describe where the option trades.**
- **Letting an ITM option expire unexercised** — the historical failure that produced automatic exercise policies.
- **Treating at-the-money as just another out-of-the-money option** — the ATM has special properties throughout the book.
- **Forgetting the exercise-notice threshold** in either direction: a 0.03-ITM option will not be auto-exercised for a retail account; a 0.06-ITM option *will* be unless you file "do not exercise".
- **Assuming a futures option expires with its future.** It usually expires in the preceding month.

## Reference Tables

**Figure 3-2 — Intrinsic value and time value**

| Position | Premium | Intrinsic value | Time value |
|---|---|---|---|
| April 400 call, underlying $435 | $50 | $435 − $400 = **$35** | $50 − $35 = **$15** |
| October 70 put, underlying $62 | $11 | $70 − $62 = **$8** | $11 − $8 = **$3** |

**Figure 3-3 — Moneyness map**

| Relative to current underlying price | Calls | Puts |
|---|---|---|
| Lower exercise prices | In the money | Out of the money |
| Exercise price ≈ underlying | At the money | At the money |
| Higher exercise prices | Out of the money | In the money |

**Settlement into the physical underlying** (stock options always settle this way)

| Event | Cash | Shares |
|---|---|---|
| Exercise one January 110 call | pay 100 × $110 = **$11,000** | receive 100 shares |
| Assigned on six April 40 calls | receive 600 × $40 = **$24,000** | deliver 600 shares |
| Exercise two July 60 puts | receive 200 × $60 = **$12,000** | deliver 200 shares |
| Assigned on three October 95 puts | pay 300 × $95 = **$28,500** | receive 300 shares |

**Settlement into a futures position** (future at 85.00, point value $1,000, margin $3,000/contract)

| Event | Resulting position | Margin | Variation |
|---|---|---|---|
| Exercise one February 80 call | long 1 future at 80 | $3,000 | (85 − 80) × $1,000 = **+$5,000** |
| Assigned on six May 75 calls | short 6 futures at 75 | 6 × $3,000 = $18,000 | (75 − 85) × $1,000 × 6 = **−$60,000** |
| Exercise four August 100 puts | short 4 futures at 100 | 4 × $3,000 = $12,000 | (100 − 85) × $1,000 × 4 = **+$60,000** |
| Assigned on two November 95 puts | long 2 futures at 95 | 2 × $3,000 = $6,000 | (85 − 95) × $1,000 × 2 = **−$20,000** |

**Settlement into cash** (index fixed at 300, $500 per index point; no underlying position results)

| Event | Cash flow |
|---|---|
| Exercise three March 250 calls | (300 − 250) × $500 × 3 = **+$75,000** |
| Assigned on seven June 275 calls | (275 − 300) × $500 × 7 = **−$87,500** |
| Exercise two September 320 puts | (320 − 300) × $500 × 2 = **+$20,000** |
| Assigned on four December 340 puts | (300 − 340) × $500 × 4 = **−$80,000** |

## Worked Example
**Reading two real exchange contracts.**

A **crude oil October 90 call** on the New York Mercantile Exchange gives the buyer the right to take a **long position in one October crude oil futures contract for 1,000 barrels** (the underlying) at **$90 per barrel** (the exercise price) **on or before October expiration** (the expiration date). Because the underlying is a future, exercise settles into a futures position: the exerciser becomes long one October future at **90**, posts the exchange's margin, and immediately receives or pays variation equal to (futures price − 90) × 1,000.

A **General Electric March 30 put** on the Chicago Board Options Exchange gives the buyer the right to take a **short position in 100 shares of GE** (the underlying) at **$30 per share** (the exercise price) **on or before March expiration**. Being a stock option it is American-style and settles into the physical underlying: on exercise the holder receives 100 × $30 = **$3,000** and must deliver 100 shares.

**Decomposing the premium.** With the underlying at $435 and the April 400 call trading at $50: intrinsic value is $435 − $400 = $35, so the market is paying **$15 of time value** for the protective characteristic of the option relative to owning the underlying outright. With the underlying at $62 and the October 70 put at $11: intrinsic $70 − $62 = $8, time value **$3**. If either option's price fell to exactly its intrinsic value it would be trading **at parity**.

## Key Takeaways
- All rights with the buyer, all obligations with the seller — this asymmetry is what the premium buys.
- Premium = intrinsic + time value, always; intrinsic value is expiration-independent, time value is not.
- Exercise settles into the physical underlying (stock options), a futures position (with immediate margin and variation), or cash (index options).
- ATM options are technically out of the money but are singled out because of their special characteristics and heavy volume.
- Exercise style, serial/midcurve listings, AM/PM expiration, and auto-exercise thresholds are contract details that quietly change P&L — check them per exchange.

## Connects To
- [Chapter 1 — Financial Contracts](ch01-financial-contracts.md): stock-type vs futures-type settlement, referenced throughout the exercise examples.
- [Chapter 4 — Expiration Profit and Loss](ch04-expiration-profit-and-loss.md): intrinsic value becomes the parity graph.
- [Chapter 16 — Early Exercise of American Options](ch16-early-exercise-american-options.md): exercise style made economic; negative time value and sub-parity pricing.
- [Chapter 22 — Stock Index Futures and Options](ch22-stock-index-futures-and-options.md): cash settlement and AM expiration in practice.
