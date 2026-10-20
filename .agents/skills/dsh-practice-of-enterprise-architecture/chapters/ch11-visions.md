# Chapter 11 — Visions

## Core Idea
**Visions are business-focused structures EA artifacts.** They provide high-level conceptual descriptions of an organization from the business perspective — "shared views of an organization and its future agreed by business and IT". If Considerations are the compass, **Visions are the global map**: they show exactly where the organization needs to go in the long run and what steps get it there. Their purpose is to align IT investments with long-term business outcomes.

## Frameworks Introduced

### Questions Visions answer
What does an entire organization do? · What are its business activities and capabilities? · What is the relationship between main customers, processes, data and systems? · What should IT deliver in the long term? · Which business areas should receive future IT investments? · Which business capabilities should be uplifted with IT? · What types of IT investments should be made? · Which specific business needs should be addressed and when?

### Five subtypes
| Subtype | Prevalence | Definition |
|---|---|---|
| **Business Capability Models** (BCMs; business capability maps/canvases, capability reference models, "business architectures") | Essential | Structured graphical representations of all organizational business capabilities, their relationship and hierarchy |
| **Roadmaps** (investment/capability/application/technology roadmaps) | Essential | Structured graphical views of all planned IT initiatives in specific business areas having direct business value |
| **Target States** (target architectures, future state architectures, business reference architectures, architecture visions) | Common | High-level graphical descriptions of the desired long-term future state |
| **Value Chains** (value reference models, business activity models, process models) | Uncommon | Structured graphical representations of the added value chain |
| **Context Diagrams** (business context diagrams, application diagrams, concepts of operations) | Uncommon | High-level graphical descriptions of the current operational flows |

**Less popular subtypes**: *process models/maps* (heatmappable, focused on processes rather than capabilities); *core diagrams* (one-page depictions of the target operating model with standardized processes, shared data and their relationship — more overarching and far more stable than Target States, used to launch fundamental initiatives and build a reusable digitized platform); *business model canvases* (nine elements: partners, activities, resources, value propositions, customer relationships, channels, customer segments, cost structure, revenue streams); *organizational structures* and *product catalogs*.

### Four aspects of investment alignment Visions address
- **How much money to invest in IT** — the magnitude of expense needed to accommodate strategic demands
- **Where to invest IT dollars** — focus on the most strategically important areas, minimize ineffective expenses
- **What types of IT investments are needed** — identify critical new system types required to execute the strategy
- **When IT investments should be made** — allocate and schedule per strategic priorities and organization-wide plans

## Key Concepts
- **Development.** Part of **Strategic Planning**, collaboratively by senior business executives and architects. Discussions cover which capabilities to improve long-term and which specific business needs to address with IT. Physically created by architects, "mentally shaped by senior business stakeholders". Reviewed, approved and signed off yearly after re-approval of the business strategy.
- **Four uses**: focus future IT investments on strategically important areas; identify potential future initiatives; prioritize initiatives by actual business importance (including selecting among bottom-up local initiatives); determine which initiatives to launch now, supplying **planned business needs** to portfolio management and then Initiative Delivery.
- **Permanent with an "infinite" lifetime** (especially BCMs and Value Chains) — they exist as long as the organization does; updated multiple times a year. Discarded only after major restructurings or business-model transformations.
- **Format.** Brief, informal, easily understandable, simplistic schematic pictures rather than wiring diagrams, **full-colored stylish** rather than monotonous black-and-white technical drawings. Usually simple one-page diagrams; MS Visio, less often PowerPoint.
- **Time focus.** Long-term future, up to 3–5 years, depending on environmental turbulence. Exception: **Context Diagrams are facts artifacts** focused on the current state — yet still change-oriented and still intended to support the strategic dialog.

### Business Capability Models in detail
Hierarchical, typically **two to four nested abstraction levels**. At **level zero** capabilities split into customer-facing ("front office") and operational ("back office"). Top-level structure correlates with the operating model: **coordination** organizes by functional responsibilities of units (enable, manage, run); **replication and unification** by successive value chain activities (supply, production, distribution, sales, service); **diversification** may need completely separate models per independent unit. Capabilities can be titled noun-verb or verb-noun ("Product development" / "Develop products"). Created relatively quickly — perfect correctness and accuracy are not critical qualities; intuitive simple models resonate better with executives. Industry-standard or vendor reference models can accelerate development, and common capability groups (customer management, product management) can be borrowed across industries.

Complex BCMs also carry mission, strategy and vision, long-term goals, objectives, constraints, and external context: key competitors, strategic partners, major suppliers, target markets, core customer groups, industry regulators.

**Heatmapping approaches** — all answer "What improvements are necessary for our organization?":
1. **Collective judgment** of over- and underinvested capabilities → shift focus from overinvested to underinvested.
2. **Intuitive strategic marking** by executives; improvements can be typed (add fundamentally new operations / raise process quality / perform at lower cost) and colour-coded; brand-new capabilities added, obsolete ones removed.
3. **Maturity assessment and gap analysis** — assess current maturity, define required maturity, estimate the gap (delta), colour-code. **Relative** assessments compare with competitors, peers or the industry average (below / on par with / above the industry) and may need external benchmarking. **Absolute** assessments use objective measures often derived from the **Capability Maturity Model (CMM)**: initial, repeatable, defined, managed, optimizing. Executives may weight by strategic importance — focus on low-maturity, high-importance capabilities; safely neglect high-maturity, low-importance ones.
4. **Capability component analysis** — assess the underlying people, processes, information, IT systems and physical facilities separately, revealing that some capabilities need process optimization or staff training rather than new systems.
5. **Strategy mapping** — place strategic goals, objectives, problems and blockers on the model to show which capabilities are crucial to their realization.
6. Other schemes: differentiating vs. non-differentiating (non-differentiating → packaged systems or ERP modules; differentiating → custom in-house applications); core vs. non-core (non-core → outsourcing candidates); safety and regulatory requirements of handled information (cloud migration or strengthened authentication decisions).

Other applications: **initiative "footprinting"** (mapping initiatives to affected capabilities to understand contribution, sponsors, stakeholders, scope, effect and disruption, and to compare initiatives or options); discovering synergies, overlaps, dependencies and duplications across the portfolio; showing where the IT budget goes; providing a common language and shared vocabulary. BCMs are "an 'entry point' to IT for business executives" but, because of their conceptual simplicity, suit **incremental capability improvements** rather than complex structural transformations.

### Roadmaps in detail
Show planned initiatives with approximate start dates, completion timelines and the current point in time. Contain fundamental, strategic and local initiatives. Usually planned **up to three years ahead**, more rarely up to five. Complementary to BCMs: "Business Capability Models operate with business capabilities and help business executives decide **where** future IT investments should go; Roadmaps operate with concrete IT initiatives and help business executives decide **when** these investments should be made." Often organized around capabilities (initiatives then called **business capability increments**), or around major business units, cross-cutting concerns (customer, information), strategic themes (improving customer experience, reducing costs), or core platforms with business meaning (ERP, CRM, BI). Initiatives can be grouped into programs and workstreams, or called **work packages**.

**Initiative approval-status colour-coding**: **Planned** (proposed as an idea, preliminarily approved, placed in the Roadmap, no further work started) → **Approved** (discussed in more detail, early Outlines started to explore options) → **Funded** (sufficiently elaborated, approved on its Outlines, signed off, included in the current program of work) → **Active** (being implemented, project team assembled, Designs agreed). Other colour-codings: order-of-magnitude size (small / medium / large), business sponsors.

Complex Roadmaps also carry strategic business drivers, goals and objectives; expected contributions and outcomes; links to business priorities, problems and pain points; interdependencies; targeted capability components (processes, staff, information, technology); current and desired future states per area; financial estimates (per-initiative price tags, aggregate yearly and per-area investment volumes).

**The industry-standard Roadmap format**: relevant business capabilities with current underpinning systems on the **left**, the same capabilities with desired systems on the **right**, and all the initiatives in between in temporal sequence. Exotic Roadmaps use unusual visualizations more appealing to business or aligned with highly specific needs. Large decentralized companies keep a hierarchy: local unit Roadmaps plus one or a few consolidated organization-wide Roadmaps for C-level or board audiences. Sometimes required for **external regulatory compliance** — in energy, utilities and public sector, organizations may be legislatively obliged to submit long-term IT investment Roadmaps to regulators.

### Target States in detail
Describe some or all business-enabling domains (business, applications, data) and the relationships between them; often planned up to three years, less often five. Two kinds:
- **Evolutionary Target States** — only incremental capability improvements; change the *composition* of the landscape but not its *structure*. Practically materialized as **target system or application portfolios** — the "to-be" portfolio contrasted with the "as-is". Simplest form: mappings of current and target systems to capability areas with colour-coding (local slices of Enterprise System Portfolios). Complex form: broader views of core systems, data repositories and other objects plus textual comments on goals, problems, constraints, expected improvements and outcomes.
- **Transformational Target States** — modify the very structure of the landscape: relationships between core systems, data sources, departments, customer groups, locations. Examples: refocusing sales and distribution channels to different customer segments; consolidating systems after a merger; replacing diverse local systems with a single global platform (changing the operating model); decoupling business units to remove interdependencies (also changing the operating model). Expressed as **connections-based** (lines and arrows) or **overlays-based** (overlaying elements) diagrams.

Target States are more sophisticated than BCMs and Roadmaps, found more often in **mature EA practices**, and require more experienced architects. Transformational states suit large-scale "qualitative" rearrangements; evolutionary states suit "quantitative" capability increments.

### Value Chains and Context Diagrams in detail
**Value Chains** structure organizational activities by value chain position (inbound logistics, operations, outbound logistics, marketing and sales, service). Conceptually similar to BCMs but dynamic rather than static; business activities offer roughly the same abstraction level as capabilities and suit 3–5 year planning. All BCM colour-coding, maturity, gap analysis and strategy mapping techniques apply with minor modification; activities can be benchmarked against best practices, industry averages or competitors on time, quality, cost and other indicators. Their **relative advantage is familiarity** — value chains are taught at business schools and understandable to MBA-holding executives, so they "cause less confusion, discomfort and resistance among senior business leaders".

**Context Diagrams** contain any elements critical for understanding main operational flows: customers, products, services, activities, physical production and storage facilities, business units and functions, geographical locations, workforce, information, IT systems. Unlike other Visions they are **facts** artifacts of the current "as-is" state — yet still support the strategic dialog by exposing problems, bottlenecks, pain points and limitations, and by identifying initiative drivers, owners, stakeholders and organizational impact.

## Mental Models
- **The Strategic Planning sequence.** Many organizations progress from BCMs → Target States → Roadmaps: first heatmap capabilities to specify *what* improvements are required, then plan Target States to show *how approximately* they will be realized with IT, then develop Roadmaps to explain *exactly how* the future will be achieved step by step. BCMs are the most abstract Visions, Target States the next level down, Roadmaps the most detailed.
- **Roadmaps are the main suppliers of planned business needs to Initiative Delivery.** Each initiative in a Roadmap eventually goes through portfolio management and budgeting and turns into a full-fledged instance of Initiative Delivery elaborated with Outlines.

## Anti-patterns
**The single biggest threat: overly detailed and fine-grained Visions.** The environment is volatile, future plans are speculative, and "even technologies that might be available after 3-5 years are rarely known". Per the EA uncertainty principle, detailed long-horizon planning is impractical or unachievable. "There are no practical reasons for investing significant efforts in describing the desired long-term Target States in great detail when even slight shifts in the business environment can instantly invalidate these plans."

**Kotusev's rule of thumb:** *Visions should be detailed enough to help business and IT leaders decide which of the proposed IT investments to fund in the next budgeting cycle, or which initiatives to include into the forthcoming program of work — but no more detailed than that.*

The standard BCM and Roadmap formats naturally resist over-detailing: capabilities can be quickly re-heatmapped and initiatives quickly reprioritized, added or removed when conditions change.

## Reference Tables

### Visions vs. adjacent types
| Compared with | Difference |
|---|---|
| **Considerations** | Considerations are less specific, directive and actionable; they describe only *how* an organization wants to work. Visions provide a high-level direction of *what* it wants to do, specify where investments should focus, when they should be made and what sorts of projects should be initiated. |
| **Landscapes** | Landscapes are highly technical, incomprehensible and irrelevant to business leaders, and focus on documenting the current state. Visions concentrate on the long-term strategic direction endorsed directly by executives and let business leaders guide initiatives indirectly without knowing which systems will be built. |
| **Outlines** | Outlines are more specific, narrow and actionable, describing separate initiatives with limited scopes and timelines. Visions express the global strategic direction and do not explain exactly what needs to be done. |

## Worked Example
**A maturity gap heatmap.** "An organization may assess the present maturity level of a particular business capability as two (on a five-point scale), characterize its desirable target level as four, then estimate the volume of change required to achieve this level (delta) and finally depict this volume in Business Capability Models to indicate where the biggest IT investments should be made."

**A complex Roadmap (Figure 11.4).** Header carries three goals and a problems list; three business functions each with current systems on the left and target systems on the right; initiatives 1.1–1.3, 2.1–2.5 and 3.1 laid across a three-year timeline with a "Now" marker at Q2; dependency arrows ("Start after"); yearly investment totals of \$5–8 million (2021), \$7–13 million (2022) and \$3–5 million (2023); initiative size legend Small (\$0–1m) / Medium (\$1–3m) / Large; an outcomes list on the right.

**Rejecting a local initiative with a BCM.** "If a proposed IT initiative intends to uplift an already mature but strategically insignificant capability, then it is likely to be rejected."

## Key Takeaways
- Visions align IT investment with long-term business outcomes across four dimensions: how much, where, what type, when.
- BCMs = *where*; Roadmaps = *when*; Target States = *what the end state looks like*.
- Heatmapping is the core technique — from intuitive marking to formal maturity gap analysis.
- Keep Visions abstract: detailed enough to fund the next budget cycle, no more.

## Connects To
- [Chapter 5 — The Dialog Between Business and IT](ch05-the-dialog-between-business-and-it.md) — business capabilities and specific business needs as discussion points; the EA uncertainty principle
- [Chapter 12 — Landscapes](ch12-landscapes.md) — Enterprise System Portfolios and IT Roadmaps as the IT-focused twins
- [Chapter 13 — Outlines](ch13-outlines.md) — where Roadmap initiatives get elaborated
- [Chapter 7 — IT Initiatives and Enterprise Architecture](ch07-it-initiatives-and-enterprise-architecture.md) — the initiative types Visions prioritize
