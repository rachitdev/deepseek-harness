# Patterns — Cost Engineering (Domanski, 2020)

The book's reusable moves, grouped by the part of the discipline they belong to. Each is **When / How / Trade-offs**.

---

## Part 1 — Estimating

### P1. Build the estimate bucket by bucket, from the ground up
**When** — Any new part, any quote evaluation, any negotiation preparation. Always, in preference to referencing historical cost alone.
**How** — Work the fifteen buckets in flow order: raw material → purchased components → packaging (in) → logistics (in) → material overhead → direct labor → indirect labor → setup → manufacturing overhead → depreciation → scrap → SG&A → profit → packaging (out) → logistics (out). Handle tooling and fixtures separately, outside piece price. Record the *assumption* next to every number, not just the number.
**Trade-offs** — Slower than a historical lookup and demands process knowledge you may not have. But without the assumptions written down there is nothing to negotiate, and the estimate cannot be updated as the design matures. Best achievable accuracy is still only 5%–10%.

### P2. Cost the gross weight, credit recovery only when real
**When** — Any process with material loss: molding (gates/runners 10%–60% of shot weight), stamping (offal up to 70%), forging (flash up to 30%), casting (oxidation 2%–7%), machining (removed material).
**How** — Gross weight = net weight from the drawing + everything discarded. For molding, gross × cavities = shot weight. Then apply a credit only if regrind is actually permitted and practised, or if scrap resale is actually invoiced (stamping scrap resells at 25%–80% of original value; molding regrind at ~10%).
**Trade-offs** — Assuming recovery you cannot prove understates cost; ignoring recovery that is happening overstates it and loses the bid. Early in development the gross weight is a judgement call until the mold flow analysis exists.

### P3. Break material overhead out as its own bucket
**When** — Always; especially for large low-volume parts, JIT/sequenced parts, and parts with troubled sub-suppliers.
**How** — Estimate warehousing, in-plant handling and procurement staff (buyers, material planners) separately from other overhead. Start at ~5% of total material cost; raise to ~10%+ where storage space, shelf time or procurement effort is disproportionate. Sanity-check the percentage in absolute dollars and hours.
**Trade-offs** — Not standard practice in the US (it is in Germany), so you may have to build the data. The upside is avoiding a 2%-to-10% error hidden inside a "peanut butter" spread — which for a low-margin business is decisive.

### P4. Charge real capital investment, not GAAP depreciation
**When** — Whenever the supplier or plant owns paid-off, used or unusual equipment; whenever a project requires new investment.
**How** — Ask what was actually paid, what loan principal and interest remain, and what must be replaced in the next N years. Divide that total over years, annual hours and machine count. Add maintenance and repair explicitly (technician salary + replacement parts ÷ machines ÷ hours), rising with equipment age. For product-specific custom equipment, amortize over contracted program-life volume only.
**Trade-offs** — Requires internal data the customer usually lacks, so as a customer, start with the conservative GAAP method as a placeholder and refine in negotiation. Note that the GAAP method is often the *higher* number, which is why suppliers accept it.

### P5. Trade setup frequency against inventory holding, explicitly
**When** — Low-volume parts where run size is contested.
**How** — Compute both: (a) setup cost per part = total setup cost ÷ parts per run; (b) annual-run cost per part = setup ÷ annual volume + financing cost of holding the inventory (loan interest on the manufactured value). Choose the cheaper and put the number in the breakdown.
**Trade-offs** — Annual runs shift working capital and floor space onto the supplier, and lean plants resist them; weekly runs multiply setup cost. Neither party can pretend the low volume is free.

### P6. Model scrap cumulatively, per step
**When** — Any multi-step process where material content changes between steps: painting, plating over stamping, assembly then over-molding.
**How** — Table the batch volume, unit cost of each material and the scrap percentage at each step; compute scrapped material cost per step from the cost of everything already in the part at that point; report both units scrapped and total scrap cost as percentages.
**Trade-offs** — More work than one plant-wide percentage, and requires per-product scrap data suppliers may not track. But a single percentage of revenue conceals the difference between 0.5% automated and 5% manual assembly — which is exactly the number that justifies an automation investment.

### P7. Price SG&A on minimum absolute support cost, and profit on ROI/payback
**When** — Every quote, and especially any low-volume or high-investment project.
**How** — Determine the minimum annual cost to support *any* project regardless of size (the book's example: $5,000/yr) and divide by volume. For profit, ask what investment the project requires and over what payback the company needs it recovered; divide. Then check the resulting percentage for plausibility at the other end of the volume range and commit to symmetry.
**Trade-offs** — Produces prices that look outrageous on low volume ($0.50 SG&A and $1.00 profit on a $1 part) and require an open-book relationship to defend. Symmetry is the price of the argument: at 10,000,000 units the same logic must reduce SG&A to ~2% and profit to ~1.25%.

### P8. Run a business case for any program longer than a day
**When** — Program life of 2–10 years, i.e. almost all manufacturing work.
**How** — Escalate each bucket separately over the program (the book uses +3% raw material, −2% purchased parts, +3% direct labor, +3% manufacturing overhead, −3% price giveback), add new investment and cost of capital, apply the tax rate and borrowing rate, and compute IRR and payback. Then run the alternatives: 1-year payback, 2-year payback, or an IRR hurdle, and see what price each demands.
**Trade-offs** — Requires forecasting, which people resist as futile. The book's answer: assuming today's cost is also a forecast, and a worse one. The calculation does not set the price — the market does — it tells management whether the business is worth taking.

### P9. Sanity-check every percentage in absolute dollars and hours
**When** — Whenever a bucket is estimated as a percentage of another bucket.
**How** — Multiply out to annual dollars, then convert to hours or headcount at a plausible rate. "10% material overhead = $506/yr = 50 hours at $10/hr — is 50 hours enough to manage this part's material for a year?"
**Trade-offs** — Costs a minute and catches order-of-magnitude errors that percentages hide.

---

## Part 2 — Allocation

### P10. Rank your allocation method honestly before trusting any estimate
**When** — Before defending your own cost, and before challenging a supplier's.
**How** — Identify which of the six methods is actually in use — contribution/gross margin %, labor ratio, standard hour, capital, machine rate, ABC — and ask the diagnostic question: has the product mix, process mix, machine mix or volume mix changed since the ratio was derived from the P&L? If yes, the number is wrong in a predictable direction.
**Trade-offs** — Moving up the ladder costs effort. Machine rates are the practical compromise; ABC is the target but is only worth it if you capture the key activities and drivers rather than every measurable activity.

### P11. Model the future state you are quoting into
**When** — Quoting business that will change utilization, mix or scale.
**How** — Build the cost model on the utilization the new business creates, not last year's. If a quote takes the factory from 65% to 80% utilization, the fixed-cost absorption improves and the model must show it.
**Trade-offs** — Optimistic assumptions become losses if the volume does not arrive. Treat the future-state model as a scenario alongside the status-quo estimate, not a replacement.

### P12. Test an overhead rate by reverse-engineering the plant P&L
**When** — A supplier quotes a burden rate you cannot challenge with your own data.
**How** — From plant-visit observations (headcount by category, square footage, capital employed) plus regional cost-per-unit assumptions, rebuild the plant's total annual cost by category. Then take the quoted hourly rate × annual uptime hours ÷ the share of plant capacity your product consumes, and compare the implied total plant cost with the reverse-engineered actual. The gap is the negotiation.
**Trade-offs** — Needs a plant visit and regional rate data, and every input is an assumption. Its value is not proof but momentum: it converts a stalled argument into a data-driven one.

---

## Part 3 — Negotiation

### P13. Make a standardized cost breakdown mandatory
**When** — Every RFQ, before any gap analysis is possible.
**How** — Standardize the bucket structure and the assumption fields, and require submission as a condition of quoting; use online collection software to reduce the burden on both sides.
**Trade-offs** — Suppliers resist, sometimes citing corporate policy, and may submit a modified breakdown with profit and SG&A hidden in purchased parts or manufacturing overhead. You still get more than nothing — and the hiding pattern is itself detectable.

### P14. Work the gaps in Pareto order with a waterfall
**When** — After both estimates exist.
**How** — Lay the two breakdowns side by side, chart the gap per bucket as a waterfall, and attack the largest first. For each gap, determine which of the two assumptions is wrong — or whether both are, and the gap is actually a cost reduction opportunity.
**Trade-offs** — Tempting to skip the small gaps; the book advises asking about them anyway ("even if they are only fractions of a penny") because the questions expose structural facts, such as whether resin price includes delivery.

### P15. Treat every gap as an idea generator
**When** — Throughout the negotiation.
**How** — Record what each disagreement implies could be changed: shot weight → improve mold layout, add cavities; resin grade → switch to a commonly used resin; manning → combine stations or automate; scrap → root-cause Pareto; logistics → localize. Use the Chapter 3 strategy table as the prompt list.
**Trade-offs** — Slows the price conversation. But the ideas are the durable output; the price is a one-off.

### P16. Accept a supplier's number when it is lower — then still audit it
**When** — A bucket comes in below your own estimate.
**How** — Take the number, but review the sub-supplier assumptions behind it and extend the collaborative approach one tier down.
**Trade-offs** — Effort spent on a bucket where you already "won." The book calls skipping it "a missed opportunity," particularly if the supplier is hiding margin in purchased components.

### P17. Prepare the negotiation like a rehearsal
**When** — Before any cost-based negotiation.
**How** — Gather cost breakdowns, process detail and value stream maps; align on which documents will be shared; assign roles; role-play the top gaps; anticipate the other side's responses; identify or create leverage with the buyer or sales rep; understand the other side's motives; establish collaboration and mutual respect; lead the agenda; keep it data-driven; leave purely commercial items (profit, future business) to the buyer or sales rep; and aim for a compromise both sides can call a win.
**Trade-offs** — Preparation time. The alternative is an opinion-driven meeting that stalls.

### P18. Make the plant visit an evidence-gathering exercise
**When** — Before negotiating with any supplier whose processes you do not know.
**How** — Send a data form and request the plant layout beforehand; brief your team on their roles (counting operators, timing cycles, splitting by process); open by stating your objectives; protect the touring time from long presentations and long lunches; insist on knowledgeable guides; talk to operators; take time at each process; learn to count cycle time without a stopwatch; ask permission before photographing; complete a visit form and agree the findings with the supplier before leaving; record cost reduction opportunities; thank them.
**Trade-offs** — Suppliers may add operators, slow machines, rush you, show handpicked processes, or block plant-wide data. Knowing the tactics is the counter.

### P19. Train with mock negotiations before the real one
**When** — Before rolling out cost-based negotiation, and as ongoing capability development.
**How** — Split trainees into customer and supplier teams with private, conflicting briefs (the book supplies SPARK vs Mold Experts Inc.), give each a negotiation tracker recording their walk-away price, target price, belief about the other side, each counter-offer and the final agreement plus any cost reductions agreed, and run to a settlement.
**Trade-offs** — Takes people out of the day job. The book calls it "the most useful" element of the training it recommends, and notes companies that abandoned open-book programmes purely for lack of trained people.

---

## Part 4 — Optimization

### P20. Prioritize optimization by share of total cost
**When** — Before committing any optimization resource.
**How** — Split total product cost between what you make (often 5%–10% for an assembler) and what you buy (often up to 80%). Aim the resource at the larger share, and extend it through the supplier's own value chain rather than stopping at their factory.
**Trade-offs** — Your own factory is accessible and the supply base is not; optimizing outward requires supplier relationships and travel budget.

### P21. Map the value stream before generating ideas
**When** — At the start of a cost optimization effort, especially for a part you have never traced end to end.
**How** — Map material movement from customer order through raw material supply, processing and assembly to delivery, recording operators, cycle times, uptime, inventory at each step, shipping time and physical travel.
**Trade-offs** — Takes time and access. First-time maps almost always expose redundant transport, excess inventory and bottlenecks, and the map doubles as the prioritization tool.

### P22. Benchmark with costs attached
**When** — Assessing design competitiveness, and before any teardown workshop.
**How** — Tear down competitors' products, estimate cost for **every** component individually, tabulate your part against each competitor, and name the specific best practice behind the lowest cost (e.g. "zinc coating instead of silver," "lower casting weights," "less copper and smaller magnets").
**Trade-offs** — Costing every component is expensive. Skipping it produces "improvements" that increase cost, because participants often have no costing background. Requires top-management drive and trained facilitation to overcome engineers' attachment to their designs.

### P23. Run functional analysis, then TRIZ when it stalls
**When** — Design-driven cost, at concept and at every gate.
**How** — Enumerate every function the product performs, assign cost to each, and ask whether the same or better function can be delivered for less. When the team runs out of alternatives — the usual roadblock — use the TRIZ 40 principles to break the deadlock (e.g. #22 convert harm into benefit; #28 substitute a mechanical system with an electromagnetic one).
**Trade-offs** — Some ideas will be infeasible now (the connector team's "replace it all with wireless"); record them for the next generation rather than discarding them.

### P24. Concentrate the tools into a time-boxed workshop
**When** — At concept finalization, at the quote gate, and whenever a design change is large.
**How** — Secure top-leadership support and attendee dedication; assemble cross-functional product knowledge plus at least one person deliberately ignorant of the product; set challenging targets; appoint a skilled facilitator with a defined process; choose the tools to fit the problem rather than prescribing them; provide proper facilities; and allocate 3–5 days. Invite suppliers for the later rounds — they know their processes and materials better than you do.
**Trade-offs** — Pulls a cross-functional team out of the business for a week and stresses the development timeline. The alternative is that cost work never happens, because performance always outranks it in the day job.

---

## Part 5 — Process and organization

### P25. Put a cost gate at every product development checkpoint
**When** — From napkin sketch to launch, and continuing after SOP.
**How** — Estimate at concept (±10% is enough to decide direction), then re-estimate at every gate, tightening to 5% or better as detail accumulates, covering every purchased component, raw material, capital item and tool. At each gate the cross-functional team compares cost to target, produces a gap-closure plan and an idea list with implementation owners. Run value engineering between gates; value analysis after SOP.
**Trade-offs** — Adds steps and work to development. Without the gate the review does not happen, and cost influence is nearly exhausted by the time tooling is cut.

### P26. Always compare your own estimate to the quote
**When** — At the supplier and in-house manufacturing quote gate, and for every post-award design change.
**How** — Maintain two columns — your estimate and their quote — and two gaps: gap to target and gap to quote. The difference between an over-target estimate and an over-target quote separates **over-design** (an engineering problem) from **sandbagging** (a negotiation problem). The book's illustration: a $40,000 overrun with a $115,000 "should be" estimate implies roughly $25,000 of over-design and $15,000 of sandbagging.
**Trade-offs** — Requires an independent estimating capability. Without it you cannot tell which function owns the problem, and the default is to blame Purchasing.

### P27. Keep Engineering engaged after the quotes come in
**When** — At the quote gate, when the instinct is to hand the problem to Purchasing.
**How** — Run a supplier-inclusive workshop in parallel with the commercial gap analysis, feed identified design opportunities back into the drawings, and iterate through each design cycle to final design.
**Trade-offs** — Stresses development timing and may need more resources. The book's position: significant design opportunity almost always remains at this stage, and adding resource is cheaper than launching unprofitable.

### P28. Do not release production tooling until the profit is credible
**When** — At final design.
**How** — Confirm the estimate and quote both support the profit target within estimating error before kicking off tools.
**Trade-offs** — Delays tooling and therefore launch. Changes after tooling require new or modified tools, disrupt supply chains and plants, risk quality, and — for a supplier — require customer approval and usually a share of the savings, so the ROI on late change is often insufficient.

### P29. Abolish givebacks and lump-sum payments; source at best price up front
**When** — Setting commercial policy, especially in industries where annual price reductions are customary.
**How** — Remove the requirement for annual productivity givebacks and upfront lump sums; use new-business leverage for price instead; achieve real reductions through cost engineering.
**Trade-offs** — Procurement performance objectives are usually built on annual price reductions, and executives want to see cost-structure reductions, so this needs a change at objective-setting level. Otherwise suppliers simply pre-load year-one price and the reported savings are offset by higher prices on new programmes.

### P30. Build the organization before relying on the method
**When** — Any time cost optimization depends on one person or one sponsor.
**How** — Appoint a head of cost engineering reporting to the CFO; embed experts in engineering, purchasing, operations and finance with matrix reporting; keep a central staff owning standards and deployable expertise; align objectives across functions; enforce culture through the gate reviews; and build the career path from junior cost engineer upward, with internal training and mentorship.
**Trade-offs** — Real headcount and budget (the book's scale: ~70 people and ~$10M/yr for a $10B company). The alternative fails predictably when the sponsor leaves.

### P31. Hire for curiosity
**When** — Recruiting cost estimators and cost optimization engineers.
**How** — Rank curiosity above analytical mindset and attention to detail; probe for it through interests such as travel and history. Rank change-agent ability second: ask whether the candidate has proposed changes and driven them to completion, whether they have been part of an organization seeking change, and whether they can present and sell concepts to large groups.
**Trade-offs** — Cuts against the conventional analytical-skills screen. The book's reasoning: curiosity about what sits behind an assumption is what produces both accurate estimates and optimized solutions.

### P32. Measure cost engineering at company profit, not project savings
**When** — Two to three years after implementation.
**How** — Compare company profit before and after, holding other factors equal.
**Trade-offs** — No attributable per-project number to report in the meantime, which is uncomfortable. But early-development cost engineering delivers **cost avoidance**, which never appears in a savings ledger, and forcing attribution invites functions to fight over credit — the failure mode recorded in the book's Tier 1 anecdote.
