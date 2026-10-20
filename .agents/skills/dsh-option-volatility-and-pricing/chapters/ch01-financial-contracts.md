# Chapter 1 — Financial Contracts

## Core Idea
Every derivative is a contract whose value is *derived* from something else. Natenberg builds the whole taxonomy from one story: Jerry wants Farmer Smith's land for a restaurant that depends on a highway that may never be built. Paying $100,000 today is a **spot (cash) transaction**. Agreeing today on a price but exchanging money and goods in one year is a **forward contract**. Paying Farmer Smith a separate, non-refundable **premium** for the *right* — but not the obligation — to buy at the agreed forward price in one year is a **call option**. Paying a young couple for the right to *sell* his house to them in one year is a **put option**. Everything later in the book is pricing that separate payment.

## Frameworks Introduced

**The contract ladder (cash → forward → futures → option).**
| Contract | Terms agreed | Money/goods exchanged | Obligation |
|---|---|---|---|
| Cash / spot | Now | Now | Both parties, immediate |
| Forward | Now | At maturity/expiration | Both parties, deferred |
| Futures | Now (standardized, exchange-traded) | At maturity, exchange guarantees | Both parties, deferred |
| Option | Now, plus a premium | Premium now; underlying only if exercised | Buyer has the right; seller has the obligation |

**Insurance as the canonical option.** A homeowner's policy is a put: the right to "sell" the home back to the insurer for the insured amount. It has an expiration date (six-month/one-year policy), an exercise price (the insured amount, possibly net of a deductible, analogous to an agreed forward price), and a premium the insurer keeps regardless. Insurers price by assigning **probabilities** to outcomes and charging more than fair value. Option pricing uses the same logic — this is the seed of Chapter 5.

**The two settlement types.** This distinction drives real cash-flow surprises and recurs throughout the book (Chapters 2, 15, 16).

| | Stock-type settlement | Futures-type settlement |
|---|---|---|
| Payment at trade | Full and immediate | None; a **margin** deposit only |
| Profits before close | **Unrealized** (paper profit) | Realized daily as **variation** |
| Daily cash flow | None | Variation credit/debit each day |
| Interest | Buyer loses interest on full purchase price | Margin still belongs to trader and earns interest; variation credits earn / debits cost interest |
| Also called | — | Margin-and-variation settlement |

## Key Concepts
- **Derivative** — forward, future, option (and swap). Value derives from an underlying asset. Natenberg restricts the text to forwards, futures, and options; **swaps** (most commonly a plain-vanilla fixed-for-floating interest-rate swap) are excluded because they are non-standardized and mostly off-exchange.
- **Futures contract** — an exchange-traded forward with standardized quantity, quality, delivery date, delivery place, and payment method, plus an exchange guarantee against default.
- **Opening trade / closing trade / open interest** — the first trade (buy or sell) opens a position; the reversing trade closes it. **Open interest** is the number of contracts not yet closed out. Long and short open interest are necessarily equal.
- **Long = debit, short = credit.** For multi-leg trades, a net debit is a "long" position, a net credit a "short" position.
- **Long/short contract position vs. long/short market position.** A trader long a derivative may want the underlying to *fall* (e.g., a long put). Natenberg keeps these separate deliberately.
- **Notional (nominal) value.** Physical commodity: units × unit price. 1,000 units at $75 = **$75,000**. Cash-settled index: index price × point value. Index at **825.00** with a **$200** point value = **$165,000**. Exchanges set the point value so the contract is neither too risky nor too costly per unit of exposure.
- **Cash settlement** — futures with no physical delivery (stock indexes) settle with one final variation payment against the index price at maturity.
- **Clearinghouse** — on execution the buyer–seller link is broken and replaced by two links to the exchange, which becomes buyer to every seller and seller to every buyer. Clearing firms are members that collect margin and guarantee their traders. In the U.S.: the **Options Clearing Corporation** (equity options), **CME Clearing House** (CME Group), **DTCC** (stock and bonds). A U.S. clearinghouse has **never failed**.
- **Haircut** — the name for a margin requirement on a professional trader at an equity options exchange.

## Mental Models
- **Options are compensation for hostage-taking.** Farmer Smith and the young couple are held hostage to Jerry's decision for a year; the premium buys their patience. This is the intuitive root of time value.
- **Price is unrealized until you trade back.** A stock buyer at $50 seeing $60 has a $1,000 paper profit on 100 shares he cannot spend until he sells.
- **Variation is not a price change on top of the trade price.** A futures buyer at $75 who takes delivery at $90 pays $90 × 1,000 = $90,000 at maturity but was credited $15/unit in variation along the way — net $75, exactly the contract price.

## Anti-patterns
- **Ignoring mismatched settlement on U.S. futures options.** The underlying future is margin-and-variation settled; the option on it is stock-type settled. A perfectly hedged position can still generate an immediate variation cash drain on the futures leg while the offsetting option gain stays unrealized. Most non-North American exchanges avoid this by matching option settlement to the underlying's.
- **Funding only the margin requirement.** A trader must fund margin *and* the variation he may owe, or he will be forced out of a position earlier than intended.
- **Confusing margin with variation.** Margin is security that still belongs to the trader; variation is realized P&L moving between accounts.

## Reference Tables

**Figure 1-1 Stock-type settlement** (opening trade at day-1 settlement price, interest ignored)

| Day | Stock price | Trade | Cash flow | Position | Cumulative realized P&L | Unrealized P&L |
|---|---|---|---|---|---|---|
| 1 (open) | $53 | buy 1,200 | −$53 × 1,200 = −$63,600 | long 1,200 | 0 | 0 |
| 2 | $57 | sell 500 | +$57 × 500 = +$28,500 | long 700 | ($57−$53) × 500 = **+$2,000** | ($57−$53) × 700 = +$2,800 |
| 3 | $51 | none | 0 | long 700 | +$2,000 | ($51−$53) × 700 = −$1,400 |
| 4 (close) | $54 | sell 700 | +$54 × 700 = +$37,800 | 0 | $2,000 + ($54−$53) × 700 = **+$2,700** | 0 |

**Figure 1-2 Futures-type settlement** (contract size 1,000 units; margin $3,000 per contract)

| Day | Futures price | Trade | Position | Margin requirement | Variation | Cumulative realized P&L |
|---|---|---|---|---|---|---|
| 1 (open) | $75 | sell 9 | short 9 | 9 × $3,000 = $27,000 | 0 | 0 |
| 2 | $77 | none | short 9 | $27,000 | ($77−$75) × −9 × 1,000 = −$18,000 | −$18,000 |
| 3 | $74 | buy 2 | short 7 | 7 × $3,000 = $21,000 | ($74−$77) × −9 × 1,000 = +$27,000 | +$9,000 |
| 4 | $70 | buy 4 | short 3 | 3 × $3,000 = $9,000 | ($70−$74) × −7 × 1,000 = +$28,000 | +$37,000 |
| 5 (close) | $80 | buy 3 | 0 | 0 | ($80−$70) × −3 × 1,000 = −$30,000 | **+$7,000** |

**Figure 1-3 The clearing process** — (1) buyer and seller agree contract, quantity, price; (2a/2b) each reports to his clearing firm; (3a/3b) each clearing firm reports to the clearinghouse; (4) if reports match, the trade is official; (5) if they do not match, both clearing firms are informed and return the trade to buyer and seller for resolution.

## Worked Example
**Reconciling the futures short in Figure 1-2.** Nine contracts of 1,000 units are sold at $75 on day 1; margin of 9 × $3,000 = $27,000 is posted (it stays the trader's money).

- Day 2, price to $77: the short loses $2/unit. Variation = ($77 − $75) × (−9) × 1,000 = **−$18,000** debited that day.
- Day 3, buy 2 back at $74 (position short 7), margin falls to $21,000. Variation is computed on the *previous day's* position of 9: ($74 − $77) × (−9) × 1,000 = **+$27,000**. Cumulative = **+$9,000**.
- Day 4, buy 4 back at $70 (short 3), margin $9,000. Variation on the prior 7: ($70 − $74) × (−7) × 1,000 = **+$28,000**. Cumulative = **+$37,000**.
- Day 5, buy the last 3 at $80. Variation on 3: ($80 − $70) × (−3) × 1,000 = **−$30,000**. Cumulative = **+$7,000**, and all margin is returned.

Every dollar of that $7,000 moved through the account **daily**. Contrast Figure 1-1, where the stock trader's $2,700 exists only on paper until each closing sale.

## Key Takeaways
- A forward defers both payment and delivery; a futures contract is a standardized, exchange-guaranteed forward; an option adds a separately negotiated premium in exchange for the right to walk away.
- The premium exists to compensate the counterparty for being frozen out of alternatives — the economic content of an option's price.
- Options are priced by assigning probabilities to outcomes, exactly as insurers do.
- Stock-type vs. futures-type settlement changes *when* money moves and therefore the interest carried on a position — a theme that returns in forward pricing (Ch 2), risk measures (Ch 9), and arbitrage (Ch 15).
- In North America all exchange-traded options are stock-type settled; this text's examples assume that convention.

## Connects To
- [Chapter 2 — Forward Pricing](ch02-forward-pricing.md): why Farmer Smith wants more than $100,000 for a one-year forward.
- [Chapter 3 — Contract Specifications and Option Terminology](ch03-contract-specifications-and-terminology.md): the exchange-standardized version of Jerry's option.
- [Chapter 5 — Theoretical Pricing Models](ch05-theoretical-pricing-models.md): the insurance/probability logic made formal.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): where settlement differences become tradable risk.
