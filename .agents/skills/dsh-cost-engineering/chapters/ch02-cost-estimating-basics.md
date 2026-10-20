# Chapter 2 — Cost Estimating Basics

## Core Idea

"The capability to accurately estimate cost is the most important factor in the cost engineering process." The chapter defines the fifteen cost buckets that make up a manufactured part's price, then builds a complete ground-up estimate for one part — a six-pin over-molded connector for the SPARK truck — twice, once as the supplier and once as the customer, and shows why the two land 2.5× apart. It then extends point-in-time estimating into a multi-year business case with IRR and payback, and closes with the two commercial practices that distort price (productivity givebacks, lump-sum payments), the quoting process, and change management control.

## Frameworks Introduced

**The 15 cost buckets** (Figure 2.1 shows their flow through manufacturing):

1. Raw material — 2. Purchased components — 3. Packaging (inbound) — 4. Logistics (inbound) — 5. Material overhead — 6. Manufacturing labor (direct + indirect) — 7. Manufacturing overhead — 8. Setup — 9. Equipment depreciation — 10. Scrap — 11. Corporate overhead (SG&A) — 12. Profit — 13. Packaging (outbound) — 14. Logistics (outbound) — 15. Tooling and fixtures *(not in piece price)*.

**Two estimate types.** *Point in time* (today's cost, most common) and *business case* (cost fluctuation, cash flows, IRR and payback over the 2–10 year program life).

**Gross weight, not net weight.** Always cost the gross weight — net part weight plus everything discarded (runner, sprue, gate, offal/flash) — with a credit only if regrind or resale is real.

## Key Concepts

- **Raw material** is material in the form the plant receives it (coiled steel for a stamper, resin pellets for a molder), not its mined form.
- **Purchased components** are pre-made parts from sub-suppliers: screws, connectors, or whole sub-assemblies (seats, radios, engines).
- **Packaging (inbound)**: returnable (molded plastic trays, good ROI on long high-volume programs) vs expendable (cardboard, for low volume or overseas travel).
- **Logistics (inbound)**: freight plus duties. Incoterm choice trades cost against inventory ownership and risk; the party with the larger freight volume usually has better leverage. Tariffs are a real cost driver — the book cites 25% tariffs on Chinese imports turning a $10 part into $12.50. Overseas supply chains also carry bulk-breaking, foreign representation, audit travel and cross-culture inefficiency costs.
- **Material overhead** = warehousing plus in-plant handling plus procurement staff (buyers, material planners). Broken out separately in Germany, rarely in the US, but recommended because it swings from ~2% to ~10%+ of purchased price between parts.
- **Direct labor** = workers who add value, plus those who touch the part without adding value (sorting, inspection). **Indirect labor** = in-process material handlers, shift supervisors, machine/maintenance technicians, manufacturing and quality engineers. Plant managers and controllers belong in **manufacturing overhead**, not indirect labor, because they are not responsible for any specific part.
- **Manufacturing overhead** = plant costs not tied to a specific product: rent, maintenance, utilities, supplies, outside services (uniforms, cafeteria), plant management staff. Split fixed (lease, plant management) vs variable (equipment electricity).
- **Setup**: negligible per piece at high volume, dominant at low volume. Should include lost machine time, not only technician labor.
- **Equipment depreciation**: an accounting smoothing device. Past investment is sunk and irrelevant to future decisions; what matters is cash needed to maintain/replace capability. Charging depreciation on paid-off assets inflates cost and loses bids; assuming zero equipment cost wins bids and starves replacement.
- **Scrap** ranges from ~0.5% (fully automated, controlled assembly) to ~30% (decorative painting with multiple coats and laser etching). Always clarify: percentage *of what* (cost, revenue, material), and *when* (launch vs full production). Scrap is cumulative — see Table 2.1.
- **Corporate overhead (SG&A)**: sales/marketing, non-manufacturing finance/accounting/HR/IT, top management, their buildings. GAAP usually folds R&D into SG&A; IFRS often reports it separately; some countries (Germany) put plant management staff into corporate overhead. Companies may legitimately keep two cost views, one for accounting and one for quoting.
- **Efficiency vs OEE vs capacity utilization**: inefficiency = planned + unplanned downtime during manufacturing; uptime/efficiency factor is its complement; OEE additionally discounts time spent making bad parts; capacity utilization additionally counts time lost to lack of business (two shifts of three = 66.67%).
- **Tooling and fixtures** have three cost elements — design, raw material/purchased components, tool/fixture manufacture — with design and manufacture usually quoted per hour. Tool life varies hugely: ~1,000,000 cycles for an injection mold vs ~100,000 for a die casting die. Confirm tool life covers program life; if not, plan purchase or amortization of replacements.

## Mental Models

- **"Cost estimates are the navigation tools."** The recurring metaphor: a product development team without cost estimates is a navigator without instruments.
- **"Peanut butter on toast."** Spreading a cost bucket evenly across products because it is hard to allocate is the central error the book attacks; it appears first here (material overhead) and again in Ch. 3 and Ch. 5.
- **Sanity-check every percentage in absolute dollars.** 10% material overhead on the connector = $506/yr; at $10/hr that is 50 hours per year to manage this part's material. Is 50 hours enough? Ask the question in hours and headcount, not percent.
- **Estimates, not "exactomates."** Two estimators at the *same* company will differ. Best achievable accuracy is 5%–10% of actual; transparency and collaboration narrow it as the design matures.
- **Cycle-time reduction ≠ cost reduction.** Saving 2 s/part at 15 s frees one hour per shift — not enough to release an operator, so unit cost does not change.

## Anti-patterns

- **Markup instead of margin.** The customer estimator in the worked example applies X × (1 + 0.10) where X ÷ (1 − 0.10) is meant. The book flags this explicitly as "a common mistake."
- **Flat SG&A % and flat profit % on every project.** A 10% assumption gives $5,000,000 SG&A to a $50M project and $500 to a $5,000 project when the true minimum support cost is $5,000 per project regardless of size. You lose the overpriced work and win the underpriced work.
- **Point-in-time estimating on a 5-year program.** "Although some might argue that trying to predict the economic future is futile, they are actually predicting it by not trying" — assuming no change is itself a forecast, and a wrong one.
- **Ignoring inventory holding when assuming a single annual production run.** The customer's setup estimate divides $19 over 10,000 parts and silently assumes the supplier warehouses a year of stock for free.
- **Sandbagging in the quote team.** Purchasing or manufacturing padding cost to cover risk, inflating the quote and losing the business.
- **Sales objectives on revenue only.** The sales rep optimizes for winning, not for profit.
- **"Sharpen your pencils."** Re-quoting rounds that manipulate cost to reach a winning price rather than defining a real walk-away price.
- **No change management control.** Post-award design and process changes erode an awarded-at-profit program into a loss with no time and no customer sympathy.

## Reference Tables

**Table 2.1 — Cumulative scrap calculation, decorative plastic part** (as printed; batch of 1,000 moldings through three paint layers)

| | Molding | Paint #1 | Paint #2 | Paint #3 | Pack |
|---|---|---|---|---|---|
| Batch volume at each step | 1,000 | 980 | 931 | 838 | 754 |
| Resin cost per unit | $1.00 | $1.00 | $1.00 | $1.00 | $0.00 |
| Paint cost per unit | $0.00 | $0.30 | $0.30 | $0.30 | $0.00 |
| Total cost of processed resin | $1,000.00 | $980.00 | $931.00 | $837.90 | $0.00 |
| Total cost of processed paint | $0.00 | $294.00 | $558.60 | $754.11 | $0.00 |
| Total material processed at step | $1,000.00 | $1,274.00 | $1,489.60 | $1,592.01 | $0.00 |
| New material used at step | $1,000.00 | $294.00 | $279.30 | $251.37 | $0.00 |
| Scrap % | 2.00% | 5.00% | 10.00% | 10.00% | 0.00% |
| Cost of scrapped resin | $20.00 | $49.00 | $93.10 | $83.79 | $0.00 |
| Cost of scrapped paint | $0.00 | $14.70 | $55.86 | $75.41 | $0.00 |
| **Total scrap cost** | $20.00 | $63.70 | $148.96 | $159.20 | $0.00 |
| Cumulative scrap % of total material cost | 2% | 6% | 15% | 21% | 21% |
| **% units scrapped: 25% — % total scrap cost: 21%** | | | | | |

**Table 2.2 — Sample financial calculation (profit terminology)**

| Line | Value |
|---|---|
| Revenue | $100 |
| Cost of goods sold | $50 |
| Gross profit | $50 |
| Operating expenses | $30 |
| Operating profit (EBIT) | $20 |
| Interest | $3 |
| Taxes | $5 |
| Net profit | $12 |

Definitions given: **Gross margin** = (revenue − COGS) ÷ revenue, excluding corporate overhead. **Operating profit / EBIT** = gross profit − operating expenses (corporate overhead). **EBITDA** = EBIT before depreciation and amortization, used as a cash-flow proxy. **Net profit** = after all costs and taxes.

**Table 2.4/2.5/2.6 — Business case assumption set** (identical drivers, three hurdle-rate scenarios)

| Assumption | Value |
|---|---|
| Volume | 10,000/yr for 5 years |
| Raw material escalation | +3%/yr |
| Purchased parts | −2%/yr |
| Direct labor escalation | +3%/yr |
| Manufacturing overhead escalation | +3%/yr (electricity, gas, indirect labor) |
| Scrap | 2% |
| Price escalation (customer giveback) | −3%/yr |
| New investment | $10,000 total ($0.20/part/yr) |
| Cost of capital | $4,500 total ($0.09/part/yr) |
| Borrowing interest rate | 9% |
| Tax rate | 30% |
| Years of depreciation | 10 |
| SG&A | $5,000/yr ($0.50/part) |

| Scenario | Year-1 price | Lifetime profit | IRR | Payback |
|---|---|---|---|---|
| Table 2.4 — 1-year payback | **$3.00** | $48,470 (34.3% lifetime) | 91% | 1.00 yr |
| Table 2.5 — 2-year payback | **$2.32** | $16,450 (15.1% lifetime) | 36% | 2.00 yr |
| Table 2.6 — 20% IRR hurdle | **$2.15** | $8,450 (8.3% lifetime) | 20% | 3.00 yr |

## Worked Example

**The connector should-cost, built twice.** A six-pin connector for the SPARK truck: six pins with attached wires, over-molded in a plastic housing. Volume 10,000/yr (one per vehicle). Two-cavity mold on a 60-ton press; assembly on a table-top station. The supplier's estimator has internal data; SPARK's estimator does not.

| Bucket | Supplier calculation | Customer calculation |
|---|---|---|
| **Raw material** | Net weight 10 g (drawing) + 5 g discarded (runner/sprue/offal) = **15 g gross**; shot weight 30 g for two cavities. 30% glass-filled resin at $2.50/kg = $0.0025/g. 15 g × $0.0025 = **$0.0375** | Assumes 26 g shot = 13 g/part, best-case resin $2.00/kg. 13 × $0.0020 = **$0.0260** |
| **Purchased components** | Sub-supplier quotes $0.03/pin, $0.04/wire. 6 × $0.03 + 6 × $0.04 = **$0.4200** | Historical data $0.03/pin, $0.05/wire → **$0.4800** |
| **Packaging (in)** | 1% of purchased components: $0.42 × 1% = **$0.0042** | $0.48 × 1% = **$0.0048** |
| **Logistics (in)** | Internal logistics quote = **$0.0210** | 2% of material cost ($0.0260 + $0.48 = $0.5060) = **$0.0101** |
| **Material overhead** | Actual plant data: $1,500/yr ÷ 10,000 = **$0.1500** | 10% of material cost = **$0.0506**. Sanity check: $506/yr ÷ $10/hr ≈ 50 hours/yr of material management |
| **Direct labor** | Assembly: 15 s manual, southern Mexico, $3.00/hr wage + $1.50 fringes = $4.50/hr fully fringed; 15% inefficiency. (15 ÷ 3600) × 1 op × $4.50 × 1.15 = **$0.0215**. Over-molding: 30 s per 2-cavity shot = 15 s/part, same math = **$0.0215**. Total **$0.0430** | 13 s/part, 1.5 operators, 90% efficiency: (13 ÷ 3600) × 1.5 × $4.50 × 1.10 = **$0.0268** |
| **Indirect labor** | Plant has 4 departments (table-top assembly, assembly lines, 60-ton molding, 150-ton molding). Charge: 2 shift leaders @ $6/hr, ½ manufacturing engineer + ½ quality engineer @ $16/hr, 1 technician @ $10/hr → **$48/hr** *(printed as $48; the listed components sum to $38)*. Spread over 12 assembly stations + 12 60-ton presses at 15 s each = 2,880 parts/hr. $48 ÷ 2,880 = **$0.0167** | 50% of direct labor = **$0.0134** |
| **Setup** | Weekly production: 10,000 ÷ 50 weeks = 200 parts/run. Assembly setup ½ hr × 1 tech @ $6 = $3; molding setup 1 hr × 2 techs @ $8 = $16. Total **$19/setup** ÷ 200 = **$0.0950** | Assumes one run per year: $19 ÷ 10,000 = **$0.0019** (ignores inventory holding) |
| **Manufacturing overhead** | ABC-derived machine rates: assembly $15/hr, molding $20/hr. (15 ÷ 3600) × $35 × 1.15 = **$0.1677** | Allocated on original equipment cost: assembly $5/hr, molding $30/hr. (13 ÷ 3600) × $35 × 1.15 = **$0.1453** |
| **Depreciation** | Real capital: 12 assembly stations bought for $1,000 each = $12,000 over 10 yrs ÷ 5,000 hr ÷ 12 = $0.02/hr. Molding: $1,000,000 outstanding loan + $200,000 for two replacement machines = $1,200,000 ÷ 10 yrs ÷ 5,000 hr ÷ 12 = $2.00/machine-hr. Per part: (15 ÷ 3600) × ($0.023 + $2.30) × 1.15 = **$0.0111**, 50% below GAAP depreciation. Plus maintenance: ($40,000 technician + $30,000 parts) ÷ 12 machines ÷ 5,000 hr = $1.16/hr → $0.0055/part. **Total $0.0166** | GAAP method on original cost: assembly $10,000 ÷ 8 yrs ÷ 5,000 hr = $0.25/hr; molding $200,000 ÷ 10 yrs ÷ 5,000 hr = $4.00/hr. Per part $0.0019 + $0.0192 → table shows **$0.0211** |
| **Scrap** | 2% of all incurred cost = **$0.0194** | 2% of incurred cost = **$0.0156** |
| **Total manufacturing cost** | **$0.9911** | **$0.7956** |
| **Corporate overhead (SG&A)** | Minimum real support cost $5,000/yr ÷ 10,000 = **$0.5000** | 10% markup = **$0.0796** |
| **Profit** | Company requires 1-year payback on the $10,000 project investment: $10,000 ÷ 10,000 = **$1.0000** | 10% markup = **$0.0796** |
| **Price** | **$2.4911** | **$0.9547** |
| Packaging (out) | Cardboard + pallet + shrink wrap $100/pallet ÷ 2,000 parts = **$0.0500** | 2% of price = **$0.0191** |
| Logistics (out) | Internal shipping quote = **$0.0300** | 3% of price = **$0.0286** |
| **Price w/ packaging & logistics** | **$2.5711** | **$1.0025** |

The gap is **$1.5686** — 2.5×. Almost all of it sits in two buckets: corporate overhead ($0.4204) and profit ($0.9204), both driven by the supplier pricing a genuinely low-volume, investment-heavy project on absolute cost and payback while the customer applies flat percentages.

**The business case layer.** Rerunning the same connector as a five-year business case (Table 2.4) with +3% inflation on raw material, labor and manufacturing overhead, −2%/yr on purchased parts, a −3%/yr customer giveback, $10,000 investment, 9% borrowing rate and 30% tax, the one-year-payback requirement produces a **year-one price of $3.00**, lifetime revenue $141,270, lifetime profit $48,470 (34.3%), **IRR 91%**. Relaxing to a two-year payback (Table 2.5) drops price to **$2.32** with IRR still 36%. Targeting a 20% IRR hurdle instead (Table 2.6) drops price to **$2.15** with a three-year payback. The point: the calculation does not set the price — the market does — but it tells management whether the business is worth pursuing, and whether a strategic loss (customer target ~$1.03) is affordable.

## Key Takeaways

1. Build the estimate bucket by bucket from the ground up; do not price from history alone.
2. Cost the gross weight, credit regrind/resale only when real.
3. Break out material overhead separately — it swings 2% to 10%+ of material cost between parts.
4. Setup cost is a low-volume killer; trade it explicitly against inventory holding cost.
5. Charge real future capital, not GAAP depreciation on sunk assets — and add maintenance/repair.
6. Model scrap cumulatively through the process sequence.
7. Replace flat SG&A and profit percentages with minimum absolute support cost and ROI/payback.
8. Know the difference between markup and margin.
9. For any program longer than a day, run the business case: inflation, givebacks, cash flows, IRR, payback.
10. Abolish productivity givebacks and lump-sum payments — both are simply pre-loaded into year-one price; the lump sum is a high-interest loan from the supplier.
11. Run quoting as a timed, cross-functional process with a quoting manager and a defined walk-away price.
12. Control post-award changes with an ERP/PLM-backed process, or an awarded profit becomes a loss.

## Connects To

- [ch01-what-is-cost-engineering.md](ch01-what-is-cost-engineering.md) — where SPARK and the target cost come from
- [ch03-cost-based-negotiation.md](ch03-cost-based-negotiation.md) — the same connector estimate taken into gap-closure negotiation, bucket by bucket, to a settled $2.4722
- [ch04-manufacturing-process-estimating.md](ch04-manufacturing-process-estimating.md) — cost drivers and rate ranges for the molding, stamping and plating steps behind these numbers
- [ch05-cost-allocation-methods.md](ch05-cost-allocation-methods.md) — why the customer's original-equipment-cost machine rates are wrong
