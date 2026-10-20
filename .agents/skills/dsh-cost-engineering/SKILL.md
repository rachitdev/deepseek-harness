---
name: dsh-cost-engineering
description: "Cost Engineering: A Practical Method for Sustainable Profit Generation in Manufacturing by Chris Domanski (CRC Press / Taylor & Francis, 2020) — a ground-up method for should-costing manufactured parts, allocating overhead, and negotiating on cost data. Use when estimating piece price or should cost, building a cost breakdown by bucket (raw material, purchased components, material overhead, direct/indirect labor, setup, manufacturing overhead, depreciation, scrap, SG&A, profit, packaging, logistics), setting target costs, running a cost-based (open-book) supplier negotiation or gap analysis, choosing a cost allocation method (labor, standard hour, capital, machine rate, ABC), estimating molding, stamping, casting, forging, machining, sintering, plating, painting, or SMT processes, building a multi-year business case with IRR and payback, or standing up a cost engineering organization and product-development cost gates."
---

# Cost Engineering — A Practical Method for Sustainable Profit Generation in Manufacturing

**Author**: Chris Domanski (25+ years in automotive cost engineering, purchasing, finance and engineering; Ford, Continental, TRW, ZF, Methode Electronics, Nexteer)
**Publisher**: CRC Press / Taylor & Francis Group, 2020 (ISBN-13 978-0-367-44083-1)
**Pages**: 156 (front matter + 147 numbered pages)
**Chapters**: 9
**Generated**: 2026-08-23

## How to Use This Skill

- **No args** — read this page for the whole method: what cost engineering is, the fifteen cost buckets, the estimating and negotiation loop, allocation methods, and where each tool belongs in the product development cycle.
- **With a topic** — jump via the Topic Index below (e.g. "should cost a molded part", "machine rate vs ABC", "supplier plant visit", "productivity givebacks").
- **With a chapter** — open `chapters/chNN-slug.md` for study depth: frameworks, the chapter's own tables and numbers, and a reproduced worked example.
- **Browse** — `cheatsheet.md` for decision tables, default percentages and process cost ranges; `patterns.md` for the reusable moves; `glossary.md` for the book's vocabulary.

## Core Frameworks & Mental Models

**1. Cost engineering is the umbrella, not one of the tools.** Target costing, cost estimating, should costing, cost modeling, ABC, VA/VE, TRIZ, cost/value management, management accounting, cost accounting, standard costing and marginal costing are each *portions* of it (Figure 1.1). Companies routinely believe they "have cost engineering" when they own one subset. Cost engineering = engineering a product to a pre-defined cost requirement, across the full life cycle and across every function.

**2. Target cost is derived, not negotiated.** Market price − required profit margin = target cost. SPARK's fictional hybrid truck: $100,000 market price × 10% required profit → $90,000 target cost. The hard part is not the vehicle target, it is cascading it to the 10,000 components. Failure to cascade is what kills programs.

**3. Estimate from the ground up, in buckets, twice.** The book's spine is a 15-bucket cost breakdown built bottom-up: raw material → purchased components → packaging (in) → logistics (in) → material overhead → direct labor → indirect labor → setup → manufacturing overhead → depreciation → scrap → corporate overhead (SG&A) → profit → packaging (out) → logistics (out); with tooling and fixtures handled outside piece price. Both sides of the table build the same structure — the supplier from internal data, the customer from assumptions — and the *difference between the two* is the negotiation agenda.

**4. Estimates are estimates, not "exactomates".** Best achievable accuracy is 5%–10% of actual. Concept-stage accuracy of ±10% is enough to decide whether a design is a $1, $10 or $100 part; by later development gates it must tighten to 5% or better.

**5. Percentage markups destroy companies.** Applying a flat SG&A % and a flat profit % to every project systematically underprices low-volume work and overprices high-volume work. Result: you win the "dogs" and lose the good business. Replace flat percentages with (a) a minimum absolute cost per project for SG&A (e.g. $5,000/yr) and (b) ROI/payback-driven profit. Also: markup (X × 1.10) is not margin (X ÷ 0.90) — confusing them is a common, real error the book calls out in its own worked example.

**6. Allocation accuracy beats every other refinement.** Ranked worst to best: contribution/gross margin % → labor ratio → standard hour → capital (original equipment cost) → machine rates → activity-based costing. Each shortcut is a ratio derived from last year's P&L, which is only valid if the product mix and process mix never change. ABC is most accurate; "accuracy is much more important than precision" (Hicks) — capture the key activities and the right causality drivers rather than measuring everything.

**7. Depreciation is an accounting artifact; capital investment is the real cost.** Past investment is sunk. What matters is cash that must be spent to maintain or replace capability. A supplier charging GAAP depreciation on fully paid-off assets is collecting hidden profit; a supplier assuming zero equipment cost will win business and then be unable to fund replacement.

**8. Cycle-time savings are not cost savings unless a resource actually leaves.** Cutting 15 s to 13 s saves nothing if no operator, shift or machine is released. Savings only materialize when freed capacity absorbs another product. Same logic for lean events.

**9. The bottleneck sets the cost.** If two stations are sequenced, both are charged at the slowest cycle time — operator and machine are "captive" (theory of constraints). Only unsequenced stations may carry different cycle times.

**10. Scrap is cumulative.** Scrap accumulates through the process sequence and the material content of a scrapped part differs at each step. A part scrapped after three paint layers carries resin *and* three coats. Model scrap step by step (Table 2.1), not as one plant-wide percentage of revenue.

**11. Negotiate assumptions, not prices.** Cost-based negotiation is the nine-step gap-closure process: mandatory standardized cost breakdown → compare bucket by bucket → waterfall/Pareto the gaps → attack the biggest drivers with data → convert disagreements into cost reduction ideas. "Collect & compare" (take the lowest quote) transfers risk instead of removing cost.

**12. Cost influence collapses over time.** Impact on final cost is largest at concept and approaches zero after tooling is cut. Value engineering (before design freeze) is cheap; value analysis (after production start) is expensive and often needs customer approval and shared savings. Cost reduction workshops held three months before SOP are theatre.

**13. Distortions accumulate.** Annual productivity givebacks and upfront lump-sum payments do not lower structural cost — suppliers bake both into year-one price (a lump sum is effectively a high-interest loan from the supplier). The book recommends abolishing both and sourcing at best price up front.

**14. Cost engineering must be an organization, not a person.** A lone cost optimizer inside Engineering or Purchasing has no authority, no aligned objectives, and disappears with the next reorganization. The benchmark structure: a head of cost engineering reporting to the CFO with matrixed experts embedded in engineering, purchasing, operations and finance. The most important hiring trait is **curiosity**, then the ability to be a change agent.

## Chapter Index

| # | Chapter | File | What it gives you |
|---|---------|------|-------------------|
| 1 | What Is Cost Engineering? | [chapters/ch01-what-is-cost-engineering.md](chapters/ch01-what-is-cost-engineering.md) | The twelve methodologies under the umbrella; target cost derivation; SPARK failure case |
| 2 | Cost Estimating Basics | [chapters/ch02-cost-estimating-basics.md](chapters/ch02-cost-estimating-basics.md) | The 15 cost buckets defined; full connector should-cost, supplier vs customer; business case with IRR/payback; giveback and lump-sum distortions; quoting and change control |
| 3 | Cost-Based Negotiation | [chapters/ch03-cost-based-negotiation.md](chapters/ch03-cost-based-negotiation.md) | Gap closure bucket by bucket; reverse-engineered supplier P&L; negotiation strategy table; plant visit tips; mock negotiation exercise |
| 4 | Cost Estimating for Various Manufacturing Processes | [chapters/ch04-manufacturing-process-estimating.md](chapters/ch04-manufacturing-process-estimating.md) | Cost-driver tables and dollar ranges for sand casting, die casting, forging, machining, PM sintering, stamping, plating, molding, painting, SMT, assembly, additive |
| 5 | Cost Allocation Methods | [chapters/ch05-cost-allocation-methods.md](chapters/ch05-cost-allocation-methods.md) | Six allocation methods ranked; capital and machine-rate worked tables; ABC |
| 6 | Cost Optimization Tools | [chapters/ch06-cost-optimization-tools.md](chapters/ch06-cost-optimization-tools.md) | Prioritise by spend; lean, VSM, benchmarking/teardown, functional analysis, TRIZ, cost reduction workshop |
| 7 | Optimal Cost Engineering Process | [chapters/ch07-optimal-cost-engineering-process.md](chapters/ch07-optimal-cost-engineering-process.md) | Gate-by-gate cost engineering in product development; the SPARK turnaround with its four gate tables |
| 8 | Cost Engineering Organization | [chapters/ch08-cost-engineering-organization.md](chapters/ch08-cost-engineering-organization.md) | Report-to-CFO matrix structure; culture; capability development; the cost engineer interview; software vendors |
| 9 | Cost Engineering in Conclusion | [chapters/ch09-cost-engineering-in-conclusion.md](chapters/ch09-cost-engineering-in-conclusion.md) | Cost engineering as a core function; 10:1 return claim; how to measure it |

## Topic Index

| Topic | Chapters |
|---|---|
| Target costing, cost target cascade | 1, 7 |
| Should costing / clean-sheet estimating | 1, 2, 4 |
| Cost buckets and their definitions | 2 |
| Gross vs net weight, offal, shot weight | 2, 4 |
| Direct vs indirect labor, efficiency vs OEE vs utilization | 2, 3 |
| Setup cost and lot sizing vs inventory holding | 2, 3 |
| Depreciation vs real capital investment | 2, 3 |
| Scrap, cumulative scrap modelling | 2, 3, 4 |
| SG&A and profit allocation, ROI/payback pricing | 2, 3, 5 |
| Business case, IRR, inflation, productivity givebacks | 2 |
| Quoting process, sandbagging, change management | 2, 7 |
| Cost-based negotiation, gap analysis, waterfall | 3 |
| Reverse-engineering a supplier P&L | 3 |
| Supplier plant visits, negotiation tactics, mock training | 3 |
| Process cost drivers and rate ranges | 4 |
| Machine rates, ABC, capital allocation | 5 |
| Lean, VSM, benchmarking, functional analysis, TRIZ | 6 |
| Cost gates in product development, VE vs VA | 7 |
| Organization design, culture, careers, software | 8, 9 |

## Supporting Files

- [glossary.md](glossary.md) — the book's own glossary plus terms defined in the chapter text
- [patterns.md](patterns.md) — reusable moves as When / How / Trade-offs
- [cheatsheet.md](cheatsheet.md) — decision tables, default percentages, process cost ranges, tells and smells

## Scope & Limits

- **Edition/date**: 2020 first edition. Dollar figures, labor rates, resin/metal prices and tariff commentary (the 25% China tariffs of the Trump administration are cited as current) are 2019–2020 vintage and must be re-indexed before use. The author states repeatedly that the process-cost ranges in Chapter 4 are "a rough guide only" with many outliers.
- **Domain**: discrete manufacturing, heavily automotive Tier-1/OEM. Piece-price and program-life economics. Not construction/project cost engineering (AACE-style), not software costing, not services.
- **Not covered by the book**: no code, no software implementation, no spreadsheet formulas beyond the tables reproduced here; "speeds and feeds" calculation is explicitly out of scope; additive manufacturing cost implications are explicitly declined as too fast-moving; electronic component should-costing is acknowledged as not practised in that industry (price/volume curves dominate).
- **No AACE cost-estimate classes.** The brief asked for "estimating-class definitions and accuracy ranges." This book does **not** use the AACE International Class 1–5 estimate classification. Its own typology of estimate types is: *point in time* vs *business case*, plus optional *best-in-class (BIC)*, *supplier-specific*, and *regional* scenarios (Ch. 3). Its stated accuracy expectations are: best achievable 5%–10% of actual; ±10% acceptable at concept; 5% or better required at later development checkpoints. Nothing else in the book defines estimate classes, and none has been invented here.
- **No published index tables.** The book contains no commodity/labor escalation index tables. Its escalation handling is the business case in Table 2.4–2.6, which applies flat assumption rates (raw material +3%/yr, purchased parts −2%/yr, direct labor +3%/yr, manufacturing overhead +3%/yr, price −3%/yr giveback, 9% borrowing rate, 30% tax rate, 10-year depreciation). Those are the only "index" numbers available and are reproduced as such.
- **Extraction quality**: the PDF has a usable text layer throughout; all nine chapters, the glossary, the recommended-literature list and the index extracted cleanly. Figures are images and did not extract — Figure 1.1 (methodology umbrella), 2.1 (cost category flow), 2.5–2.10 (utilization and distortion charts), 3.1–3.4 (collaboration, nine-step gap process, waterfall, bottleneck), 5.1 (ABC diagram), 6.1 (VSM), 6.2 (TRIZ 40 principles), 7.1–7.2 (cost gates, cost impact curve), 8.1 (org chart) are referenced by the text but their content is only available as described in prose. Numeric tables extracted well; OCR noise occasionally corrupts a currency symbol (`S` for `$`) or a digit, and those spots are flagged inline where they affect a calculation.
- **Arithmetic inconsistencies in the source** (reproduced as printed, flagged where found): the indirect-labor component list on p.25 sums to $38/hr but the book uses $48/hr; the assembly depreciation per unit is printed as $0.0019 where the stated inputs give ≈$0.0012; the marginal-costing example on p.6 states ~$50,000 incremental variable cost where the stated $1,000,000/300,000 units implies ~$33,000; the setup inventory-financing interest of $270.47 on $9,911 at 5% implies an average rather than full balance. These are the book's numbers, not corrections.
