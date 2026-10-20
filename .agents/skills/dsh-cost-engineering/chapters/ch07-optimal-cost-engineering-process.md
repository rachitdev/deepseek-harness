# Chapter 7 — Optimal Cost Engineering Process

## Core Idea

Cost optimization must have a **cadence tied to the product development gates**, not be applied at random. The chapter's opening image is the failure state: automotive OEMs organizing cost reduction workshops "only a couple of months before the start of production, after designs are already finalized and tools are already built," kicked off by a vehicle line executive saying "We are 20% over on our vehicle's cost, so we need your ideas on how to reduce cost so that we can make this vehicle profitable." The book's verdict: "It would be akin to building a house, then deciding to completely remodel it because it is over budget."

The chapter then reruns the SPARK case *with* the process, gate by gate, from a −$15,000 concept to a launched +$8,693 profit.

## Frameworks Introduced

**Cost engineering in the product development process** (Figure 7.1) — checkpoints and activities at every stage:

| Stage | Cost engineering activity | Required accuracy |
|---|---|---|
| Concept design | Ballpark cost estimate of the sketch/markup; compare to target; decide go/no-go | ~10% is sufficient ("is it a $1, $10 or $100 part?") |
| Between gates | **Value engineering** — functional analysis, benchmarking, TRIZ | — |
| Product development checkpoints (gates) | Re-estimate at each gate, and more often; estimate every purchased component, raw material, capital investment and tool | **5% or better** |
| Supplier / in-house quote evaluation | Compare own estimates to quotes; gap analysis by bucket; separate over-design from sandbagging | — |
| Product launch and production | Cost every design change; control change management | — |
| Post-SOP | **Value analysis** — same tools, applied to production parts and the next generation | — |

**Cost impact by stage** (Figure 7.2) — influence on final cost is largest at concept and decays through development; after tooling it is near zero.

**VE vs VA.** Value engineering = all cost optimization before design freeze / production start. Value analysis = all cost optimization after. Same tools, radically different economics.

**Two failure paths when the gate process is missing.** SPARK, two years in and 45% over ($130,000 vs $90,000 target):
1. **Redesign** — another 1–2 years at ~$10,000,000/yr (100 engineers × $100,000), probably running out of cash first, probably still too costly because component targets still do not exist, and most likely de-contenting the vehicle (eliminating functions), compromising performance and losing sales.
2. **Raise price** to ~$150,000 — but the market analysis says $100,000 is required to sell 10,000 vehicles, so volumes fall, the same fixed expenses spread over fewer vehicles, cost per vehicle rises further, and profit disappears.

"Both paths likely lead to the same result: the company losing money on their product and possibly going bankrupt."

## Key Concepts

- **Cost MUST be evaluated at concept.** "Based on raw materials used, the rough part weight, and manufacturing processes needed to make the part, a cost estimator should be able to tell if the design will cost $1, $10, or $100." Even ±10% is enough to decide direction.
- **Cost restrictions do not kill creativity — they already exist.** Designers already work under space, weight and safety constraints; it is rarely a blank sheet. Even on a blank sheet, designers know roughly what consumers will pay for a given functionality (a self-driving mass-market car that doubles in price becomes a niche product). The counter-argument: a cost restriction "would actually increase creativity since it forces designers to dig deeper," and the alternative is designing a product nobody can buy.
- **Why companies find out after the tools are cut.** Short development windows mean tool manufacture starts after only initial designs are complete. Initial designs often do not work; redesigns are pushed fast under performance pressure, are conservative, ignore cost, and require significant tool changes. At that point suppliers hold all the leverage and charge premiums on both piece price and tooling.
- **Sandbagging is expected in first quotes.** Suppliers and in-house plants pad for uncertainty, especially on new designs, unvalidated designs, or with customers who have made them absorb cost before. This is precisely why the company's own estimates are needed: to split an overrun into over-design vs sandbagging.
- **Buyers are not incentivized to fight change-driven price increases.** Purchasing is judged on annual savings, not on design change pricing — and a premium-priced change actually *helps* the buyer later by creating supplier margin to negotiate against. Cost estimators must therefore price design changes independently.
- **Post-SOP change is expensive and shared.** Tools are built; changes need tooling modification or new tools; savings often must be shared with suppliers as motivation; OEM customers must approve and usually take a share; hidden costs (resources to execute changes) are real but unmeasured; supply-chain and quality risk rises. "As a consequence, the ROI is often not good enough to pursue changes after production start."
- **But the search is never wasted.** Ideas that cannot be implemented in development may be implementable in production; some still have good ROI even after sharing; others feed the next product generation.
- **Refresh cycles are shortening.** Cars: new versions every 4–5 years with refreshes every 2–3, shrinking further with CAD/CAM. Appliances and consumer electronics may refresh every few months. Designers should start the next cost-effective concept **as soon as production starts**, capturing lessons learned while the launch is fresh, and use benchmarking to see how the just-launched product stacks up against competitors' recent launches.
- **Ego is the obstacle.** "The number one goal of any company is to make money, not to make egos. Developing designs is only a means to an end."

## Mental Models

- **Cost influence decays; cost commitment does not.** By the time cost is visible in a quote, most of it is already designed in.
- **The gate is the forcing function.** Without a formal cost review at each gate, cost review does not happen, because performance requirements always outrank it in the day job.
- **Estimate vs quote is a two-column comparison, always.** Every SPARK gate table has both a "Estimated" (own cost model) and a "Quote" (supplier/plant) column, plus gaps to each — the estimate is the instrument that makes the quote interpretable.
- **A modest, deliberate profit beats a large, hoped-for one.** SPARK proceeds from concept finalization at $5,000 profit against a $10,000 target, with a direction to keep improving — a controlled decision, not a gamble.

## Anti-patterns

- **The pre-SOP cost reduction workshop** — organized months before launch after tools are built.
- **Engineering "washing their hands"** after Phase 1 quotes come in high and handing the problem to Purchasing to negotiate and everyone else to "sharpen their pencils." The book insists Engineering stays engaged: "It is very common, even at this stage, after so much analysis has taken place, that the opportunities to further improve the design still exist and may be significant."
- **De-contenting under time pressure** — eliminating functions to hit cost, compromising performance and sales.
- **Raising price above the market** to cover an overrun.
- **Waiting for finalized drawings before estimating** — "having a well-modeled estimate of cost is infinitely better than having no estimate at all. Otherwise, the engineers will be 'flying blind'."
- **Relying only on value analysis** (post-production optimization) — "a terrible idea."
- **Kicking off production tooling without confidence in the profit** — any later change destroys the ROI, disrupts supply chains and risks quality.

## Reference Tables

The four SPARK gate tables. Target price is $100,000 throughout; the target BOM is $75,000; factory labor $2,000, factory overhead $8,000, other $1,000, SG&A $4,000, target profit $10,000.

**Table 7.1 — Concept design checkpoint (hybrid truck)**

| Line | Target cost | Concept estimated | Gap to target |
|---|---|---|---|
| Motor 1 | $1,500 | $2,500 | $1,000 |
| Motor 2 | $1,500 | $2,500 | $1,000 |
| Inverter 1 | $1,000 | $1,500 | $500 |
| Inverter 2 | $1,000 | $1,500 | $500 |
| Battery | $5,000 | $13,000 | $8,000 |
| Gasoline engine | $10,000 | $10,000 | $0 |
| Electronic controls | $2,000 | $3,000 | $1,000 |
| Frame | $15,000 | $23,000 | $8,000 |
| Body | $10,000 | $15,000 | $5,000 |
| Lighting | $1,500 | $1,500 | $0 |
| Other | $26,500 | $26,500 | $0 |
| **Total purchased parts** | **$75,000** | **$100,000** | **$25,000** |
| **Profit** | **$10,000** | **($15,000)** | **($25,000)** |

**Table 7.2 — Concept design checkpoint, updated (switched to full electric)**

| Line | Target cost | Concept estimated | Gap to target |
|---|---|---|---|
| Motor 1 / Motor 2 | $1,500 each | $2,000 each | $500 each |
| Inverter 1 / Inverter 2 | $1,000 each | $1,250 each | $250 each |
| Battery | $15,000 | $20,000 | $5,000 |
| Gasoline engine | $0 | $0 | $0 |
| Electronic controls | $2,000 | $2,500 | $500 |
| Frame | $15,000 | $16,000 | $1,000 |
| Body | $10,000 | $11,000 | $1,000 |
| Lighting | $1,500 | $1,500 | $0 |
| Other | $26,500 | $26,500 | $0 |
| **Total purchased parts** | **$75,000** | **$84,000** | **$9,000** |
| **Profit** | **$10,000** | **$1,000** | **($9,000)** |

**Table 7.3 — Concept finalization checkpoint (after the 5-day workshop)**

| Line | Target cost | Concept estimated | Gap to target |
|---|---|---|---|
| Motor 1 / Motor 2 | $1,500 each | $2,000 each | $500 each |
| Inverter 1 / Inverter 2 | $1,000 each | $1,000 each | $0 |
| Battery | $15,000 | $18,000 | $3,000 |
| Electronic controls | $2,000 | $1,500 | ($500) |
| Frame | $15,000 | $16,000 | $1,000 |
| Body | $10,000 | $11,000 | $1,000 |
| Lighting | $1,500 | $1,500 | $0 |
| Other | $26,500 | $26,000 | ($500) |
| **Total purchased parts** | **$75,000** | **$80,000** | **$5,000** |
| **Profit** | **$10,000** | **$5,000** | **($5,000)** |

**Table 7.4 — Phase 1 design checkpoint (quote level)**

| Line | Target cost | Phase 1 estimated | Phase 1 quote | Gap to target | Gap to quote |
|---|---|---|---|---|---|
| Motor 1 / Motor 2 | $1,500 each | $1,756 each | $1,955 each | $256 each | $455 each |
| Inverter 1 / Inverter 2 | $1,000 each | $951 each | $1,125 each | ($49) each | $125 each |
| Battery | $15,000 | $16,597 | $17,567 | $1,597 | $2,567 |
| Electronic controls | $2,000 | $1,850 | $2,156 | ($150) | $156 |
| Frame | $15,000 | $15,098 | $17,231 | $98 | $2,231 |
| Body | $10,000 | $10,250 | $11,422 | $250 | $1,422 |
| Lighting | $1,500 | $1,300 | $1,328 | ($200) | ($172) |
| Other | $26,500 | $25,124 | $27,894 | ($1,376) | $1,394 |
| **Total purchased parts** | **$75,000** | **$75,633** | **$83,758** | **$633** | **$8,758** |
| Labor (factory) | $2,000 | $2,000 | $2,145 | $0 | $145 |
| Overhead (factory) | $8,000 | $8,000 | $8,745 | $0 | $745 |
| Other | $1,000 | $1,200 | $1,244 | $200 | $244 |
| SG&A | $4,000 | $4,000 | $4,000 | $0 | $0 |
| **Profit** | **$10,000** | **$9,167** | **$108** | **($833)** | **($9,892)** |

**Table 7.5 — Final design checkpoint**

| Line | Target cost | Final estimated | Final quote | Gap to target | Gap to quote |
|---|---|---|---|---|---|
| Motor 1 / Motor 2 | $1,500 each | $1,756 each | $1,825 each | $256 each | $325 each |
| Inverter 1 / Inverter 2 | $1,000 each | $894 each | $900 each | ($106) each | ($100) each |
| Battery | $15,000 | $16,258 | $16,667 | $1,258 | $1,667 |
| Electronic controls | $2,000 | $1,750 | $1,923 | ($250) | ($77) |
| Frame | $15,000 | $14,963 | $15,247 | ($37) | $247 |
| Body | $10,000 | $10,250 | $10,536 | $250 | $536 |
| Lighting | $1,500 | $1,300 | $1,328 | ($200) | ($172) |
| Other | $26,500 | $24,620 | $24,954 | ($1,880) | ($1,546) |
| **Total purchased parts** | **$75,000** | **$74,441** | **$76,105** | **($559)** | **$1,105** |
| Labor (factory) | $2,000 | $2,000 | $1,958 | $0 | ($42) |
| Overhead (factory) | $8,000 | $8,000 | $8,000 | $0 | $0 |
| Other | $1,000 | $1,200 | $1,244 | $200 | $244 |
| SG&A | $4,000 | $4,000 | $4,000 | $0 | $0 |
| **Profit** | **$10,000** | **$10,359** | **$8,693** | **$359** | **($1,307)** |

## Worked Example

**SPARK, rerun with the process — from −$15,000 to +$8,693 per vehicle.**

**Gate 1 — Concept design.** Finance develops target costs for the vehicle *and for each individual component* first. Engineering produces the hybrid concept; cost estimates are developed for that concept (taking the low volume into account) and compared line by line (Table 7.1). Purchased parts estimate: $100,000 against a $75,000 target — a $25,000 gap, all in purchased components, giving a **−$15,000 loss per vehicle**. Crucially this happens at concept, not two years in.

**Design direction change.** With component-level visibility, the designers can see *why*: the gasoline engine makes the battery hard to package, forcing an expensive shape, and the weight of two powertrains forces a much more robust frame. The conclusion — "a purely electric vehicle is the only feasible option that could meet the $100,000 market price requirement" — is a drastic change, but "proceeding with the original hybrid design would have been disastrous."

**Gate 2 — Updated concept (Table 7.2).** The EV concept brings purchased parts to $84,000 (gap $9,000) and profit to **+$1,000**. Still short of the $10,000 target, but now a feasible starting point.

**Concept finalization workshop.** Before freezing the concept, Engineering runs a **5-day cost reduction workshop** using functional analysis, TRIZ and benchmarking data. Yield funnel: **350 ideas generated → 210 deemed feasible → 75 prioritized for investigation → 35 implemented**, mostly in the electronic connection concepts between battery, motors and controllers. Result (Table 7.3): purchased parts $80,000 (gap $5,000), **profit $5,000** — half the target, but the company proceeds with a direction to keep improving during drawing development. Suppliers, not present at the first workshop, are now engaged for design feedback while drawings are developed.

**Gate 3 — Phase 1 design / quote level (Table 7.4).** Drawings released, RFQ package issued. Two columns now diverge: SPARK's own **estimate says $9,167 profit is achievable**; the **supplier and in-house quotes say $108** — a $9,892 gap. This is the moment where most companies hand the problem to Purchasing. Instead SPARK does three things in parallel:
- another workshop, this time **with suppliers**, focused on the value stream map and lean manufacturing opportunities (functional analysis having been done already);
- **gap analysis on every cost bucket of every part**, comparing quotes to cost targets, identifying each gap driver and assigning an action plan — with design opportunities fed back to Engineering;
- the same gap analysis plus a lean study on **SPARK's own factory**, whose costs also came in high ($2,145 labor vs $2,000; $8,745 overhead vs $8,000).

The chapter notes explicitly that first-time quote conservatism is expected on something as new as EV components, which is why collaborative gap analysis — not pressure — is the right instrument.

**Gate 4 — Phase 2 to final design (Table 7.5).** Iterating design refinement against workshop output and gap analysis through each design cycle: purchased parts land at **$74,441 estimated / $76,105 quoted** (now *below* the $75,000 target on the estimate), and profit at **$10,359 estimated / $8,693 quoted**. The company accepts $8,693 — slightly below target, within estimating error, and with the cost targets indicating $10,359 is reachable during launch.

**Production tooling release.** Only now, with profit confidence established, is production tooling kicked off. The reasoning given: any further change would require new or updated tools, diminishing ROI, disrupting supply chains and plants with parts on order, risking quality, and — for a supplier — requiring customer approval and probably a share of the savings.

**Production phase.** Optimization continues immediately after SOP: collect lessons learned while the launch is fresh, benchmark the launched product against competitors' recent launches, and start the refresh concept at once rather than waiting.

**Real-Life Anecdotes bracketing the chapter.** (a) A major automotive OEM held cost reduction workshops three months before SOP, after all tools were built, on **almost all vehicle launches** — because engineers had learned cost was not important during development and adjusted their behaviour to performance only. Fixing cost just before or after SOP "became company's process and culture" and cost millions per program. (b) A Tier 1 supplier told by executives to increase savings on current production failed completely: the CEO was not engaged, so functional VPs bickered and working teams had no direction; **engineering actively sabotaged the effort** to protect its designs and resources; **no dedicated resources** were assigned (the new group was staffed by people with other primary jobs); and **objectives were not aligned**, leaving functions fighting over credit for each cost reduction. "Ultimately nothing changed."

## Key Takeaways

1. Put a cost checkpoint at every product development gate, and value engineering between them.
2. Estimate at concept — ±10% is enough to change direction, and it is the only point where direction is cheap to change.
3. Tighten to 5% or better as design detail accumulates; estimate every component, material, capital item and tool.
4. Always compare your own estimate to the quote; the difference separates over-design from sandbagging.
5. Engineering must stay engaged after quotes arrive — significant design opportunity typically remains.
6. Expect and plan for first-quote conservatism; resolve it with collaborative gap analysis, not pressure.
7. Do not release production tooling until the profit is credible.
8. Price every post-award design change independently; buyers are not incentivized to.
9. Keep optimizing after SOP for the refresh and the next generation, and benchmark while the launch is fresh.
10. CEO engagement, dedicated resources and aligned functional objectives are the three preconditions; missing any one of them causes the effort to fail.

## Connects To

- [ch01-what-is-cost-engineering.md](ch01-what-is-cost-engineering.md) — the original SPARK failure this chapter reverses
- [ch02-cost-estimating-basics.md](ch02-cost-estimating-basics.md) — the estimating capability and quoting/change-control processes these gates depend on
- [ch03-cost-based-negotiation.md](ch03-cost-based-negotiation.md) — the gap analysis run at the quote gate
- [ch06-cost-optimization-tools.md](ch06-cost-optimization-tools.md) — the workshop, functional analysis, TRIZ, benchmarking and VSM used between gates
- [ch08-cost-engineering-organization.md](ch08-cost-engineering-organization.md) — the organization and culture required to enforce the gates
