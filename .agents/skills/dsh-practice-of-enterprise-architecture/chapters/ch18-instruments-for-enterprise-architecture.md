# Chapter 18 — Instruments for Enterprise Architecture

## Core Idea
The "technical" side of an EA practice — modeling languages, software tools, quantitative measurements, templates, straw-man architectures, project checklists, architecture debt and "agile" EA. The chapter's unifying verdict: **modeling languages and software tools play only secondary and supporting roles**. Notations are a "hygiene factor" at best and an obsession at worst; tools "cannot organize an EA practice for people". What genuinely helps are the lightweight management instruments — templates, straw-men, checklists, debt registers — and a conscious choice of where to sit on each **dimension of agility**.

## Frameworks Introduced

### The four relevant modeling languages
| Language | Origin | Covers | Verdict |
|---|---|---|---|
| **ArchiMate** ("architecture" + "animate") | Telematica Instituut (Netherlands) 2002; transferred to The Open Group 2008 | Business, application and technology layers (loosely mapping the EA domain stack), each in active structure / behavior / passive structure aspects; plus strategy, implementation-and-migration, physical and motivation extensions | Too detailed and formal for executive-level Visions; business stakeholders find it "excessively complex". Despite being promoted as *the* EA language, "its real practical applicability is largely limited only to Landscapes and Designs" |
| **UML** | Rational Software 1994; OMG 1997; ISO standard 2005 | Structural diagrams (class, component, composite structure, deployment, object, package, profile) and behavioral diagrams (activity, communication, interaction overview, sequence, state machine, timing, use case) | Very detailed, technical, formal — incomprehensible to business, too low-level even for Landscapes. Widely used for **Solution Designs** |
| **BPMN** | BPMI 2004; OMG from 2005 | Business processes as step-wise activities in swimlanes, plus events, gateways and artifacts | No notations for applications, data or infrastructure. Inapplicable to Visions and Landscapes. Useful in **Outlines and Designs** to show process change — usually in **simplified, cut-down versions** for Outlines |
| **ARIS** | August-Wilhelm Scheer 1980s; IDS Scheer; Software AG from 2009 | Functional, organizational and data views plus the integrating **control view** | Mature and widely adopted but "presently arguably losing its popularity". Usable in **Designs and Landscapes**; too formal for Visions and Outlines |

### Applicability grid (Figure 18.5)
Considerations and Standards are largely **textual** and need no full-fledged diagrams. **Visions** require high-level, intuitive, conceptual visualizations "not offered by any modeling languages". The languages reach only **Designs, Landscapes and (partly) Outlines**.

### Three categories of software tool
| Category | Examples | Role |
|---|---|---|
| **MS Office suite** | Word, Visio, Excel, PowerPoint (equivalents: LibreOffice, FreeOffice, iWork, Google Docs/Sheets/Slides) | The mainstay. Simple, familiar, ubiquitous, flexible, standard file formats. Most EA artifacts end up as ordinary MS Office files even when their diagrams were generated elsewhere |
| **Specialized EA tools** | Abacus (Avolution), Alfabet (Software AG), Enterprise Architect (Sparx Systems), Enterprise Studio (BiZZdesign), HOPEX (MEGA International), iServer (Orbus), System Architect (UNICOM Systems), Troux (Planview); free/open-source **Archi** (mostly for teaching) | Architectural repository (a meta-model-governed graph of elements and relationships), modeling and visualization, analysis — above all **impact analysis**. Plus querying/reporting, XML import-export, access control, versioning, locking, check-in/check-out, workflow, alerts |
| **CMDBs** | Configuration management databases owned by IT operations and support; assets are **configuration items (CIs)** | Auto-discovery, native integration with change/release/deployment/asset management, a single source of truth shared by architects and operations |

## Key Concepts
- **Why modeling languages are secondary — Kotusev's five reasons.** (1) Perfectly correct modeling is not the goal; usefulness is determined by support for communication and decision-making, and "less formal modeling approaches allow some interpretive flexibility and ambiguity which is often conducive to reaching agreements". (2) The most critical stakeholders — business executives — do not understand formal notations. (3) Drawing stakeholder-friendly diagrams "is more art than engineering or science". (4) Full versions of languages are rarely if ever used; they are cut down to essentials. (5) The obsession with correct modeling substitutes the genuine goal with a false objective — **"While the proper use of modeling languages alone cannot guarantee the success of an EA practice, the obsession with correct modeling can easily guarantee its failure."**
- **Why software tools are secondary — two reasons.** (1) The communication problems EA solves are unrelated to tooling: "software tools cannot help architects establish effective communication with pertinent stakeholders and make balanced planning decisions." (2) An EA practice is a set of interrelated processes, actors and documents requiring mandate, involvement and institutionalization — "software tools cannot organize an EA practice for people."
- **Adoption statistics.** "Around two-thirds of organizations use some specialized EA tools, while the remaining one-third of organizations still rely only on general-purpose, non-EA-specific tools."
- **Tools correlate with facts, not decisions.** "The capabilities of EA-specific tools highly correlate with the purpose of facts EA artifacts, but do not correlate with the purpose of decisions EA artifacts." A repository can effectively substitute Inventories, Landscape Diagrams, Technology Reference Models and Logical Data Models as standalone documents, and capture the interrelationships between them.
- **When to introduce an EA tool.** "Not at the outset of an EA practice, but only at its later stages when the need for these tools is widely understood." Start with Visio drawings and Excel spreadsheets; migrate when consistency becomes too burdensome and a clear "business case" exists. If a populated CMDB is already integrated with IT service management, an extra EA tool "may not be worthwhile due to its minimal potential added value." **"If an EA practice does not work as expected, software tools are the last candidates to blame."** EA tools also should not **"wag the dog"**.
- **Storage.** MS SharePoint, Atlassian Confluence, Atlassian Jira, Google Drive, protected network folders, or the EA repository itself. Folder hierarchies often follow business capabilities; Outlines and Designs are split into current vs. completed initiatives; other artifacts are organized by EA domain.

### Architecture debt
An **architecture debt** (or technical debt) is **"a temporary deviation from the ideal long-term architectural direction"** — a step in the architectural direction opposite to strategic, taken for compelling short-term benefits and requiring corrective measures later. It arises when initiative-level decisions in Outlines and Designs contradict Considerations, Standards, Visions or Landscapes but are granted **exemptions** by governance committees on the basis of tactical business value.

It is a **dual notion**:
- **Actions** — the work needed to return to the ideal trajectory: rework or decommission a deviating solution; migrate a misaligned system to a newer platform. This side drives **Technology Optimization**.
- **Financial** — "the cost of a temporary 'detour' from the main strategic 'highway'"; the IT dollars needed to return to the unspoiled state. Often only a "guesstimation". This side informs **Initiative Delivery** — estimate a debt from the Outlines and add it to total costs (subtract it from the business case) to expose the hidden delayed price.

Debts are logged in **architecture debt registers** (Figure 18.12) capturing description, corrective actions, financial estimate, redemption deadline (or N/A) and a minor / medium / major classification. Registers "can be regarded as a special type of Landscapes somewhat resembling IT Roadmaps" and act as one of the sources of technical rationalization suggestions. The metaphor's value: it explains long-term consequences of shortcuts to non-IT-savvy business leaders — **"all debts must be paid"**.

## Mental Models
- **Measurements inform, they never drive.** "Organizations should not consider achieving specific indicator values as their ultimate goals... companies should not make concrete IT investment or technology selection decisions with the intention to increase some or the other EA-related indicators." Use the numbers to ask *why* so many investments add little value or so many projects need exemptions — then fix the decision process. "Organizations should not establish measurements for the sake of measurements."
- **"Agile" EA is a rhetorical migration, not a methodology.** Unlike agile software development with concrete methodologies (Scrum), "agile enterprise architecture" has "neither definitive texts describing the corresponding practices, nor even a clear definition of its meaning". It can mean anything from less documentation to modular styles to ignoring TOGAF. Often it reduces to "trivial motherhood statements that becoming agile and flexible is better than staying rigid and inflexible."
- **The real question behind agility.** All discussions of agile EA "can arguably be reduced to the eternal question of how much upfront planning is desirable or necessary." Not "agile" vs. "traditional" but **"enough" planning** for the specific circumstances. Every dimension is a continuous spectrum, and the dimensions do not necessarily correlate with each other.

## Anti-patterns
- Obsessing over notational correctness — it "can only distract the attention of architects from solving real problems" and guarantees failure.
- Installing a sophisticated EA tool at the start of a practice, especially with architects inexperienced in it.
- Letting the tool dictate the artifacts and processes.
- Treating aggregate indicators as targets rather than signals.
- Building a second architectural repository when a fully populated, operations-owned CMDB already exists.
- Assuming "agile EA" is a defined method that can be adopted.

## Reference Tables

### Table 18.1 — Specialized EA tools vs. CMDBs
| | Specialized EA tools | CMDBs |
|---|---|---|
| Area | Enterprise architecture | IT service management |
| Goal | Business and IT alignment | Quality of IT services |
| Owners | Architects | IT operations and support teams |
| Coverage | Both physical and logical elements from all EA domains | Only concrete physical instances, mostly applications and infrastructure |
| Information | Rather high-level, updated periodically by architects, usually manually | Very detailed, updated by IT operations almost in real time, often semi-automatically |
| Focus | Relationships between assets (resemble **Landscape Diagrams**) | Properties of specific assets (resemble **Inventories**) |
| Features | Modeling, visualization, analysis (impact analysis) | Auto-discovery; integration with ITSM processes |
| Advantages | Store all necessary information at the right abstraction level; powerful planning support | Reliable, accurate, up-to-date; single source of truth for architects and operations |

CMDBs also solve the stale-Landscape problem: minor changes invisible to architects are still tracked by operations. "The information from CMDBs tends to be in a less convenient format for architectural purposes, but more accurate and up-to-date."

### The three families of EA measurements
| Family | Audience | Collected at | Indicators |
|---|---|---|---|
| **Quality of IT investments** (business-related; senior business leaders) | Executives allocating budget | Approval of Outlines and business cases by **investment committees**; portfolio management and budgeting | **Strategic focus** — % of investments uplifting capabilities heatmapped in Business Capability Models · **Strategic contribution** — % contributing to long-term Target States · **Roadmaps alignment** — ratio of planned (Roadmap) to unplanned (urgent) investments · **Assets development** — % invested in strategic systems from Enterprise System Portfolios and Inventories · **Platform utilization** — ratio of investment in improving existing assets to creating new ones · **Investments breakdown** — split into fundamental / strategic / local / urgent / architectural |
| **Magnitude of technical deviations** (IT-related; architects and IT leaders) | Standard-setters | Endorsement of Designs by **design committees** | **Technological heterogeneity** — % of projects on technologies missing from or retiring in Technology Reference Models · **Technical diversity** — % inconsistent with Guidelines, Patterns and IT Principles · **Reuse quality** — % reusing assets marked undesirable in Landscapes · **Risk volume** — % introducing significant technical risks · **Technical adaptivity** — % that led to creating or updating Standards (higher = faster organizational learning) |
| **Complexity of the IT landscape** (IT-related) | Architects, senior IT leaders | Extracted from **Landscapes and some Standards** (advanced ones need a tool-based repository) | **Number of assets** (scale) · **Number of connections** (topology) · **Number of technologies** (diversity) · **Number of undesirable assets** |

Asset counts correlate directly with maintenance costs; interconnectedness counts correlate with rigidity and brittleness. Reports are produced yearly or quarterly. **Quantitative measurements "are the features of advanced EA practices" and are "seldom found in organizations with emerging or immature EA practices."**

### Templates and straw-man architecture
**Templates** define presentation formats, schematic structures and informational contents; they are "a 'single source of truth' regarding the desirable composition of all instances." Ideal candidates are all **temporary** artifacts (Outlines and Designs) with many instances; also permanent artifacts with per-business-area instances (Roadmaps, Target States). Rarely created for **Business Capability Models and Enterprise System Portfolios** — most companies have only one instance. Templates accumulate the organization's own experience-based best practices and are improved from stakeholder feedback.

**Straw-man ("napkin") architecture** — "an elementary architectural draft of an EA artifact that can be produced very quickly and used to discuss principal planning decisions and available alternatives." Always **one format lighter** than the real artifact: if the artifact will be MS Word, the straw-man is PowerPoint; if the artifact will be PowerPoint, the straw-man can be photographed whiteboard drawings. It sits between the initial discussions and the finalized decisions artifact, and it excludes unfeasible alternatives before effort is spent elaborating them.

### Project checklists and assessment forms — two categories
| | **Governance** checklists/forms (Figure 18.10) | **Compliance** checklists/forms (Figure 18.11) |
|---|---|---|
| Purpose | Determine the formality and strictness of governance to apply — whether Designs need design-committee review, or whether architects need to be involved at all | Help project teams achieve conformity to Standards **with minimal architect involvement** via self-assessment |
| Criteria | Size, complexity, alignment, integration, security requirements | Lists of IT Principles, Guidelines and other requirements, sometimes solution-type-specific (infrastructure, web, high-availability) |
| Used when | Early stages of an initiative, often filled by project managers at inception | Later stages, during design and implementation |
| Effect | Avoids wasting architects' time on trivial projects; refocuses them on exceptional ones | Simplifies project management, increases team autonomy, engages architects only for unusual situations |

### The three impact tiers
| Tier | Example project | Treatment |
|---|---|---|
| **High impact** | Large, completely new fault-tolerant system on unusual technologies, highly integrated with existing applications | Full project governance; Designs studied and approved by all competent experts |
| **Moderate impact** | Medium-sized, fully compliant with corporate Standards, ordinary functionality, no strict security or availability needs | A solution architect in the team creating written Designs, but **no** formal design-committee endorsement |
| **Low impact** | Small routine extension of existing functionality, no unusual characteristics | Implemented by developers alone, no architectural involvement or governance |

### Figure 18.13 — Dimensions of agility
| Process | Dimension | "Traditional" / heavyweight / total planning | "Agile" / lightweight / full agility |
|---|---|---|---|
| **Strategic Planning** | Planning effort | High effort | Low effort |
| | Organizational scope | Entire company | Only critical areas |
| | Planning horizon | Long horizon | Short horizon |
| | Desired future | Concrete state | Only initiatives |
| **Initiative Delivery** | Logical flow | Strictly sequential | Highly iterative |
| | Volume of Outlines | Elaborate Outlines | Basic Outlines |
| | Volume of Designs | Extensive Designs | Lean Designs |
| **Technology Optimization** | Standardization | Many Standards | Few Standards |
| | Landscapes coverage | Full IT landscape | Only some areas |
| | Landscapes granularity | Rather detailed | Very abstract |
| **Other aspects** | Investment portfolio | Planned initiatives | Urgent initiatives |
| | Budget allocations | On a yearly basis | Continuous |
| | Governance procedures | Written decisions | Verbal agreements |
| | Adherence to plans | Strict adherence | Loose adherence |
| | Number of architects | Many architects | Few architects |
| | Involvement in projects | All IT projects | Only key projects |
| | Set of EA artifacts | Broad collections | Only essentials |
| | Software tools | Complex toolkits | Simplest tools |

Stable environments gravitate to the left extreme; dynamic environments to the right — "though this observation has countless nuances and should not be treated superficially." Neither extreme is satisfactory in pure form.

## Worked Example
**An architecture debt escalation policy.** "A corporate governance policy may stipulate that the planning decisions with a small architecture debt (e.g. less than 100,000 dollars) can be approved at the discretion of a design committee, the decisions with a moderate debt must be escalated either to an investment committee or technology committee, while all the IT investments introducing formidable volumes of architecture debt (e.g. exceeding one million dollars) must be sanctioned directly by a strategy committee."

**A populated architecture debt register (Figure 18.12).** Thirteen debts with estimate and deadline: Debt 1 \$350k Q1 2023 · Debt 2 \$40k N/A · Debt 3 \$1.6m Q4 2023 · Debt 4 \$200k N/A · Debt 5 \$90k Q4 2021 · Debt 6 \$25k Q3 2022 · Debt 7 \$600k Q2 2023 · Debt 8 \$65k N/A · Debt 9 \$50k Q2 2022 · Debt 10 \$1.1m Q3 2023 · Debt 11 \$30k N/A · Debt 12 \$280k N/A · Debt 13 \$20k Q1 2022 — colour-banded Minor / Medium / Major.

**Mapping IT investments to a Business Capability Model (Figure 18.7).** Capability 1: \$12.4m (17.9%) — sub-capabilities 1.1 \$3.5m (5.0%), 1.2 \$0.0m (0.0%), 1.3 \$7.6m (11.0%), 1.4 \$1.3m (1.9%). Capability 2: \$34.7m (50.0%) — 2.1 \$10.8m (15.6%), 2.2 \$13.6m (19.6%), 2.3 \$3.9m (5.6%), 2.4 \$6.4m (9.2%). Capability 3: \$12.2m (17.6%) — 3.1 \$2.8m (4.1%), 3.2 \$0.0m (0.0%), 3.3 \$5.3m (7.6%), 3.4 \$4.1m (5.9%). Capability 4: \$10.1m (14.5%) — 4.1 \$1.9m (2.7%), 4.2 \$7.7m (11.1%), 4.3 \$0.0m (0.0%), 4.4 \$0.5m (0.7%). Totals by priority band: Regular \$14.2m (20.5%), Important \$23.2m (33.3%), Strategic \$32.0m (46.2%).

**A governance impact checklist (Figure 18.10, "Project Alpha").** Nine yes/no features — business-critical · planned for three months or more · creates new IT assets · requires new infrastructure · uses unproven technologies · handles personal data · deals with payments · requires users to be authenticated · has some other special requirements. Rules: *"If 'Yes' is ticked at least once, then a solution architect must be invited. If 'Yes' is ticked more than twice, then presenting to Design Authority is required."*

**An architecture assessment form (Figure 18.10, "Project Beta").** Nine aspects scored 0–4 — Functionality, User Experience, Complexity, Hardware Infrastructure, Data Integration, Security, Operability, Maintainability, Roadmap Alignment. Bands: **0–2 negligible impact · 3–9 minor impact · 10–18 medium impact · 19–36 major impact**. Overall Score for Project Beta: **15** (medium impact).

**A compliance checklist (Figure 18.11, "Project Gamma").** Nine IT architecture principles — Simplicity, Modularity, Reusability, Maintainability, Manageability, Reliability, Scalability, Security, Compliance — each marked Not / Partly / Fully aligned. Rule: *"If at least one principle is not aligned to or at least two principles are partly aligned to, then a consultation with enterprise architects is necessary."* The companion Standards Conformance Form for "Solution Delta" is gated by triggering questions ("Exposure through web interfaces? Yes/No", "Operations with sensitive data? Yes/No", "Interactions with other systems? Yes/No") each unlocking checks such as "All web pages are accessible only via HTTPS", "Session ID is always passed in cookies", "All sensitive data is stored in an encrypted form", "All interactions with external systems are done via ESB", "All data is passed in standard open formats, e.g. XML" — answered ✓ / ✗ / N/A, with "In case of any nonconformance, please contact the architecture team."

**Industry-specific agility calibration.** "Retailers tend to experience higher business turbulence than utility providers and, thus, are likely to invest less efforts in long-range planning, plan for shorter horizons, adopt more flexible budgeting mechanisms and launch a greater number of urgent initiatives." And within one company, agility can be initiative-specific: iterative approaches for software solutions that can be reworked and redeployed, formal documentation-led approaches for infrastructure solutions requiring expensive hardware purchase and installation.

## Key Takeaways
- ArchiMate, UML, BPMN and ARIS reach only Designs, Landscapes and partly Outlines; Visions and Outlines have no suitable standard notation, and Considerations and Standards are largely textual.
- Successful practices run on MS Office plus a CMDB, with a specialized EA tool as an optional later addition justified by a specific need.
- Measurements come in three families — investment quality, technical deviations, landscape complexity — and belong to mature practices only.
- Templates, straw-men and two kinds of checklist (governance and compliance) are the cheap, high-leverage instruments.
- Architecture debt is dual: financial (informs Initiative Delivery and escalation thresholds) and action-based (drives Technology Optimization).
- "Agile EA" is not a method; it is a position chosen on ~17 continuous dimensions.

## Connects To
- [Chapter 6 — Processes of Enterprise Architecture Practice](ch06-processes-of-ea-practice.md) — the three processes whose agility dimensions are tuned here
- [Chapter 12 — Landscapes](ch12-landscapes.md) — the artifacts EA tools and CMDBs actually serve
- [Chapter 14 — Designs](ch14-designs.md) — where UML and ArchiMate are genuinely used
- [Chapter 17 — Architecture Functions in Organizations](ch17-architecture-functions-in-organizations.md) — the exemptions and escalation thresholds architecture debt informs
- [Chapter 19 — The Lifecycle of Enterprise Architecture Practice](ch19-the-lifecycle-of-ea-practice.md) — why measurements and debt management are signs of maturity
