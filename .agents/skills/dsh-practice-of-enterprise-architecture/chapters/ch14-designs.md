# Chapter 14 — Designs

## Core Idea
**Designs are IT-focused changes EA artifacts.** They provide detailed technical and functional descriptions of separate IT projects actionable for project teams and act as the **communication interface between architects and IT project teams**. Designs are the **"last" EA artifacts in the EA delivery chain** — "the only existing mechanism in an EA practice to convert all intangible architectural decisions reflected in other types of EA artifacts into tangible IT systems." Without peer-reviewed Designs, "most other artifacts are often ignored, while the money invested in these artifacts are usually wasted."

## Frameworks Introduced

### Questions Designs answer
What specific business requirements should be met? · What infrastructure should be provided? · What hardware and software should be installed? · What applications should be developed? · Which data entities should be used? · How exactly should system components communicate and interact? · How exactly should the new system interact with the surrounding environment? · How exactly should current business processes be modified?

### Two subtypes
| Subtype | Prevalence | Definition |
|---|---|---|
| **Solution Designs** (detailed/technical/physical/high-level designs, project-start architectures, solution architectures, full solution architectures, solution definitions, solution specifications) | Essential | Detailed technical and functional specifications of approved IT solutions actionable for project teams |
| **Preliminary Solution Designs** (preliminary solution architectures, solution architectures, logical designs) | Uncommon | Preliminary high-level technical and functional designs of specific approved IT solutions |

**Less popular subtypes**: *key design decisions (KDDs)* — one-page summaries of the most significant technical decisions and justifications; *architectural directions for Designs* — purely technical requirements extracted from Technology Optimization; *mini-designs* — a few pages for minor solutions that merely extend existing functionality, deviating from standard structure and approved via simplified procedures, "helps reduce the level of architectural overhead and bureaucracy"; *business requirements documents (BRDs)* — detailed functional, non-functional and architectural requirements separated out, prepared by business analysts with architects and business representatives, serving as formal contracts with delivery partners; *technology-specific supplementary materials* (e.g. ERP configuration documents, integration bus routing-rule XML files); *release designs* — for lean organizations that fully outsource delivery, bundling all changes of one platform delivered by one partner in one release cycle ("produced on a per-release, per-partner and per-platform basis" rather than per project).

## Key Concepts
- **Development.** At the **implementation step** of Initiative Delivery, collaboratively by architects, IT project teams and business representatives, based on Outlines previously agreed with business executives. For small and medium solutions delivered as single projects, Designs describe the complete end-to-end solution and "may be produced merely by extending Outlines, i.e. via adding new, more detailed technical sections to existing Outlines". Large solutions get one Design per constituent project.
- **Three parties, three concerns**: *architects* — conformance to Standards and seamless integration into Landscapes, plus incorporating technical rationalization suggestions from Technology Optimization; *IT project teams* — feasibility, practicality and credibility from an implementation-centric perspective, plus adequate schedules and resource availability; *business representatives* (actual owners or business analysts acting for them) — correct, consistent and complete functional requirements.
- **Discussion point: business requirements.** Techniques used: user stories, traceability matrices, and the **MoSCoW** framework (must have, should have, could have, will not have).
- **Domain coverage.** End-to-end: "the entire IT project stack starting from the 'top' business layer (business requirements, use cases and processes) and ending with the 'bottom' infrastructure layer (hardware, operating systems and networks)". Different project types shift the focus — application projects concentrate on applications and data; infrastructure projects on infrastructure. Designs may also formulate **local project-specific architecture principles, patterns and guidelines** limited to the respective system.
- **Governance.** Peer-reviewed by other architects for fit into Landscapes and compliance with Standards, then formally approved and signed off under project governance. Reasonable justified deviations tolerated, registered, later analysed for organizational learning; incurred architecture debt may be estimated and recorded. In highly regulated industries (banking, finance) finalized Designs may need to be sent to national regulators for compliance verification and endorsement. Critically: "at the Designs stage of IT initiatives, the fundamental investment decisions to implement these initiatives have already been made... based on their high-level Outlines."
- **Time focus**: short-term future, up to one year ahead, seldom longer for large projects.
- **Format**: mix of text, tables and complex diagrams; long textual requirement descriptions, extensive configuration tables with technical parameters, sophisticated IT-specific diagrams using **UML or ArchiMate**, less often **BPMN or ARIS**. Almost always MS Word.
- **Lifecycle.** Temporary. After release, Designs are **updated to reflect the "as-implemented" state**, sometimes with additional lower-level details, and stored in searchable document repositories. They then serve as passive reference materials on the current landscape — for user training, and for operations and support teams monitoring, maintaining and troubleshooting running systems. "While Landscapes describe at a high level what IT systems constitute the IT landscape and how these systems are connected to each other, Designs describe in great detail how exactly each of these systems works internally."

### Designs vs. project management plans
| | Designs | Project management plans |
|---|---|---|
| Nature | Highly IT-specific architectural documents driven by architects | Largely non-technical documents driven by project managers |
| Content | Low-level descriptions of planned systems, composition in technical terms, volume of development effort | Milestones and schedules, procurement, staffing, budgeting, stakeholder management, risk mitigation, quality assurance |
| Methods | Count components, estimate person-hours per component, produce a work breakdown structure; or send specifications to contractors for precise price quotations | Managerial considerations for delivering the project |
| Purpose | Describe **what** systems and components should be developed | Explain **how, when and with what resources** they should be delivered |

Developed together; managerial considerations are based on Designs, and Designs are influenced by feasibility, financial, personnel-availability and time restrictions.

### After delivery
Systems are handed over to IT operations and support teams — "the corresponding systems enter from the delivery phase into the realm of IT service management." Architects update Landscapes; CMDB-based Landscapes (most often Inventories) are usually updated directly by operations and support as part of the transition. Business owners then organize follow-up people and process activities: educate end users, assign new decision-making responsibilities, lead process modification, address conflicts arising along the transition.

## Mental Models
- **Designs = system architecture.** "Designs as EA artifacts are equivalent to what is typically called 'system architecture', i.e. the architecture of a single IT system. Hence, Designs in the context of this book should be understood as full-fledged system architectures, rather than merely as 'designs'."
- **Organization-wide significance is the filter.** Designs "tend to define only what is really important from the perspective of the whole organization, while all minor decisions negligible in the overall organizational context are normally omitted" and left to project team discretion — those lower-level design documents are not EA artifacts.
- **Designs de-risk delivery** by providing a "single source of truth" to all project participants and by surfacing risks and mitigation strategies beforehand.

## Anti-patterns
**The single biggest threat: excessively detailed Designs, from the inability to distinguish "architecture" from "design".** Fine-grained voluminous Designs require unreasonable time and effort and often end in **analysis paralysis** — "an enormous number of available planning options merely overwhelm planners and prevent them from making any specific decisions." (Developing new information systems is a "wicked problem" where the perfect solution cannot be found analytically.) Even when produced, meticulous Designs are viewed as too rigid, inflexible and bureaucratic by project teams.

### Kotusev's three distinctions between architecture and design
| Architecture | Design |
|---|---|
| Concerned with the **fitness for purpose** | Concerned with the **engineering optimization** |
| Consists of choices **dictated by needs** | Consists of choices **compliant with architectures** |
| Different architectures address **different basic needs** | Different designs can address **the same basic need** |

**Three aspects Designs must always specify**, regardless of the desired level of agility: **utilized technologies, interactions with the external IT environment, and security considerations.** "All these project-level decisions can have substantial consequences for an entire organization; none of them can be neglected or simply left at the discretion of the project teams."

## Reference Tables

### Solution Design sizing
| Delivery methodology | Design characteristics |
|---|---|
| Agile methodologies | Brief and lean; only critical technical information — key technologies, main components, integration with external applications and data sources |
| Traditional, waterfall-like | Extensive Solution Designs with detailed project implementation plans |
| "Average" | **25–50 pages**; in extreme cases a few hundred pages |
| Preliminary Solution Designs | "Average" about **20–40 pages** |

Infrastructure projects involving procurement and installation of costly equipment are likely to need thoroughly planned Solution Designs specifying hardware configuration and precise capacity before any implementation begins.

### Designs vs. adjacent types
| Compared with | Difference |
|---|---|
| **Landscapes** | Landscapes are broader in scope and more abstract in granularity, focused on the current environment. Designs focus on separate projects in great detail, describe only near-future delivery, and "explain exactly how separate changes in Landscapes are going to be implemented." |
| **Outlines** | Outlines present and "sell" entire solutions to senior business stakeholders and give only conceptual requirements. Designs may cover separate components of large solutions, specify detailed business requirements, explain exactly how they should be met and how the resulting systems should work, with sufficient low-level detail to start implementation. |

### Standards and Landscapes referenced by Designs
Designs often explicitly refer to the Standards they conform to — **Patterns**, **Guidelines** and **Logical Data Models** — and integrate with the systems shown in **Landscape Diagrams** and the assets listed in **Inventories**.

## Worked Example
**Solution Design structure (Figure 14.1).** *Application design*: 1 Brief Overview · 2 Goals and Objectives · 3 Detailed Requirements · 4 Solution Context · 5 Data Architecture · 6 Application Architecture · 7 Infrastructure Architecture (Linux Server 1 with VM 1 and VM 2, firewall, switch, Linux Server 2 as hot standby). *Infrastructure design*: 1 Introduction · 2 Business Problem · 3 Solution Specification · 4 Contextual View · 5 Deployment View · 8 Utilized Technologies · 9 Necessary Equipment (Hardware A qty 3, Hardware B qty 2, Appliance C qty 5).

Solution Designs are "structured into multiple chapters, sections or views corresponding to the relevant EA domains, e.g. usage scenario view, application component view, data processing view and deployment view."

**Preliminary Solution Design structure (Figure 14.2).** 1 Brief Overview · 2 Goals and Objectives · 3 Scope and Stakeholders · 4 Business Requirements · 5 Involved Partners (IBM, Accenture) · 6 Key Technologies (Java EE Platform; IBM BPM and DB2) · 7 Accurate Estimations (Time 8–9 months; Cost \$2.1–2.3 million) · 8 Technical Risks · 9 High-Level Architecture · 10 Application Interaction.

**What Preliminary Solution Designs are for.** They are "intermediate 'halfway' EA artifacts between Outlines... and Solution Designs". Their primary purpose is to refine and reaffirm the earlier Outlines-based time and cost estimates. "If the refined estimates confirm the original Outlines-based estimations, or at least do not differ from them significantly, then corresponding IT projects can smoothly proceed further to developing more detailed Solution Designs in a regular manner. However, if the updated estimates for IT projects are dramatically different from the earlier estimates produced during their initiation steps, then these projects may need to be renegotiated with their executive business sponsors and even the very decision to implement these projects might need to be reconsidered."

**Key design decisions (KDDs) — examples.** Decision to select a specific technical implementation approach or reuse a proven Pattern · to implement the project based on existing IT assets from current Inventories · to use a non-standard technology missing in Technology Reference Models due to the unsuitability of supported technologies for unique project needs · to deviate from typical best-practice Guidelines because of the project's highly specific nature.

**Architectural directions for Designs — examples.** The system should store specific data entities in the format defined by Logical Data Models · should follow critical data encryption Guidelines · should interact with existing systems shown in Landscape Diagrams · should not reuse certain classes of IT assets included in Inventories.

**Designs in outsourcing.** External partner architects join development; Designs become the key communication instrument between internal and external specialists, the basis for RFTs and RFQs (including hardware and network equipment procurement), the formal contract, the estimation base before implementation, and the specification against which contractors' work is validated, accepted or rejected.

## Key Takeaways
- Designs are the conversion point where all architectural intent becomes real systems.
- Solution Designs essential; Preliminary Solution Designs uncommon and used to re-baseline estimates.
- Keep Designs at the architecturally significant level — always specify technologies, external interactions and security.
- Update Designs post-implementation; they become reference material complementing Landscapes.

## Connects To
- [Chapter 13 — Outlines](ch13-outlines.md) — what Designs are elaborated from
- [Chapter 10 — Standards](ch10-standards.md) — the rules Designs must conform to
- [Chapter 12 — Landscapes](ch12-landscapes.md) — the environment Designs fit into and update
- [Chapter 18 — Instruments for Enterprise Architecture](ch18-instruments-for-enterprise-architecture.md) — modeling languages, project checklists, agility and Design volume
