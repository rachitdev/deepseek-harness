# Chapter 22 — Stock Index Futures and Options

## Core Idea
Stock index futures and options are among the most actively traded derivatives, and the two are so entangled that neither can be discussed alone. The distinguishing feature is that **an index is not a deliverable thing**: it cannot be physically settled, cannot be perfectly replicated with shares, and the futures contract on it carries a **built-in structural bias** because portfolio managers are almost always hedging long equity by selling futures.

## Frameworks Introduced

**Index construction methods.** Take the ABC Index:

| Stock | Price | Shares outstanding | Market capitalization |
|---|---|---|---|
| A | 80 | 100 | 8,000 |
| B | 20 | 2,000 | 40,000 |
| C | 50 | 400 | 20,000 |

| Method | Formula | ABC value | Who dominates |
|---|---|---|---|
| **Price weighted** | Σ priceᵢ | 80 + 20 + 50 = **150** | The **highest-priced** stock (A, 53.33%) |
| **Capitalization weighted** | Σ (priceᵢ × sharesᵢ) | 8,000 + 40,000 + 20,000 = **68,000** | The **largest-cap** stock (B, 58.82%) |
| **Equal weighted** | Σ (priceᵢ / base priceᵢ) | 1 + 1 + 1 = **3** | Every stock equally |
| **Geometric weighted** | [Π(today's priceᵢ / yesterday's priceᵢ)]^(1/n) | (80/80 × 20/20 × 50/50)^⅓ = **1.00** | Every stock equally, multiplicatively |

Index weightings:

| Stock | Price weighted | Cap weighted |
|---|---|---|
| A | 80/150 = **53.33%** | 8,000/68,000 = 11.76% |
| B | 20/150 = 13.33% | 40,000/68,000 = **58.82%** |
| C | 50/150 = 33.33% | 20,000/68,000 = 29.41% |

**The index divisor.**
```
raw index value / divisor = target index value
divisor = raw index value / target index value
```
ABC targeting 100: price-weighted divisor = 150/100 = **1.50**; cap-weighted = 68,000/100 = **680**.

**The universal impact rule.**
```
percent change in the index = percent change in the stock × the stock's weighting
new index price = old index price × (1 + Σ percent changeᵢ × weightᵢ)
```
This holds for price-weighted, cap-weighted, and equal-weighted indexes alike. For a **price-weighted** index there is a shortcut: **every one-point move in any component changes the index by `1 / divisor`** — the same amount for every component, every time. (Counterintuitive, but the shrinking percentage effect of each successive point is exactly offset by the rising weighting.)

**Forward price of a stock index.** Exactly the individual stock formula, `F = S × (1 + r·t) − D`, but the dividends arrive spread out over time from many companies. The practical approximation treats the dividend flow as a **negative interest rate**:
```
F = S × [1 + (r − d) × t]
```
where d is the average annualized dividend yield in percent.

**The futures delta.** Ignoring dividends, `F = S × (1 + r·t)`, so for each point the index rises the futures contract rises by **1 + r·t**. Treating the cash index as the underlying, **the futures contract's delta is 1 + r·t**, and a delta-neutral index arbitrage requires holding **1 + r·t** times the exact index replication in stock.

**Index arbitrage / program trading.** Buy the underpriced side, sell the overpriced side, carry to expiration where futures and index converge.
- **Buy program** — buy the component stocks, sell futures (effectively borrowing cash).
- **Sell program** — sell the component stocks, buy futures (effectively lending cash).

**Hedging instruments for cash index options** when no matching futures month exists — use the nearest futures contract beyond expiration:

| Cash index option expiration | Hedging instrument |
|---|---|
| January, February, March | March futures |
| April, May, June | June futures |
| July, August, September | September futures |
| October, November, December | December futures |

## Key Concepts
- **Broad-based vs. narrow-based indexes.** Broad-based ones represent a market or region: S&P 500 (500 stocks, US), Nasdaq 100, Russell 2000, FTSE 100 (UK), Nikkei 225 (Japan), ASX 200 (Australia), KOSPI 200 (South Korea), BOVESPA (Brazil, 67), TSE (Canada, 60), MSCI EAFE (1,500), CSI 300 (China). Some with few components still count as broad based if they span the economy: Dow Jones Industrials (30), DAX (30), CAC 40, EuroStoxx 50, OMX 30, FTSE MIB (40), AEX (25), Hang Seng (30), Sensex 30, IPC (Mexico, 35), Tel Aviv 25, Straits Times (30), Swiss Market (20). **Narrow-based** indexes cover a segment: Dow Jones Transportation (20), S&P Utilities (31), Bank Index (24), Semiconductor Index (30), Oil Service (15), UK Homebuilders (6), EuroStoxx Technology (16), FTSE Mining (24).
- **Equal-weighted indexes drift and must be rebalanced.** If A, B, C move to 76, 25, 51 the index is 0.95 + 1.25 + 1.02 = **3.22**, with B now dominating. Rebalancing resets it to 76/76 + 25/25 + 51/51 = **3.00** — but to keep the published series continuous, subsequent values are multiplied by 3.22/3.00 = **1.0733**, the 7.33% gain of the previous rebalancing period.
- **Price weighting is a legacy of arithmetic.** "It is a relatively easy task to add up a list of individual stock prices," so the earliest indexes — the **Dow Jones Industrial Average, introduced in 1896** — were price weighted. Capitalization weighting gives a truer picture of each company's value, and with computers now doing the work, most widely followed indexes are cap weighted.
- **Free float.** Shares in the company treasury, held by officers, or in employee investment plans are not available to the public. Cap-weighted indexes typically use only the **free float** — the tradable shares.
- **When the divisor must change.** A **2-for-1 split** of Stock A (price 40, shares 200) leaves capitalization at 8,000, so a **cap-weighted index needs no adjustment**. But the price-weighted raw value falls to 110, which would drop the index to 73.33 — economically absurd for an investor whose holdings did not change — so the divisor is reset to **110/100 = 1.10**. The index organization announces it: "The new ABC Index divisor is 1.10 as a result of the 2 for 1 stock split of Company A." A **component substitution** also requires adjustment: replacing C with D (75, 500 shares) gives divisors of `(80+20+75)/100 = **1.75**` (price weighted) and `(8,000+40,000+37,500)/100 = **855**` (cap weighted).
- **Total-return indexes** assume all dividends are immediately reinvested, so a price decline caused by a dividend payout does not lower the index. ABC at 100.00 with divisor 1.50; Stock A pays 1.00 and opens ex-dividend at 79 — a traditional index falls to 99.33, but a total-return index adjusts the divisor to **1.49** to hold 100.00. The **German DAX** is the best-known total-return index; the S&P 500 is published in both forms, though the traditional version is far more widely followed.
- **Volume-weighted average price (VWAP).** A closing print can be an anomaly. With a closing quote of 43.10–43.30 and the last three trades being 1,800 at 43.10, 2,400 at 43.15, and 300 at 43.30, some exchanges compute the closing price as `[(300 × 43.30) + (2,400 × 43.15) + (1,800 × 43.10)] / 4,500 = **43.14**` for the index calculation.
- **Cash settlement is a necessity, not a choice.** Physical delivery of an index would require the right number of shares in many companies, "which would be unmanageable for most clearing organizations," and might require **fractional shares**, which are impossible. Settlement is a final variation payment against the expiration index value.
- **The dividend approximation fails on short-dated contracts.** Dividends arrive in discrete, unevenly spaced bundles. For the Dow Jones Industrial Index over an October–December cycle, the total annualized dividend is about **2.75%** — accurate for a position taken at the start of the cycle, but a **gross overstatement** for one taken after all dividends have been paid, when the true remaining payout is near zero.
- **Market-on-close orders and the move to AM expiration.** Index arbitrage is unwound at expiration with a **market-on-close order**, guaranteeing the liquidation prices are the ones that set the final index value. As program trading grew, the resulting closing orders "often caused disruptions in the normal trading process, with unexpected jumps in the prices of component stocks," so most derivative exchanges moved to settling on **opening** prices — **AM expiration**. A small number of contracts still use **PM expiration**, including options on exchange-traded funds.
- **Triple witching.** Most stock index futures trade on the March–June–September–December cycle, so four times a year index futures, options on those futures, and cash index options **all expire simultaneously**, typically the third Friday, all settled in cash.
- **Replicating a price-weighted index is easy.** Because each stock's weighting is proportional to its price, buying an **equal number of shares of each component** replicates it. A cap-weighted index requires holdings proportional to capitalization instead.
- **Index arbitrage risks** mirror any stock-futures arbitrage: unfixed interest rates, mis-estimated dividends, short-sale restrictions, and an unfavorable short-stock rate. Rate moves hurt a **buy program** (borrowing) when rates rise and help a **sell program** (lending) — magnified for long-dated futures. Dividend increases help a buy program and hurt a sell program; in a broad index one company's dividend change is immaterial, but **in a narrow-based index even one firm's change can decide profitability**.
- **The structural downward bias in index futures.** "The vast majority of equity portfolio managers take long positions in equities," and even a manager who thinks a stock will underperform rarely sells short. So hedging almost always means **selling futures**, and "this constant selling pressure tends to depress the price of futures contracts compared with theoretical value." Arbitrageurs cannot fully correct it: taking the other side means buying futures and **selling stock short**, which may be prohibited, is never as easy as buying, and does not earn full interest. "In stock index markets around the world, there tends to be constant downward pressure on futures prices."
- **Two kinds of stock index options.** Options on **index futures** (CME began trading options on S&P 500 futures in **January 1983**) settle into a futures position — with margin and immediate variation — *except* when the option and its underlying future expire together, in which case they settle in cash. Options on a **cash index** never produce an underlying position; they always settle in cash.
- **OEX and the end of American cash index options.** The CBOE listed the first cash index options in **March 1983**. Unable to obtain rights to the S&P 500 or Dow at the time, it created its own **Options Exchange Index (OEX)** of 100 large U.S. companies, and made the options American to match its equity options. "Once trading began, it became obvious that the early exercise feature resulted in additional and unforeseen risks and also greatly complicated theoretical evaluation. As a result, **all exchange-traded cash index options are now European**." (The CBOE later reached an agreement with Standard and Poor's; OEX was renamed the **S&P 100 Index** but kept the ticker.)
- **Cash index tickers commonly end in X:** SPX, DJX, DAX, AEX, OMX 30, ASX 200.
- **Why hedge cash index options with futures.** Hedging with a basket of component stocks would require trades in many stocks, possibly fractional shares, and continual rebalancing as the delta changes — "impractical for most traders." The futures contract on the same index is liquid and closely correlated. It is not a perfect solution, since the future can trade above or below theoretical value against the cash index.
- **The futures price *is* the forward price.** For March/June/September/December options the future and cash index converge at expiration, so a trader can treat the futures contract as the underlying — which also makes theoretical sense, because "option values are derived from the forward price of the underlying contract, and the futures contract is simply the traded form of the forward price." Where both cash options and futures options exist on an index and expire together, **there is effectively no difference between them; they trade at the same prices** (barring a very slight early exercise value in deeply in-the-money American futures options).

## Anti-patterns
- **Using the annualized dividend yield to price a short-dated index forward** without checking where you are in the dividend cycle.
- **Hedging a short index future with an exact index replication.** The stock side is stock-type settled (unrealized P&L) while the futures side is futures-type settled (daily variation earning or costing interest) — a **settlement-risk mismatch**. Hold **1 + r·t** times the replication instead, and adjust it as time passes and rates change.
- **Assuming an index's last-trade value is its true value** when a component's trading is halted, or when the closing print is an anomaly.
- **Using the December futures price as the underlying for November cash options.** The forward prices differ.
- **Expecting arbitrage to eliminate the futures market's downward bias.** Short-selling constraints prevent it.

## Reference Tables

**Effect of a 10% rise in one stock on the price-weighted ABC Index** (divisor 1.50, index 150 raw)

| Stock rising 10% | New price | Index change | Percent change |
|---|---|---|---|
| A (80 → 88) | 88 | 150 → 158 | 8/150 = **5.33%** |
| B (20 → 22) | 22 | 150 → 152 | 2/150 = 1.33% |
| C (50 → 55) | 55 | 150 → 155 | 5/150 = 3.33% |

**Replicating the ABC Index with $100,000**

| Stock | Price-weighted holdings | Capitalization-weighted holdings |
|---|---|---|
| A | 53.33% × $100,000 / 80 ≈ **667 shares** | 11.76% × $100,000 / 80 ≈ **147 shares** |
| B | 13.33% × $100,000 / 20 ≈ **667 shares** | 58.82% × $100,000 / 20 ≈ **2,941 shares** |
| C | 33.33% × $100,000 / 50 ≈ **667 shares** | 29.41% × $100,000 / 50 ≈ **588 shares** |

Check: (667 × 80) + (667 × 20) + (667 × 50) ≈ $100,000 ✓ and (147 × 80) + (2,941 × 20) + (588 × 50) ≈ $100,000 ✓. Note the price-weighted replication is simply **an equal number of shares of each stock**.

**Adjusting the hedge for the futures delta** (cap-weighted ABC, 6.00% interest)

| Stock | Shares to replicate the index | Shares to offset a 4-month future (× 1.02) | Shares to offset a 3-month future (× 1.015) |
|---|---|---|---|
| A | 147 | ≈ **150** | ≈ **149** |
| B | 2,941 | ≈ **3,000** | ≈ **2,985** |
| C | 588 | ≈ **600** | ≈ **597** |

**Program trade exposures**

| | Buy program (buy stocks, sell futures) | Sell program (sell stocks, buy futures) |
|---|---|---|
| Economic effect | Borrowing cash | Lending cash |
| Rates rise | Hurts | Helps |
| Rates fall | Helps | Hurts |
| Dividends increase | Helps | Hurts |
| Dividends cut | Hurts | Helps |

**Serial-month forward price reconciliation** (index options)

| | November | December |
|---|---|---|
| 1,000 call | 34.80 | 44.60 |
| 1,000 put | 29.85 | 34.75 |
| Combo (C − P) | 4.95 | 9.85 |
| Implied forward | **1,005** | 1,010 (the futures price) |

November/December 1,000 roll = 9.85 − 4.95 = **4.90**.

## Worked Example

**A. Setting and adjusting the divisor.** ABC targets an index of 100.
- Price-weighted divisor: 150/100 = **1.50**. Cap-weighted: 68,000/100 = **680**.
- Stock B rises to 25: price-weighted index = (80 + 25 + 50)/1.50 = 155/1.50 = **103.33**; cap-weighted index = (8,000 + 50,000 + 20,000)/680 = 78,000/680 = **114.71**.
- Stock A splits 2-for-1 (price 40, 200 shares): the price-weighted raw value falls to 110, which would show 73.33 — wrong, since no shareholder's wealth changed. New divisor = **110/100 = 1.10**. The cap-weighted index needs no change; A's capitalization is still 8,000.
- Company C is replaced by Company D (75, 500 shares): price-weighted divisor becomes **1.75**, cap-weighted **855**.

**B. Verifying the impact rule two ways.** Stock A, currently 80, rises one point.

*Price-weighted (index 100, divisor 1.50):* directly, `(81 + 20 + 50)/1.50 = 151/1.50 = **100.67**`. By the rule, A's weighting is 53.33% and the stock moved 1/80 = 1.25%, so the index moves 0.5333 × 0.0125 = **0.0067**, giving (1 + 0.0067) × 100 = **100.67** ✓.

*Cap-weighted (index 100, divisor 680):* directly, `[(81 × 100) + 40,000 + 20,000]/680 = 68,100/680 = **100.147**`. By the rule, A's weighting is 11.76%, so 0.1176 × 0.0125 = **0.00147**, giving **100.147** ✓.

And the price-weighted shortcut: every 1.00 change in any component moves the index by **1.00/1.50 = 0.67** ✓.

**C. Pricing an index during a trading halt.** The index is **1,425.50**. A component last traded at **63.00** but trading is halted pending news; the exchange indication for the reopening is **67.50–68.00**, and the stock's weighting is **2.5%**. Using a midpoint of 67.75:
```
1,425.50 × [1 + 0.025 × (67.75 − 63.00)/63.00] = 1,428.19
```
Or, if the trader already knows that each point of this stock moves the index by 0.57:
```
1,425.50 + (4.75 × 0.57) = 1,428.21
```
Either estimate "will enable the trader to make a more informed decision."

**D. Pricing an index forward.**
```
Current index price      = 100.00
Time to maturity         = 4 months
Interest rate            = 6.00%
Average annualized yield = 2.25%

F = 100.00 × [1 + (0.06 − 0.0225) × 4/12] = 100.00 × 1.0125 = 101.25
```

**E. Exercising an option on an index future vs. holding to expiration.** A trader owns a **February 1,000 call** on a stock index future. February is a serial month, so the underlying is the **March future**, and the option is **PM expiration**. With the March future at **1,025** at February expiration, the trader exercises, becoming **long one March future at 1,000**. Unless he sells it immediately he must post margin, and his account is credited `25.00 × $100 = **$2,500**`. The assigned trader is short the March future and **debited $2,500**. Both still hold market positions with opposite wishes.

Now the **March 1,000 call** at March expiration. Because the March future is **AM expiration**, so is the option: its value is set by the **opening** prices of the component stocks. If the call is out of the money it expires worthless; if in the money the exchange settles it in cash. With an opening index of **1,040** and a $100 multiplier, the long is credited `40 × $100 = **$4,000**` and the short debited the same. Once the cash moves, **both traders are out of the market entirely** — subsequent index moves are irrelevant.

**F. Backing out a serial-month forward price with put-call parity.**
```
November 1,000 call = 34.85 (34.80 in the calculation)
November 1,000 put  = 29.90 (29.85 in the calculation)
Time to November expiration = 2 months
Annual interest rate = 6.00%

From C − P = (F − X)/(1 + r·t):
F = (C − P) × (1 + r·t) + X
F_Nov = (34.80 − 29.85) × 1.01 + 1,000 = 1,005
```
The December futures contract is trading at **1,010**, so the November forward sits **5.00 below** it. To value November cash options as the December future fluctuates, **use the December futures price less 5.00** as the underlying.

*Consistency check on December.* With the December future at 1,010, three months out, at 6.00%:
```
C − P = (1,010 − 1,000) / 1.015 = 9.85
```
So if the December 1,000 call trades at 44.60, the December 1,000 put should be **44.60 − 9.85 = 34.75**. The November/December 1,000 roll is then `9.85 − 4.95 = **4.90**`.

## Key Takeaways
- Price-weighted indexes are dominated by the highest-priced stock; cap-weighted by the largest company; the Dow (1896) is the archetypal price-weighted index and most modern indexes are cap-weighted on free float.
- Any index change = percent change in the stock × the stock's weighting; for a price-weighted index, one point in any component always moves the index by 1/divisor.
- The divisor absorbs splits and substitutions in price-weighted indexes and dividend payouts in total-return indexes; cap-weighted indexes are unaffected by splits.
- Index forwards use F = S × [1 + (r − d) × t] as a practical shortcut, unreliable for short maturities because dividends come in bundles.
- Index futures have a delta of 1 + r·t against the cash index, so index arbitrage requires holding more stock than an exact replication — and adjusting it as time passes and rates move.
- Program trading forced the move from PM to AM expiration for most index contracts; triple witching is the quarterly simultaneous expiration.
- Index futures carry a persistent downward bias because portfolio hedging is one-sided and short-selling constraints prevent full arbitrage correction.
- All exchange-traded cash index options are now European (the American OEX experiment proved too risky and hard to value); they are hedged with the nearest futures month, and where cash and futures options expire together they trade at the same prices.

## Connects To
- [Chapter 2 — Forward Pricing](ch02-forward-pricing.md): the forward price formula the index version adapts.
- [Chapter 3 — Contract Specifications and Option Terminology](ch03-contract-specifications-and-terminology.md): cash settlement, AM vs. PM expiration, serial options.
- [Chapter 15 — Option Arbitrage](ch15-option-arbitrage.md): settlement risk and put-call parity, both used here.
- [Chapter 16 — Early Exercise of American Options](ch16-early-exercise-american-options.md): why American cash index options were abandoned.
- [Chapter 1 — Financial Contracts](ch01-financial-contracts.md): notional value, point value, and stock-type vs. futures-type settlement.
- [Chapter 21 — Position Analysis](ch21-position-analysis.md): the stock split arithmetic that parallels the divisor adjustment.
- [Chapter 25 — Volatility Contracts](ch25-volatility-contracts.md): index-based volatility products such as the VIX.
