# Chapter 9 — Cost Engineering in Conclusion

## Core Idea

Cost engineering is not an optional overhead layered on top of core functions — it **is** a core function whether the company acknowledges it or not. "Any company manufacturing any product must already cost optimize it in order to make money; some are just doing it better than others. A consumer is willing to pay only so much for any given product, which means that a cost target is automatically established."

Therefore the only real decision is **how good you intend to be at it**, and how you will resource that.

## Frameworks Introduced

**The "it is already a core function" argument.** Three steps:
1. Consumers will pay only so much → a cost target exists by default, whether or not anyone writes it down.
2. Exceeding that target erodes or wipes out profit → the product must be cost optimized to the target or better.
3. "If your company is not doing this basic thing, then it should not and will not be in business."

**The 10:1 return heuristic.** "It is not an exaggeration to say that for every dollar spent on cost engineering, ten dollars should be expected in return." Worked at scale: a **$10B company** will probably need to spend **$10M a year** on cost engineering resources — approximately **70 personnel**, costing software, travel to supplier factories, teardown and benchmarking resources, and cost reduction workshops — and should see **$100M or more improvement in profit per year**.

**How to measure it (and why not to try too hard).** Do **not** try to quantify cost engineering's contribution project by project. If resources are deployed early in development, most of the benefit is **cost avoidance**, not annual cost savings, and cost avoidance is inherently hard to attribute. The recommended measurement: **compare company profit two or three years after implementation**. "If the profits for that $10B company jump from $500M annually to $600M or more in that span while everything else remains equal, then most likely cost engineering was the culprit."

**The minimum resource set.** Two distinct expert types, both required: **cost estimating** experts to provide cost knowledge and guidance, and **cost optimization** experts to "scientifically and creatively find cost reduction opportunities." Their coverage must span every cost category of the product: raw material, purchased components from suppliers, logistics, engineering design, in-house manufacturing, and corporate overhead.

## Key Concepts

- **What companies that embrace it stop doing.** They will not sell products that cost more than their selling price; they will not redesign products mid-cycle and blow their development budgets; they will not end up with suppliers unable to manufacture components or manufacturing them at a loss.
- **What they start doing.** They hold accurate cost data that supports good decisions; they focus on meeting cost targets throughout development; and they work with suppliers early to design manufacturable, cost-effective products that sell at a profit "on a consistent and sustainable basis."
- **Start at napkin sketches.** "The cost engineering process must start as early as possible in the product development process, even as early as napkin sketches, and the cost engineering discipline must be maintained throughout." Review at every gate; adjust design direction when targets are missed; engage cost engineering resources to find solutions.
- **The full commitment list.** Methodology alone is insufficient. A company must also build the organization, develop people to work in it, give them career advancement options, and provide resources: **software, benchmarking facilities and personnel, and a travel budget to visit supplier factories**. Training must be developed in-house to capture and share knowledge.
- **Competitive necessity, then competitive advantage.** "Some of your competitors have probably already embraced it or are embracing it. Your company must do the same simply to keep up. By fully embracing it yourself, it will give you an unprecedented competitive advantage."
- **The evidence base is thin and the author says so.** "Although there is a lack of statistics to prove it, companies that are using it will almost unanimously agree that they have benefited greatly from its use." The claim is practitioner consensus, not measured study — consistent with the Preface, where the author says he could find no studies quantifying the cost of poor cost management and offers his own estimates instead (from 25+ years in manufacturing: at least three quarters of manufacturing companies do not fully understand their own costs; roughly 90 percent have no existing or effective cost engineering process).
- **The cost of the discipline is real.** It adds complexity to product development, which means more work. "The witnesses of this methodology will tell you that the rewards far outweigh the effort" — and the benefits are tangible, arriving as profit and cash.
- **Credit-fighting is the last obstacle.** "Various company functions often fight to give themselves credit for any success, but as a CEO or general manager, you must see the bigger picture. You must give credit to the cross-functional embrace of cost engineering methodology."
- **Institutionalize it so it outlives you.** "You must engrain cost engineering into your company culture so much that it becomes just like breathing... This will give your company a method to sustain its existence even after the people who implemented cost engineering are long gone."

## Mental Models

- **The cost target exists whether you set it or not.** The market sets it; your only choice is whether you know its value.
- **Cost avoidance is invisible and is most of the benefit.** Judge the discipline by company-level profit two to three years out, not by a savings ledger.
- **Two roles, not one.** An estimator who cannot optimize and an optimizer who cannot estimate are each half a capability.
- **Budget objection inverted.** "The honest answer is that cost engineering is a core function, whether you acknowledge it or not, and you better find the budget for it."

## Anti-patterns

- **"We don't have budget for resources beyond our core functions."** Rebutted directly: this *is* a core function.
- **Trying to measure cost engineering ROI project by project** — the effect is cost avoidance and will not appear in a savings ledger.
- **Functions competing for credit** for individual cost reductions instead of the cross-functional method being credited.
- **Implementing methodology without organization, people, career paths, software, benchmarking capability and travel budget** — the method alone does not survive.
- **Starting after concept.** The discipline must begin at napkin sketches.

## Reference Tables

**The 10:1 investment case, at the book's own scale**

| Item | Value |
|---|---|
| Company revenue | $10B |
| Annual cost engineering spend | ~$10M |
| Headcount implied | ~70 personnel |
| Other spend in that budget | Costing software; travel to supplier factories; teardown and benchmarking resources; cost reduction workshops |
| Expected annual profit improvement | $100M or more |
| Stated ratio | **~10:1 return** |
| Measurement method | Profit two to three years after implementation, everything else equal (e.g. $500M → $600M+) |

**The minimum capability coverage**

| Expert type | Purpose | Must cover |
|---|---|---|
| Cost estimating | Provide cost knowledge and guidance | Raw material, purchased components from suppliers, logistics, engineering design, in-house manufacturing, corporate overhead |
| Cost optimization | Scientifically and creatively find cost reduction opportunities | The same six cost categories |

**What a company must provide beyond methodology**

| Resource | Why |
|---|---|
| Cost engineering organization | Method without an owner is person-dependent (Ch. 8) |
| People development and career advancement | Otherwise leaders leave or never enter the field |
| Costing software | Removes rate-collection effort; pushes feedback to design time |
| Benchmarking facilities and personnel | Teardown is a primary idea generator (Ch. 6) |
| Travel budget to visit supplier factories | Plant visits are where cost knowledge and negotiation evidence come from (Ch. 3) |
| Internally developed training | Universities do not teach this (Ch. 8) |

## Worked Example

**The $10B company's cost engineering investment case — the only quantified organization-level example in the book.**

*The question a CEO asks:* we already have engineering, purchasing, finance and operations; why fund a separate cost engineering capability?

*Step 1 — establish that the target already exists.* Consumers will pay only so much for the product, so a cost target is set by the market automatically. Exceeding it erodes or eliminates profit. The company is therefore already obliged to cost optimize; the only question is competence.

*Step 2 — size the investment.* For a **$10B revenue** company, the annual spend is estimated at **$10M**, made up of:
- approximately **70 personnel** (cost estimators and cost optimization experts, embedded across engineering, purchasing, operations and finance per Chapter 8);
- **costing software** (the FACTON / Siemens / aPriori / Costimator / Munro category from Chapter 8);
- **travel to supplier factories** (the plant visits that make Chapter 3's negotiation evidence and reverse-engineered P&Ls possible);
- **teardown and benchmarking resources** (Chapter 6's Table 6.1 analysis);
- **cost reduction workshops** (Chapter 6's 3–5 day format).

*Step 3 — state the expected return.* **$100M or more of annual profit improvement** — a ~10:1 return. That is 1% of revenue, which for a company earning $500M/yr is a 20% profit uplift.

*Step 4 — decide how to measure it, in advance.* Do not try to attribute savings to individual projects. Early-development cost engineering produces **cost avoidance** — designs that were never allowed to become expensive — which never appears as a line-item saving. Instead, hold everything else equal and look at company profit **two to three years after implementation**: $500M → $600M or more is the signature.

*Step 5 — accept the caveat.* The author is explicit that no statistics prove the ratio; it rests on practitioner consensus and his own 25+ years of observation. Treat 10:1 as a framing argument for a budget conversation, not as a measured benchmark.

*Step 6 — remove the last obstacle.* When results arrive, functions will compete to claim them. The CEO must credit the **cross-functional method**, not any single function, or the alignment that produced the result decays — precisely the failure recorded in Chapter 7's Tier 1 anecdote, where functions "fought over credit for each cost reduction" and nothing changed.

## Key Takeaways

1. Cost engineering is already a core function; the market sets a cost target whether or not you write one down.
2. Budget objections are answered by that fact, not by a business case alone.
3. The claimed return is ~10:1 — for a $10B company, ~$10M/yr and ~70 people against $100M+ of annual profit improvement — with the author openly noting no statistics prove it.
4. Measure at company profit two to three years out; most of the benefit is cost avoidance and cannot be attributed project by project.
5. Staff both roles: cost estimating and cost optimization, covering all six cost categories.
6. Start at napkin sketches and review at every gate; adjust design direction whenever targets are missed.
7. Fund the surrounding resources — software, benchmarking, supplier travel, training — or the methodology will not survive.
8. Credit the cross-functional method, and engrain it so the capability outlasts the people who built it.

## Connects To

- [ch08-cost-engineering-organization.md](ch08-cost-engineering-organization.md) — the organization, culture and career path this chapter says must be funded
- [ch07-optimal-cost-engineering-process.md](ch07-optimal-cost-engineering-process.md) — the gate process that starts at napkin sketches
- [ch01-what-is-cost-engineering.md](ch01-what-is-cost-engineering.md) — the definition and umbrella this conclusion returns to
- [ch06-cost-optimization-tools.md](ch06-cost-optimization-tools.md) — the benchmarking and workshop resources named in the budget
