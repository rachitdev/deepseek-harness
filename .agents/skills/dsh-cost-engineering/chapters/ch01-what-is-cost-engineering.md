# Chapter 1 — What Is Cost Engineering?

## Core Idea

Cost engineering (sometimes called *design to cost*) is "the practice of engineering a company's products to meet pre-defined cost requirements." Wikipedia's definition, which the author quotes, is broader: "the engineering practice devoted to the management of project cost, involving such activities as estimating, cost control, cost forecasting, investment appraisal, and risk analysis."

The chapter's argument is that cost engineering is an **umbrella** (Figure 1.1) over twelve named methodologies, each of which addresses only a slice — estimating, control, or optimization. Companies mistake owning one slice for owning the discipline: "Companies often believe they have cost engineering capabilities when in fact they actually have none or, at best, a subset of those capabilities."

## Frameworks Introduced

**The umbrella of cost engineering methodologies** (Figure 1.1) — twelve items, each defined in its own section:

| Methodology | What it actually is | Is it cost engineering? |
|---|---|---|
| Target Costing | Setting cost targets for every component during development; US-origin, made regimented by Japanese carmakers, tied to *kaizen* | Closest relative; some argue identical, but cost engineering is broader |
| Cost Estimating | Estimating the cost of products, purchased and produced, ideally from the "ground up" | Critical element; but excludes optimization |
| Should Costing | Same thing as cost estimating; "should" is a rhetorical device to imply attainability because "estimate" reads as guessing | Identical accuracy limits as estimating |
| Cost Modeling | Deeper analysis anchored on the company's/supplier's P&L; models where cost *should* be assigned via activities and drivers, and predicts the effect of structural change (e.g. utilization 65% → 80%) | Yes; more predictive than point-in-time estimating |
| Activity-Based Costing (ABC) | Formal method: identify activities, assign all costs to activities by causality, then activities to products | Yes; the most accurate allocation method |
| Value Analysis / Value Engineering (VA/VE) | Optimization by function: value = function ÷ cost. VE = before launch; VA = after launch | Yes; a vital part |
| TRIZ | Altshuller's 40 inventive principles derived from patent analysis; brainstorming aid | Idea-generation input |
| Cost & Value Management | Very broad; may mean nothing more than tracking to budget or stream-mapping the value chain | Weak; depends entirely on content |
| Management Accounting | Using accounting information for management decisions rather than pure GAAP history; IMA's tool of choice is ABC | Adjacent |
| Cost Accounting | Setting up a system to track costs as they occur, tied to areas/departments | **No** — accounting tool, not required to design to a cost target |
| Standard Costing | Budgeted cost per standard hour per product/step, tracked as variances | **No** — accounting tool |
| Marginal Costing | Only costs that vary with the decision enter the decision analysis | Useful for incremental decisions; wrong when assumptions change materially |

**Target cost derivation.** Market price − required profit margin = target cost. Trivially easy for a single-BOM-line product (a plastic toy duck); the difficulty is decomposing a $90,000 vehicle target into targets for ~10,000 components.

## Key Concepts

- **Cost modeling vs cost estimating.** A cost estimate uses today's status quo; a cost model adjusts the cost structure for changes in the business. If a quoted business would move factory utilization from 65% to 80%, the model reflects the improved fixed-cost absorption; the estimate probably still assumes 65%. Corollary: garbage in — wrong activities or wrong drivers — makes the model *worse* than the estimate.
- **ABC's real scope.** Companies typically stop applying ABC at the manufacturing process, which is the visible activity. The invisible ones — design development, product launch, parts procurement, transaction accounting, sales and marketing, production scheduling — also cost money and differ between products (marketing a truck ≠ marketing a car).
- **Value = function ÷ cost.** Raise function or lower cost and value rises. The VA/VE purist route to ideas is functional analysis: enumerate every function the product performs, cost each function, then brainstorm cheaper ways to deliver it.
- **TRIZ by example.** "Merging" — combine several systems into one for synergy; e.g. combine the compute power of several computers, or use one processor chip for many functions.
- **Marginal costing example.** 300,000 units, $1,000,000 variable cost, $2,000,000 fixed cost. Adding 10,000 units adds "about $50,000" of variable cost and no fixed cost → $5/unit incremental. Full absorption would say $10/unit ($3,000,000 ÷ 300,000), overstating the true incremental impact. *(Note: $1M/300,000 units implies ~$33,000, not $50,000, for 10,000 units — the book's figure is reproduced as printed.)*

## Mental Models

- **Umbrella, not synonym.** When someone says "we do should costing" or "we run VA/VE workshops," map their claim onto the twelve-cell table above and identify the missing slices — usually optimization if they estimate, and estimating if they optimize.
- **Accounting looks backward; cost engineering looks forward.** Cost accounting and standard costing serve monitoring; they do not tell you what a not-yet-designed part should cost.
- **Early or fatal.** "Waiting to cost optimize products until after design is finalized could be a fatal mistake. There will not be enough time and resources to make any significant changes and the price that a product can bear in the market will not change."

## Anti-patterns

- **Claiming capability from a subset.** Owning cost accounting, or a VA/VE facilitator, and calling it cost engineering.
- **Using historical cost as "estimating."** Referencing past cost is useful data but is not ground-up estimating and cannot price a new design.
- **Treating "should cost" as more accurate than "estimate."** It is the same calculation with the same assumptions and the same error band.
- **Engineering-first, cost-later escalation.** SPARK's sequence: design freeze → discover overrun → order Engineering to redesign (refused on performance grounds) → blame Purchasing → squeeze suppliers → still over target.
- **Assuming purchasing leverage can fix a design problem.** SPARK had 10,000 units/yr and no production history; suppliers had no reason to move.

## Reference Tables

**Table 1.A — SPARK hybrid truck, concept economics (from the chapter's narrative)**

| Item | Value |
|---|---|
| Planned annual capacity/sales | 10,000 hybrid trucks/yr |
| Market price | $100,000 per vehicle |
| Owner-required profit | 10% |
| **Target cost** | **$90,000 per vehicle** |
| Engineering spend to preliminary design | $20,000,000 (100 engineers × $100,000 avg salary × 2 years) |
| BOM cost from supplier RFQs | $115,000 |
| In-house manufacturing + corporate cost | $15,000/vehicle ($150,000,000 ÷ 10,000) |
| **Total estimated vehicle cost** | **$130,000** — $40,000 over target |
| After Purchasing's best negotiation | BOM $110,000 → total still ~$35,000 over target |

**Table 1.B — Marginal vs full-absorption unit cost (p.6)**

| | Marginal | Full absorption |
|---|---|---|
| Base volume | 300,000 units | 300,000 units |
| Variable cost | $1,000,000 | $1,000,000 |
| Fixed cost | $2,000,000 (unchanged) | $2,000,000 |
| Incremental 10,000 units | +~$50,000 variable | — |
| Cost per incremental unit | **$5.00** | **$10.00** ($3,000,000 ÷ 300,000) |

## Worked Example

**SPARK hybrid truck — how a target cost is set and then lost.**

1. *Derive the target.* SPARK's executives believe they can sell their full capacity of 10,000 hybrid trucks per year at $100,000 each. The owners need 10% profit for the desired payback on their investment. $100,000 − 10% = **$90,000 target cost per vehicle**. So far, straightforward.
2. *Fail to cascade.* Nobody sets a cost target for the steering wheel, the tires, the windows — the ~10,000 individual components. SPARK is a startup with a new design, so there is no historical component cost to start from.
3. *Spend two years.* 100 engineers × $100,000 average salary × 2 years = **$20,000,000** in development, producing a preliminary design that meets design and performance requirements.
4. *Discover the gap.* Purchasing sends RFQs to component suppliers and spends about three months collecting data: **BOM = $115,000**. Manufacturing and corporate cost is estimated at $150,000,000 ÷ 10,000 vehicles = **$15,000/vehicle**. Total = **$130,000**, i.e. **$40,000 over target** (44% over).
5. *Push on Engineering.* Finance challenges each component engineering group. Gasoline powertrain refuses: must have a four-cylinder, 250 hp engine. Electric powertrain refuses: cannot compromise battery size. Chassis, body and brakes give the same answer. Redesign is off the table.
6. *Push on Purchasing.* With no vehicle yet produced and only 10,000 parts of volume — "not a lot of business for most suppliers" — the team lacks leverage and gets the BOM only down to **$110,000**. The vehicle is still **$35,000 over target**.
7. *Outcome.* "This is a very common way for things to unfold. The result is often an unsuccessful product launch and, in the worst case, the company's bankruptcy."

The counterfactual the book states: the engineering team should have worked up front with the other functions to design each component against a specific performance **and cost** target. Chapter 7 reruns this same case with the process applied and lands at a profitable design.

## Key Takeaways

1. Cost engineering = engineering products to pre-defined cost requirements; it subsumes twelve narrower methodologies and is not equal to any of them.
2. Cost accounting and standard costing are monitoring tools, not cost engineering tools.
3. Cost modeling differs from cost estimating by predicting the effect of structural change (utilization, scale), not just recording today's cost.
4. Target cost = market price − required margin; the discipline lives in the cascade to components.
5. Implementation is cross-functional (sales, finance, engineering, operations, purchasing) and spans the whole development process.
6. Late cost optimization cannot work: design is frozen, market price is fixed, so only profit can absorb the miss.

## Connects To

- [ch02-cost-estimating-basics.md](ch02-cost-estimating-basics.md) — the ground-up estimating method this chapter says is critical, bucket by bucket
- [ch05-cost-allocation-methods.md](ch05-cost-allocation-methods.md) — ABC and the alternatives introduced here, ranked and worked
- [ch06-cost-optimization-tools.md](ch06-cost-optimization-tools.md) — VA/VE, functional analysis and TRIZ in operating detail
- [ch07-optimal-cost-engineering-process.md](ch07-optimal-cost-engineering-process.md) — the SPARK case rerun correctly, gate by gate
