# Cheatsheet — The Practice of Enterprise Architecture (Kotusev)

Dense, scannable reference. Chapter numbers in parentheses.

---

## 1. The CSVLOD grid at a glance (Ch 4, 8)

| | **Rules** — how we work | **Structures** — what we have | **Changes** — what we change now |
|---|---|---|---|
| **Business-focused** | **CONSIDERATIONS** · dual · decisions · permanent · stateless or long-term · textual | **VISIONS** · dual · decisions · permanent · 3–5 yr future · one-page diagrams | **OUTLINES** · dual · decisions · **temporary** · 1–2 yr future · text + simple diagrams |
| **IT-focused** | **STANDARDS** · not dual · decisions (+ some facts) · permanent · stateless or current · varied, strict notations | **LANDSCAPES** · not dual · **facts** (+ some decisions) · permanent · current state · strict, often ArchiMate | **DESIGNS** · dual (architects↔teams) · decisions · **temporary** · <1 yr · text, tables, UML |

**Users**: Considerations, Visions, Outlines = business executives + architects. Standards, Landscapes = architects only. Designs = architects + project teams.
**Purpose**: top row lets business **manage** IT; bottom row lets architects **organize** IT.

## 2. All 24 subtypes by prevalence (Ch 8)

| Type | **Essential** (>50%) | **Common** (~25–50%) | **Uncommon** (~10–25%) |
|---|---|---|---|
| Considerations | Principles | Policies | Conceptual Data Models · Analytical Reports · Direction Statements |
| Standards | Technology Reference Models · Guidelines | Patterns · IT Principles | Logical Data Models |
| Visions | Business Capability Models · Roadmaps | Target States | Value Chains · Context Diagrams |
| Landscapes | Landscape Diagrams | Inventories · Enterprise System Portfolios · IT Roadmaps | — |
| Outlines | Solution Overviews | Options Assessments | Initiative Proposals |
| Designs | Solution Designs | — | Preliminary Solution Designs |

8 essential + 8 common + 8 uncommon = 24. **Target set: 10–15 subtypes.**

## 3. Decisions vs. facts — the fork that decides everything (Ch 2)

| | **Decisions artifacts** | **Facts artifacts** |
|---|---|---|
| State | Future or stateless | Current only |
| Nature | Subjective (people's interests) | Objective (acknowledged facts) |
| Developed by | All stakeholders, collaboratively | Individual architects, alone |
| Architect's role | Driver and facilitator | Sole developer |
| Value realized | **During** development | **After** development |
| Priority | **Process over product** | **Product over pro­cess** |
| Success factor | Involvement of all stakeholders | Accuracy of all descriptions |
| Format optimized for | Teamwork, editing, distribution | Storage, searchability, analysis |
| Which artifacts | Everything except… | **Most Landscapes and some Standards** |

Exceptions to memorize: **IT Roadmaps** are the one Landscapes subtype that is *decisions*. **Context Diagrams** are the one Visions subtype that is *facts*. **TRMs, Inventories and Enterprise System Portfolios** are both — the list is facts, the status colour-coding is decisions.

## 4. The EA uncertainty principle — sizing test (Ch 5)

| Scope | Horizon | Discussion point | Artifact |
|---|---|---|---|
| Entire organization | ~5 yr (>3–5) | Operating model | dual Considerations |
| Organization or major units | ~4 yr (3–5) | Business capabilities | Business Capability Models, dual Visions |
| Separate business areas | ~3 yr (2–3) | Specific business needs | Roadmaps, dual Visions |
| Separate IT initiatives | ~2 yr (1–2) | Business processes | Solution Overviews, dual Outlines |
| Separate IT projects | ~1 yr (<1) | Business requirements | Solution Designs, dual Designs |

**Too abstract for a narrow/short cell → Useless Zone. Too detailed for a broad/long cell → Unachievable Zone.**

## 5. Operating model selector (Ch 5)

| | Low data integration | High data integration |
|---|---|---|
| **High process standardization** | **Replication** — similar, independent units; globally standardized but locally owned apps and DBs | **Unification** — similar, interdependent units; global apps, DBs, services, often comprehensive ERP |
| **Low process standardization** | **Diversification** — diverse, independent units; local apps and DBs, global infrastructure only | **Coordination** — diverse but interdependent; local apps, global DBs (customers, products, suppliers) |

Fits: price leadership → replication (misfits coordination). Customer intimacy → coordination (misfits replication).
Downstream: **centralization of the architecture function** — unification/replication = very centralized (domain architects only); coordination = centralized; diversification = decentralized to very decentralized (business area architects). Conceptual/Logical Data Models are most useful under **coordination and unification**.

## 6. Thresholds and default values

| Thing | Value | Ch |
|---|---|---|
| Organizations that practise EA | Those with >**30–50 IT specialists** where IT supports main business activities | 3 |
| Architects per IT staff | **1 per 20–30** (1:20 aggressive transformation, 1:30 maintenance mode) = **3–5% of the IT workforce** | 17 |
| Solution architects' share of all architects | **60–80%** | 16 |
| Years of IT experience before an architecture position | **8–12+** | 16 |
| Organization-wide Principles | **5–10** | 9 |
| IT Principles | **10–20 or more**, grouped by EA domain | 10 |
| EA artifact subtypes in a mature practice | **10–15** of the 24 | 8, 15, 19 |
| Solution Overview length | **15–30 pages** | 13 |
| Solution Design length | **25–50 pages** average; a few hundred in extreme cases | 14 |
| Preliminary Solution Design length | **20–40 pages** | 14 |
| Roadmap horizon | Up to **3 years**, more rarely 5 | 11 |
| IT Roadmap horizon | Often only **1–2 years** | 12 |
| Vision / Target State horizon | **3–5 years** | 11 |
| Outlines horizon | **1–2 years**, longer for large programs | 13 |
| Designs horizon | Up to **1 year** | 14 |
| Options to analyse | Often **at least 3**, plus optionally "Do nothing" | 13 |
| Organizations using a specialized EA tool | ~**two-thirds** | 18 |
| Time to a reasonably mature practice | ~**2 years** small; **several years** large | 19 |
| Review cadence for Considerations, Standards, Visions | **Yearly**, after business strategy re-approval | 9, 10, 11 |

## 7. Governance quick reference (Ch 17)

| | **Strategic** (senior, less often) | **Tactical** (less senior, more often) |
|---|---|---|
| **Business** | **Strategy committee** — Considerations, Visions · CEO, C-level, heads of units, CIO · **monthly or as necessary** | **Investment committee** — Outlines + business cases · CEO, CFO, sponsors, CIO · **monthly or biweekly** |
| **IT** | **Technology committee** — most Standards, some Landscapes · CIO, senior IT managers, SMEs · **monthly or as necessary** | **Design committee** — Designs · solution architects, project managers, SMEs, operations · **biweekly, weekly or more** |

### Resolution matrix
| Business impact ↓ / Deviation → | None | Minor | Major |
|---|---|---|---|
| **Critical** | Endorsement | Exemption | **Escalation** |
| **Moderate** | Endorsement | Exemption | Rejection |
| **Negligible** | Endorsement | Rejection | Rejection |

### Escalation triggers
| From → To | Trigger |
|---|---|
| Design → Technology | Technologies missing from TRMs; departure from IT Principles; conflict with IT Roadmaps |
| Design → Investment | Cannot capture data per Conceptual Data Models; deviates from Target States |
| Investment → Strategy | Non-compliance with critical Policies; deviation from Direction Statements; uplifts no strategic capability |
| Technology → Strategy | Technology portfolio decisions conflict with Analytical Reports; reuse decisions conflict with heatmapped Value Chains |

### Committee count by size
Small → **1** committee (all four types, invitees vary). Medium → **2** (business committee chaired by CIO + IT committee chaired by architecture manager) — *"arguably the most popular structure"*. Large → **4**. Very large/decentralized → **multiple per type** plus global escalation committees.

## 8. Status colour-coding cribs

**Technology Reference Model** (Ch 10): Current (reuse safely) · Emerging (adopt in new solutions) · Unsupported (in use, no vendor support, do not reuse) · Retiring (retire soon, do not reuse) · Prohibited (critical defects, eliminate immediately) · Retired (recently removed, left for clarification) · Unknown.

**Inventory assets** (Ch 12): Reuse (healthy) · Invest (strategic, enhance) · Maintain (toxic, reuse discouraged) · Decommission (legacy, decommission shortly) · Undecided.

**Enterprise System Portfolio systems** (Ch 12): Active · Strategic · Legacy (unfit, needs replacement) · Inactive (deployed, unused, safely decommission) · Unclear.

**Roadmap initiatives** (Ch 11): Planned → Approved (early Outlines started) → Funded (in the program of work) → Active (team assembled, Designs agreed).

**IT Roadmap funding** (Ch 12): CIO · Business · Not Sponsored · Not Required.

**Architecture debt** (Ch 18): Minor · Medium · Major.

## 9. Which architect does what (Ch 16, 17)

| Archetype | Scope × expertise | Processes | Artifacts |
|---|---|---|---|
| **Solution architects** (60–80%) | Narrow × narrow | Initiative Delivery only | Outlines, Designs |
| **Program architects** | Wider initiative scope | Initiative Delivery | Initiative roadmaps, Outlines, Designs; coordinate other solution architects |
| **Domain architects** — business-supporting (integration, infrastructure, security) | Broad × narrow | Technology Optimization + Initiative Delivery | Standards, Landscapes; review Outlines and Designs |
| **Domain architects** — business-enabling (business, applications, data) | Broad × narrow | Strategic Planning + Technology Optimization + Initiative Delivery | Considerations, Visions, plus Standards and Landscapes |
| **Business area architects** | Narrow × broad | Strategic Planning + Technology Optimization for their area | All types for their area |
| **Enterprise architects** | Broad × broad | Strategic Planning + Technology Optimization | Considerations, Standards, Visions, Landscapes org-wide |
| **Architecture managers** | Outside the matrix | Organize all three | None directly — "meta-architects" |

**Not architects**: engagement managers (strategic business analysts; a sign of an immature practice) and technical designers ("microarchitects" in IT delivery functions).

### Function staffing by IT department size (Ch 17)
| IT staff | Architects | Composition |
|---|---|---|
| <25 | 0 | CIO acting part-time |
| 30–40 | 1 | 1 enterprise architect |
| 100–200 | 4–8 | Solution architects, "20% enterprise / 80% solution" |
| 250–400 | 10–15 | 1 enterprise architect (also architecture manager) + solution architects |
| 500–800 | 20–30 | 3–5 domain + 15–25 solution architects |
| 1200–2000 | 50–80 | 1 architecture manager + 3–5 business area + 4–8 domain + 40–70 solution |
| >2500 | >100 | 2–3 managers + 3–5 enterprise + 4–8 business area + 10–15 domain + 80–120 solution |

## 10. Initiative type selector (Ch 7)

| Type | Origin | Direction | Motivation |
|---|---|---|---|
| **Fundamental** | Strategic Planning | Top-down, planned | Grow permanent capabilities; build a reusable digitized platform (often from operating model requirements) |
| **Strategic** | Strategic Planning | Top-down, planned | Execute the current business strategy |
| **Local** | Strategic Planning (proposed by unit) | **Bottom-up**, planned | Address essential tactical needs of business units |
| **Urgent** | **External environment** | Bottom-up, **unplanned** | Address critical, unexpected needs |
| **Architectural** | Technology Optimization | Top-down, planned | Optimize the landscape structure |

Goal: maximize fundamental + strategic; minimize urgent + architectural. A mature practice converts many urgent initiatives into foreseen local ones.

## 11. Modeling language and tool selector (Ch 18)

| Artifact type | Notation | Tool |
|---|---|---|
| Considerations | Plain text — no diagrams needed | **MS Word** |
| Standards | Varied; often text | MS Word; EA tool for TRMs and Logical Data Models |
| Visions | **No standard notation exists** — intuitive ad hoc, full-colour, stylish | **MS Visio**, sometimes PowerPoint |
| Landscapes | **ArchiMate** (less often ARIS) | MS Visio, **MS Excel** (Inventories), **CMDB**, specialized EA tool |
| Outlines | Simplified **BPMN** for process change; otherwise informal | MS Word (official) or PowerPoint (informal) |
| Designs | **UML** or ArchiMate; BPMN or ARIS less often | **MS Word**, almost always |

### EA tool vs. CMDB
| | EA tool | CMDB |
|---|---|---|
| Resembles | Landscape Diagrams (relationships) | Inventories (properties) |
| Owner | Architects | IT operations and support |
| Coverage | All EA domains, physical + logical | Applications and infrastructure, physical instances only |
| Currency | Periodic, manual | Near real-time, semi-automatic, auto-discovery |
| Strength | Abstraction level, impact analysis | Accuracy, single source of truth, ITSM integration |

**Rule**: start on MS Office + existing CMDB. Introduce an EA tool only at a later stage against a stated need. If a populated CMDB integrated with ITSM exists, an extra EA tool "may not be worthwhile".

## 12. Measurement menu (Ch 18)

| Family | Collected at | Indicators |
|---|---|---|
| **Investment quality** (business) | Investment committee / portfolio management | Strategic focus · Strategic contribution · Roadmaps alignment · Assets development · Platform utilization · Investments breakdown |
| **Technical deviations** (IT) | Design committee | Technological heterogeneity · Technical diversity · Reuse quality · Risk volume · Technical adaptivity |
| **Landscape complexity** (IT) | Landscapes and Standards | Number of assets · connections · technologies · undesirable assets |

Report yearly or quarterly. **Measurements inform, never drive.** Features of advanced practices only.

## 13. Project impact triage (Ch 18)

| Tier | Profile | Treatment |
|---|---|---|
| **High** | Large, new fault-tolerant system, unusual technologies, highly integrated | Full project governance; Designs approved by all competent experts |
| **Moderate** | Medium, fully Standards-compliant, ordinary functionality, no strict security/availability needs | Solution architect in the team writing Designs; **no** design-committee endorsement |
| **Low** | Small routine extension of existing functionality | Developers alone; no architectural involvement |

Sample checklist rule: *one "Yes" → invite a solution architect; more than two "Yes" → present to the Design Authority.* Sample assessment bands (nine aspects scored 0–4): **0–2 negligible · 3–9 minor · 10–18 medium · 19–36 major**.
Sample debt thresholds: **<\$100k** design-committee discretion · moderate → investment or technology committee · **>\$1m** → strategy committee.

## 14. Tells and smells — symptom → diagnosis → fix

| Symptom | Diagnosis | Fix | Ch |
|---|---|---|---|
| Artifacts are produced, filed and never referenced again | Decisions artifacts built like facts artifacts, without their stakeholders — the "ivory tower" | Rebuild collaboratively; the process *is* the product | 2, 6 |
| Executives cannot say where the IT budget goes | Strategic Planning immature; Visions missing or too detailed | Introduce Business Capability Models and Roadmaps; map investments to capabilities | 11, 18, 19 |
| Every project reinvents the wheel; costs and risks keep climbing | Standards missing or unenforced | Introduce TRMs and Guidelines; enforce via peer-review of Outlines and Designs | 10 |
| Projects stall in endless negotiation; the function is called "architecture police" | Standards overly strict, too numerous, at the wrong level | Prune; build a hierarchy by organizational level; tolerate justified deviations as logged debt | 10 |
| Landscape documentation is comprehensive but nobody trusts it | Too detailed to keep current — minor changes happen without architects | Raise abstraction to architecturally significant, slow-changing elements; source detail on demand; use the CMDB | 12 |
| Architects propose an IT target state; executives will not fund it | Future-state Landscapes created ahead of Visions — IT trying to lead business | Reverse the order: capabilities in Visions first, then portfolios, then diagrams | 12 |
| Five-year plans invalidated by every strategy shift | Visions too detailed and fine-grained; EA uncertainty principle violated | Detail Visions only enough to fund the next budget cycle | 11, 5 |
| Funding decisions take forever and miss the point | Outlines over-elaborated to Design level | Cut to value, risks and cost; move detail to Designs | 13 |
| Designs never finish; teams call them rigid | Inability to distinguish architecture from design → analysis paralysis | Keep only technologies, external interactions and security plus the org-significant; delegate the rest | 14 |
| Portfolio dominated by urgent initiatives | Strategic Planning is not sifting needs; or genuinely turbulent industry | Strengthen Roadmaps; accept a floor in dynamic sectors | 7, 19 |
| Governance meetings are debates that end without decisions | Peer-review and stakeholder pre-approval skipped | Do the deciding beforehand; use committees to authorize | 17 |
| Systems built for the last strategy are liabilities for the next | The alignment trap — planning started from the business strategy | Anchor on the operating model; build a reusable digitized platform | 5 |
| An architecture initiative was launched and sponsorship evaporated | Wrong deliberate-path entry point for the executives' actual problem | Re-ask "What is our biggest business problem in relation to IT?" and re-enter there | 19 |
| Architects spend their days on the tool | Tool introduced too early or "wagging the dog" | Revert to MS Office and the CMDB until a real need is articulated | 18, 19 |
| Diagrams are notationally perfect and nobody reads them | Modeling obsession substituting for communication | Simplify; use ad hoc intuitive formats for business-facing artifacts | 18, 3 |
| Communication with a stakeholder keeps failing | The architect's fault, not the stakeholder's | Change language, format and framing; adopt the stakeholder's own models (BCG matrix, SCOR) | 16, 19 |
| An engagement manager is needed to talk to executives | Sign of an immature practice | Build direct architect–executive dialog | 16, 19 |
| Consultants deliver a documentation set and leave | The BSP anti-pattern | Restructure toward initiative-based, strategic or developmental engagements with knowledge transfer as the outcome | 19 |

## 15. Process templates

### Initiative Delivery, end to end (Ch 6, 7, 13, 14)
1. Need arrives — planned from a **Roadmap**, urgent from the external environment, or technical from an **IT Roadmap**.
2. Passes **IT investment portfolio management** (the sieve) → enters the **program of work**, signed off including by the CFO.
3. **Initiation** — business executives + architects. Discussion point: business processes. Artifacts: **Initiative Proposal** → **Options Assessment** → **Solution Overview**, paired with a **business case**. Decisions: buy vs. build, outsource vs. in-house. → **Investment committee** approves and funds.
4. **Implementation** — architects + project teams + business representatives. Discussion point: business requirements. Artifacts: optional **Preliminary Solution Design** (re-baseline estimates) → **Solution Design**, paired with a **project management plan**; techniques: user stories, traceability matrices, MoSCoW. → **Design committee** endorses.
5. Build, test, deploy. Hand over to IT service management.
6. Post-release: update Designs to "as-implemented" and archive; update **Landscapes**; update **Standards** if the initiative pioneered something; business owners run people and process change.
7. Years later: retrieve Outline + business case for a **post-implementation benefit review**.

### Mastering one artifact type (Ch 19)
Mandate → stakeholder commitment → architect owner + template → consistent process (develop, update, review, re-approve) → governance body with scheduled meetings → enforcement mechanism (decisions artifacts only) → benefits felt and acknowledged. Then, and only then, start the next type. Pilot in one business unit before propagating.

### Establishing a practice — two routes (Ch 19)
**Historical (safest)**: Outlines + Designs ("solution architecture") → Standards + Landscapes ("IT architecture") → Considerations + Visions ("true" EA). Also serves as a rough three-level maturity model.
**Deliberate (faster, riskier)**: ask "What is our biggest business problem in relation to IT?" → run costs / legacy risk / inflexibility → Standards + Landscapes · no strategic transparency or impact → Considerations + Visions · overruns and unrealized benefits → Outlines + Designs.

### Straw-man escalation ladder (Ch 18)
Whiteboard photos → PowerPoint → Word/Visio artifact → stakeholder approval → governance endorsement. Always draft one format lighter than the destination.

## 16. Maturity checklist (Ch 19)

**Ask executives**: Do you understand what IT is doing? · How IT contributes to your business goals? · Where your IT budget is spent? · How IT is transforming your business? · What business value IT brings?

**Six signs**: all six general types mastered (10–15 subtypes) · processes predictable and repeatable · artifacts and processes reviewed and optimized, often yearly · direct business–architect dialog without intermediaries · lower ratio of urgent and local initiatives than peers · higher percentage of strategic investments than peers.

**Remember**: maturity is uneven — assess **per process** (Initiative Delivery is usually most mature, Strategic Planning least) and **per business unit**. It is manifested "not in doing some special activities... but rather in doing ordinary things better, more systematically and in a more predictable manner."

## 17. Refuse-to-do list

- Fill in the Zachman Framework or follow the TOGAF ADM steps (Ch 3)
- Build an ROI business case for the EA practice itself (Ch 3)
- Start information systems planning from the business strategy (Ch 5)
- Produce a decisions artifact alone on behalf of its stakeholders (Ch 2)
- Change even the colours of a decisions artifact without discussion (Ch 2)
- Deviate from a **Policy** — deviations are tolerated everywhere else (Ch 9)
- Create future-state Landscapes ahead of Visions (Ch 12)
- Copy the book's sample artifacts as ideal specimens (Ch 15)
- Count artifacts as a maturity metric (Ch 15)
- Install a specialized EA tool at the outset of a practice (Ch 18, 19)
- Set indicator values as goals (Ch 18)
- Outsource the architecture function, or hire consultants to build facts artifacts (Ch 19)
- Blame stakeholders for communication failures (Ch 16)
