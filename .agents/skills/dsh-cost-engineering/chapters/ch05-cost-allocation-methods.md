# Chapter 5 — Cost Allocation Methods

## Core Idea

"The lack of accurate cost allocation method is one of the biggest reasons why companies are unsuccessful." Misallocation produces under- or over-priced products — selling work that loses money, or failing to sell work you could have won. Worse, most companies do not know they have the problem: "If a company does not know that it has a cost allocation problem, it will not seek a solution to that problem."

The chapter ranks six allocation methods from least to most accurate and shows the failure mode of each. Its closing claim: "Cost allocation is the most important building block of product cost estimating."

## Frameworks Introduced

**The allocation accuracy ladder** (worst → best):

1. **Contribution / gross margin %** — one margin percentage on everything
2. **Labor allocation** — overhead as a multiple of labor cost or operator count
3. **Standard hour allocation** — cost per standard machine hour from last year's P&L
4. **Capital allocation** — overhead split by original equipment acquisition cost
5. **Machine rates** — per-machine rates built from actually-attributable costs, with a residual plant burden spread on top
6. **Activity-based costing (ABC)** — activities identified, costs assigned by causality drivers

**The universal failure mode.** Methods 1–4 are all *one ratio derived from last year's P&L*. They are defensible only when the operation is single-product or genuinely homogeneous — "one product or very similar products using the exact same process." The moment product mix, process mix, machine mix or volume mix varies, the ratio misprices systematically and in a self-reinforcing direction.

**The death spiral (standard hour method).** Rate set from last year's P&L → some products underpriced → win them, lose money → total cost rises → next year's rate rises for *all* products → less competitive → lose business → machine utilization falls → rate rises further → plant closes.

## Key Concepts

- **Contribution vs gross margin.** Contribution margin = price − all variable costs. Gross margin = price − all manufacturing cost. Both are historical targets (e.g. "our gross margin was 20%, let's quote above 20%") applied indiscriminately.
- **Why a flat margin % is wrong.** In absolute terms the required margin differs by product: a low-engineering, high-volume product needs a smaller percentage than a highly engineered, customer-intensive, low-volume product. Flat percentages penalize some products and underprice others; over time you win the underpriced ones.
- **Labor allocation.** Plant controller sees $1M labor and $5M overhead in last year's P&L → assume $5 of overhead per $1 of labor forever. Fails whenever labor intensity and overhead intensity decouple. A casting requiring eight operators does not consume more machine overhead than one requiring four on the same line — "the machine and the cost to run it are the same regardless of how many operators are needed."
- **Standard hour allocation.** $20M total operating cost ÷ 500,000 machine hours (100 machines × 100 hr/week × 50 weeks) = $20 per standard hour, used for variance reporting *and* for all future quotes. Accurate only if all 100 machines and all products are alike. A part needing two operators, a more expensive machine, or more energy still gets $20/hr.
- **Capital allocation.** Overhead split in proportion to original acquisition cost, then divided by annual hours to give a machine rate. "The only thing that is the same, in fact, is the original cost of the equipment" — a $500k assembly line and a $500k molding press have entirely different space, headcount, energy and maintenance profiles.
- **Machine rates.** Assign what can genuinely be attributed per machine — space rental, electricity, indirect support, maintenance and supplies, depreciation — then spread the unattributable remainder (plant manager, accountants, HR, IT, office space, parking, utilities, cafeteria, buses, uniform washing) as a crude uplift, e.g. 200% of the directly allocated manufacturing cost. Becoming the most popular method because it offers "decent accuracy with affordable effort." Its residual is still its weakness: if most non-machine resources actually serve products on the $200k machines, the method underestimates those and overestimates the rest.
- **Corporate overhead is usually still bolted on flat.** Sales, finance, purchasing and executive management typically get added as a straight percentage on top, which is inaccurate because excessive re-quotes, engineering changes and "freebies" make customer-service load vary by customer.
- **ABC.** Define the activities that happen in the company — manufacturing processes, but also selling, processing accounting transactions, moving raw material inside the factory — and allocate cost to them with causality drivers, so that little or nothing is left to spread "like peanut butter over toast."
- **Accuracy vs precision (Hicks).** "Being accurate means that you are hitting very close to the bull's eye, while precision means that you are hitting exactly the same spot every time, except that the spot may not be anywhere close to the bull's eye." ABC fell out of favor at some companies because they measured every identifiable activity — excessive effort, little return. Capturing the *key* activities with the *right* drivers is sufficient; failing to capture the right activities or drivers is the real accuracy risk.

## Mental Models

- **Follow cost logically; don't lock into a rule.** "As a cost estimator, one must follow cost logically instead of locking himself/herself into a box of specific rules and methodologies."
- **Don't be bound by past cost.** "Try to predict what the cost will be in the future state that the company seeks, e.g. with optimal machine utilization." Last year's P&L describes last year's utilization, not the utilization the quoted business would create.
- **No one size fits all.** A method that works at one company may be useless at another; common sense adjustment is part of the job.
- **The navigator metaphor closes the chapter.** "As navigators of your company's future, it is up to you to use the proper navigational tools."

## Anti-patterns

- **Quoting a flat SG&A percentage on a transformational deal.** See the Worked Example — the flat 10% was defensible only if SG&A scaled linearly with revenue, which it does not.
- **Reusing a stamping overhead ratio for a new assembly business.** 10:1 overhead-to-labor is plausible for one operator running a large press with heavy capital, space and energy; for table-top assembly the real ratio may be 4:1. The company quoted 21 assembly jobs, lost all of them, and concluded it was "uncompetitive at assembly."
- **Ignoring downtime inside labor cost.** The ECU anecdote: appliance business was low volume, high mix, with operators changing over or idle; treating labor as labor regardless of downtime under-allocated overhead to appliances and over-allocated it to automotive. Management believed appliance was profitable and automotive was not — "the opposite was true."
- **Assuming the flat percentage averages out.** It only averages out if your whole portfolio sits with one supplier or one customer and is balanced between small and large volumes.
- **Measuring every activity under ABC.** Extreme detail is not necessary for a predictive model and is the reason companies abandoned ABC.

## Reference Tables

**Table 5.1 — Capital cost allocation method example** ($10,000,000 of factory overhead, five machines, 5,200 annual hours each)

| Machine | Original acquisition cost | % of total machine cost | Cost allocation per machine | Hourly rate per machine |
|---|---|---|---|---|
| 1 | $200,000 | 10% | $1,000,000 | $192.31 |
| 2 | $200,000 | 10% | $1,000,000 | $192.31 |
| 3 | $500,000 | 25% | $2,500,000 | $480.77 |
| 4 | $500,000 | 25% | $2,500,000 | $480.77 |
| 5 | $600,000 | 30% | $3,000,000 | $576.92 |
| **Total** | **$2,000,000** | **100%** | **$10,000,000** | |

Estimating use: multiply each part's cycle time on a machine by that machine's hourly rate to get overhead cost per part.

**Table 5.2 — Machine rate cost allocation example** (same five-machine factory; 5,200 annual production hours; 10-year depreciation schedule; hourly plant overhead = 200% of all other manufacturing cost per machine)

| Machine | Original cost | Depreciation $/hr | Space rental $/hr | Electricity $/hr | Indirect support $/hr | Maintenance & supplies $/hr | Plant overhead $/hr (200%) | **Total rate $/hr** | Total annual cost |
|---|---|---|---|---|---|---|---|---|---|
| 1 | $200,000 | 3.85 | 22.00 | 20.00 | 16.00 | 20.00 | 163.69 | **245.54** | $1,276,800 |
| 2 | $200,000 | 3.85 | 22.00 | 20.00 | 16.00 | 20.00 | 163.69 | **245.54** | $1,276,800 |
| 3 | $500,000 | 9.62 | 39.00 | 40.00 | 16.00 | 40.00 | 289.23 | **433.85** | $2,256,000 |
| 4 | $500,000 | 9.62 | 39.00 | 40.00 | 16.00 | 40.00 | 289.23 | **433.85** | $2,256,000 |
| 5 | $600,000 | 11.54 | 52.00 | 50.00 | 16.00 | 60.00 | 379.08 | **568.62** | $2,956,800 |
| **Total** | **$2,000,000** | | | | | | | | **$10,022,400** |

Note the shape of the failure: the residual "plant overhead" line is by far the largest component of every rate, and it is allocated by a crude 200% multiplier — so most of the money is still being spread rather than assigned.

**Allocation method selection summary**

| Method | Accurate when | Fails when | Typical symptom of failure |
|---|---|---|---|
| Contribution/gross margin % | Never reliably | Any mix of volume or engineering intensity | Win low-volume "dogs", lose high-volume work |
| Labor ratio | One product, one process | Machines differ; labor intensity varies independently of overhead | Quoting a new process type with the old ratio and losing every bid |
| Standard hour | 100 identical machines, identical products | Any machine or process variation | Rising rate each year; shrinking utilization; death spiral |
| Capital (original cost) | Very consistent, static operations | Equipment of equal cost with unequal space/energy/support needs | Assembly overpriced, machining underpriced |
| Machine rates | Most cost is genuinely attributable to machines | Large unattributable residual concentrated on particular products | Rates dominated by the residual uplift |
| ABC | Key activities and causality drivers identified | Wrong activities or wrong drivers chosen; or over-measurement | Excessive measurement effort with little return |

## Worked Example

**The SG&A percentage that was blocking the company's biggest ever deal** (chapter's first Real-Life Anecdote).

Setting: a Tier 1 supplier of electronic components. Its estimating practice was a flat **10% SG&A** added to piece price on every quote, derived from historical SG&A cost for the company. Observed pattern: the company won some smaller business but was consistently uncompetitive on larger business — despite being "very lean and agile," which should have made it competitive.

The trigger: a quote for a piece of business that would **double the size of the company**.

The insight, from the new quote manager: SG&A does not scale linearly with revenue. Doubling revenue would not double SG&A in absolute terms — the increase would be **about 25%**. Working through it:

- Under the flat rule, SG&A allocated to the new business = 10% of the new revenue, i.e. an amount equal to the company's entire current SG&A base (since the business doubles the company).
- Under the incremental reality, the new business only causes about a **25% increase** in absolute SG&A spend, roughly a quarter of what the flat rule charges it.

Applying the incremental logic, the company was able to **submit a quote several percentage points lower than typical** — and **won the large business**.

Two companion anecdotes in the chapter show the same allocation error in the other direction:

- **Stamper adding assembly.** Historical stamping ratio: $10 of overhead per $1 of labor. Applied to a new assembly business with 10 assemblers, the quote carried $10 overhead per labor dollar. The company lost that bid and the next 20, then concluded it was uncompetitive at assembly. In reality, table-top assembly stations need little space and energy and no heavy capital — the true ratio was closer to **4:1** — so the company had been dramatically overpricing assembly and could have won profitable business.
- **ECU plant serving automotive and appliance.** Both used SMT plus final assembly, so the plant used a single labor-based overhead allocation. But downtime was not counted in labor cost, and the appliance business was low-volume, high-mix, so operators spent much of their time changing over or waiting. Overhead was therefore **under-allocated to appliance and over-allocated to automotive**: appliance parts were underpriced, automotive parts overpriced, and management steered the company believing appliance was highly profitable and automotive was underperforming — the exact reverse of the truth. (Chapter 2's material-overhead anecdote is the same plant seen from the warehousing side: 70% of warehousing served appliance customers who were only 40% of volume.)

## Key Takeaways

1. Allocation accuracy is the foundation of estimating — get it wrong and nothing downstream can save the estimate.
2. Contribution/gross margin percentages are the least accurate method and the most widely used.
3. Any allocation ratio taken from last year's P&L encodes last year's mix; it misprices the moment mix changes.
4. Labor, standard-hour and capital methods all share this defect in different clothing.
5. Machine rates are the practical modern default — decent accuracy for affordable effort — but watch the size of the unattributable residual.
6. ABC is the most accurate; capture the key activities and the right causality drivers, not every measurable activity.
7. Accuracy beats precision.
8. Model the future state (e.g. improved utilization) you are quoting into, not the historical state.

## Connects To

- [ch02-cost-estimating-basics.md](ch02-cost-estimating-basics.md) — the manufacturing overhead and SG&A buckets these methods populate
- [ch03-cost-based-negotiation.md](ch03-cost-based-negotiation.md) — reverse-engineering a supplier P&L to test an allocated burden rate
- [ch04-manufacturing-process-estimating.md](ch04-manufacturing-process-estimating.md) — the process facts that make a machine rate defensible
- [ch01-what-is-cost-engineering.md](ch01-what-is-cost-engineering.md) — where ABC, standard costing and marginal costing sit in the umbrella
