# Chapter 2 — Forward Pricing

## Core Idea
A forward price is not a forecast. It is the cash price adjusted for the costs and benefits of owning the asset now rather than later — costs and benefits that a forward contract *defers* rather than eliminates.

```
forward price = current cash price + costs of buying now − benefits of buying now
```

Every option pricing model in the book prices off the **forward price**, not the spot price, so this arithmetic is load-bearing for everything downstream.

## Frameworks Introduced

**Costs and benefits by instrument class** (Natenberg's master table):

| Instrument | Costs of buying now | Benefits of buying now |
|---|---|---|
| Physical commodity | Interest on cash price; storage costs; insurance costs | Convenience yield |
| Stock | Interest on stock price | Dividends (if any) + interest on dividends |
| Bonds and notes | Interest on bond/note price | Coupon payments + interest on coupons |
| Foreign currency | Interest cost of borrowing the domestic currency | Interest earned on the foreign currency |

**The four forward-price formulas.**
- Physical commodity (C = commodity price, s = annual storage, i = annual insurance):
  `F = C × (1 + r × t) + (s × t) + (i × t)`
- Stock, exact: `F = [S × (1 + r × t)] − Σ[dₙ × (1 + rₙ × tₙ)]`
- Stock, simplified (aggregate dividends D, ignore interest on them): `F = [S × (1 + r × t)] − D`
- Bonds/notes: `F = [B × (1 + r × t)] − Σ[cₙ × (1 + rₙ × tₙ)]`
- Foreign currency: `F = S × (1 + r_d × t) / (1 + r_f × t)` where `S = C_d / C_f`

**Implied values — invert the formula for whatever you don't know.** From `F = [S × (1 + r × t)] − D`:
- Implied spot: `S = (F + D) / (1 + r × t)`
- Implied interest rate: `r = ([(F + D)/S] − 1) / t`
- Implied dividend: `D = [S × (1 + r × t)] − F`

If a trader believes the contract is fairly priced, the implied value **is the marketplace's consensus estimate** of the missing input. This is the template for implied volatility later.

**Long rate vs. short rate.** `r_l − r_s = r_bc` (borrowing costs). Ordinary borrowing/lending uses the long rate; short stock earns only the **short-stock rebate** r_s. An option is *not* a deliverable security — it is created between buyer and seller and need not be borrowed to be sold — so **always apply the ordinary long rate to option cash flows**.

## Key Concepts
- **Basis** — cash price minus forward price. Usually negative (costs of carry exceed benefits). Turns positive when benefits (e.g., oil revenue, high coupons) exceed carrying costs.
- **Contango / normal market** — long-term futures at a premium to short-term. **Backwardation / backward market** — cash price above the futures price.
- **Convenience yield** — the premium a user pays for immediate physical access. A factory that must not shut down will overpay for spot supply; this is what produces backwardation despite always-positive interest and storage costs. Hard to measure directly, but **inferable** by comparing observed cash price with the computed theoretical cash price.
- **The forward price of a futures contract is the futures price.** A three-month future at $75.00 has a three-month forward price of $75.00 — which makes futures options easier to evaluate than stock options.
- **Arbitrage** — "buying and selling of the same or very closely related instruments in different markets to profit from an apparent mispricing." Natenberg refuses "riskless profit" as a definition.
- **Cash-and-carry arbitrage** — buy cash, sell futures, carry to maturity.
- **Carry trade** — borrow low-rate domestic currency, buy high-rate foreign currency. Often *called* arbitrage but carries rate risk *and* exchange-rate risk; Natenberg says the term is "probably misapplied."
- **Forward rate / FRA** — a rate applying to a period beginning in the future. A **3 × 9 forward rate** is a six-month rate beginning in three months; a **1 × 5** is a four-month rate beginning in one month; a **4 × 12** is an eight-month rate beginning in four months.
- **Dividend timeline** — *Declared date* (amount and payment date announced; dividend risk eliminated), *Record date* (must own to receive; ownership official only at settlement, T+3 in the U.S.), *Ex-dividend date* (two business days before the record date; last purchase date to receive the dividend is three business days before), *Payable date*.
- **Ex-div quoting** — stock closes at $67.50, dividend $0.40, opens next day (ex-date) at $68.25. Unchanged would have been $67.10, so the quote reads **68.25 +1.15 ex-div 0.40**.
- **Short-stock squeeze** — inability to borrow shares. **Locked / limit** market — a futures contract at its daily price limit; no trading until price comes off the limit.
- **OTC market** — trading not on an organized exchange. Almost all exchange-traded options on commodities, bonds, and currencies are actually **futures options**.

## Mental Models
- **Deferral, not elimination.** Everything you avoid paying today, you pay in the forward price.
- **Implied = consensus.** Whatever input you back out of a market price is the market's opinion of that input.
- **Arbitrage has a no-trade band, not a point.** Different rates on the long and short side create a range of forward prices within which no arbitrage is possible.

## Anti-patterns
- **Assuming borrowing and lending rates are equal.** Natenberg flags this simplification in a footnote; the short-sale section shows exactly what it costs you.
- **Calling the carry trade arbitrage.** Rates float and exchange rates move; repaying the borrowed domestic currency can cost far more than expected.
- **Trusting an estimated dividend.** An unannounced dividend is a guess based on past payments; a cut destroys the arbitrage profit.
- **Sloppiness about the dividend date.** Interest on a dividend is small, but if the payment date falls near the derivative's maturity, a small date error significantly changes value.
- **Ignoring transport/insurance in a location arb.** London $700 vs New York $710 is not $10 of profit if shipping plus insurance exceeds $10.

## Reference Tables

**Jerry's land — a full forward price build-up** (cash price $100,000, one year, r = 8.00%)

| Item | Calculation | Amount |
|---|---|---|
| Interest on cash price | 8% × $100,000 | +$8,000 |
| Property taxes due in 9 months, financed 3 months | $2,000 + ($2,000 × 8% × 3/12) = $2,000 + $40 | +$2,040 |
| **Total costs of buying now** | | **+$10,040** |
| Oil revenue | 12 × $500 | −$6,000 |
| Interest on oil revenue | ($500 × 8% × 11/12) + … + ($500 × 8% × 1/12) | −$220 |
| **Total benefits of buying now** | | **−$6,220** |
| **Fair one-year forward price** | $100,000 + $10,040 − $6,220 | **$103,820** |
| **Basis** | $100,000 − $103,820 | **−$3,820** |

**Inferring the convenience yield** (three-month forward)

| Input | Value |
|---|---|
| Three-month forward F | $77.40 |
| Interest rate r | 8% |
| Annual storage s | $3.00 |
| Annual insurance i | $0.60 |
| Theoretical cash C = [F − (s + i)t] / (1 + rt) | (77.40 − 3.60 × 0.25) / 1.02 = 76.50 / 1.02 = **$75.00** |
| Observed cash price | $76.25 |
| **Convenience yield** | **$1.25** |

**Worked forward prices from the chapter**

| Asset | Inputs | Forward price |
|---|---|---|
| Stock (exact) | S = 67.00, t = 8 mo, r = 6.00%, d = 0.33 semiannual, next in 1 mo (t₁ = 7 mo, r₁ = 6.20%; t₂ = 1 mo, r₂ = 6.50%) | 69.68 − 0.3419 − 0.3318 = **69.0063** |
| Stock (simplified) | Same, D = 2 × 0.33 = 0.66 | 67.00 × (1 + 0.06 × 8/12) − 0.66 = **69.02** |
| Bond | B = 109.76, t = 10 mo, r = 8.00%, c = 5.25 semiannual, next in 2 mo (t₁ = 8 mo, r₁ = 8.20%; t₂ = 2 mo, r₂ = 8.50%) | 117.0773 − 5.5370 − 5.3244 = **106.2159** |
| FX | €1.00 = $1.50, r$ = 6.00%, r€ = 4.00%, six months | 1.50 × (1 + 0.03)/(1 + 0.02) = **1.5147** |

## Worked Example
**Cash-and-carry arbitrage on the eight-month stock forward, and how short-stock rates kill the reverse trade.**

Theoretical forward = 67.00 × (1 + 0.06 × 8/12) − 0.66 = **69.02**. The market shows the eight-month forward at **69.50**.

Sell the forward at 69.50, buy the stock at 67.00. Cash flows to maturity:

| Cash flow | Amount |
|---|---|
| Interest on borrowing $67.00 for 8 months at 6% (67.00 × 0.06 × 8/12) | −2.68 |
| Cost of buying the stock | −67.00 |
| Dividend received | +0.66 |
| Forward price received at maturity | +69.50 |
| **Net** | **+0.48** |

Both the entry price (67.00) and the exit price (69.50) are fixed, so **stock price fluctuations do not affect the result**. What *can* hurt: rising interest rates (higher carry) and an unannounced dividend cut.

Now question the inputs. Backing out from the observed 69.50:
- Implied interest rate: r = [(69.50 + 0.66)/67.00 − 1] / (8/12) = **7.07%** — the market may simply be using a higher rate than 6%.
- Implied dividend: D = [67.00 × (1 + 0.06 × 8/12)] − 69.50 = **0.18** — i.e., two payments of about **$0.09** each, not $0.33.

**Now the reverse.** Suppose the forward trades at **68.75** instead. Against a theoretical 69.02, buying the forward and selling the stock looks like +0.27 — and it *is*, for a trader who already owns the stock. A trader who must sell short does not earn the 6% long rate. If the lending firm keeps 2% in borrowing costs, he earns the 4% short rate:

`67.00 × (1 + 0.04 × 8/12) − 0.66 = 68.13`, and 68.13 − 68.75 = **−0.62**. The arb loses money.

**Conclusion: a trader without stock can only profit if the forward price is below $68.13 or above $69.02.** Between those two prices there is no arbitrage — the no-trade band created by the borrowing cost r_bc.

## Key Takeaways
- Forward price = cash + carry costs − carry benefits; the basis is normally negative.
- Options are priced off the forward, and for futures options the forward price *is* the futures price.
- Invert the forward-price identity to extract implied spot, implied rate, or implied dividend — the market's consensus on whichever input you distrust.
- Convenience yield explains backwardation and is inferred, not observed.
- The long/short rate split (r_l − r_s = r_bc) creates a band of forward prices inside which no arbitrage exists. Options always use the long rate.
- Short selling requires borrowing the stock, pays only the short-stock rebate, and obliges the short to pay away any dividends.

## Connects To
- [Chapter 1 — Financial Contracts](ch01-financial-contracts.md): the forward contract this chapter prices.
- [Chapter 5 — Theoretical Pricing Models](ch05-theoretical-pricing-models.md): the forward price becomes the mean of the distribution.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): synthetic and conversion arbitrage built on these same carry calculations.
- [Chapter 16 — Early Exercise of American Options](ch16-early-exercise-american-options.md): dividends and interest drive early exercise.
- [Chapter 20 — Volatility Revisited](ch20-volatility-revisited.md): implied volatility as the same "solve for the missing input" move.
