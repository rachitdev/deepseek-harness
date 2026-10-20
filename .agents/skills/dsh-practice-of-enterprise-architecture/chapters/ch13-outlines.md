# Chapter 13 — Outlines

## Core Idea
**Outlines are business-focused changes EA artifacts.** They provide high-level descriptions of separate IT initiatives understandable to business leaders and essentially represent **benefit, time and price tags for proposed IT investments**. Their job is to let executives approve, reject or reprioritize initiatives at the *initiation* step of Initiative Delivery — before any implementation money is spent.

## Frameworks Introduced

### Questions Outlines answer
What business need is fulfilled by the proposed initiative? · What solution will be implemented? · How will it change current business processes? · What is its tactical and strategic value? · What is its overall organizational impact? · What financial investments are required? · When can it be delivered? · What risks are associated with it?

### Three subtypes
| Subtype | Prevalence | Definition |
|---|---|---|
| **Solution Overviews** (solution outlines, conceptual architectures, preliminary solution architectures, conceptual designs, solution briefs) | Essential | High-level descriptions of specific proposed IT solutions understandable to business leaders |
| **Options Assessments** (options analyses/papers, solution options/assessments, discussion papers) | Common | Lists of available high-level implementation options with their pros and cons |
| **Initiative Proposals** (solution proposals, initiative summaries, investment cases, idea briefs, project briefs) | Uncommon | Very early idea-level descriptions of proposed initiatives and their justifications |

**Less popular subtypes**: *key decisions for Outlines* (one-page bullet-point digests of the most significant planning decisions and justifications); *architectural directions for Outlines* (very high-level conceptual and technical architectural requirements handed to the architects writing Outlines — a documents-based communication interface between enterprise and solution architects); *one-page initiative overviews* (informal, intuitive single-page graphical views for early feedback and "T-shirt size" estimates — "condensed, purely graphical Initiative Proposals"); *initiative roadmaps* (also program roadmaps, transition plans — temporary mini-roadmaps for multi-project initiatives); *analytical papers* for initiatives implying far-reaching technology choices ("local, initiative-specific Analytical Reports, or purely analytical versions of Options Assessments").

## Key Concepts
- **Development.** At the **initiation step** of Initiative Delivery, collaboratively by architects and relevant senior business stakeholders (executive sponsors), often assisted by business analysts. Lifecycle: early informal discussions of the general idea (often from an original business proposal prepared by the business owners) → elaboration through ongoing discussions, becoming more formalized and "meaty" → completion, review and official approval by executive sponsors. Discussion point: **business processes** — how new solutions should modify current processes. Techniques used: formal business process modeling, customer journey mapping for customer-facing initiatives. Complex, multi-domain solutions require multiple specialized architects contributing.
- **Two critical early decisions**: the **"buy versus build"** decision (study the product market, investigate vendor products, evaluate against requirements, assess architectural fitness, comparative analysis) and the **outsourcing decision** (external contractor vs. in-house IT staff). RFIs and RFPs may be submitted and contracts discussed.
- **Timing relative to portfolio management.** Outlines may be developed before or after inclusion in the program of work. Sometimes elaborate or finalized Outlines are created beforehand to inform portfolio management with no further sign-offs needed; other initiatives enter the program conditionally with no or tentative Outlines, and the final investment decision follows when the Outlines are complete.
- **Time focus**: mid-term future, up to 1–2 years, sometimes longer for large transformation programs. Beyond 2–3 years, neither the high-level details nor the business cases can be planned with acceptable precision — longer horizons belong to Visions.
- **Temporary.** After approval and elaboration into Designs, Outlines "lose their value as EA artifacts and get archived" — but they are retrieved later, with their business cases, for **post-implementation benefit reviews**.
- **Format**: mix of textual descriptions and simple diagrams; avoid long narratives, elaborate explanations, sophisticated diagrams and technical details; avoid strict formal modeling notations, though a **simplified version of BPMN** may be used for process changes. MS Word (more official) or MS PowerPoint (less formal). Solution Overviews typically **15–30 pages**.

### Outlines vs. business cases
| | Outlines | Business cases |
|---|---|---|
| Nature | IT-specific architectural documents driven by architects | Largely IT-agnostic documents driven by business sponsors |
| Content | High-level overview of the proposed solution; business value in qualitative terms; helps assess costs | Financial side; may also touch non-technology business aspects (e.g. legal matters) |
| Methods | Estimate license fees, hardware, development efforts; send solution descriptions to vendors for tentative estimates | Formal quantitative valuation: discounted cash flow (DCF), net present value (NPV), internal rate of return (IRR); total ROI |
| Purpose | Substantiate initiatives from a conceptual perspective | Justify them from a purely financial point of view |

They are developed together: business cases use the time and cost estimates from Outlines; Outlines use business cases to compare options against the original business motivation. Not all benefits quantify easily — e.g. customer satisfaction and security enhancements.

### Cost concepts in Outlines
**CAPEX** (initial investments: purchasing hardware, developing software) · **OPEX** (recurring: hosting costs, license fees) · periodical future maintenance (bug fixes, updates, security patches) · **delayed costs of redeeming architecture debts** · **total cost of ownership (TCO)** over the whole lifecycle.

### Post-implementation benefit reviews
Retrieved Outlines and business cases are revisited a few years after completion to assess how far the declared benefits were realized and whether the estimated ROI was achieved. Three benefits: (1) more realism and accuracy in future value estimations, discouraging exaggerated benefits and overstated business proposals, increasing IT's credibility; (2) ensuring business ownership, commitment and accountability for delivering claimed benefits — "the business success of any IT initiative cannot be guaranteed by the IT department alone merely via delivering the requested system on time and budget"; (3) organizational learning, eliminating systematic bias in benefit and cost estimations, increasing management satisfaction with IT.

## Mental Models
- **Outlines "sell" initiatives to potential business sponsors.**
- **Filter early, elaborate late.** Initiative Proposals filter out futile initiatives at the earliest stage; Options Assessments narrow the alternatives; Solution Overviews finalize the chosen option for funding.
- **Where Outlines come from depends on initiative type.** Fundamental, strategic and local initiatives arrive as planned initiatives from Roadmaps; urgent initiatives launch instantly, possibly bypassing normal prioritization and budgeting; architectural initiatives arrive from IT Roadmaps and are funded directly by the CIO, with senior IT managers acting as sponsors instead of business leaders.

## Anti-patterns
**The single biggest threat: highly detailed Outlines** — "redundant and too 'heavyweight' for preliminary decision-making purposes." Elaborating to the level of implementable Designs has no practical justification and may even be unachievable per the EA uncertainty principle. Voluminous Outlines "complicate the assessment of proposed IT initiatives, distract attention from the most critical questions and eventually even reduce the quality of decision-making."

**Kotusev's rule:** *Outlines should be detailed enough to adequately evaluate proposed IT initiatives, but no more detailed than necessary for this purpose.* They should focus on exactly three things: the anticipated business value and how it will be realized; potential risks, problems and negative consequences; the cost, estimated with reasonable accuracy.

## Reference Tables

### Evaluation criteria at investment governance
Tactical and strategic business value · expected financial returns · conceptual alignment to Considerations and Visions · technical alignment to key Standards and Landscapes · timelines, costs and risks. Outcome: approve and fund, or reject as inexpedient.

### Solution Overview contents
**Core**: abstract technical architecture + high-level business process models, often showing both the current and expected "to-be" states to accentuate the beneficial contrast.
**Supporting information**: goals and objectives of the initiative · business benefits expected · key business stakeholders and sponsors · essential requirements for the solution · third parties involved in implementation · estimates of time and cost · identified risks.
**Strategy alignment additions**: a checklist of established Principles with an explanation of adherence to each; a capability "footprint" mapping the initiative to Business Capability Models; positioning within the agreed investment Roadmaps.
**Mini-roadmaps** for large solutions delivered as a series of related projects — though multi-project solutions usually still have a single business case for the whole solution.

### Options Assessment scoring criteria
Business functionality provided · technical feasibility · time, effort and cost estimates · security, risks and potential problems · alignment to the general strategic direction · estimated overall bottom-line impact. Total scores can be calculated as a sum of weighted scores; sponsors are often asked to prioritize and weight business requirements so options can be scored against them separately. **In some organizations architects are expected to propose and analyse at least three different conceptual options**, and the **"Do nothing"** option may be included to explore the consequences of not acting. When no reasonable alternatives exist, Options Assessments are skipped.

### Outlines vs. adjacent types
| Compared with | Difference |
|---|---|
| **Visions** | Visions are more conceptual, abstract and global, describing the long-term organization-wide direction and talking in capabilities or activities; Outlines describe short-term initiatives, often in terms of specific business processes. Visions use rough qualitative assessments (large, small) and usually have no business cases; Outlines contain financial estimations and are closely associated with business cases. |
| **Designs** | Designs are very technical with extensive implementation-specific detail; Outlines are abstract, avoid "scary" IT terminology, and describe solutions conceptually from the business perspective — impact, value, costs. Outlines support initial decision-making; Designs support practical implementation. |

## Worked Example
**A Solution Overview (Figure 13.1, ordinary form).** 1 Overview and Goals · 2 Scope and Stakeholders · 3 Essential Requirements · 4 Business Benefits · 5 Capability Impact (Order Fulfilment High, Order Management Low, Customer Analytics Low) · 6 Involved Partners (IBM, Accenture) · 7 Estimations (Time 6–8 months, Cost \$1.2–1.5 million) · 8 Business Process Changes (Process 1 Now: Step 1 = 3 days, Step 2 = 5 days, Step 3 = 2 days, 10 days in total; Process 1 Will Be: Step 1 = 2 days, Step 2 = 3 days, 5 days in total) · 9 Architectural Overview.
*Strategy-aligned form*: 1 Business Intent · 2 Business Sponsors · 3 Basic Requirements · 4 Solution Scope and Value · 5 Principles Alignment (Principle 1, 2, 3) · 6 Strategy Alignment · 7 Conceptual Architecture · 8 Current Estimates (Time 9–12 months, Cost \$1.6–2.0 million, Risks).

**An Options Assessment (Figure 13.2).** Three options scored on Functionality / Feasibility / Alignment:
| Option | Time | Cost | Functionality | Feasibility | Alignment | **Total** |
|---|---|---|---|---|---|---|
| Solution 1 | 8–13 months | \$2.0–3.5 million | 5 | 2 | 4 | **11** |
| Solution 2 | 4–7 months | \$1.0–1.7 million | 3 | 3 | 1 | **7** |
| Solution 3 | 3–5 months | \$0.7–1.3 million | 2 | 5 | 2 | **9** |
Each option also carries Advantages, Disadvantages and Risks. Prepared in MS Word or PowerPoint; the comparison itself is often done in MS Excel.

**Three ways to address one need.** "If business executives articulated a specific business need to enable order tracking for customers, then architects can offer different options for addressing this business need (e.g. develop a mobile app, create an order tracking page on the website or upgrade the existing system to automatically send status notifications via SMS) and explain the pros and cons of these options from the business perspective in Options Assessments."

**Planned vs. urgent initiative flow.** *Planned*: the need to enable order tracking is placed in a Roadmap and scheduled for mid-2021; at the beginning of the closest preceding budgeting cycle it enters the program of work; in mid-2021 it becomes an active instance of Initiative Delivery — starting from the initiation step if Outlines are not yet prepared, or proceeding straight to implementation if they are. *Urgent*: "the urgent need to be able to provide specific compliance reports to a regulatory agency from the beginning of the next financial year can emerge unexpectedly as a result of a recent legislative change" — a new instance is launched instantly, possibly bypassing normal prioritization and budgeting, with Outlines developed immediately. *Architectural*: "the planned technical need to decommission unsupported legacy databases and migrate their records to newer storage platforms" appears in an IT Roadmap scheduled for end-2021, is financed directly by the CIO through a simplified procedure, and Outlines discuss exactly how it can be accomplished.

**Key decisions for Outlines — examples.** Decision to select a specific implementation approach from available options · to limit the scope to certain business functionality · to depart from established Principles for compelling reasons · to deviate from desired Target States due to a lucrative business case or other tactical benefits.

**Architectural directions for Outlines — examples.** The solution should be suitable for all points of presence per the established Principles · should adhere to national privacy Policies · should use certain technologies from Technology Reference Models · should be based on a particular strategic system from the Enterprise System Portfolio.

## Key Takeaways
- Outlines are the business's approval instrument, produced at initiation and archived after.
- Solution Overviews essential, Options Assessments common, Initiative Proposals uncommon.
- Always paired with a business case; the two inform each other.
- Detailed enough to decide, no more — over-elaboration degrades decision quality.

## Connects To
- [Chapter 6 — Processes of Enterprise Architecture Practice](ch06-processes-of-ea-practice.md) — the initiation step of Initiative Delivery
- [Chapter 7 — IT Initiatives and Enterprise Architecture](ch07-it-initiatives-and-enterprise-architecture.md) — the five initiative types whose Outlines differ in origin
- [Chapter 11 — Visions](ch11-visions.md) — Roadmaps that supply planned business needs
- [Chapter 14 — Designs](ch14-designs.md) — what Outlines are elaborated into
- [Chapter 17 — Architecture Functions in Organizations](ch17-architecture-functions-in-organizations.md) — investment committees and exemption procedures
