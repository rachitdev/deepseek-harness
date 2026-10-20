# Chapter 3 — Cost-Based Negotiation

## Core Idea

Cost-based negotiation replaces "collect & compare" (gather quotes, take the lowest, leverage suppliers against each other) with a data-driven process that resolves the *gaps between two cost estimates* and converts those gaps into cost reduction ideas. "Even if the initial price is higher than other competitors, the quote would not be automatically disqualified, but instead, a process would begin to understand it and find ways to improve it."

The precondition is non-negotiable: **a mandatory, standardized cost breakdown submission by the supplier.** "This is required for the gap closure process to be successful; nothing can be achieved without it."

## Frameworks Introduced

**The nine-step gap closure process** (Figure 3.2 — the figure itself did not extract; the text describes the flow): mandatory standardized cost breakdown → compare bucket by bucket against the customer's own estimate → visualize as a waterfall chart (Figure 3.3) to Pareto the gaps → interrogate the biggest gap drivers with data → agree or task an action → capture every disagreement as a potential cost reduction idea → re-price → agree → carry the agreed cost model forward to future parts.

**The collaboration ladder.** Three states, from Figure 3.1: (a) price hammers — customers chasing suppliers with no cost data; (b) high-stakes poker — some cost data, both sides holding cards; (c) open-book collaborative — negotiation purely on cost data and assumptions, "without the uncomfortable lying."

**Pre-agreed cost models.** In a mature open-book relationship, customer and supplier agree a cost model that simulates the supplier's cost structure *before* any quote. The customer can then estimate cost during development without an RFQ, and both sides carry less financial risk.

**Reverse-engineered P&L.** When the supplier hides cost in manufacturing overhead, rebuild the supplier's plant P&L from data gathered on the plant visit (headcount by type, square footage, capital employed) and test whether the quoted burden rate is consistent with the plant's actual total cost.

**Types of cost estimates.** Beyond *point in time* and *business case*: **best-in-class (BIC)**, **supplier-specific**, and **regional** scenarios — or a mixture. Example: estimate a part three ways (manual, semi-automated, fully automated in Mexico) to understand the full option space before negotiating.

## Key Concepts

- **What happens without open book.** In the worked non-collaborative path, the supplier's team re-checks every assumption, concludes they are all correct, knows the customer will only accept ~10% SG&A and ~5% profit, and therefore either refuses a breakdown ("corporate policy") or submits a **modified** breakdown with the excess buried in purchased parts and manufacturing overhead (Table 3.1: purchased parts inflated $0.4200 → $0.7984, manufacturing overhead $0.1677 → $0.8922, SG&A shown as 10% markup = $0.2500, profit as 5% markup = $0.1250 — same $2.4911 price). The customer sources elsewhere, and both sides lose: the supplier loses the business, the customer loses the cost reductions and takes on the risk of a supplier who underquoted out of ignorance.
- **Motion study.** Suppliers determine cycle time motion by motion. Manufacturing engineers are conservative because they face post-production efficiency targets — so review the motion study rather than accepting or rejecting the number.
- **Sequenced vs unsequenced stations.** Different cycle times per station are only legitimate when the processes are independent. If sequenced, both operator and machine are captive to the bottleneck and both are charged at the slowest time (theory of constraints; the book cites Goldratt's *The Goal*).
- **Labor rates are local, not national.** Government averages hide wide variation: a Mexican plant that is the only employer in town pays less; rates rise sharply when competitors open nearby or when scarce skilled labor is required.
- **Custom equipment amortization.** Product-specific lines have no reuse value, so amortize over contracted program-life volume only. Example: $2M investment, 5-year contracted life, 2,000,000 units/yr → $0.20/piece. Do not assume program extensions.
- **Scrap gaps are common and large.** Suppliers often quote plant-wide scrap as % of sales and apply it to everything, or quote launch scrap as if it were production scrap. Push for per-product, cumulative scrap. If automated assembly scrap is 0.5% and manual is 5%, that fact justifies an automation investment decision.
- **Depreciation gaps are often trivially small.** In the example the customer's GAAP method ($0.0211) is *higher* than the supplier's real-capital method ($0.0166) — which is why suppliers usually accept the customer's conservative method.
- **Volume symmetry is the price of the deal.** The customer accepts $5,000/yr minimum SG&A and $1.00/part profit on a 10,000-unit part only in exchange for the supplier applying the same absolute logic on high-volume work: at 10,000,000 units, SG&A drops from a flat 10% ($796,000) to ~2% (~$159,000), and a one-year payback on a $100,000 investment is $0.0100/part ≈ 1.25% profit.

## Mental Models

- **The gap is the agenda.** Every difference between two estimates is either (a) a wrong assumption on one side, or (b) a cost reduction opportunity. Raw material alone produced two ideas: improve the mold layout to cut shot weight, and switch to a more common resin.
- **Transparency reduces risk for both sides.** The customer avoids sourcing a supplier who will discover its loss later; the supplier avoids the "dogs" — low-volume projects that flat-percentage pricing systematically underprices and that customers "have no scruples unloading on unsuspecting victims."
- **Properly priced beats favourably priced.** A supplier holding one overpriced high-volume project and one underpriced low-volume project is one cancellation away from dire straits.
- **Small gaps still deserve a question.** Inbound logistics gapped only $0.0210 vs $0.0101 but the question "does the resin price include delivery? where are the purchased components shipping from?" is still worth asking — "even if they are only fractions of a penny."

## Anti-patterns

- **Refusing to supply a cost breakdown** (or citing "corporate policy") — kills the process outright.
- **Hiding profit and SG&A in manufacturing overhead, purchased parts or raw material** — the modified breakdown in Table 3.1.
- **Accepting the lower number and moving on.** Purchased components came in *below* the customer estimate ($0.42 vs $0.48). The book calls taking the win and moving on "a missed opportunity" — the sub-supplier assumptions should still be reviewed, and the collaborative approach extended down the value chain.
- **Assuming cycle time cuts are savings.** A press running five products 24/6/50 that goes 15 s → 13 s on one of them releases no operator and yields no savings until enough capacity accumulates for a sixth product.
- **Supplier tricks during plant visits** (listed as things customers must watch for and suppliers do use): extra operators at the machine, deliberately slowed machines, rushed tours, handpicked processes, blocked access to accountants/buyers, blocked plant-wide data that would enable P&L reverse-engineering.
- **Running cost-based negotiation untrained.** The book notes companies that abandoned open-book programmes purely for lack of people who could do the job.

## Reference Tables

**Table 3.2/3.5 — Connector cost breakdown, before and after negotiation**

| Bucket | Supplier before | Customer before | Supplier after | Customer after |
|---|---|---|---|---|
| Raw material | $0.0375 (15 g, $2.50/kg) | $0.0260 (13 g, $2.00/kg) | $0.0315 (14 g, $2.25/kg) | $0.0315 |
| Purchased parts | $0.4200 | $0.4800 | $0.4200 | $0.4200 |
| Packaging (in) | $0.0042 | $0.0048 | $0.0042 | $0.0042 |
| Logistics (in) | $0.0210 | $0.0101 | $0.0210 | $0.0090 |
| Material overhead | $0.1500 | $0.0506 | $0.1500 | $0.1355 |
| Direct labor | $0.0430 | $0.0268 | $0.0309 | $0.0296 |
| Indirect labor | $0.0167 | $0.0134 | $0.0167 | $0.0148 |
| Setup | $0.0950 | $0.0019 | $0.0289 | $0.0289 |
| Manufacturing overhead | $0.1677 | $0.1453 | $0.1549 | $0.1693 |
| Depreciation | $0.0166 | $0.0211 | $0.0166 | $0.0211 |
| Scrap | $0.0194 | $0.0156 | $0.0175 | $0.0173 |
| **Total manufacturing cost** | **$0.9911** | **$0.7956** | **$0.8922** | **$0.8811** |
| Corporate overhead (SG&A) | $0.5000 | $0.0796 | $0.5000 | $0.5000 |
| Profit | $1.0000 | $0.0796 | $1.0000 | $1.0000 |
| **Price** | **$2.4911** | **$0.9547** | **$2.3922** | **$2.3811** |
| Packaging (out) | $0.0500 | $0.0191 | $0.0500 | $0.0500 |
| Logistics (out) | $0.0300 | $0.0286 | $0.0300 | $0.0300 |
| **Price w/ pkg & logistics** | **$2.5711** | **$1.0025** | **$2.4722** | **$2.4611** |

**Table 3.3 — Reverse-engineered supplier P&L** (built from plant-visit data: headcount, square footage, capital employed)

| Category | Driver | Quantity | Cost per | Annual |
|---|---|---|---|---|
| Hourly employees | People | 200 | $7,500 | $1,500,000 |
| Salaried employees | People | 40 | $40,000 | $1,600,000 |
| Building rental | Sq ft | 200,000 | $2.00 | $400,000 |
| Building insurance | Sq ft | 200,000 | $0.20 | $40,000 |
| Building depreciation | Years | 30 | $0 | $0 |
| Building utilities | Sq ft | 200,000 | $0.25 | $50,000 |
| Building maintenance | Sq ft | 200,000 | $0.05 | $10,000 |
| Equipment depreciation | Years | 10 | $40,000,000 | $4,000,000 |
| Equipment utilities | Units | 1,000,000 | $0.50 | $500,000 |
| Equipment maintenance | Capital | $40,000,000 | 0.50% | $200,000 |
| Equipment supplies | Capital | $40,000,000 | 0.20% | $80,000 |
| Indirect materials (solder, oils, rags) | Capital | $40,000,000 | 0.25% | $100,000 |
| Perishable tooling | Capital | $40,000,000 | 0.50% | $200,000 |
| Tool room | Capital | $40,000,000 | 0.50% | $100,000 |
| Customer unpaid tooling | Years | 4 | $1,000,000 | $250,000 |
| Supplies | Sq ft | 200,000 | $0.10 | $20,000 |
| Outside services | Capital | $40,000,000 | 0.20% | $80,000 |
| Phones, computers, office supplies | People | 240 | $1,000 | $240,000 |
| Travel & entertainment | People | 240 | $500 | $120,000 |
| Shipping supplies | Capital | $40,000,000 | 0.20% | $80,000 |
| **Total annual plant cost** | | | | **$9,670,000** |
| — Direct labor cost | | | | $1,500,000 |
| — Salary labor cost | | | | $1,600,000 |
| — Equipment depreciation | | | | $4,000,000 |
| — Plant burden | | | | $2,570,000 |
| **Ratio: overhead (excl. equipment depreciation) to direct labor** | | | | **278%** |

Underlying labor build-up: $2.50/hr + $1.25 fringes × 2,000 hours = $7,500 per hourly employee per year.

**Table 3.4 — Overhead rate comparison (testing the quoted burden rate)**

| Line | Value |
|---|---|
| OH rate provided in supplier breakdown | $150.00/hr |
| Annual uptime hours | 4,800 (8 hr/shift × 3 shifts × 5 days × 50 weeks × 80% efficiency) |
| Annual cost per year (with depreciation) | $720,000 |
| Annual depreciation ($1,000,000 equipment ÷ 10 years) | $100,000 |
| Annual cost per year (without depreciation) | $620,000 |
| Plant capacity (or cost) utilized by this product | 5% |
| **Projected plant cost implied by the quoted burden rate** | **$14,400,000** |
| Actual plant cost (Table 3.3) | $9,670,000 |
| **Gap** | **$4,730,000 (49%)** |

**Table 3.6 — Negotiation strategies and cost reduction opportunities** (condensed; every row is: gap driver → how to interrogate it → what reduction it can yield)

| Gap driver | Negotiation strategy | Potential cost reduction |
|---|---|---|
| Material type | Assume the same material per design spec | Cheaper material; relax spec |
| Material price | Request invoices; supply market/internal data | Consolidate spend for volume; change raw material supplier |
| Gross material weight | Ask for weight calculation method; request mold/strip layout samples | More cavities per mold; better mold layout; less offal/gates/risers |
| Scrap | Ask for internal scrap data with Pareto of root causes | Specific scrap-reduction steps; recycle more material |
| Scrap resale value | Request invoices; provide market resale data | Sell to a different scrap buyer |
| Material overhead | Request inventory/warehousing detail; check allocation between products | Eliminate handling; improve ordering frequency |
| Packaging/logistics | Request packaging detail (type, size, density) and value stream map | Switch expendable ↔ returnable per business case; localize sourcing |
| Process flow/layout | Verify with PPAP data and supplier quality; get flow and layout; tour the plant | Optimize process to best in class |
| Machine selection | Challenge supplier to meet best in class | Move to BIC equipment long term |
| Capital investment | Obtain and challenge equipment cost breakdown; show target equipment detail | Buy equipment from outside vendors; low-cost-country vendors |
| Tooling & maintenance | Verify what is amortized in piece price; determine tool life; challenge tooling breakdown | Source tooling in LCC; eliminate design charges on duplicate/replacement tools |
| Cycle time | Obtain and challenge motion study per station; verify in person | Reduce tool changes, increase machining speed, change materials; better mold-space utilization |
| Manning | Challenge process layout and motion study per operator; verify in person | Combine stations; automate where justified; eliminate shifts |
| OEE/efficiency | Get root-cause analysis of downtime; challenge scheduled and unscheduled downtime | Improve uptime (temps during lunches/breaks); reduce scrapped process time |
| Manufacturing overhead | Obtain detailed indirect allocation analysis | Allocate per ABC; optimize electricity; reduce support staff |
| Capacity utilization | Challenge under-utilization; pay only for the utilization your product uses | Improve utilization; add shifts (6-day pattern) |
| SG&A | Provide market data (financial statements) | Allocate SG&A accurately |
| R&D | SG&A should already include core R&D; application-specific R&D belongs in ED&T | Strip application-specific engineering from the R&D quote |
| Profit | Provide market data | Move to best-in-class profit levels |
| Cost of capital | Challenge interest rates; allow only on investment, not expenses | Use actual interest rates, not internal hurdle rates |
| ED&T | Verify amortization assumes correct spend and life; challenge total spend | Reduce ED&T (eliminate tests, fewer resources) |
| Payment terms | Challenge piece-price impact of payment days; verify interest calculation | Shorten payment days; lower working-capital interest |
| Risk | Profit already rewards risk — no separate risk cost | Eliminate risk cost |
| Startup cost | Separate and challenge | Reduce startup cost |
| Outbound logistics & packaging | Verify with a logistics cost model | Optimize; in-source to company provider |

## Worked Example

**Closing the connector gap, bucket by bucket, from $2.5711/$1.0025 to $2.4722/$2.4611.**

- **Raw material** — gap $0.0375 vs $0.0260. Two drivers: gross weight (15 g vs 13 g) and resin price ($2.50 vs $2.00/kg). Supplier cites a preliminary resin flow analysis recommending 30 g shot for a two-cavity mold; customer argues 26 g from similar parts in production and can challenge the proposed mold layout. Settled at **14 g and $2.25/kg = $0.0315 both sides**, plus two new cost reduction ideas (reduce shot weight via mold layout; switch to a more common resin).
- **Purchased components** — supplier $0.42 is *lower* than the customer's $0.48; the team takes the lower number, and the book flags this as a missed chance to push cost engineering down to the sub-suppliers.
- **Packaging (in)** — $0.0042 vs $0.0048, both 1% of material cost; closes automatically once material cost is agreed.
- **Logistics (in)** — $0.0210 (≈4% of material) vs $0.0101 (2%). Worth asking whether resin price excludes delivery and where components ship from; settles at supplier $0.0210 / customer $0.0090.
- **Material overhead** — $0.1500 ($1,500/yr, historical) vs $0.0506 (10% of material, $506/yr). Neither side can prove its number without drivers. The supplier has the upper hand if it can show ABC drivers ("$500 per pallet ordered per month," "$100 per invoice") — which also educates the customer into updating its cost model. Customer moves to **30% of material cost = $0.1355**.
- **Direct labor** — $0.0430 vs $0.0268. Drivers: cycle time (15 s vs 13 s), operators (2 vs 1.5), efficiency (85% vs 90%). Reviewing the motion study, both agree **15 s for assembly and 13 s for over-molding** — legitimate only because the two processes are unsequenced and sit in different departments. Manning revised to **1 / 0.5 operators**. Result: supplier $0.0309, customer $0.0296.
- **Indirect labor** — $0.0167 vs $0.0134; the customer's crude "50% of direct labor" rule lands close to the supplier's machine-allocated $48/hr. Both agree to spend their time elsewhere.
- **Setup** — the biggest process gap: $0.0950 (weekly runs of 200) vs $0.0019 (one annual run of 10,000, inventory cost ignored). Resolution by explicit trade-off: financing 12 months of inventory at $9,911 (10,000 × $0.9911) at 5% costs **$270.47/yr = $0.0270/part**; add the once-a-year setup at $0.0019 and the total is **$0.0289/part**, cheaper than $0.0950 for weekly setups. Both sides adopt **$0.0289** — the customer having conceded that low volume costs *something*, either setups or inventory. *(The book's $270.47 implies an average rather than full outstanding balance; 5% of $9,911 is $495.55.)*
- **Manufacturing overhead** — with cycle times agreed, supplier updates to $0.1549 and customer to $0.1693, closing most of the gap. The book notes this is "probably the easiest manufacturing overhead negotiation that ever took place," and that normally this bucket is the biggest discrepancy — hence the reverse-engineered P&L technique: a quoted $150/hr burden rate over 4,800 annual uptime hours implies a $14,400,000 plant against an actual reverse-engineered $9,670,000, a **49% gap** that reopens the conversation with data.
- **Depreciation** — $0.0166 vs $0.0211; the customer's conservative GAAP method is higher, so the supplier accepts it.
- **Scrap** — $0.0194 vs $0.0156, both 2% of incurred cost; moves with the other buckets to $0.0175/$0.0173.
- **Corporate overhead** — gap $0.4204. The customer's flat-percentage argument ("it averages out") fails because a customer has multiple suppliers, so the underpriced low-volume work concentrates on one victim. The customer accepts **$5,000/yr minimum = $0.50/part**, conditional on the supplier reciprocating on high volume (at 10,000,000 units, ~2% not 10%).
- **Profit** — gap $0.9204. Replaced by payback logic: $10,000 investment recovered in one year over 10,000 units = **$1.00/part**, with the same method applied symmetrically at high volume ($100,000 investment over 10,000,000 units = $0.0100/part ≈ 1.25%).

**Settled price: $2.4722 (supplier) / $2.4611 (customer).** Far above the customer's opening estimate, but built on real assumptions. The stated lesson: the customer's cost models must be revised for low-volume scenarios and pre-agreed with suppliers, so that development-phase estimates predict production reality — and SPARK should have known this before launch, not at it.

**Mock negotiation exercise.** The book supplies a runnable training simulation. Split trainees into a customer team (SPARK) and a supplier team (Mold Experts Inc.) and give each a private brief, then negotiate to a price using the Customer/Supplier Negotiation Tracker (Tables 3.7/3.8: current quote; highest price you will pay / lowest you will sell at; price you would be happy with; price you believe the other side will accept; their first, second and third counter-offers; final agreed price; any other cost reductions agreed).

*Customer brief (SPARK):* connector suppliers are large global corporations and hard to leverage; only a handful can make this part. Mold Experts' first quote is 5% below the next lowest supplier but not enough to justify adding a regional supplier. You must extract a further **30%** to sell the sourcing internally; full program spend is in play, not just year one; prior commercial negotiation failed, so it is on the cost engineer to find real cost reductions; this is the last chance.

*Supplier brief (Mold Experts Inc.):* desperate for the job to grow globally in automotive; mostly low-volume, short-life business, and costs are poorly understood because the controller uses plant-wide rates from last year's budget; investors demand a three-year payback, so gross margins were quoted at **50%** with a floor of **30%**; the cost breakdown could not show margin above **12%**, so the rest is hidden in raw material and overheads; productivity givebacks demanded by SPARK were also built into year-one price and hidden; concede nothing without real data.

## Key Takeaways

1. Make a standardized cost breakdown a condition of quoting — the process is impossible without it.
2. Work the gaps in Pareto order using a waterfall chart; treat each gap as either a wrong assumption or a savings idea.
3. Reverse-engineer the supplier's plant P&L from plant-visit data to test quoted burden rates.
4. Motion studies, mold/strip layouts, PPAP data, invoices and scrap Paretos are the evidence base; opinions are not.
5. Concede volume-appropriate SG&A and profit only in exchange for symmetry at high volume.
6. Trade setup frequency against inventory financing explicitly rather than arguing about lot size.
7. Prepare: gather data, align on what is shared, assign roles, role-play the top gaps, know the leverage, let the buyer/sales rep own purely commercial items, and leave both sides feeling they won.
8. Run mock negotiations before real ones — the book calls this its most useful training tool.

## Connects To

- [ch02-cost-estimating-basics.md](ch02-cost-estimating-basics.md) — the two estimates being reconciled here and the bucket definitions
- [ch04-manufacturing-process-estimating.md](ch04-manufacturing-process-estimating.md) — the process knowledge needed to challenge cycle times, cavitation and scrap
- [ch05-cost-allocation-methods.md](ch05-cost-allocation-methods.md) — why "$150/hr burden" is usually an allocation artifact
- [ch06-cost-optimization-tools.md](ch06-cost-optimization-tools.md) — where the cost reduction ideas generated in negotiation get developed
