# Cheatsheet — Cost Engineering (Domanski, 2020)

All figures are the book's own, 2019–2020 vintage. Chapter 4's ranges are explicitly "a rough guide only" with many outliers.

---

## 1. The cost breakdown template

| # | Bucket | Typical basis | Default starting assumption (customer side, no internal data) |
|---|---|---|---|
| 1 | Raw material | Gross weight × price per unit weight | Gross = net + discarded; price from indexes/suppliers, adjusted for annual volume purchased |
| 2 | Purchased components | Sub-supplier quotes; else historical or cost model | Validate historical data against a ground-up estimate eventually |
| 3 | Packaging (inbound) | % of purchased component cost | **1%** of material cost |
| 4 | Logistics (inbound) | Freight quote + duties | **2%–5%** of material cost (2% = fully localized supply base) |
| 5 | Material overhead | Actual warehousing + handling + procurement staff | **5%** of total material cost; **10%+** if disproportionate storage/shelf time/procurement effort |
| 6 | Direct labor | (cycle time ÷ 3600) × operators × fully fringed rate × inefficiency factor | Fringed rate = wage + benefits; inefficiency 1.15 (85% efficiency) |
| 7 | Indirect labor | Allocated headcount ÷ parts per hour | **50%** of direct labor cost |
| 8 | Setup | Setup cost ÷ parts per run | Compare weekly-run setup cost vs annual run + inventory financing |
| 9 | Manufacturing overhead | (cycle time ÷ 3600) × machine rate × inefficiency | ABC-derived machine rates preferred; original-equipment-cost rates are a fallback |
| 10 | Depreciation | Real future capital ÷ years ÷ annual hours ÷ machines, + maintenance | Customer fallback: original cost ÷ GAAP years ÷ annual hours |
| 11 | Scrap | % of incurred cost, cumulative per step | **2%** of incurred cost as a generic start; use process ranges below |
| 12 | Corporate overhead (SG&A) | Minimum absolute support cost ÷ volume | Flat % is the anti-pattern; minimum ≈ **$5,000/project/yr** in the book's example |
| 13 | Profit | Investment ÷ payback ÷ volume | ROI/payback, not flat % |
| 14 | Packaging (outbound) | Cost per pallet ÷ parts per pallet | **2%** of price if no data |
| 15 | Logistics (outbound) | Shipping quote | **3%** of price if no data |
| — | Tooling and fixtures | Design + material/components + manufacture, per hour | Not in piece price; verify tool life ≥ program life |

**Formulas to keep straight**
- Margin = X ÷ (1 − rate). Markup = X × (1 + rate). They are not the same; confusing them is the book's named common mistake.
- Direct labor per part = (cycle time in seconds ÷ 3600) × operators × fully fringed hourly rate × inefficiency factor
- Machine-based cost per part = (cycle time ÷ 3600) × hourly rate × inefficiency factor
- Cost per part (cavity processes) = cost per mold ÷ cavities per mold
- Target cost = market price − required profit
- Setup per part = total setup cost ÷ parts per run
- Inventory financing per part = (annual inventory value × interest rate) ÷ annual volume

---

## 2. Default percentages and rates

| Item | Default | Range / notes |
|---|---|---|
| Inbound packaging | 1% of material cost | |
| Inbound logistics | 2% of material cost | 2%–5%; 2% = localized base |
| Material overhead | 5% of material cost | up to 10%+ for slow-moving or hard-to-procure parts; 30% used after negotiation on the low-volume connector |
| Indirect labor | 50% of direct labor | crude but landed close to the ABC number in the book's example |
| Scrap (generic) | 2% of incurred cost | see process table below |
| Outbound packaging | 2% of price | |
| Outbound logistics | 3% of price | |
| Inefficiency factor | 15% (efficiency 85%) | applied as ×1.15 |
| Fringe benefits | +50% on wage in the Mexico example | $3.00 wage + $1.50 fringes = $4.50 fully fringed |
| Estimate accuracy | 5%–10% of actual, best achievable | ±10% acceptable at concept; ≤5% required at later gates |
| Business case escalation | +3% raw material, −2% purchased parts, +3% direct labor, +3% mfg overhead, −3% price | book's Table 2.4–2.6 assumption set |
| Borrowing interest rate | 9% | business case |
| Tax rate | 30% | business case |
| Depreciation schedule | 10 years (equipment) | 8 years used for assembly stations; 30 years building |
| Annual hours per machine | 5,000–5,200 | 4,800 uptime hours used in the reverse-engineered P&L (8 hr × 3 shifts × 5 days × 50 weeks × 80% efficiency) |
| Machine-rate residual uplift | 200% of directly allocated cost | crude; the weakness of the machine-rate method |

---

## 3. Process cost quick reference (Chapter 4)

| Process | Raw material | Scrap | Process cost | Cycle time | Tooling |
|---|---|---|---|---|---|
| Sand casting | $0.15–$0.25/lb (50/50 pig iron + scrap); sand ~$0.01/lb of iron | 5% oxidation; 2%–5% process | Melting $0.05–$0.10/lb; casting $2.50–$6.00 per mold; cores $0.40–$1.50 each | 40–60 s per mold | $100–$300k |
| Die casting | $0.50–$1.50/lb Al; $3.00–$4.50/lb Mg | 3%–5% dross; 2%–3% process | Melting ~$0.05/lb; casting $1.50–$4.50 per mold | 35–65 s per mold | $100–$300k |
| Forging | $0.30–$1.00/lb steel | 20%–30% of raw material (flash + oxidation); 3%–5% process | $50–$900/hr by press size (100–16,500 ton) | 3–25 s per part | $100–$300k |
| Machining | material-dependent; cost gross less removed | 2%–3% | machine- and speeds-and-feeds-dependent | — | perishable tools amortized in piece price |
| PM sintering | $0.60–$1.50/lb granules | 2%–3% (no offal) | Pressing $0.10–$0.50; sintering ~$0.05; sizing ~$0.05 per part | 5–7 s press, ~5 s sinter | $20–$60k |
| Stamping | $0.25–$0.50/lb steel; $0.50–$1.50/lb Al; $2.50–$4.50/lb copper | 2%–3% process; **offal 10%–70%**; scrap resells at 25%–80% | $0.01–$0.30 per part (20–1,500 ton presses) | ≤2 s | $40–$200k |
| Plating | zinc $0.75–$1.50/lb; nickel $5.00–$7.50/lb; chrome $1.00–$1.50/lb; silver $15–$20/oz; gold $1,000–$2,000/oz | 2%–3% (no plating-material loss) | $0.005–$0.50 per part; equipment $0.5–$3M + ~$1M filtration/waste | 0.5 s coil → 15 s rack | $10–$20k racks |
| Molding | $1.50–$2.50/lb resin | 0.5%–2% bad parts; **gates/runners 10%–60% of shot weight**; regrind or resell at ~10% | $0.05–$1.00 per part (40–600 ton presses, $100k–$1M) | 15–65 s per mold | $40–$200k |
| Painting | base coat $40–$60/gal; paint $60–$150/gal; top coat ~$70/gal | **5%–30% cumulative**, plus 10%–30% overspray | $30–$200/hr or $0.02–$2.00 per part; pad print ~$0.02; laser etch $0.02–$0.10 | 15–60 s per part | $20–$100k fixtures |
| SMT | solder paste ~$0.01/sq in; PCB $0.05–$0.12/sq in | 0.5%–2% process; 5%–20% of PCB panel | $0.005–$0.02 per component placement; line $1.5–$2.5M | ≤0.5 s per placement (200 components = 15–30 s) | $20–$60k fixtures |
| Custom assembly | — | — | thousands to millions in investment; 2 to hundreds of steps | seconds to hours | product-specific |
| Electronic components | — | — | priced on historical data and price/volume curves, not should cost | — | — |
| Additive manufacturing | — | — | explicitly not costed by the book | — | none (CAD change replaces retooling) |

**Operator manning rules of thumb**
| Process | Operators |
|---|---|
| Sand casting furnace | 2 per furnace area; ladle crane operator |
| Sand casting line (DISAMATIC-type) | 4–10 depending on part design; +1 for manual core insertion (robotizable) |
| Die casting melting | 1 per 2–3 furnaces |
| Die casting press | 1 per press, sometimes 1 per 2 presses; robot extracts parts |
| Forging press | 1 per press |
| PM pressing | 1 per 1–2 presses |
| Stamping | 1–2 per press, or robots |
| Rack plating | up to 4 (manual hang/un-hang) |
| Molding, drop-to-bucket | 1 per 4–6 presses |
| Molding, insert/over-mold | 1 operator or robot per press |
| Painting line | 2–8 depending on design and line speed |
| SMT line | 2–3 (mainly replacing tape reels) |

---

## 4. Allocation method selection matrix

| Method | Use when | Never use when | Failure signature |
|---|---|---|---|
| Contribution / gross margin % | (effectively never) | Any mix of volume or engineering intensity | Winning low-volume "dogs", losing high-volume work |
| Labor ratio | One product, one process, static | Machines differ in cost, space or energy; labor intensity varies independently of overhead | New process type quoted with the old ratio and losing every bid |
| Standard hour | 100 identical machines and identical products | Any machine or process variation | Rate rises every year; utilization falls; death spiral to plant closure |
| Capital (original acquisition cost) | Very consistent, static operations | Equal-cost machines with unequal support needs (assembly line vs molding press) | Assembly overpriced, capital-intensive processes underpriced |
| Machine rates | Most cost is genuinely attributable to machines | Large unattributable residual concentrated on particular products | Residual uplift is the biggest line in every rate |
| ABC | Key activities and causality drivers identifiable | You would need to measure every activity to feel safe | Measurement effort exceeds return; or the wrong activities/drivers chosen |

**The one diagnostic question**: has the product mix, process mix, machine mix or volume mix changed since the ratio was derived from last year's P&L? If yes, the number is wrong — and wrong in a self-reinforcing direction.

---

## 5. Tells and smells

| Symptom | Likely diagnosis | Fix |
|---|---|---|
| You win low-volume, low-margin work and lose high-volume work | Flat SG&A/profit percentage allocation | Minimum absolute SG&A per project; ROI/payback profit (P7) |
| You lose every bid in a process type you recently added | Overhead ratio imported from a different process | Rebuild the ratio for that process (stamping 10:1 vs assembly 4:1) |
| Management believes one product line is profitable and another is not, and operations disagrees | Overhead allocated on labor without counting downtime, or warehousing not traced to the customer consuming it | ABC drivers per activity |
| Standard-hour rate rises every year while volume falls | Standard hour death spiral | Move to machine rates or ABC; model future-state utilization |
| Supplier refuses a cost breakdown, or cites corporate policy | Profit and SG&A are outside what the customer will accept | Require breakdowns as a quoting condition; build a pre-agreed cost model |
| Manufacturing overhead is the biggest gap in the breakdown | Profit/SG&A hidden in overhead | Reverse-engineer the plant P&L (Table 3.3/3.4) |
| Quoted burden rate implies a plant far larger than the one you toured | Overhead rate inflated | Present the implied vs actual plant cost gap |
| Extra operators at machines and slow cycle times during your visit | Plant-visit tactic | Verify with motion study, PPAP data, and repeat/unannounced observation |
| Supplier will not share plant size or headcount | Blocking P&L reverse-engineering | Make the data a pre-visit form requirement |
| A cost reduction workshop is scheduled 3 months before SOP | The gate process is missing | Move cost gates to concept and every checkpoint (P25) |
| Cost is discovered only after tools are cut | Tooling started on initial designs; conservative redesigns ignored cost | Concept-stage estimate; gate reviews; do not release tooling until profit is credible |
| Engineering hands the overrun to Purchasing after quotes arrive | No independent estimate to separate over-design from sandbagging | Two-column estimate vs quote with gaps to both (P26) |
| First quotes look uniformly conservative | Sandbagging under design/relationship uncertainty | Collaborative gap analysis, not pressure |
| Reported annual savings, but total cost structure is flat | Productivity givebacks and lump sums pre-loaded into new-programme prices | Abolish both; source at best price up front |
| Lean savings never show in the P&L | Cycle time reduced but no resource released | Only count savings when an operator, shift or machine is released or freed capacity is filled |
| Your factory is lean but supplier prices are creeping | Inventory pushed upstream | Extend lean/kanban across the chain |
| Teardown workshop generates ideas that raise cost | Components were not costed before the workshop | Cost every component first (Table 6.1 format) |
| Cost optimization dies when a manager leaves | Person-dependent capability | CFO-reporting cost engineering organization with matrix embeds (P30) |
| Cost engineers leave or cannot be recruited | No career path; treated as a dead-end specialism | Ladder to VP/chief executive; treat as candidates for other functions |
| Functions fight over credit for savings | Attribution-based measurement | Measure at company profit 2–3 years out; credit the cross-functional method |

---

## 6. Process templates

**Nine-step gap closure (cost-based negotiation)**
1. Require a standardized supplier cost breakdown as a quoting condition
2. Build your own ground-up estimate with the same bucket structure
3. Lay the two side by side; compute the gap per bucket
4. Chart as a waterfall; Pareto the gaps
5. Interrogate the largest gaps with evidence (motion study, mold/strip layout, invoices, scrap Pareto, PPAP, plant tour)
6. Where neither side can prove its number, reverse-engineer the plant P&L
7. Convert every disagreement into a recorded cost reduction idea with an owner
8. Agree the assumptions; re-price
9. Update the pre-agreed cost model so the next part starts closer

**Cost engineering gates (product development)**
| Gate | Activity | Accuracy |
|---|---|---|
| Concept design | Estimate the sketch; compare to component targets; go/no-go or change direction | ~10% |
| Between gates | Value engineering: functional analysis, TRIZ, benchmarking | — |
| Development checkpoints | Re-estimate every component, material, capital item and tool; gap plan with owners | ≤5% |
| Quote evaluation | Own estimate vs supplier/in-house quote; gap analysis per bucket; separate over-design from sandbagging; supplier-inclusive workshop | — |
| Final design | Confirm profit before releasing production tooling | — |
| Launch and production | Price every design change; change management control via ERP/PLM | — |
| Post-SOP | Value analysis; benchmarking while the launch is fresh; start the refresh concept immediately | — |

**Quoting process**
1. Align objectives across sales, purchasing, manufacturing, engineering and finance (revenue-only sales objectives and sandbagging both break the quote)
2. Define a walk-away price; let sales fight for anything above it
3. Time the process to the customer's deadline, not to internal convenience
4. Sequence: engineering completes design detail → manufacturing defines process and investment → sub-suppliers quote (or cost models simulate them) → finance and sales assemble the business case → executive review
5. Appoint a quoting manager to keep the team on track, consolidate data, develop the business case and hold profit integrity
6. Use software as the data hub for transparency and documentation
7. After award, run change management control with equal discipline

**Cost reduction workshop**
Sponsor and mandate → dedicated attendees released from day jobs → cross-functional product knowledge plus at least one deliberately ignorant participant (and suppliers in later rounds) → challenging targets set in advance → skilled facilitator with a defined process → tools chosen to fit the problem (teardown, functional analysis, TRIZ, VSM) → proper facilities → **3–5 days** → idea funnel with feasibility screen, priority screen and named implementation owners.

*The SPARK funnel, for calibration*: 350 ideas generated → 210 feasible → 75 prioritized → 35 implemented.

---

## 7. Reference numbers from the book's worked cases

| Case | Number |
|---|---|
| SPARK target cost | $100,000 price × 10% profit → $90,000 |
| SPARK concept overrun | $130,000 vs $90,000 (45% over) |
| Connector supplier price | $2.5711 (mfg cost $0.9911 + SG&A $0.50 + profit $1.00 + pkg/logistics $0.08) |
| Connector customer estimate | $1.0025 (mfg cost $0.7956 + 10% + 10% + 2%/3% of price) |
| Connector settled price | $2.4722 supplier / $2.4611 customer |
| Business case prices | $3.00 (1-yr payback, IRR 91%) / $2.32 (2-yr, 36%) / $2.15 (20% IRR hurdle, 3-yr) |
| Reverse-engineered plant | $9,670,000 actual vs $14,400,000 implied by a $150/hr quoted rate — 49% gap |
| Overhead-to-direct-labor ratio (that plant, excl. equipment depreciation) | 278% |
| Setup trade-off | $0.0950/part weekly setups vs $0.0289/part annual run + 5% inventory financing |
| Cost engineering investment case | $10B company → ~$10M/yr, ~70 people → $100M+ profit improvement (~10:1) |
