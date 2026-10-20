# Chapter 4 — Enterprise Architecture and City Planning

## Core Idea
The mechanics of an EA practice are explained through a close analogy with **city planning**. Both are continuous activities controlling the evolution of complex, semi-organic systems that cannot be stopped, redesigned and restarted. Out of the analogy come the **six general types of EA artifacts — Considerations, Standards, Visions, Landscapes, Outlines and Designs (CSVLOD)** — the backbone of the entire book.

## Frameworks Introduced

### Table 4.1 — Commonalities between main actors
| EA | City planning | Commonality |
|---|---|---|
| Business executives | City governors | Interested in long-term prosperity from the ultimate-value perspective, but unaware of technical infrastructure |
| IT project teams | Construction project teams | Responsible for completing technical projects on time, but may be unaware of long-term impact and ultimate value |
| Architects | City planners | Accountable for translating the long-term ultimate-value perspective of executives/governors into the short-term technical perspective of project teams |

### The CSVLOD taxonomy (Figure 4.1)
Two axes. **Columns**: Rules → Structures → Changes. **Rows**: business-or-livability perspective (visible ultimate value, used by business executives / city governors to *manage* the landscape) vs. IT-or-urban-infrastructure perspective (invisible technical infrastructure, used by architects / city planners to *organize* the landscape).

| | Rules | Structures | Changes |
|---|---|---|---|
| **Business / livability** | **Considerations** — how is the organization or city organized from the business or livability perspective? | **Visions** — what is the high-level structure from the business or livability perspective? | **Outlines** — what are the specific proposed changes from the business or livability perspective? |
| **IT / urban infrastructure** | **Standards** — how is it organized from the IT or urban infrastructure perspective? | **Landscapes** — what is the high-level structure from the IT or urban infrastructure perspective? | **Designs** — what are the specific proposed changes from the IT or urban infrastructure perspective? |

## Key Concepts — the six types with their city-planning twins

| Type | EA example | City example | Nature |
|---|---|---|---|
| **Considerations** | Architecture principles | Urbanism principles | Abstract high-level imperatives defining the whole organization/city; implicitly dual; permanent, very long-lived; established collaboratively by executives + architects |
| **Standards** | Technology standards | Construction standards | Highly specialized low-level technical guidelines; not dual, invisible to executives; permanent, relatively stable; established collectively by architects |
| **Visions** | Business capability map | Zoning map | Abstract, often one-page diagrams; 3–5 year strategic development plan; implicitly dual; permanent but evolving |
| **Landscapes** | Landscape diagram | Infrastructure map (cables, gas and water pipes) | Formal models/diagrams of the technical landscape; **facts**, may be maintained by individual architects; usually current "as-is"; permanent, updated reactively |
| **Outlines** | Solution overview | Building model | High-level descriptions of separate projects understandable to executives; **explicitly dual**; temporary, short-lived |
| **Designs** | Solution design | Building blueprint | Detailed technical descriptions actionable for implementers; dual (architects ↔ project teams); temporary, limited to project duration |

### Common properties of cities and organizations
Both have visible valuable objects and invisible supporting infrastructure; cannot be perfectly planned in every detail; future needs can be anticipated but not detailed; cannot be built from scratch; run and evolve simultaneously and cannot be stopped; change only through step-wise small increments; major changes take considerable time; evolve slowly and path-dependently; future evolution is limited by current structures and past decisions; poor decisions hinder further development; evolution is endless with no final state; there is no single best way to evolve, only options with different benefits, costs and risks.

### Relationships between the six types (Figure 4.8)
- **Considerations** influence everything: the development of Visions, the *selection* of Standards, the evolution of Landscapes, and the architectures in Outlines and Designs.
- **Standards** provide *guidelines* for Outlines and Designs, and thereby *eventually shape* Landscapes.
- **Visions** *initiate* the *structure* of Outlines; also guide Standards selection, Landscape evolution and Designs.
- **Landscapes** provide the *environment* for new systems described in Outlines and Designs.
- **Outlines** *provide the basis* for Designs.
- **Designs** are shaped by all the others and influence none of them directly — but provide the basis for updating Landscapes after project completion.

## Mental Models
- **Managing vs. organizing.** Considerations, Visions and Outlines let executives *manage* the landscape (set fundamental rules, define strategic direction, approve tactical steps). Standards, Landscapes and Designs let architects *organize* it (reuse proven approaches, leverage assets, plan technical change in detail).
- **Communication interfaces.** Considerations, Visions and Outlines are the interface between business executives and architects. Designs are the interface between architects and project teams. Standards and Landscapes are pure reference materials used only by architects.
- **Practising EA implies not "architecting" organizations, but discussing their desired evolution.**

## Reference Tables

### Figure 4.9 — Complementarity of the six types
| | Considerations | Standards | Visions | Landscapes | Outlines | Designs |
|---|---|---|---|---|---|---|
| Examples | Architecture principles / urbanism principles | Technology standards / construction standards | Business capability maps / zoning maps | Landscape diagrams / infrastructure maps | Solution overviews / building models | Solution designs / building blueprints |
| Who uses them | Business executives and architects | Only architects | Business executives and architects | Only architects | Business executives and architects | Architects and IT project teams |
| How used | Established collaboratively, then influence all planning decisions | Established, then influence all project designs and the landscape structure | Developed collaboratively, then direct future investments and projects | Maintained current to support technical decision-making and project planning | Developed collaboratively, then inform project funding decisions | Developed collaboratively, then guide project implementation |
| Why used | Improve overall conceptual consistency and mutual alignment | Reduce complexity, risks and costs; achieve homogeneity and compliance | Improve strategic effectiveness of future investments | Rationalize infrastructure, reuse assets, accelerate project planning | Improve cost-benefit ratio and efficiency of all investments | Improve quality of project delivery according to requirements |
| Dual | Yes | No | Yes | No | Yes | Yes |
| Meaning | Decisions | Decisions | Decisions | Facts | Decisions | Decisions |
| Lifecycle | Permanent | Permanent | Permanent | Permanent | Temporary | Temporary |

## Worked Example
**The dual reading of a Consideration.** The architecture principle *"All lines of business work with the shared list of customers"*: to business executives it means their company can cross-sell products offered by different lines of business to the same customers; to architects the same principle means all IT systems supporting different lines of business should use the single shared customer database.

**Kotusev's sample artifact contents (Figures 4.2–4.7):**
- Architecture principles: "Business processes are standardized across all points of presence"; "All lines of business work with the shared list of customers"; "All business operations are maintained despite system failures". Urbanism twins: "All locations are accessible via public transport and private vehicles"; "All buildings adhere to the traditional architectural style"; "All buildings are commensurable with the pedestrian scale".
- Technology standards: "All applications should be implemented on the Java EE platform"; "All databases should use the Oracle RDBMS platform"; "Enterprise Service Bus should be used to integrate all applications". Construction twins: "High alumina cement should be used for brick masonry"; "Plastic glasses should be used in all residential buildings"; "All traffic lanes should be 3.0 meters wide".
- Solution overview sections: 1 Overview and Goals · 2 Scope and Stakeholders · 3 Essential Requirements · 4 Business Benefits · 5 Capability Impact (Order Fulfilment High, Order Management Low, Customer Analytics Low) · 6 Involved Partners (IBM, Accenture) · 7 Estimations (Time 6–8 months; Cost \$1.2–1.5 million) · 8 Business Process Changes (Process 1 now: 3 days + 5 days + 2 days = 10 days; will be: 2 days + 3 days = 5 days) · 9 Architectural Overview · 10 Key Risks. Building-model twin: Type Commercial, Location Zone 2, Height 9 storey, Total area 2300 m², Cost \$50–60 million, Date 2022Q4, Contractor BuildCo.

**What the combined use guarantees for every IT project:** fulfils local short-term needs (approved by executives via Outlines, requirements documented in Designs) · contributes to long-term goals (Outlines aligned and prioritized against Considerations and Visions) · implemented predictably and risk-free (Designs follow Standards) · reuses available assets (Outlines and Designs leverage Landscapes) · does not create redundant assets (checked against Landscapes) · reusable later (Designs allow reusability) · built on technologies the organization wants to keep (Standards) · consistent with similar projects (Standards + Considerations) · no complexity beyond necessity (Standards + Landscapes).

## Key Takeaways
- Six types, two axes: what is described (rules / structures / changes) × for whom (business-visible value / invisible technical infrastructure).
- Considerations, Visions, Outlines = business control levers. Standards, Landscapes, Designs = architect tools.
- The six types are complementary and synergistic — none of them works alone.

## Connects To
- [Chapter 2 — The Concept of Enterprise Architecture](ch02-the-concept-of-enterprise-architecture.md) — duality, decisions/facts, permanent/temporary properties reused here
- [Chapter 8 — The CSVLOD Model of Enterprise Architecture](ch08-the-csvlod-model.md) — the formal two-dimensional taxonomy and 24 subtypes
- [Chapter 6 — Processes of Enterprise Architecture Practice](ch06-processes-of-ea-practice.md) — the three processes that pair the six types
