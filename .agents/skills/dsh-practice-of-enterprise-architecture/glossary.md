# Glossary — The Practice of Enterprise Architecture (Kotusev, 2nd ed., 2021)

**Alignment trap** — the condition where chasing each successive business strategy with strategy-specific systems makes the IT landscape increasingly complex, fragile and inefficient, turning IT into a bottleneck rather than a strategic asset (Ch 5)

**Analytical Reports** — uncommon Considerations subtype; executive-level analyses of relevant technology trends and their potential business impact, taking the form of hype cycles, technology radars, SWOT analyses or vendor analyses; also called whitepapers, position papers, strategy papers (Ch 9)

**Anti-pattern, EA frameworks as** — Kotusev's characterization of TOGAF, Zachman, FEAF and DoDAF: marketing-driven management fads whose prescriptions successful practices never follow (Ch 3, 16, 19)

**ArchiMate** — EA-specific modeling language (Telematica Instituut 2002, The Open Group from 2008) covering business, application and technology layers in active-structure/behavior/passive-structure aspects; too formal for business audiences, practically limited to Landscapes and Designs (Ch 18)

**Architectural directions** — very high-level conceptual and technical architectural requirements handed to the architects writing Outlines or Designs; a documents-based interface between enterprise and solution architects (Ch 13, 14)

**Architectural initiatives** — one of the five initiative types; no business component, only IT; improve landscape quality; proposed top-down by architects from Technology Optimization, approved by IT executives, often funded directly by the CIO (Ch 7)

**Architectural repository** — a tool-based store of all architecturally significant elements and their relationships, governed by a meta-model; the core value proposition of specialized EA tools (Ch 18)

**Architecture debt** (technical debt) — "a temporary deviation from the ideal long-term architectural direction"; a dual notion, characterized both by the corrective actions needed and by the estimated financial cost of those actions (Ch 18)

**Architecture debt register** — a log of taken debts with description, corrective actions, financial estimate, redemption deadline and a minor/medium/major classification; "a special type of Landscapes somewhat resembling IT Roadmaps" (Ch 18)

**Architecture function** — a specialized supporting corporate function, usually a subunit of the IT department reporting to the CIO, responsible for the EA practice; hosts all architects and the governance bodies (Ch 2, 17)

**Architecture managers** — the least represented archetype; heads of architecture functions who define roles, size and structure the function, allocate architects and hire; "meta-architects, i.e. architects of architecture functions and EA-related processes" (Ch 16)

**"Architecture police"** — the reputation an architecture function acquires when it stops projects over minute deviations from overly strict Standards (Ch 10)

**ARIS** (Architecture of Integrated Information Systems) — enterprise modeling approach from August-Wilhelm Scheer (1980s), covering functional, organizational, data and integrating control views; mature but losing popularity (Ch 18)

**Assessment forms** — see *Compliance checklists* and *Governance checklists* (Ch 18)

**Assets development** — an investment-quality indicator; the percentage of IT investments in strategic systems and assets identified in Enterprise System Portfolios and Inventories (Ch 18)

**"Big brain" ideal** — the unattainable notion of an organization acting as one entity able to make globally and locally optimal decisions on all available information (Ch 1)

**BPMN** (Business Process Model and Notation) — process modeling language (BPMI 2004, OMG from 2005) using swimlanes, activities, events, gateways and artifacts; used in Outlines (usually simplified) and Designs (Ch 18)

**BSP** (Business Systems Planning) — IBM's late-1960s/1970s architecture-based planning methodology; the origin of the step-wise planning lineage that runs through Method/1, Information Engineering, EAP, TAFIM, FEAF and TOGAF (Ch 3, 19)

**Budgeting** — the largely administrative process of allocating money to the initiatives selected by IT investment portfolio management (Ch 7)

**Business area architects** — archetype covering all EA domains for one business area, unit, cross-cutting aspect or capability group; "local enterprise architects"; drive local flexibility, autonomy and responsiveness; more common in large decentralized companies (Ch 16)

**Business capabilities** — a general ability or capacity to perform a specific business activity, encompassing the underlying processes, procedures, people, knowledge, incentives and IT systems; one of the five key discussion points (Ch 5)

**Business Capability Models (BCMs)** — essential Visions subtype; structured graphical representations of all organizational business capabilities, their relationship and hierarchy, typically two to four nested levels; answer *where* investments should go (Ch 11)

**Business case** — the largely IT-agnostic financial justification for an initiative, driven by business sponsors and paired with an Outline; uses DCF, NPV, IRR and ROI (Ch 13)

**Business-enabling domains** — the functional domains business, applications and data: they define core business functionality, are visible to business stakeholders, and their decisions are agreed with business (Ch 2)

**Business-focused EA artifacts** — the top row of the CSVLOD model (Considerations, Visions, Outlines): technology-neutral business language, brief and informal, dual, used by business leaders and architects to *manage* IT (Ch 8)

**Business processes** — sequences of concrete activities by specific actors producing valuable outcomes; the discussion point of the initiation step of Initiative Delivery (Ch 5, 6)

**Business requirements** — detailed functional and non-functional specifications for concrete IT systems; the discussion point of the implementation step of Initiative Delivery (Ch 5, 6)

**Business-supporting domains** — the non-functional domains integration, infrastructure and security: unrelated to business functionality, invisible to business, usually not discussed with business (Ch 2)

**Capability Maturity Model (CMM)** — the initial/repeatable/defined/managed/optimizing scale often used for absolute maturity assessment when heatmapping Business Capability Models (Ch 11)

**Changes EA artifacts** — the right column of the CSVLOD model (Outlines, Designs): specific proposed incremental changes, narrow scope, temporary, tactical plans (Ch 8)

**CMDB** (configuration management database) — an ITSM tool owned by IT operations that tracks configuration items; offers auto-discovery, real-time accuracy and a single source of truth, and can substitute for or complement an EA repository (Ch 12, 18)

**Compliance checklists and assessment forms** — lists of IT Principles, Guidelines and other requirements used by project teams for architectural self-assessment, engaging architects only where compliance cannot be met (Ch 18)

**Conceptual Data Models** — uncommon Considerations subtype; abstract, business-oriented definitions of the main data entities critical for the business and their relationships; basis for Logical Data Models (Ch 9)

**Considerations** — business-focused rules; global conceptual rules and fundamental considerations important for business and relevant to IT; the "global compass" saying which directions are right or wrong (Ch 4, 9)

**Context Diagrams** — uncommon Visions subtype; high-level graphical descriptions of current operational flows; uniquely among Visions they are **facts** artifacts of the current state (Ch 11)

**Core diagram** — a less popular Visions subtype; a one-page depiction of the target operating model with standardized processes and shared data, far more stable than Target States (Ch 11)

**CSVLOD model** — Kotusev's central conceptualization: **C**onsiderations, **S**tandards, **V**isions, **L**andscapes, **O**utlines, **D**esigns, produced by intersecting *what* (rules/structures/changes) with *how* (business-focused/IT-focused) (Ch 4, 8, 15)

**Decision path** — the flow of planning decisions from strategy to implemented systems; the typical path is Principles → Business Capability Models → Roadmaps → Solution Overviews → Solution Designs (Ch 15)

**Decisions EA artifacts** — artifacts representing made planning decisions: future-focused or stateless, subjective, developed collaboratively by all stakeholders, value realized during development; **process over product** (Ch 2)

**Deliberate path** — the newer, proactive route to establishing an EA practice: identify the biggest business problem in relation to IT and introduce the artifacts that solve it (Ch 19)

**Designs** — IT-focused changes; detailed technical and functional descriptions of separate IT projects actionable for project teams; "the last EA artifacts in the EA delivery chain" (Ch 4, 14)

**Design committees** — tactical IT governance bodies (design authorities, architecture review boards) approving all Designs; meet biweekly, weekly or more often (Ch 17)

**Developmental engagements** — the most advanced EA consulting type; consultants act as architecture managers to establish or improve the internal practice; outcome is practice improvement plus knowledge transfer (Ch 19)

**Direction Statements** — uncommon Considerations subtype; conceptual messages communicating major organization-wide decisions with far-reaching consequences; the most action-oriented Considerations; mostly temporary (Ch 9)

**Discussion points** — see *Five key discussion points* (Ch 5)

**Disposable architectural diagrams** — ad hoc temporary drawings with no consistent meaning, created for a single discussion; the first class of exception to the CSVLOD model (Ch 15)

**Diversification** — the operating model with low process standardization and low data integration; diverse independent business units; local applications with global infrastructure (Ch 5)

**Domain architects** — archetype covering one or a few EA domains organization-wide; drive global consolidation and optimization; more common in centralized companies; split into business-enabling and business-supporting types with different process duties (Ch 16)

**Duality** — the property of an EA artifact carrying information relevant to two audiences at once in a format appealing to both; **explicit** (different parts for different audiences) or **implicit** (the same part read differently) (Ch 2)

**EA artifact** — a special document describing some aspect of an organization from an integrated business and IT perspective (Ch 2)

**EA practice** — the organizational practice of using EA artifacts; also called enterprise architecture management (EAM) (Ch 2)

**EA uncertainty principle** — organizations can be planned for broader scopes and longer horizons in less detail, or narrower scopes and shorter horizons in more detail, but not for broad scopes and long horizons in great detail (Ch 5)

**Enterprise Architecture on a Page** — Kotusev's single-page poster placing all 24 subtypes with schematic representations on the CSVLOD structure, colour-coded by prevalence; at eaonapage.com (Ch 15)

**Enterprise architects** — the "broadest" archetype: organization-wide planning across all EA domains; in small organizations they do everything, in large ones they act as chief integrators (Ch 16)

**Enterprise System Portfolios** — common Landscapes subtype; structured high-level mappings of all essential IT systems to relevant business capabilities, mirroring the organization's BCM (Ch 12)

**Engagement managers** — a role adjacent to but distinct from architects: "strategic business analysts" who visit executives, elicit considerations and collect demand; "seemingly more characteristic for immature EA practices" (Ch 16)

**Escalation** — a governance resolution delegating a decision to a more authoritative committee; used for major deviations with critical business impact (Ch 17)

**Essential artifacts** — the eight subtypes used in more than 50% of established EA practices: Principles, Technology Reference Models, Guidelines, Business Capability Models, Roadmaps, Landscape Diagrams, Solution Overviews, Solution Designs (Ch 8)

**Exemption** (dispensation) — a governance resolution approving a decision despite misalignment, sometimes with an obligation to correct later (a *conditional endorsement*) and usually with an architecture debt registered (Ch 17)

**Facts EA artifacts** — artifacts documenting objective current-state facts: developed by individual architects alone, optimized for storage and searchability, value realized after completion; **product over process** (Ch 2)

**FEA / FEAF** — the U.S. Federal Enterprise Architecture program initiated 1999, which "delivered only the heaps of largely useless architectural documents and reportedly wasted more than one billion dollars" (Ch 3)

**Five key discussion points** — operating model, business capabilities, specific business needs, business processes, business requirements: the dual middle ground between business and IT, in descending order of abstraction, scope and horizon (Ch 5)

**Fundamental initiatives** — strategy-agnostic initiatives addressing profound permanent needs, often derived from the requirements of the adopted operating model; grow permanent capabilities and build the digitized platform (Ch 7)

**Governance checklists and assessment forms** — criteria-based forms filled early in an initiative to judge its architectural significance (low/moderate/high impact) and set the required level of architect involvement and governance (Ch 18)

**Guidelines** — essential Standards subtype; IT-specific implementation-level prescriptions in narrow technology-specific areas; specify *how* technologies should be used (Ch 10)

**Heatmapping** — marking a Business Capability Model to show where improvement is needed; approaches include collective judgment, intuitive strategic marking, maturity assessment and gap analysis, capability component analysis and strategy mapping (Ch 11)

**Historical path** — the mainstream evolutionary route to an EA practice, from simple to complex: Outlines and Designs ("solution architecture") → Standards and Landscapes ("IT architecture") → Considerations and Visions ("true" EA); doubles as a rough three-level maturity model (Ch 19)

**Initiation** — the first inherent step of Initiative Delivery: business executives and architects formulate goals, generate and evaluate options and secure approval; the key artifacts are Outlines (Ch 6)

**Initiative-based engagements** — the most basic EA consulting type; consultants act as solution architects within Initiative Delivery, filling technology knowledge gaps or absorbing workload peaks (Ch 19)

**Initiative Delivery** — the *need-to-solution* process; multiple parallel instances, one per active initiative; a sequential two-step process (initiation, implementation) revolving around Outlines and Designs (Ch 6)

**Initiative Proposals** — uncommon Outlines subtype; very early idea-level descriptions of proposed initiatives and their justifications, used to filter out futile ideas (Ch 13)

**Inventories** — common Landscapes subtype; structured catalogs of available IT assets with their properties; the only EA artifacts typically represented in tabular form (Ch 12)

**"Ivory tower" syndrome** — the failure mode where insufficient stakeholder involvement makes artifacts inexplicable, irrelevant and useless — "architecture is produced for the sake of architecture" (Ch 6)

**IT initiative** — the key unit of work in an EA practice; understood as all change initiatives with an IT component, not only those with only an IT component (Ch 7)

**IT investment portfolio management** — the executive-level process of selecting the most critical candidate initiatives within budget; it acts as a filter or sieve around Initiative Delivery and produces the program of work (Ch 7)

**IT Principles** — common Standards subtype; global high-level IT-specific guidelines influencing all IT-related decisions; typically 10–20 or more, grouped by EA domain (Ch 10)

**IT Roadmaps** — common Landscapes subtype; structured graphical views of planned purely technical initiatives with no visible business impact, usually 1–2 year horizons; uniquely among Landscapes they are **decisions** artifacts (Ch 12)

**IT-focused EA artifacts** — the bottom row of the CSVLOD model (Standards, Landscapes, Designs): technical language, potentially voluminous and formal, internal IT tools invisible to business, used to *organize* IT (Ch 8)

**Key decisions / Key design decisions (KDDs)** — one-page bullet-point digests of the most significant planning or technical decisions and their justifications, attached to Outlines or Designs (Ch 13, 14)

**Landscape Diagrams** — essential Landscapes subtype; technical "boxes and arrows" schemes of varying scope and granularity; multidomain (free-form or layered) or domain-specific (Ch 12)

**Landscapes** — IT-focused structures; high-level technical descriptions of the corporate IT landscape acting as a knowledge base of reference materials for architects; predominantly **facts** artifacts (Ch 4, 12)

**Local initiatives** — initiatives addressing critical tactical needs of individual business units; proposed bottom-up via business proposals, then judged by global executives and architects for organizational fitness (Ch 7)

**Logical Data Models** — uncommon Standards subtype; logical or platform-specific definitions of common data entities and their relationships; the IT-specific counterpart of Conceptual Data Models (Ch 10)

**Mini-designs** — a few pages for minor solutions extending existing functionality, approved by simplified procedures; reduce architectural overhead and bureaucracy (Ch 14)

**Minutes of the meeting (MoM)** — the record of a governance committee meeting: date, participants, agenda items with associated artifacts and, above all, the approval decisions to be followed up (Ch 17)

**MoSCoW** — must have / should have / could have / will not have; a requirements prioritization framework used when developing Designs (Ch 14)

**Operating model** — the desired level of organization-wide business process standardization and data integration across major business units; the most fundamental and stable discussion point (Ch 5)

**Options Assessments** — common Outlines subtype; lists of high-level implementation options with pros, cons, risks and weighted scores; some organizations require at least three options plus a "Do nothing" option (Ch 13)

**Outlines** — business-focused changes; high-level descriptions of separate IT initiatives understandable to business leaders; "benefit, time and price tags for proposed IT investments" (Ch 4, 13)

**Patterns** — common Standards subtype; generic reusable solutions to commonly occurring design problems, specified as description, applicability, rationale and solution; intangible until instantiated (Ch 10)

**People / Process / Technology** — the three aspects that must change in harmony for an information system to deliver business value (Ch 1)

**Permanent EA artifacts** — broad-scope, long-lived artifacts created once, periodically updated and constantly used (Ch 2)

**Platform utilization** — an investment-quality indicator; the ratio of investment in improving existing IT assets to investment in creating new ones (Ch 18)

**Policies** — common Considerations subtype; overarching organizational norms of a restrictive nature giving compulsory prescriptions; specify what must **not** be done and **cannot be deviated from** (Ch 9)

**Preliminary Solution Designs** — uncommon Designs subtype; intermediate "halfway" artifacts between Outlines and Solution Designs whose primary purpose is to refine and reaffirm earlier time and cost estimates; average 20–40 pages (Ch 14)

**Prevalence bands** — essential (>50% of practices), common (~25–50%), uncommon (~10–25%); eight subtypes in each (Ch 8)

**Principles** — essential Considerations subtype; global high-level guidelines influencing all decision-making and planning, defined by title, statement, rationale and implications; usually 5–10 organization-wide (Ch 9)

**Program architects** — a subclass of solution architects: large-scale architects for broad-scope transformation programs who also coordinate the other solution architects on the initiative (Ch 16)

**Program of work** — the document listing the initiatives selected and funded for the forthcoming budgeting period, with brief descriptions, estimated costs and commencement dates; signed off including by the CFO (Ch 7)

**Project management plans** — the largely non-technical, project-manager-driven counterpart to Designs, covering milestones, procurement, staffing, budgeting and risk (Ch 14)

**Replication** — the operating model with high process standardization and low data integration; similar but independent units; globally standardized, locally owned applications (Ch 5)

**Roadmaps** — essential Visions subtype; structured graphical views of all planned IT initiatives in specific business areas having direct business value; answer *when* investments should be made; usually up to three years (Ch 11)

**Roadmaps alignment** — an investment-quality indicator; the ratio of planned Roadmap investments to unplanned urgent ones (Ch 18)

**Rules EA artifacts** — the left column of the CSVLOD model (Considerations, Standards): general global rules, very broad scope, often textual, permanent, intangible and uncountable (Ch 8)

**Solution architects** — the "narrowest" and most numerous archetype (**60–80% of all architects**); carry out only Initiative Delivery; develop Outlines and Designs (Ch 16)

**Solution Designs** — essential Designs subtype; detailed technical and functional specifications actionable for project teams; average 25–50 pages; equivalent to full system architectures (Ch 14)

**Solution Overviews** — essential Outlines subtype; high-level descriptions of proposed IT solutions understandable to business leaders; typically 15–30 pages (Ch 13)

**Specific business needs** — a general idea to address a particular business problem using IT; the discussion point that corresponds directly to individual initiatives (Ch 5)

**Standards** — IT-focused rules; global technical rules, standards, patterns and best practices representing proven reusable means for IT systems implementation; "backend" artifacts by architects for architects (Ch 4, 10)

**Straw-man architecture** ("napkin" architecture) — an elementary draft of a decisions EA artifact produced very quickly, always one format lighter than the real artifact, used to discuss principal decisions and exclude unfeasible alternatives early (Ch 18)

**Strategic contribution** — an investment-quality indicator; the percentage of IT investments contributing to the achievement of long-term Target States (Ch 18)

**Strategic engagements** — the EA consulting type where consultants act as enterprise, business area or domain architects within Strategic Planning and Technology Optimization; the critical outcome is knowledge transfer (Ch 19)

**Strategic focus** — an investment-quality indicator; the percentage of IT investments uplifting the capabilities heatmapped in Business Capability Models (Ch 18)

**Strategic initiatives** — initiatives stemming directly from the current business strategy; the "workhorses" of strategy execution, identified top-down from required business capabilities (Ch 7)

**Strategic Planning** — the *strategy-to-portfolio* process; continuous and largely unstructured; translates external environment factors into Considerations and Visions; integrated with regular strategic management (Ch 6)

**Strategy committees** — strategic business governance bodies approving all significant changes in Considerations and Visions; participants include the CEO and other C-level executives; meet monthly or as necessary (Ch 17)

**Structures EA artifacts** — the middle column of the CSVLOD model (Visions, Landscapes): high-level structures, broad scope, usually graphical, permanent, "maps" facilitating decision-making (Ch 8)

**Subtypes** — the 24 concrete recurring artifact classes beneath the six general types; many useful artifacts are unique and belong to no subtype (Ch 8)

**Target States** — common Visions subtype; high-level graphical descriptions of the desired long-term future state; **evolutionary** (change composition, not structure) or **transformational** (change the landscape structure itself) (Ch 11)

**Technical adaptivity** — a deviation indicator; the percentage of IT projects that led to creating new or updating existing Standards; higher means faster organizational learning (Ch 18)

**Technical designers** — "microarchitects" inside project teams who co-create Designs and then produce concrete implementation plans; belong to IT delivery functions, not architecture functions (Ch 16)

**Technological heterogeneity** — a deviation indicator; the percentage of IT projects based on technologies missing from or tagged retiring in Technology Reference Models (Ch 18)

**Technology committees** — strategic IT governance bodies approving substantial changes in most Standards and some Landscapes; chaired around the CIO; meet monthly or as necessary (Ch 17)

**Technology Optimization** — the *structure-to-rationalization* process; an internal IT-specific housekeeping process run largely by architects alone; revolves around Standards and Landscapes; also addresses architecture debt (Ch 6)

**Technology Reference Models (TRMs)** — essential Standards subtype; structured graphical representations of all technologies used in an organization, hierarchically organized and status colour-coded; specify *what* technologies should be used; combine facts and decisions (Ch 10)

**Temporary EA artifacts** — narrow-scope, short-lived artifacts created for a purpose, used as intended and then immediately discarded (Ch 2)

**Three boundaries** — the three axes of diversity separating all relevant actors: business vs. IT, strategic vs. tactical, internal vs. external (Ch 1)

**Total cost of ownership (TCO)** — the whole-lifecycle cost concept used in Outlines alongside CAPEX, OPEX, future maintenance and the delayed costs of redeeming architecture debts (Ch 13)

**Unification** — the operating model with high process standardization and high data integration; similar and interdependent units; global applications, databases and infrastructure, often comprehensive ERP (Ch 5)

**Urgent initiatives** — initiatives addressing unanticipated critical needs arriving directly from the external business environment; fundamentally unplanned; the best that can be done is align them to the strategic direction (Ch 7)

**Useless Zone** — the region of the EA uncertainty principle where highly abstract artifacts are applied to narrow scopes and short horizons, lacking the implementation-level detail the stage requires (Ch 5)

**Unachievable Zone** — the region where highly detailed artifacts are attempted for broad scopes and long horizons, where inherent uncertainty is too high (Ch 5)

**Value Chains** — uncommon Visions subtype; structured graphical representations of the added value chain; conceptually similar to BCMs but dynamic; their relative advantage is familiarity to MBA-holding executives (Ch 11)

**Visions** — business-focused structures; high-level conceptual descriptions of an organization from the business perspective; the "global map" showing where the organization needs to go and what steps get it there (Ch 4, 11)

**Wicked problem** — the class of problem, including developing new information systems, where a perfect solution cannot be found analytically; the reason excessively detailed Designs end in analysis paralysis (Ch 14)
