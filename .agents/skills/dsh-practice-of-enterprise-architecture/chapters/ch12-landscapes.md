# Chapter 12 — Landscapes

## Core Idea
**Landscapes are IT-focused structures EA artifacts.** They provide high-level technical descriptions of the corporate IT landscape and act as **a knowledge base of reference materials** for architects — a shared organizational repository describing what IT assets exist, how they are connected and how they are used. Landscapes are the only general type that is predominantly **facts** rather than decisions, and the chapter's strongest warning is that they must **not** be used for strategic planning.

## Frameworks Introduced

### Questions Landscapes answer
What IT systems, databases and infrastructure are available? · How are existing assets connected? · What is the information flow and interaction between them? · How are they used to support business capabilities and processes? · Which are duplicated, unused or redundant? · Which are considered strategic or legacy? · Which should be reused or decommissioned? · What technical improvements are required and when?

### Four subtypes
| Subtype | Prevalence | Definition |
|---|---|---|
| **Landscape Diagrams** (architectural repository; relational diagrams, system interaction diagrams, information exchange diagrams, one-page diagrams, platform architectures, enterprise system models, integration contexts) | Essential | Technical "boxes and arrows" schemes of different scopes and granularities describing the IT landscape |
| **Inventories** (asset inventories/registers, registries, master lists, item profiles, architectural repositories; or domain-specific: application inventories, infrastructure inventories) | Common | Structured catalogs of currently available IT assets describing their essential properties and features |
| **Enterprise System Portfolios** (applications-to-capabilities mappings, capability models with system overlays, application portfolios/models, IT system value maps, IT strategy maps) | Common | Structured high-level mappings of all essential IT systems to relevant business capabilities |
| **IT Roadmaps** (technology/platform/infrastructure/integration roadmaps) | Common | Structured graphical views of all planned IT initiatives of a purely technical nature having no visible business impact |

**Less popular subtypes**: *asset roadmaps* (lifecycle milestones per asset — "a mixture of Inventories and IT Roadmaps"); *IT target states* (technical counterparts of Target States, covering integration, infrastructure and security, developed by architects with senior IT stakeholders — "should be developed with great caution"); *enterprise data portfolios* and *enterprise infrastructure portfolios* (capability overlays for data and infrastructure; data portfolios are effectively very high-level master data maps); *IT capability models* (application, data or infrastructure capabilities, heatmapped for gaps); *extended inventories* (clusters of related assets with textual descriptions, development strategy, current-state and future-state diagrams — "a mix of regular Inventories and Landscape Diagrams"). **Architecture debt registers** can also be loosely related to Landscapes.

### Landscape Diagram taxonomy
- **Multidomain** — combine objects from several domains and explain their interrelationship.
  - **Free-form diagrams** — no particular structure; objects placed anywhere and connected to anything. Advantages: simplicity, flexibility, freedom of expression. Risk: overly complicated, entangled, hard to understand.
  - **Layered diagrams** — structured into layers aligned with the stack of EA domains; business entities at the top, infrastructure at the bottom, applications and data in between; often only adjacent-layer connections permitted. Advantages: clarity, transparency, orderliness, amenability to standardization. Risk: too rigid, burdensome, unnecessary.
- **Domain-specific** — in-depth slices of a single domain. Common examples: **application interaction diagrams** (exactly how applications "talk" to each other) and **master data maps** (master data sources and their replicas per data type, where information can be found and how retrieved).

Landscape Diagrams vary in **granularity** (only the most critical systems → all systems including minor ones), **scope** (narrow areas supporting single activities → entire lines of business; in small companies the whole landscape on one page), **domains** (any single domain or any combination) and **elements** (customers, processes, roles, services, applications, communication interfaces, databases, data types, integration platforms, physical and virtual servers, network equipment, interaction protocols).

## Key Concepts
- **Development.** Part of **Technology Optimization**, largely by architects alone — mostly gathering, aggregating and materializing existing information. Sources: questioning people acquainted with parts of the landscape (users, developers, support teams); studying project documentation (Designs of implemented systems); searching information repositories (portals, wikis, CMDBs); extracting facts from IT management and monitoring tools (issue trackers, network monitoring, project, budget and license management software). IT Roadmaps are the exception — they contain planning decisions and are developed collaboratively by architects and senior IT managers, requiring formal approval under technology governance.
- **Created reactively, then maintained.** Because comprehensive Landscapes take significant initial effort, new ones are created when needed for a specific purpose and then kept up to date. "Organizations typically start their EA practices with very patchy Landscapes, slowly fill the gaps as required and eventually cover the entire IT landscape comprehensively." Updates are usually triggered by structural changes (deployment of new systems per their Designs), accidental discovery of inconsistencies, or automated change-detection notifications from CMDBs or monitoring tools.
- **Three uses**: rationalize the landscape structure (find duplicated, unused, redundant, overly complicated or messy areas → produce technical rationalization suggestions); manage the lifecycle of assets (retain / remove / treat as strategic; identify obsolescent or unsupported systems and retire them without disturbing operations); plan Outlines and Designs (what other systems to interact with, where input data comes from, where output goes, where to deploy).
- **Almost never discarded**, unless a considerable part of the landscape is decommissioned at once. Chief custodians are architects, but Inventories may also be updated by IT operations, maintenance and support staff.

### Inventory asset properties (Kotusev's list)
**Purpose** (from the business perspective) · **Technology** (full stack and components) · **Unit** (owning organizational unit) · **Owners** (business and IT owners accountable) · **Lifetime** (since introduction) · **Cost** (estimated annual maintenance: license fees, hardware costs, full-time IT staff to support) · **Fitness** (for current and future business purpose) · **Problems** (identified problems and potential risks). Type-specific properties: licensing terms for software, compliance requirements for data, billing policies for cloud assets.

### Inventory asset status categories
| Status | Meaning |
|---|---|
| **Reuse** | "Healthy" assets in use, maintained in the future, safely reusable |
| **Invest** | Strategic assets in use, further enhanced, reuse highly encouraged |
| **Maintain** | "Toxic" assets in use, still maintained, reuse highly discouraged |
| **Decommission** | Legacy or problematic assets in use, decommissioned shortly, should not be reused |
| **Undecided** | No specific decisions have yet been made |

### Enterprise System Portfolio system status categories
| Status | Meaning |
|---|---|
| **Active** | Actively used, adequately fit for current purpose, maintained in future |
| **Strategic** | Actively used, fit for strategic business needs, further expanded |
| **Legacy** | At their "sunset", actively used but unfit for purpose, need replacement |
| **Inactive** | Currently deployed but not actively used, can be safely decommissioned |
| **Unclear** | Role in the current and future landscape not yet clear |

Other Enterprise System Portfolio colour-codings: how problematic systems are to the business, suitability for functional requirements, technical quality, overall architectural fitness; plus notations or comments for externally hosted systems, systems handling sensitive information, cloud-migration candidates, compliance issues, annual maintenance costs, total user count.

### IT Roadmaps in detail
IT-oriented counterparts of Roadmaps, focused on integration and infrastructure, planned for shorter horizons — **often only 1–2 years**. Their initiatives are planned technical rationalization suggestions from Technology Optimization; primary stakeholders are senior IT managers, often including the CIO. Typical operations: upgrade end-user software or operating systems; replace aging server or network infrastructure; introduce new middleware or service integration platforms; relocate systems to better data centres; decommission unsupported, redundant or unused assets; consolidate systems with similar functionality; streamline overly complicated areas; apply security patches. Organized around technology areas and domains (middleware, connectivity, authentication) rather than business areas.

**Funding colour-coding**: **CIO** (agreed, sponsored and funded by the CIO) · **Business** (sponsored by senior business stakeholders who understand their importance) · **Not Sponsored** (separate funding required, sponsors missing) · **Not Required** (no sponsorship or separate funding needed). Three funding routes in order of preference: opportunistically as part of regular business initiatives (usually no extra funding); standalone architectural initiatives financed directly by the CIO; or "sold" to business leaders. Separate IT Roadmaps suit medium and large organizations; small companies place housekeeping initiatives in business Roadmaps with distinguishing colour-coding.

Unlike all other Landscapes, **IT Roadmaps are decisions EA artifacts**.

## Mental Models
- **Interior vs. exterior.** "Standards concentrate more on the 'interior' of IT solutions, whereas Landscapes focus more on their 'exterior'."
- **Landscapes are instruments for capturing knowledge, not means of strategic planning.**
- **Higher-level and up-to-date beats detailed and outdated.**

## Anti-patterns
**Threat 1 — misusing Landscapes for strategic planning.** Business executives define the long-term direction and control the budget; IT executives respond within the allocated budget. Landscapes are technical and cannot be used in discussions with senior business stakeholders, so any future states described in them "usually cannot be understood, consciously approved and sponsored by business executives", while IT executives lack the budget for large initiatives comprehensible only to IT. "Idealistic global future-state Landscapes created by architects and senior IT stakeholders alone... often end up in an 'ivory tower' and never get implemented."

The correct order: **only after** business executives have specified target capabilities in Visions can architects place new planned systems in Enterprise System Portfolios; **only after** implementation has been approved based on Outlines can new applications or databases appear in Landscape Diagrams — "but not in the opposite order." "All the attempts to create future-state Landscapes ahead of Visions essentially represent the attempts of IT to lead business and can be considered risky and dangerous. Put it simply, IT-specific target states either should not be developed at all, or at least should be developed with great caution."

**Threat 2 — excessively detailed and fine-grained Landscapes**, especially Landscape Diagrams and Inventories. Development is tedious and time-consuming; architects who spend a substantial portion of their time merely describing the existing environment "might be considered futile and lose their credibility in the eyes of senior business and IT leaders". Maintenance is worse: **major changes involve architects and are trackable, but minor changes are often implemented without their involvement or notification** and happen unnoticed. So overly detailed Landscapes "tend to inevitably become stale", and stale information undermines trust in the whole knowledge base.

**The remedy**: focus on architecturally significant details that do not change often and are not expected to change without architects' involvement. Lower-level details, when needed, come from archived Designs, daily users, operations teams, or reverse-engineering source code and hardware configuration.

## Reference Tables

### Benefits of Landscapes
| Benefit | Mechanism |
|---|---|
| Increased reuse of IT assets | Help architects identify suitable assets to be reused in new initiatives |
| Reduced duplication of IT assets | Help identify duplicated or redundant assets and eliminate them |
| Decreased dependence on legacy IT systems | Help identify fragile legacy systems and decommission them opportunely |
| Improved IT agility | Provide a baseline of the current landscape, accelerating the planning of new solutions |

### Landscapes vs. adjacent types
| Compared with | Difference |
|---|---|
| **Standards** | Standards define abstract rules for all assets of a type; Landscapes distinguish specific tangible instances and describe their connections — snapshots explaining the logical relationships between servers, platforms, databases, applications, processes and capabilities. |
| **Visions** | Visions are conceptual, speculative and future-focused and support communication with senior business stakeholders. Landscapes document the actual current state technically; they are largely useless for business discussions but essential for IT-centric planning and optimization. |
| **Designs** | Designs are more granular, narrow and actionable, presenting one IT project in depth. Landscapes give higher-level views of the whole landscape or major parts of it and describe cohesive sets of multiple systems and connections — the general technical context, without enough implementation detail to deliver anything. |

### Storage options
MS Visio drawings (Landscape Diagrams, Enterprise System Portfolios) · MS Excel spreadsheets (Inventories) · configuration management databases (CMDBs) · specialized tool-based EA repositories (a comprehensive graph of interconnected entities). Landscape Diagrams often use ArchiMate, less often ARIS. **Inventories are the only EA artifacts typically represented in a tabular form.**

## Worked Example
**Facts and decisions in one artifact.** Inventories: "as facts EA artifacts, they accurately capture the list and properties of the existing IT assets and can easily be updated by individual architects if new assets are identified. On the other hand, as decisions EA artifacts, they suggest which IT assets should be reused in the future and the corresponding decisions to consider particular assets as strategic or legacy can be made only collectively by asset owners and all other relevant stakeholders." The same duality applies to Technology Reference Models and Enterprise System Portfolios.

**Synchronizing initiatives via Inventories.** "During the planning of Roadmaps and IT Roadmaps, architects can mark the IT assets affected by the corresponding IT initiatives with their estimated start and completion dates in order to ensure that different initiatives do not attempt to modify the same entities at the same time."

**Enterprise System Portfolio as "landscape on a page".** They map core systems to the capabilities they support, mirroring the structure of the organization's Business Capability Models (or Value Chains where those are used instead). They normally abstract internal system structure, though detailed variants reveal essential components. They can periodically be shown to senior business stakeholders as an intuitive graphical representation of the corporate IT landscape — "but they may be too conceptual to be helpful during the technical planning of new IT solutions."

## Key Takeaways
- Landscapes are the architect's knowledge base, not a planning instrument.
- Landscape Diagrams essential; Inventories, Enterprise System Portfolios and IT Roadmaps common.
- Never lead business with future-state Landscapes; derive any future content from Visions and Outlines approved earlier.
- Choose an abstraction level you can actually keep current.

## Connects To
- [Chapter 10 — Standards](ch10-standards.md) — the complementary IT-focused rules
- [Chapter 11 — Visions](ch11-visions.md) — Business Capability Models that Enterprise System Portfolios mirror; Roadmaps vs. IT Roadmaps
- [Chapter 14 — Designs](ch14-designs.md) — the source of Landscape updates at project completion
- [Chapter 18 — Instruments for Enterprise Architecture](ch18-instruments-for-enterprise-architecture.md) — EA tools vs. CMDBs; landscape complexity measurements; architecture debt registers
