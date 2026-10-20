# Chapter 17 — Architecture Functions in Organizations

## Core Idea
An **architecture function** is a specialized supporting corporate function, usually a subunit of the IT department reporting directly to the CIO, responsible for the EA practice and organization-wide business and IT alignment. Its **size follows the size of the IT department**; its **shape follows the degree of decentralization**. It also hosts **architecture governance committees**, whose structure mirrors the architecture function itself.

## Frameworks Introduced

### The three core IT-related capabilities (Figure 17.1)
Business executives → **architecture function (planning)** → *implementable designs* → **IT delivery function (implementation)** → *running systems* → **IT support function (maintenance)**. The architecture function "provides a 'front door' to the IT department for business leaders."

### Main responsibilities of architecture functions
Establishing effective communication with other organizational functions and clarifying their IT demands and requirements · achieving sufficient engagement with business executives, project teams and other stakeholders · producing required decisions EA artifacts in a timely manner and getting them approved · ensuring compliance with the approved decisions · producing and maintaining facts EA artifacts · continuously optimizing the format, content and size of artifacts based on stakeholder feedback · institutionalizing and fine-tuning EA-related decision-making and approval processes · maintaining appropriate software toolsets for producing, storing, publishing and searching artifacts · creating architecture positions and defining responsibilities (sizing, structuring and adapting themselves) · organizing peer-review mechanisms and knowledge exchange between architects · advocating and explaining the value of disciplined information systems planning.

### Sizing rule
Organizations average **one architect for every 20–30 IT department employees** — roughly **3–5% of the total IT workforce**. Companies undergoing transformation with aggressive IT investment strategies need more (one per 20); companies in maintenance mode with a conservative attitude need fewer (one per 30). The ratio is also influenced by the amount of upfront planning desirable.

### Dependence on organization size (Figure 17.2)
| IT department scale | Architects | Sample positions |
|---|---|---|
| **<25 IT employees** | None | The CIO acting as an architect on a part-time basis (tiny organizations may not need an EA practice at all; the role may be fulfilled informally by CIOs, heads of application development or other IT leaders) |
| **30–40 IT employees** | 1 | 1 enterprise architect carrying out Strategic Planning and Technology Optimization for the whole organization; also involved in Initiative Delivery — either acting as a solution architect, or reviewing and approving Outlines and Designs produced by engaged third parties |
| **100–200 IT employees** | 4–8 | 4–8 solution architects — "20% enterprise and 80% solution" architects: ~80% of their time on Outlines and Designs, ~20% collectively carrying out Strategic Planning and Technology Optimization |
| **250–400 IT employees** | 10–15 | 1 enterprise architect + 10–15 solution architects; the enterprise architect also acts as architecture manager |
| **500–800 IT employees** | 20–30 | 3–5 domain architects (business, applications, data, infrastructure) + 15–25 solution architects; domain architects also act as architecture managers for solution architects in their domains |
| **1200–2000 IT employees** | 50–80 | 1 architecture manager + 3–5 business area architects + 4–8 domain architects + 40–70 solution architects (including a few program architects) |
| **>2500 IT employees** | >100 | 2–3 architecture managers + 3–5 enterprise architects + 4–8 business area architects + 10–15 domain architects + 80–120 solution architects (including several program architects) |

Positions in real organizations deviate from the pure archetypes: enterprise architects often act as architecture managers and, in small organizations, as solution architects; business area and domain architects often manage solution architects in their areas or domains; business area architects may also act as domain architects and vice versa. Combining or **rotating architects between positions**, especially across hierarchy tiers, "facilitates effective knowledge exchange, removes the boundaries between different tiers of architecture and avoids the 'ivory tower' syndrome."

### Dependence on decentralization (Figure 17.3)
| Degree | Operating model correlation | Positions | Authority split |
|---|---|---|---|
| **Very centralized** | Unification, replication | Only domain architects (plus solution architects) | Domain architects have full authority over Strategic Planning and Technology Optimization for the whole organization; all EA domains centralized; no local flexibility |
| **Centralized** | Coordination | Domain architects for data, integration, infrastructure, security + some business area architects | Domain architects own their domains organization-wide; business area architects own Strategic Planning for their units, but only in the business and applications domains that have no organization-wide domain architects |
| **Decentralized** | Diversification | Business area architects + some domain architects for infrastructure and security | Domain architects own Technology Optimization in their domains organization-wide; business area architects own Strategic Planning and Technology Optimization for their units except infrastructure and security |
| **Very decentralized** | Diversification; holding companies with independent subsidiaries | Only business area architects | Full authority over Strategic Planning and Technology Optimization for their units; total local autonomy, no global consolidation |

### Geographic distribution
An architect's work requires intense daily communication with other architects, and face-to-face conversation is the most effective form. Splitting architecture teams across sites "can deteriorate the quality of communication... and, thus, should not be taken easily and may be justifiable arguably only in the cases of extreme decentralization." Weak communication "may undermine overall architectural consistency and result in considerable discrepancies in technology selection, investment prioritization and solution assessment approaches." Even multinationals often host all enterprise, domain and business area architects at corporate headquarters, using video conferences and periodical travel. **Only ordinary solution architects can be decentralized relatively safely**, especially where delivery is permanently overseas or outsourced; they can also be sent temporarily to regional offices to coordinate and supervise local projects. "Architecture functions are dispersed geographically to a lesser extent than the organizations they serve."

## Key Concepts — architecture governance

### Responsibilities of governance committees
Conducting periodical meetings on a regular basis · organizing discussions of relevant EA-related questions · inviting all essential stakeholders · ensuring sufficient engagement between stakeholder groups · making and approving key EA-related decisions collectively · formally reviewing and officially endorsing decisions EA artifacts.

**The bulk of governance happens before the committee meets.** "Significant EA-related decisions should not be new and surprising for participants of governance committees." All decisions artifacts should already be peer-reviewed by other, often more senior, architects. "The role of governance committees is not to provide a forum for discussions or disputes, but rather to make sure that all relevant stakeholders are fully aware of all the important EA-related decisions, do not object to these decisions and ready to act on them. Ideally, architecture governance committees only finalize decision-making processes by officially authorizing the resulting decisions EA artifacts tentatively approved earlier by their direct stakeholders."

Meetings may be recorded, with **minutes of the meeting (MoM)** documenting the date, participants, agenda items with associated artifacts and, most importantly, approval decisions to be followed up. Only facts artifacts (most Landscapes and some Standards) need no committee approval. **Architects act only as advisors in most governance procedures**, not primary decision-makers — they cannot authorize or reject investment or project initiation decisions on behalf of real managers.

### Two classification dimensions
- **Business committees** (numerous business stakeholders, only the most senior IT stakeholders) approve business-focused artifacts: Considerations, Visions, Outlines.
- **IT committees** (numerous IT stakeholders, managers and subject-matter experts) approve IT-focused artifacts: Designs, most Standards, some Landscapes.
- **Strategic committees** (more senior stakeholders, meet less often) approve rules and structures artifacts: Considerations, Visions, most Standards, some Landscapes.
- **Tactical committees** (less senior stakeholders, meet more often) approve changes artifacts: Outlines and Designs.

## Reference Tables

### Figure 17.6 — Four types of architecture governance committees
| | **Strategic** | **Tactical** |
|---|---|---|
| **Business** | **Strategy committees** (strategic architecture forums, enterprise architecture councils, architecture governance councils, IT steering committees, executive forums) — *strategic governance* | **Investment committees** (investment forums, project investment boards, funding committees, innovation forums, portfolio boards) — *investment governance* |
| **IT** | **Technology committees** (technology architecture forums, architecture steering committees, IT architecture boards, IT strategy councils, domain architecture councils) — *technology governance* | **Design committees** (design authorities, enterprise architecture authorities, architecture review boards/panels/meetings, technology architecture review forums, technical review boards, technical design boards) — *project governance* |

| Committee | Participants | Approves | Meets |
|---|---|---|---|
| **Strategy** | Senior architects (enterprise, business area, business-enabling domain); business: CEO, other C-level, heads of major business units; IT: CIO, senior IT managers | All significant changes in **Considerations and Visions** — new Principles and Direction Statements, major amendments in Policies, refocus of Business Capability Models, changes in desired Target States | Monthly or as necessary |
| **Technology** | Senior architects (enterprise, business area, domain); IT: CIO, senior IT managers, subject-matter experts | Substantial changes in **most Standards and some Landscapes** — emerging and unsupported technologies in Technology Reference Models, new Patterns, strategic and legacy systems in Enterprise System Portfolios, rearrangements of IT Roadmaps. *Not* routine Guidelines/Logical Data Model improvements or current-state updates of Landscape Diagrams and Inventories | Monthly or as necessary |
| **Investment** | Architects (enterprise, relevant business area and domain, some solution); business: CEO, CFO, executive sponsors, senior business managers; IT: CIO, senior IT managers | All **Outlines and business cases** — Initiative Proposals and Solution Overviews with business cases; decide whether initiatives are worth implementing, elaborating or modifying; ensure alignment to Considerations, Visions, major Standards and Landscapes. Members typically also compose the program of work | Monthly or biweekly |
| **Design** | Architects (some enterprise/business area/domain, solution, sometimes external vendor architects); business owners or business analysts; IT: project managers, subject-matter experts, senior IT operations and support | All **Designs** — Preliminary Solution Designs and Solution Designs; ensure projects use recommended technologies, approaches and best practices, fit the environment described in Landscapes, and incorporate relevant technical rationalization suggestions | Biweekly, weekly or more often |

Architecture functions may also organize **specialized discussion forums** — largely informal bodies with optional attendance, no formal decision-making responsibilities and no governance authority — for architects to discuss technology trends and share ideas.

### Figure 17.8 — Taxonomy of possible governance resolutions
| Potential business impact ↓ / Deviations from higher-level plans → | **No deviations** | **Minor deviations** | **Major deviations** |
|---|---|---|---|
| **Critical** | Endorsement | Exemption | **Escalation** |
| **Moderate** | Endorsement | Exemption | Rejection |
| **Negligible** | Endorsement | Rejection | Rejection |
| | *Easy resolutions* | *Difficult resolutions* | *Unresolvable cases* |

- **Endorsement** — unconditional approval due to perfect alignment.
- **Rejection** — disapproval due to inexpediency / substantial misalignment.
- **Exemption** (also *dispensation*) — approval despite misalignment, sometimes with an obligation to take corrective action later (then called a **conditional endorsement**) or a registration of the corresponding architecture debt.
- **Escalation** — delegation to a more authoritative committee.

All exemptions and their reasons are logged to enable post-implementation analysis of long-term consequences and organizational learning. "The strictness of approval and exemption procedures is determined largely by the desired level of architectural flexibility."

### Figure 17.7 — Escalation paths
| From | To | Typical trigger |
|---|---|---|
| Design committee | Technology committee | Designs require purchase, introduction and support of technologies or products missing from Technology Reference Models; depart from established IT Principles; conflict with approved IT Roadmaps |
| Design committee | Investment committee | Designs cannot capture necessary business data per agreed Conceptual Data Models; deviate from desired Target States — the very need for the initiative may be reconsidered, or reworked Outlines and business cases requested |
| Investment committee | Strategy committee | Outlines not compliant with critical Policies; deviate from approved Direction Statements; do not uplift any capabilities marked strategic or important in Business Capability Models |
| Technology committee | Strategy committee | Optimal technology portfolio decisions conflict with Analytical Reports; asset reuse decisions conflict with heatmapped activities on Value Chains |

### Figure 17.9 — Governance structures by organization size
| Scale | Structure |
|---|---|
| **Small** | A **single** architecture committee implementing all four governance types, chaired by the most senior enterprise architect, inviting different participants per topic (CEO and executives for strategy discussions; project manager, solution architects and subject-matter experts for project meetings) |
| **Medium** | **Two** committees — a business committee (strategic + investment governance, e.g. IT steering committee, chaired by the CIO) and an IT committee (technology + project governance, e.g. architecture review board, chaired by the architecture manager). Major decisions escalate from the IT committee to the business committee. **"Arguably the most popular structure of architecture governance mechanisms commonly found in many companies"** |
| **Large** | **Four** separate specialized committees, exactly as in Figures 17.6 and 17.7 |
| **Very large and decentralized** | **Multiple committees per type** — strategic and investment governance aligned with major business units and chaired by business area architects; technology and project governance aligned with core technical domains and chaired by domain architects; plus **global** strategy and technology committees chaired by enterprise architects acting as escalation points. Temporary initiative-specific committees (e.g. program design authorities) chaired by lead solution or program architects for multi-year programs. Investments above a threshold may escalate automatically to the executive committee or board of directors |

**Why explicitly separate committees even when one could do the job**: it "helps set clearer agenda for concrete meetings, minimize deviations from the agenda, focus discussions on relevant issues and better structure an EA practice", clearly separating organization-wide from initiative-level discussions and purely technical from business-related meetings.

## Mental Models
- **Structure mirrors structure.** Architecture function structure mirrors the organization (size + decentralization); governance committee structure mirrors the architecture function.
- **Regardless of structure, all four governance types must be fulfilled.** "The four types of governance procedures can be implemented differently from an organizational perspective, but all the four types should be implemented."

## Anti-patterns
- Using governance committees as debating forums instead of authorization milestones.
- Splitting senior architects across physical sites without extreme decentralization justifying it.
- Letting architects act as if they had decision-making authority they do not possess.

## Worked Example
**Architecture function in a large diversified multinational (Figure 17.4).** Divisions A, B and C in countries X, Y and Z, each with two subunits and its own operating model — A implements coordination (subunits A.1 in city X.1, A.2 in X.2; X.1 hosts the global headquarters), B implements diversification (B.1 in Y.1, B.2 in Y.2; Y.1 hosts the regional head office), C implements unification (C.1 in Z.1, C.2 in Z.2; Z.2 hosts the regional head office). The resulting design: **all architects serving both subunits of division A are located at the global headquarters in city X.1**; for coherence, **most planning for division B is also done from headquarters, except unit B.1** where closer contact with local business leaders is required; **division C is operationally independent and employs its own architecture team at its regional head office in Z.2** serving both its subunits.

**Statistical monitoring of governance.** "Design committees may log the total number of Designs submitted for approval over a specific time period (e.g. last year) as well as the percentage of these Designs that have been endorsed, rejected, endorsed with exemptions or escalated."

## Key Takeaways
- 3–5% of IT staff are architects; the function's size and shape follow organization size and decentralization.
- Domain architects → global optimization; business area architects → local flexibility; the operating model decides the balance.
- Four governance types — strategic, technology, investment, project — always present, implemented in 1, 2, 4 or many committees.
- Four possible resolutions: endorsement, rejection, exemption, escalation.

## Connects To
- [Chapter 16 — Architects in Enterprise Architecture Practice](ch16-architects-in-ea-practice.md) — the five archetypes staffed here
- [Chapter 5 — The Dialog Between Business and IT](ch05-the-dialog-between-business-and-it.md) — operating models that determine centralization
- [Chapter 13 — Outlines](ch13-outlines.md) and [Chapter 14 — Designs](ch14-designs.md) — the artifacts investment and design committees approve
- [Chapter 18 — Instruments for Enterprise Architecture](ch18-instruments-for-enterprise-architecture.md) — architecture debt thresholds driving escalation; governance formality and agility
