# Glossary — Option Volatility and Pricing (Sheldon Natenberg, 2nd ed.)

Terms as Natenberg uses them. Chapter references point to where the term is developed. Note the author's own caveat: "option terminology is not uniform. Traders may sometimes refer to different strategies or option characteristics with the same term."

**Adjustment** — a trade made primarily to return a position to delta neutral; carries no theoretical edge when executed in the underlying (Ch 8, 11).

**AM expiration** — settlement value determined by the **opening** prices of the underlying components; standard for most stock index contracts (Ch 3, 22).

**American option** — exercisable on any business day prior to expiration. Nothing to do with geography (Ch 3, 16).

**Arbitrage** — "the buying and selling of the same or very closely related instruments in different markets to profit from an apparent mispricing." Natenberg refuses "riskless profit" as a definition (Ch 2, 15).

**Arbitrage boundary** — the lowest (lower boundary) or highest (upper boundary) price at which a contract can trade without creating an arbitrage (Ch 16).

**Assignment** — notification that the seller of an option must fulfill the buyer's exercise. Assignment on exchange-traded options is essentially **random** among unclosed short positions (Ch 3).

**At the forward** — exercise price equal to the **forward** price at expiration; the true benchmark strike, and often the most actively traded (Ch 5, 12, 18).

**At the money** — exercise price equal to the current underlying price; in practice, the strike closest to it. Technically out of the money, but singled out for its distinctive characteristics (Ch 3).

**Automatic exercise** — the clearinghouse exercises in-the-money options at expiration absent contrary instructions. Thresholds differ by exchange and trader class, e.g. 0.05 retail / 0.02 professional (Ch 3).

**Backspread** — a delta-neutral ratio spread where **more options are purchased than sold**; +gamma / −theta / +vega, normally executed for a credit (Ch 11).

**Backwardation (backward market)** — cash price above the futures price; long-term futures at a discount to short-term (Ch 2, 25).

**Balanced skew** — implied volatilities roughly symmetric around the underlying; typical of currency markets where longs and shorts hedge equally (Ch 24).

**Basis** — cash price minus forward price. Usually negative, since carry costs normally exceed carry benefits (Ch 2).

**Basis point** — 1/100 of a percentage point, or 0.0001 (Ch 21).

**Black model** — the 1976 Fischer Black variation of Black-Scholes for options on futures; b = 0 (Ch 5, 18).

**Black-Scholes model** — the 1973 Black-Scholes(-Merton) European pricing model; `C = S·N(d₁) − X·e^(−rt)·N(d₂)` (Ch 5, 18).

**Blackout period** — the days before an ex-dividend date during which early exercise of a put can never earn enough interest to offset the lost dividend: `dividend / (exercise price × interest rate / 365)` (Ch 16).

**Box** — synthetic long at one strike plus synthetic short at another, same expiration. Worth the strike interval at expiration (present-valued under stock-type settlement); selling a box is equivalent to borrowing (Ch 15).

**Broad-based index** — an index representing a market or region as a whole, e.g. S&P 500, FTSE 100, Nikkei 225 (Ch 22).

**Bull / bear spread** — a spread that gains value when the underlying rises / falls (Ch 12).

**Butterfly** — three equally spaced strikes, all same type and expiration, always **1 × 2 × 1**. Worth between 0 and the strike interval at expiration, maximal at the body (Ch 11).

**Buy/write** — buying stock and simultaneously selling a call on it, quoted and executed as one price (Ch 17).

**Cabinet bid** — a price below the exchange minimum (typically one currency unit) permitted so traders can clear worthless far out-of-the-money options from their books (Ch 21).

**Calendar spread (time spread, horizontal spread)** — same type and strike, different expirations. Long = buy the long-term, sell the short-term. −gamma / +theta / **+vega** (Ch 11).

**Call option** — the right, not the obligation, to buy (take a long position in) the underlying at the exercise price on or before expiration (Ch 1, 3).

**Cap** — a protective interest-rate call establishing a maximum borrowing cost (Ch 17). Also, a limit on the expiration value of a variance swap (Ch 25).

**Capitalization-weighted index** — index value = Σ(price × shares); the largest-cap stock dominates (Ch 22).

**Carry trade** — borrowing a low-rate currency to buy a high-rate one. Often called arbitrage, but "the term is probably misapplied" given rate and exchange-rate risk (Ch 2).

**Cash-and-carry arbitrage** — buy the cash instrument, sell the forward or future, carry to maturity (Ch 2, 10).

**Cash settlement** — expiration settled by a cash payment equal to the difference between the exercise price and the underlying price; no underlying position results, and hence **no pin risk** (Ch 3, 22).

**CEV (constant-elasticity of variance) model** — Cox-Ross 1976; volatility varies with the underlying price. Mathematically complex, not widely adopted (Ch 23, 24).

**Charm (delta decay)** — sensitivity of the delta to the passage of time. Greatest at ±20/±80 delta; rises as expiration nears (Ch 9).

**Christmas tree (ladder)** — buy (sell) one option at one strike and sell (buy) one each at two further strikes; mimics a strangle with limited risk or reward on one side (Ch 11).

**Circuit breaker** — an exchange rule halting stock trading for a set period when an index moves a predetermined percentage (Ch 23).

**Clearinghouse** — the entity that becomes buyer to every seller and seller to every buyer, guaranteeing exchange-traded contracts. A U.S. clearinghouse has never failed (Ch 1).

**Collar (fence, tunnel, cylinder, range forward, split-strike conversion)** — an underlying position plus a protective option and a covered option. A long collar is a bull vertical spread; a short collar a bear vertical spread (Ch 17).

**Color (gamma decay)** — sensitivity of the gamma to the passage of time. Large and negative at the money, positive near ±5/±95 delta (Ch 9).

**Combo** — the synthetic portion of a conversion or reversal: a long call and short put, or short call and long put, at one strike (Ch 15).

**Condor** — four strikes, two inside (body) and two outside (wings), always 1 × 1 × 1 × 1, with equal spacing between the two lowest and the two highest strikes (Ch 11).

**Contango (normal market)** — long-term futures at a premium to short-term (Ch 2, 25).

**Contingency order** — an order with special execution instructions: all or none, fill or kill, immediate or cancel, market if touched, market on close, not held, one cancels the other, stop limit, stop loss (Ch 11).

**Convenience yield** — the premium a user pays for immediate physical access to a commodity; the explanation for backwardation. Inferred, not observed (Ch 2).

**Conversion (forward conversion)** — long underlying + short call + long put at one strike. The most common option arbitrage (Ch 15).

**Convergence** — the requirement that a futures price and its cash index meet at expiration (Ch 22).

**Covered write (overwrite)** — selling a call against a long underlying (or a put against a short underlying). Synthetically a short put (or short call) (Ch 17).

**Cox-Ross-Rubinstein model** — see **binomial model** (Ch 16, 19).

**Crack spread** — buying or selling crude oil futures against gasoline and heating oil; commonly 3:2:1 (Ch 10).

**Crush spread** — the soybean analogue: soybeans against soybean oil and meal (Ch 10).

**Delta (Δ)** — sensitivity of an option's value to the underlying price. Four readings: rate of change, hedge ratio, equivalent underlying position, and approximate probability of finishing in the money. In Black-Scholes, **N(d₁)** (Ch 7, 18).

**Delta neutral** — all deltas sum to approximately zero; no directional preference under current conditions (Ch 7).

**Demand skew (commodity skew)** — higher exercise prices carry higher implied volatilities; driven by end users buying protective calls (Ch 24).

**Diagonal spread** — different strikes **and** different expirations. A one-to-one diagonal with equal deltas behaves much like a calendar spread (Ch 11).

**Dividend play** — selling deeply in-the-money calls and buying stock before the ex-dividend date, hoping the calls are not exercised (Ch 16).

**Divisor** — the number by which a raw index value is divided to reach the published index; adjusted for splits, substitutions, and (in a total-return index) dividends (Ch 22).

**Dragonfly** — a kurtosis position: strangles offset by at-the-money straddles, classically 2 strangles per straddle (Ch 24).

**Driftless theta** — the volatility-decay component of theta alone, the whole of theta when r = 0 or under futures-type settlement (Ch 18).

**Dynamic hedging** — periodically re-establishing delta neutrality over an option's life to capture its mispricing (Ch 8).

**Efficiency** — |gamma / theta|, a quick risk-reward comparison between spreads of equal edge, valid when all options expire together (Ch 13).

**Equal-weighted index** — every component contributes equally; requires periodic **rebalancing** and a continuity multiplier (Ch 22).

**EWMA (exponentially weighted moving average)** — a volatility forecast weighting recent returns more heavily; λ ≈ 0.94 is common (Ch 20).

**Ex-dividend date (ex-date)** — the first day a stock trades without the right to the dividend; two business days before the record date (Ch 2).

**Exercise price (strike price)** — the price at which the underlying is delivered on exercise (Ch 3).

**Expiration straddle** — buying an at-the-money straddle close to expiration, on the reasoning that models undervalue gap risk. Usually held unadjusted to expiration (Ch 23).

**Extreme-value method (Parkinson)** — a historical volatility estimator using each period's high and low (Ch 6, 20).

**Fat tails** — a distribution with positive kurtosis: more small moves, more big moves, fewer intermediate moves than a normal distribution (Ch 23).

**Fear index** — a nickname for the VIX, reflecting that it may measure demand for protection rather than expected realized volatility (Ch 25).

**Flex option** — an exchange-traded option whose quantity, expiration, strike, and exercise style are negotiated (Ch 3).

**Floating skew** — a skew that shifts horizontally with the underlying price and vertically with implied volatility (Ch 24).

**Floor** — a protective interest-rate put establishing a minimum lending return (Ch 17).

**Forward contract** — terms agreed now, exchange of money and goods deferred to maturity (Ch 1).

**Forward price** — `current cash price + costs of buying now − benefits of buying now`. The mean of the pricing distribution (Ch 2, 5).

**Forward rate / FRA** — a rate applying to a period beginning in the future; a 3 × 9 forward rate is a six-month rate beginning in three months (Ch 2).

**Forward volatility** — the volatility implied between two expirations: `√[(σ₂²t₂ − σ₁²t₁)/(t₂ − t₁)]` (Ch 20).

**Free float** — the shares actually available for trading, used in most capitalization-weighted index calculations (Ch 22).

**Frictionless market** — the model assumption of free trading, unlimited borrowing and lending at one rate, no transaction costs, and no taxes (Ch 8, 23).

**Frontspread** — a delta-neutral ratio spread where **more options are sold than purchased**; −gamma / +theta / −vega (Ch 11).

**Fugit** — the number of days until an option becomes an immediate early exercise candidate (Ch 16).

**Gamma (Γ), or curvature** — rate of change of the delta per one-point move in the underlying. Always positive for long options, calls and puts alike. Greatest at the money (Ch 7, 9).

**Gamma rent** — the framing of volatility trading as renting gamma, with the theta as the rent. The break-even move over any interval is one standard deviation (Ch 19).

**Garman-Klass** — a historical volatility estimator using open, high, low, and close (Ch 6, 20).

**Garman-Kohlhagen model** — the 1983 variation for options on physical foreign currencies; b = r − r_f (Ch 5, 18).

**GARCH** — generalized autoregressive conditional heteroskedasticity; forecasting models combining a volatility estimate, a correlation component, and a mean-reversion component. From Engle's ARCH (1982) (Ch 20).

**Guts** — a strangle built from in-the-money options (Ch 11).

**Haircut** — a professional trader's margin requirement on an equity options exchange (Ch 1).

**Hedge ratio** — 100 divided by the option's delta; the number of underlying contracts per option needed for neutrality. Another name for the delta (Ch 5, 7).

**Historical volatility** — realized volatility measured over a past period at a stated interval; the starting point for forecasting future realized volatility (Ch 6, 20).

**Implied volatility** — the volatility that makes an option's theoretical value equal its market price; the market's consensus estimate of future realized volatility. Also applied to whole positions: `evaluation volatility + total edge / total vega` (Ch 6, 7).

**Index arbitrage (program trading)** — buying or selling a mispriced index future against the component stocks. **Buy program** = buy stocks, sell futures; **sell program** = the reverse (Ch 22).

**Interest play** — selling stock and simultaneously selling deeply in-the-money puts that should be exercised, hoping they are not (Ch 16).

**Intrinsic value** — max(0, S − X) for a call, max(0, X − S) for a put. Independent of the expiration date (Ch 3).

**Investment skew** — lower exercise prices carry higher implied volatilities; driven by protective-put buying and covered-call selling in stock markets (Ch 24).

**Iron butterfly / iron condor** — the credit versions of a butterfly and condor, built from a straddle-plus-strangle or two strangles. Buying a traditional structure equals selling the iron one (Ch 14).

**Jump-diffusion process** — the real world: mostly continuous trading punctuated by instantaneous gaps (Ch 23).

**Kappa (Κ)** — the academic name for vega, preferred because vega is not a Greek letter (Ch 7).

**Kurtosis** — the tallness or flatness of a distribution's peak. Positive = **leptokurtic** (tall peak, fat tails); negative = **platykurtic**; normal = **mesokurtic**, kurtosis 0. In skew terms, the curvature of the volatility smile (Ch 23, 24).

**Lambda (Λ), elasticity, omega, leverage value** — the percentage change in an option's value per percentage change in the underlying: `Δ × (S/TV)` (Ch 9).

**Locked (limit) market** — a futures contract at its daily price limit; no trading until it comes off the limit. Synthetic futures via options may still be available (Ch 2, 15, 23).

**Lognormal distribution** — the distribution of prices produced by continuously compounding normally distributed percent returns; skewed right, bounded by zero, mean to the right of the mode (Ch 6).

**Long premium / short premium** — a positive gamma position (wants movement) / a negative gamma position (wants quiet) (Ch 11).

**Margin vs. variation** — margin is security that still belongs to the trader; variation is realized P&L moving daily between accounts (Ch 1).

**Market-on-close order** — an order guaranteeing the closing trade price, used to unwind index arbitrage at expiration (Ch 22).

**Mean reversion** — volatility's tendency to return to a long-run average, unlike price, which can trend for years (Ch 20).

**Midcurve option** — a short-term option on a long-term futures contract; one-, two-, or five-year midcurve (Ch 3).

**Mode, mean, median** — the peak, the balance point (half the *value* on each side), and the midpoint (half the *occurrences* on each side). Identical in a normal distribution; separated in a lognormal one (Ch 18).

**Moneyness** — how far in or out of the money a strike is, expressed as a fraction of the underlying price, or in logarithmic or standard-deviation terms (Ch 24).

**N(x)** — the standard cumulative normal distribution function: the area under the standard normal curve between −∞ and x. N(0) = 0.50 and N(x) = 1 − N(−x) (Ch 18).

**n(x)** — the standard normal distribution: mean 0, standard deviation 1, total area 1, peak height ≈ 0.398942 (Ch 18).

**Narrow-based index** — an index covering a market segment, e.g. Dow Jones Transportation, Semiconductor Index (Ch 22).

**Net contract position** — what a book becomes after a move so large that all options act like underlying contracts. Computed separately for the **upside** (calls + underlying) and **downside** (puts + underlying) (Ch 21).

**NOB spread** — notes over bonds: a 10-year note future against a 30-year bond future (Ch 10).

**Notional value** — units × unit price for a physical commodity, or index price × point value for a cash-settled contract (Ch 1).

**Open interest** — contracts traded and not yet closed out. Long and short open interest are necessarily equal (Ch 1).

**Parity** — an option trading at exactly intrinsic value. A **European** option can trade **below** parity (Ch 3, 16).

**Pin risk** — the risk that the underlying finishes exactly at the exercise price, leaving the holder of a conversion or reversal unsure whether he will be assigned. Absent in cash-settled markets (Ch 15).

**PM expiration** — settlement determined by closing prices on the last trading day; still used for individual stock options and ETF options (Ch 3, 22).

**Portfolio insurance (option replication)** — creating an option's payoff through dynamic hedging when no option exists. Fell out of favor after October 1987 (Ch 17).

**Price-weighted index** — index value = Σ prices; the highest-priced stock dominates. Every one-point move in any component changes the index by 1/divisor (Ch 22).

**Pseudoprobability** — the risk-neutral p in a binomial tree, which can fall outside [0, 1] when interest rates are large relative to u (Ch 19).

**Put option** — the right, not the obligation, to sell (take a short position in) the underlying at the exercise price (Ch 1, 3).

**Put-call parity** — `C − P = (F − X)/(1 + r·t)`. The single most important pricing relationship; also called the combo value, the synthetic relationship, or the conversion market (Ch 15).

**Ratio spread** — buying and selling unequal numbers of the same type of option, same expiration, usually delta neutral (Ch 11).

**Realized volatility** — the annualized standard deviation of percent price changes of the underlying over a stated period. Split into **future realized** (what determines value) and **historical realized** (what you can measure) (Ch 6).

**Reversal (reverse conversion)** — short underlying + long call + short put (Ch 15).

**Rho (Ρ)** — sensitivity to interest rates. The least important of the risk measures for most traders. **Rho_f / phi (Φ)** is the foreign-rate sensitivity of a currency option (Ch 7, 9).

**Risk reversal** — a delta-hedged long-one-wing / short-the-other position used to trade the **slope** of the skew; typically ±25 delta options chosen for equal vegas (Ch 24).

**Roll** — synthetic long in one month against synthetic short in another at the same strike. Value ≈ `X·r·t − D`. Practical only in stock options (Ch 15).

**Round lot / odd lot** — 100 shares / fewer than 100 shares (Ch 3).

**Scalping** — quoting a bid-ask around a perceived equilibrium price without regard to theoretical value (Ch 10, 21).

**Serial option** — a futures option expiration with no corresponding futures month; the underlying is the nearest future beyond expiration (Ch 3).

**Settlement risk** — the risk created when the options and the underlying settle differently; gives a synthetic futures position a delta of `100/(1 + r·t)` rather than 100. The hedging analogue is called **tailing** (Ch 15).

**Sharpe ratio** — average return divided by the standard deviation of returns; William Sharpe, 1966 (Ch 17).

**Short-stock rebate / short rate** — the reduced interest a short seller earns on sale proceeds. `r_l − r_s = r_bc`, the borrowing cost (Ch 2).

**Short squeeze** — inability to borrow stock, which can force the holder of a call calendar to exercise the long-term option (Ch 2, 11).

**Skewness** — the lopsidedness of a distribution, or the tilt of a volatility skew. Positive = longer right tail (Ch 23, 24).

**Special opening rotation** — the CBOE procedure matching standing orders to set one opening trade price per SPX option, used to fix the VIX settlement value (Ch 25).

**Speed** — sensitivity of the gamma to the underlying price; greatest near ±15/±85 delta (Ch 9).

**Sticky-strike / sticky-delta skew** — a skew fixed at each exercise price / a skew plotted against standard deviations from at the money (Ch 24).

**Stock-type vs. futures-type settlement** — full immediate payment with unrealized P&L / margin deposit with daily variation. Also called margin-and-variation settlement (Ch 1).

**Straddle** — a call and a put at the same strike and expiration (Ch 11).

**Strangle** — a call and a put at different strikes, same expiration; conventionally out-of-the-money options (Ch 11).

**Synthetic** — a position built from other instruments that behaves like the target contract. Six exist: long/short underlying, long/short call, long/short put (Ch 14).

**Term structure of volatility** — the pattern of realized volatility ranges by measurement window (the **volatility cone**), or of implied volatility across expirations (Ch 20).

**Theoretical edge** — the average profit from a strategy if the trader's assessment of market conditions is correct (Ch 13).

**Theoretical value** — "the price one would be willing to pay now to just break even in the long run"; the present value of the expected value (Ch 5).

**Theta (Θ)** — the rate at which an option loses value as time passes; conventionally negative. Greatest at the money and accelerating toward expiration (Ch 7, 9).

**Three-way** — a conversion or reversal whose underlying leg is replaced by a deeply in-the-money option (Ch 15).

**Time box (diagonal roll)** — synthetic positions at different strikes **and** different months (Ch 15).

**Time butterfly (time fly)** — one strike, one type, three expirations. Buying the body and selling the wings is a **debit** — the reverse of a strike butterfly (Ch 11).

**Time value (time premium, extrinsic value)** — premium in excess of intrinsic value (Ch 3).

**Total-return index** — an index assuming all dividends are immediately reinvested, so dividend payouts do not lower it. The German DAX is the best known (Ch 22).

**Triple witching** — the quarterly simultaneous expiration of index futures, options on futures, and cash index options, typically the third Friday (Ch 22).

**Underlying (underlying contract)** — the security or commodity to be bought or sold under the option contract. Typically 100 shares for stock options; one futures contract for futures options (Ch 3).

**Vanna** — sensitivity of the delta to volatility, equivalently of the vega to the underlying price. Greatest at ±20/±80 delta (Ch 9).

**Variance swap (variance contract)** — a realized volatility contract settling in variance points, `value per variance point = vega notional / (2 × volatility price)` (Ch 25).

**Vega** — sensitivity to volatility, per percentage point. Positive for all long options. Greatest at the money and for long-dated options. Nearly constant in volatility for at-the-money options (Ch 7, 9).

**Vega decay (DvegaDtime)** — sensitivity of the vega to time; greatest at ±20/±80 delta (Ch 9).

**Vertical spread (credit/debit spread)** — one option bought and one sold, same type and expiration, different strikes. The only spread that keeps its directional sign under all conditions (Ch 12).

**VIX** — the CBOE volatility index; a theoretical 30-day implied volatility. Originally (1993) computed from OEX at-the-money options; since 2003 from a 1/X² strip of out-of-the-money SPX options requiring no pricing model (Ch 25).

**Volatility (σ)** — "just a trader's term for standard deviation," expressed as a one-standard-deviation annualized percentage price change (Ch 6).

**Volatility skew (smile, smirk)** — the distribution of implied volatilities across exercise prices (Ch 24).

**Volatility spread** — a spread that is approximately delta neutral and sensitive to the underlying price, implied volatility, and time (Ch 11).

**Volatility surface** — the term structure across expirations combined with the skew across exercise prices (Ch 24).

**Volga (vomma)** — sensitivity of the vega to volatility, a contraction of "volatility gamma." Greatest near ±10/±90 delta (Ch 9, 13).

**VWAP (volume-weighted average price)** — a volume-weighted closing price used in place of a possibly anomalous last trade for index calculation (Ch 22).

**Wingspread** — the family comprising butterflies and condors (Ch 11).

**Yield volatility vs. price volatility** — volatility computed from an interest-rate instrument's yield versus from its price (Ch 6).

**Zero-mean assumption** — setting μ = 0 in a historical volatility calculation, so a steadily trending contract does not register zero volatility (Ch 20).

**Zomma** — sensitivity of the gamma to volatility. Large and negative at the money, positive near ±5/±95 delta (Ch 9).
