---
name: dsh-option-volatility-and-pricing
description: "Sheldon Natenberg's \"Option Volatility and Pricing: Advanced Trading Strategies and Techniques\" (McGraw-Hill, 2nd edition, 2015) — the professional trader's reference on option theory and practice. Use when working on option pricing, Black-Scholes or binomial models, implied and realized volatility, the Greeks (delta, gamma, theta, vega, rho and higher-order measures), delta-neutral and dynamic hedging, volatility spreads (straddles, strangles, butterflies, condors, ratio spreads, calendars), bull and bear vertical spreads, synthetics and put-call parity, conversions/reversals/boxes/rolls, early exercise of American options, hedging with protective and covered options, volatility skews and surfaces, stock index futures and options, position and risk analysis, or volatility contracts such as variance swaps and the VIX."
---

# Option Volatility and Pricing

**Author**: Sheldon Natenberg
**Publisher**: McGraw-Hill Education, 2nd edition (2015)
**Pages**: 588
**Chapters**: 25 (plus a glossary and math appendix)
**Generated**: 2026-08-23

## How to Use This Skill

- **No arguments** — read this file for the book's core frameworks, then follow the Chapter Index to whatever you need.
- **With a topic** — check the Topic Index below, open the named chapter file(s), and consult `cheatsheet.md` for the relevant formula or decision table.
- **With a chapter** — open `chapters/chNN-slug.md` directly; each contains the chapter's frameworks, its actual worked calculations with the book's own numbers, reference tables reproduced from the figures, anti-patterns, and cross-links.
- **Browse** — `patterns.md` for the reusable moves as When/How/Trade-offs, `glossary.md` for terminology, `cheatsheet.md` for every formula, threshold, and selection matrix in one place.

## Core Frameworks & Mental Models

**Direction *and* speed.** A stock trader needs only direction. An option trader needs direction *and* the speed at which the market moves. "If predicting the correct market direction is difficult, correctly predicting direction and speed is probably beyond most traders' capabilities." Volatility is how speed enters a pricing model — which is why, if direction is your only view, Natenberg says trade the underlying instead.

**Theoretical value = the present value of the expected value.** Propose expiration prices, assign probabilities constrained so the expected underlying value equals the **forward price** (arbitrage-free), compute the option's expected value, and discount. The forward price — `cash price + costs of buying now − benefits of buying now` — is the mean of every distribution in the book.

**Volatility is a standard deviation.** Feed a mean and a standard deviation into a model and the distribution is fully specified. Scale by **√t**: annual ÷ 16 for a day, ÷ 7.2 for a week. Check reality against 2/3, 19/20, 369/370 — a two-standard-deviation daily move should appear about once a month. "If we have the wrong volatility, we have the wrong probabilities. And if we have the wrong probabilities, we are generating incorrect theoretical values, thereby defeating the purpose of using a theoretical pricing model in the first place."

**The Greeks as a wish list, and the gamma/theta trade.** Delta = direction, gamma = magnitude and speed, theta = time, vega = *implied* volatility, rho = rates. Gamma and theta always carry opposite signs, and their magnitudes correlate: "An option trader cannot have it both ways. Either market movement will help the position (positive gamma) or the passage of time (positive theta) will — but not both." Gamma is your view on **realized** volatility; vega is your view on **implied** volatility. They usually agree — calendar spreads are the standing exception.

**Gamma, theta, and vega all peak at the money.** Which is why at-the-money options dominate volume, why they are the most mispriced when your volatility estimate differs from the market's, and why "at-the-money options close to expiration in a low-volatility market are among the riskiest of options."

**Dynamic hedging turns one bet into many.** Re-establishing delta neutrality forces you to sell as the market rises and buy as it falls, capturing the mismatch between the option's curving value and the underlying's straight line. Crucially, **adjustments do not change the expected return** — they reduce variance. "The present value of this sum is equal to the option's theoretical value."

**Spread, because you must survive the short run.** A casino's 5% edge is identical whether one player bets $2,000 on one number or thirty-eight players bet $1,000 on all thirty-eight — but only the second is a sure profit. There is **no risk-reward tradeoff in spreading**; there is only variance reduction. Betting limits exist because "if the bet is large enough and the bettor happens to win, the short-term bad luck can overwhelm the casino."

**Judge strategies by risk at equal edge, never by edge.** Edge scales with size, so scale competing spreads to equal edge and then compare their P&L curves against price, volatility, and time. "A good spread is not necessarily the one that shows the greatest profit when things go well; it may be the one that shows the least loss when things go badly."

**Size to the margin for error.** Compute the position's implied volatility — `evaluation volatility + edge/vega` — and let the resulting margin, not the available liquidity, set the size. A 2-point margin means a small trade; a 7-point margin against a historically rare volatility justifies five times the size.

**Decision rules a practitioner uses daily:**
- Implied volatility **low** → positive-vega spreads, and **buy** the at-the-money option in a vertical. Implied volatility **high** → negative-vega spreads, and **sell** the at-the-money option.
- Long calendars want implied volatility low but expected to rise; short calendars the reverse.
- **Buy the lower strike and sell the higher to be bullish** — calls or puts alike. Only vertical spreads keep their directional sign under all conditions; ratio spreads, butterflies, and calendars all invert.
- Adjust with the **underlying** when you want to change only delta; it has zero gamma, theta, and vega.
- Hedgers: in a **high**-volatility market buy as few options and sell as many as possible; in a **low**-volatility market, the reverse.
- A stock **call** is only ever exercised early the day before the ex-dividend date; a stock **put** only outside the blackout period of `D/(X·r/365)` days.
- **Whenever possible a trader should avoid a short stock position.**

**Everything is a synthetic of everything else.** Six synthetics, one rule: hedge any option with the underlying and you hold the **companion option**. Put-call parity, `C − P = (F − X)/(1 + r·t)`, prices all of it and generates conversions, reversals, boxes, rolls, and time boxes. Companion calls and puts share gamma and vega exactly — so "the volatility risk of a position depends not on whether the contracts are calls or puts, but on the exercise prices and expiration dates."

**The model is a candle in a dark room.** Better than groping in the dark, but it flickers and it distorts, and the danger arrives with confidence and size. Every assumption fails: markets are not frictionless, volatility is not constant, trading is not continuous, prices are not lognormal, and volatility is not independent of price. Real distributions have **fat tails** — the S&P 500's biggest move in a 2,535-day sample was an 8.84 standard deviation event. Consequence: models **undervalue** options, which is one reason implied volatility persistently exceeds realized. The professional response is not a more exotic model but a **skew** treated as an input — the market's own correction — plus judgment near expiration. "A trader who slavishly uses a model to make every trading decision is heading for disaster."

## Chapter Index

| # | Chapter | Focus |
|---|---|---|
| 1 | [Financial Contracts](chapters/ch01-financial-contracts.md) | Cash, forward, futures, option; stock-type vs futures-type settlement; clearing |
| 2 | [Forward Pricing](chapters/ch02-forward-pricing.md) | Carry costs and benefits; basis; implied values; short-stock rate; arbitrage bands |
| 3 | [Contract Specifications and Option Terminology](chapters/ch03-contract-specifications-and-terminology.md) | The five specifications; intrinsic vs time value; moneyness; exercise and assignment |
| 4 | [Expiration Profit and Loss](chapters/ch04-expiration-profit-and-loss.md) | Parity graphs; the slope method for any multi-leg position; breakevens |
| 5 | [Theoretical Pricing Models](chapters/ch05-theoretical-pricing-models.md) | Expected value, theoretical value, edge; the five inputs; the riskless hedge |
| 6 | [Volatility](chapters/ch06-volatility.md) | Volatility as standard deviation; √t scaling; lognormal distributions; implied volatility |
| 7 | [Risk Measurement I](chapters/ch07-risk-measurement-i.md) | Delta, gamma, theta, vega, rho; four readings of delta; position implied volatility |
| 8 | [Dynamic Hedging](chapters/ch08-dynamic-hedging.md) | Capturing a mispricing; the five P&L components; adjustment disciplines |
| 9 | [Risk Measurement II](chapters/ch09-risk-measurement-ii.md) | How the Greeks change; vanna, charm, speed, color, volga, vega decay, zomma; lambda |
| 10 | [Introduction to Spreading](chapters/ch10-introduction-to-spreading.md) | Why spread; spread taxonomy; the casino argument; execution |
| 11 | [Volatility Spreads](chapters/ch11-volatility-spreads.md) | Straddles, strangles, butterflies, condors, ratio spreads, Christmas trees, calendars, diagonals |
| 12 | [Bull and Bear Spreads](chapters/ch12-bull-and-bear-spreads.md) | Delta inversion; vertical spreads; the at-the-money strike-selection rule |
| 13 | [Risk Considerations](chapters/ch13-risk-considerations.md) | Comparing spreads at equal edge; volga; efficiency; adjustments; liquidity |
| 14 | [Synthetics](chapters/ch14-synthetics.md) | The six synthetics; iron butterflies and condors; decoding unfamiliar positions |
| 15 | [Option Arbitrage](chapters/ch15-option-arbitrage.md) | Put-call parity; conversions, reversals, three-ways, boxes, rolls, time boxes; four risks |
| 16 | [Early Exercise of American Options](chapters/ch16-early-exercise-american-options.md) | Arbitrage boundaries; the exercise criteria; blackout period; dividend and interest plays |
| 17 | [Hedging with Options](chapters/ch17-hedging-with-options.md) | Protective options, covered writes, collars; volatility of returns; portfolio insurance |
| 18 | [The Black-Scholes Model](chapters/ch18-black-scholes-model.md) | The differential equation; N(d₁) and N(d₂); the 40% rule; closed-form Greeks |
| 19 | [Binomial Option Pricing](chapters/ch19-binomial-option-pricing.md) | Risk-neutral probabilities; working backwards; American options; gamma rent; dividends |
| 20 | [Volatility Revisited](chapters/ch20-volatility-revisited.md) | Historical volatility estimators; mean reversion; forecasting; term structure; forward volatility |
| 21 | [Position Analysis](chapters/ch21-position-analysis.md) | Reading a complex book; net contract positions; market making; stock splits |
| 22 | [Stock Index Futures and Options](chapters/ch22-stock-index-futures-and-options.md) | Index construction and divisors; index arbitrage; the futures market's downward bias |
| 23 | [Models and the Real World](chapters/ch23-models-and-the-real-world.md) | The six assumptions and how each fails; gaps; fat tails; expiration straddles |
| 24 | [Volatility Skews](chapters/ch24-volatility-skews.md) | Investment/demand/balanced skews; modeling and normalizing; risk reversals and dragonflies |
| 25 | [Volatility Contracts](chapters/ch25-volatility-contracts.md) | Variance swaps; VIX construction, futures, and options; replication; applications |

## Topic Index

| Topic | Chapters |
|---|---|
| Forward pricing, carry, basis | 2, 10, 22 |
| Settlement procedures (stock-type vs futures-type) | 1, 3, 7, 15, 22 |
| Contract specifications, exercise, assignment | 3, 16 |
| Expiration payoffs and breakevens | 4, 11, 12 |
| Theoretical value, expected value, edge | 5, 10, 13 |
| Volatility — definition, scaling, measurement | 6, 20 |
| Historical volatility estimators (Parkinson, Garman-Klass) | 6, 20 |
| Implied volatility | 6, 7, 20, 24 |
| Lognormal distributions, skewness, kurtosis | 6, 18, 23, 24 |
| Delta, gamma, theta, vega, rho | 7, 9, 18 |
| Higher-order Greeks (vanna, charm, speed, color, volga, zomma) | 9, 18, 21 |
| Dynamic hedging and adjustments | 8, 11, 13, 17, 19 |
| Volatility spreads | 11, 13 |
| Directional spreads | 12 |
| Risk comparison and position sizing | 10, 13, 21 |
| Synthetics and put-call parity | 14, 15 |
| Arbitrage — conversions, reversals, boxes, rolls | 2, 15, 22 |
| American options and early exercise | 16, 19 |
| Hedging and portfolio insurance | 17, 22, 25 |
| Black-Scholes model | 5, 18, 23 |
| Binomial model | 16, 19 |
| Volatility term structure and forward volatility | 20, 21, 24 |
| Position and book analysis; market making | 13, 21 |
| Stock indexes, index arbitrage, program trading | 22 |
| Model assumptions and their failures | 23, 24 |
| Volatility skews and surfaces | 24 |
| Variance swaps, VIX, volatility contracts | 25 |
| Interest-rate products and yield volatility | 2, 6, 23 |
| Dividends | 2, 5, 11, 13, 16, 19, 21, 22 |

## Supporting Files

- **[glossary.md](glossary.md)** — ~180 terms as Natenberg defines and uses them, with chapter references.
- **[patterns.md](patterns.md)** — the book's reusable moves as When / How / Trade-offs, grouped into nine parts from pricing through volatility contracts.
- **[cheatsheet.md](cheatsheet.md)** — every formula, the Greek sign and maximization tables, spread selection matrices, arbitrage boundaries, early exercise rules, hedging summary, tells-and-smells diagnostics, thresholds, and process templates.

## Scope & Limits

**Edition and vintage.** This is the **2nd edition (2015)**, the first revision in twenty years. Market data throughout is drawn from roughly **2001–2012**: S&P 500, gold, Bund, crude oil, euro, EuroStoxx 50, FTSE 100, wheat, and natural gas. Interest rates in several examples reflect the unusually low post-crisis environment (the gold tables use 0.50%), while other examples use 6–8% rates for illustrative clarity. Institutional details — exchange rules, auto-exercise thresholds, LIBOR as a benchmark, specific index compositions and clearinghouse names — are as of 2015 and have changed since; **LIBOR in particular has been phased out** in favor of alternative reference rates.

**What is covered thoroughly.** Option pricing theory from first principles; volatility in all its interpretations; the complete Greek apparatus including third-order measures; the full catalog of spreads with their risk signatures; synthetics and the arbitrage relationships; American early exercise; hedging; Black-Scholes and binomial models; volatility skews and surfaces; and volatility contracts.

**What is deliberately excluded.** **Swaps** — "because swaps are not standardized and therefore most often traded off exchanges, in this text we will restrict our discussion to the most common derivatives—forwards, futures, and options." **Exotic options** beyond glossary definitions (barrier, Asian, binary, compound, chooser, Bermuda, forward start). **Specialized interest-rate models**, which the book notes are necessary for bonds but does not develop. Formal derivations of Black-Scholes, jump-diffusion, CEV, and GARCH are described in intuition and cited to their source papers rather than derived. **Taxes** are set aside: "we will simply assume that each trader wishes to maximize his pretax profits."

**Conventions the book assumes.** North American settlement (all exchange-traded options stock-type settled), U.S. market structure, and options quoted in the local currency. Non-U.S. futures options are usually futures-type settled, which changes rho, theta relationships, and early exercise value — the book flags this repeatedly but works its examples on the U.S. convention.

**On the extraction.** The PDF had a clean text layer and all 25 chapters extracted completely. Figures are reproduced here as tables and described formulas; the original **graphs and charts are not reproducible in text**, so where a chapter's argument rests on a graph's shape (the volatility cone, skew curves, position P&L surfaces, volatility surfaces, the VIX correlation scatterplots), this skill describes the shape and reports the numbers the book gives, but the visual itself is in the book. Appendix A (Glossary of Option Terminology) and Appendix B (Some Useful Math) were consulted but are summarized rather than reproduced.
