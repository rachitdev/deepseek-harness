---
name: dsh-practice-of-enterprise-architecture
description: Svyatoslav Kotusev's "The Practice of Enterprise Architecture: A Modern Approach to Business and IT Alignment" (2nd edition, SK Publishing, 2021) — an evidence-based, framework-skeptical account of how EA is actually practiced. Use when working on enterprise architecture practices, the CSVLOD model, EA artifacts (principles, technology reference models, business capability models, roadmaps, landscape diagrams, solution overviews, solution designs), business and IT alignment, operating models, architecture functions and governance committees, architect roles and career paths, architecture debt, EA tools and modeling languages (ArchiMate, UML, BPMN, ARIS), EA maturity, or critiques of TOGAF, Zachman, FEAF and DoDAF.
---

# The Practice of Enterprise Architecture: A Modern Approach to Business and IT Alignment

**Author**: Svyatoslav Kotusev · **Pages**: 566 (2nd edition, SK Publishing, Melbourne, 2021)
**Chapters**: 19 · **Generated**: 2026-08-23

## How to Use This Skill

- **No args** — read this file for the CSVLOD model, the six general types of EA artifacts, the three EA-related processes, and Kotusev's case against step-wise EA frameworks.
- **With a topic** — go to `## Topic Index` below, then open the named chapter files; use `glossary.md` for a term, `cheatsheet.md` for a decision table or threshold.
- **With a chapter** — open `chapters/chNN-slug.md` directly from `## Chapter Index`. Each is study-depth: core idea, frameworks, key concepts, mental models, anti-patterns, reference tables, a worked example reproducing the book's own figures and numbers, takeaways and cross-links.
- **Browse** — Part I (Ch 1–7) concepts and mechanics · Part II (Ch 8–15) the CSVLOD model, one chapter per artifact type · Part III (Ch 16–19) architects, functions, instruments, lifecycle. `patterns.md` collects the reusable moves as When / How / Trade-offs.

## Core Frameworks & Mental Models

**EA is a collection of documents for communication — not a blueprint of the enterprise.** Kotusev's definition: *a collection of special documents (artifacts) describing various aspects of an organization from an integrated business and IT perspective, intended to bridge the communication gap between business and IT stakeholders, facilitate information systems planning and thereby improve business and IT alignment.* Misalignment's root cause is **miscommunication**, not bad technology or bad strategy. Five actor groups are separated by three boundaries: business vs. IT, strategic vs. tactical, internal vs. external. Everything else follows: modeling is a minor part, "modeling for the sake of modeling is a useless activity", and the practice is "mostly about communicating and doing, rather than thinking."

**The CSVLOD model — two orthogonal dimensions, six general types.** *What?* rules → structures → changes. *How?* business-focused → IT-focused. Their intersection:

| | Rules | Structures | Changes |
|---|---|---|---|
| **Business-focused** | **C**onsiderations (principles, policies) | **V**isions (business capability models, roadmaps) | **O**utlines (solution overviews, options assessments) |
| **IT-focused** | **S**tandards (technology reference models, guidelines) | **L**andscapes (landscape diagrams, inventories) | **D**esigns (solution designs) |

Business-focused artifacts are **dual** and let executives *manage* IT; IT-focused artifacts are architect-only tools to *organize* IT. Beneath the six sit **24 subtypes** in three prevalence bands — 8 **essential** (>50% of practices), 8 **common** (~25–50%), 8 **uncommon** (~10–25%). Both dimensions are actually **continuous axes** (Ch 15): an artifact's position on the plane predicts its properties. Successful practices use **10–15 subtypes**, not all 24; essential artifacts are a benchmark, not a mandate — but you should be able to explain why you skip one. The model is **descriptive, "food for thought", never prescriptive**.

**Decisions vs. facts artifacts — the distinction that kills practices when ignored.** *Decisions* artifacts (everything except most Landscapes and some Standards) represent made planning decisions, are subjective, cover the future or are stateless, and are developed **collaboratively by all stakeholders** — their value is realized *during* development, so **process beats product**. *Facts* artifacts document objective current-state facts, are developed by individual architects alone, and **product beats process**. The fatal shortcut is producing decisions artifacts the way facts artifacts are produced — interviewing stakeholders and then writing the document *for* them. That turns the practice "from the driver of communication into the factory of useless documents."

**Three simultaneous processes, three artifact pairs — an EA practice is not a step-wise process.** *Strategic Planning* (Considerations + Visions) = **strategy-to-portfolio**; *Initiative Delivery* (Outlines + Designs) = **need-to-solution**; *Technology Optimization* (Standards + Landscapes) = **structure-to-rationalization**. Only Initiative Delivery is linear (initiation → implementation). The value is in the three bidirectional flows between them. A parallel triad in Ch 17: the **architecture function (planning) → IT delivery function (implementation) → IT support function (maintenance)** are the three core IT-related capabilities, with architecture as "a 'front door' to the IT department for business leaders."

**Never start architectural planning from the business strategy.** A 3–5 year strategy is vague or absent, gives no direction for IT, changes constantly, and demands strategy-specific systems that outlive it — producing the **alignment trap**. It is simultaneously too abstract and too specific. Instead use the **five key discussion points**, the middle ground meaningful to both sides: operating model → business capabilities → specific business needs → business processes → business requirements, each at its own abstraction level, scope and horizon. The **operating model** (process standardization × data integration → diversification / coordination / replication / unification) is the stable foundation because "a business strategy is based on the assumptions of what is going to change, while an operating model is based on the premise of what is not going to change."

**The EA uncertainty principle.** Organizations can be planned for *broader scopes and longer horizons in less detail*, or *narrower scopes and shorter horizons in more detail*, but **never for broad scopes and long horizons in great detail**. Violate it upward and you land in the **Useless Zone** (abstract descriptions of the local short-term); violate it downward and you land in the **Unachievable Zone**. This is the single most useful sizing test for any proposed artifact, and it explains the recurring "single biggest threat" in Chapters 11–14: overly detailed Visions, Landscapes, Outlines and Designs.

**Kotusev's case against EA frameworks — treat them as proven anti-patterns.** BSP and Method/1 (1970s) → Information Engineering (1980s) → EAP and TAFIM (1990s) → FEAF (2000s) → TOGAF (2010s) are "slightly different variations of the same step-wise formal planning approach": analyze the current state, define a future architecture aligned to strategy, produce a migration roadmap. Successful practices "never fill the cells of the Zachman Framework, never follow the TOGAF ADM steps, never develop the heaps of TOGAF-recommended artifacts" — even in organizations on The Open Group's own TOGAF user list — and do not follow the general logic of "plan the future state, then implement it." Frameworks contributed to real best practices "only in the same sense in which wars contributed to the development of modern medicine." The FEA program (from 1999) "delivered only the heaps of largely useless architectural documents and reportedly wasted more than one billion dollars", after which "the very word 'architecture' subsequently became a bad word in many organizations." Framework certifications "cannot be helpful to aspiring architects (but can be harmful if taken seriously)." Real best practices **emerged in the industry**; this book is descriptive, and practices are idiosyncratic and must be fine-tuned in-house.

**Decision rules worth memorizing.** Classify every artifact on three axes before doing anything with it: contents, meaning (decisions/facts), lifecycle (permanent/temporary) — then use the matching development process. Apply the **smoke test to Considerations**: a meaningful one "implies trade-offs, can be potentially disagreed with and stays meaningful when negated." Keep **Visions** detailed enough to fund the next budget cycle and no more; keep **Outlines** detailed enough to evaluate the initiative and no more; keep **Landscapes** at an abstraction level you can actually keep current, because "higher-level and up-to-date beats detailed and outdated." Never lead the business with future-state Landscapes — those are "the attempts of IT to lead business". In **Standards**, "'the more, the better' is an inappropriate and detrimental attitude", and functions that stop projects over minute deviations become the **"architecture police"**. Tolerate justified deviations everywhere except Policies, and log them as **architecture debt**. Modeling languages and EA tools are secondary — "software tools cannot organize an EA practice for people", and "the obsession with correct modeling can easily guarantee its failure."

## Chapter Index

| # | Chapter | What it gives you |
|---|---|---|
| 1 | [Introduction](chapters/ch01-introduction.md) | The alignment problem, three boundaries, five actor groups, EA defined as communication |
| 2 | [The Concept of Enterprise Architecture](chapters/ch02-the-concept-of-enterprise-architecture.md) | Six EA domains, duality, decisions vs. facts, permanent vs. temporary |
| 3 | [The Role of EA Practice](chapters/ch03-the-role-of-ea-practice.md) | Benefits chain, no-ROI argument, the twelve "what EA is not", framework critique |
| 4 | [EA and City Planning](chapters/ch04-enterprise-architecture-and-city-planning.md) | The city analogy introducing the six CSVLOD types and their relationships |
| 5 | [The Dialog Between Business and IT](chapters/ch05-the-dialog-between-business-and-it.md) | Why strategy fails as input, four operating models, five discussion points, uncertainty principle |
| 6 | [Processes of EA Practice](chapters/ch06-processes-of-ea-practice.md) | Strategic Planning, Initiative Delivery, Technology Optimization and their three synergies |
| 7 | [IT Initiatives and EA](chapters/ch07-it-initiatives-and-enterprise-architecture.md) | Five initiative types; portfolio management and budgeting as the funding filter |
| 8 | [The CSVLOD Model](chapters/ch08-the-csvlod-model.md) | The formal two-dimensional taxonomy; all 24 subtypes with definitions and prevalence |
| 9 | [Considerations](chapters/ch09-considerations.md) | Principles, Policies, Conceptual Data Models, Analytical Reports, Direction Statements |
| 10 | [Standards](chapters/ch10-standards.md) | Technology Reference Models, Guidelines, Patterns, IT Principles, Logical Data Models |
| 11 | [Visions](chapters/ch11-visions.md) | Business Capability Models, Roadmaps, Target States, Value Chains, Context Diagrams; heatmapping |
| 12 | [Landscapes](chapters/ch12-landscapes.md) | Landscape Diagrams, Inventories, Enterprise System Portfolios, IT Roadmaps; status codings |
| 13 | [Outlines](chapters/ch13-outlines.md) | Solution Overviews, Options Assessments, Initiative Proposals; business cases; benefit reviews |
| 14 | [Designs](chapters/ch14-designs.md) | Solution Designs and Preliminary Solution Designs; architecture vs. design; project plans |
| 15 | [The CSVLOD Model Revisited](chapters/ch15-the-csvlod-model-revisited.md) | Continuous axes, decision paths, artifacts as decision points, exceptions, EA on a Page |
| 16 | [Architects in EA Practice](chapters/ch16-architects-in-ea-practice.md) | Five qualities, five archetypes, engagement managers and technical designers, career paths |
| 17 | [Architecture Functions in Organizations](chapters/ch17-architecture-functions-in-organizations.md) | Sizing, centralization, the four governance committees, resolutions and escalation |
| 18 | [Instruments for Enterprise Architecture](chapters/ch18-instruments-for-enterprise-architecture.md) | Modeling languages, tools vs. CMDBs, measurements, templates, checklists, architecture debt, agility dimensions |
| 19 | [The Lifecycle of EA Practice](chapters/ch19-the-lifecycle-of-ea-practice.md) | Historical vs. deliberate path, organizational acceptance, maturity signs, EA consulting |

## Topic Index

| Topic | Chapters |
|---|---|
| Business and IT alignment; the communication thesis | 1, 3, 5 |
| CSVLOD model, taxonomy, artifact classification | 2, 4, 8, 15 |
| Decisions vs. facts EA artifacts | 2, 8, 12, 18 |
| EA domains (business, applications, data, integration, infrastructure, security) | 2, 8, 16 |
| Operating models (diversification, coordination, replication, unification) | 5, 10, 11, 17 |
| Business capabilities, capability maps, heatmapping | 5, 11, 15, 18 |
| Roadmaps, IT Roadmaps, investment planning | 7, 11, 12 |
| Principles, policies, IT principles, guidelines | 9, 10, 18 |
| Technology reference models, technology portfolios | 10, 12, 18, 19 |
| Landscape documentation, inventories, CMDBs | 12, 18 |
| Solution overviews, options assessments, business cases | 13, 7 |
| Solution designs, architecture vs. design, MoSCoW | 14, 18 |
| The three EA-related processes | 6, 7, 16, 19 |
| IT initiatives, portfolio management, budgeting | 7, 11, 13, 18 |
| Architects: skills, archetypes, careers | 16, 17, 19 |
| Architecture functions, sizing, centralization | 17, 16 |
| Architecture governance, committees, exemptions, escalation | 17, 13, 14, 18 |
| Architecture debt | 6, 9, 10, 13, 17, 18, 19 |
| Modeling languages (ArchiMate, UML, BPMN, ARIS) | 18, 14, 3 |
| EA software tools and repositories | 18, 12, 19 |
| Measurements, indicators, landscape complexity | 18, 19 |
| Agility, "agile EA", upfront planning | 18, 15, 14 |
| Establishing an EA practice; maturity | 19, 18, 3 |
| EA consulting engagements | 19, 3 |
| Critique of TOGAF, Zachman, FEAF, DoDAF, BSP, EAP | 3, 16, 19 |
| The EA uncertainty principle | 5, 11, 13, 15 |
| Anti-patterns and "ivory tower" failures | 2, 3, 6, 10, 11, 12, 19 |

## Supporting Files

- [glossary.md](glossary.md) — ~140 terms in Kotusev's own vocabulary, each tagged with its chapter.
- [patterns.md](patterns.md) — the book's reusable moves as **When / How / Trade-offs**, grouped by the book's three parts.
- [cheatsheet.md](cheatsheet.md) — decision tables, thresholds and default values, selection matrices, tells-and-smells (symptom → diagnosis → fix), and process templates.

## Scope & Limits

- **Edition**: Second edition, 2021 (first edition 2018). Based on Kotusev's own research programs of 2013–2018 and 2018–2020, drawing on empirical evidence from tens of real organizations. Numbers, vendor lists and tool names reflect roughly 2020–2021 and will have drifted — the tool vendor list, the "two-thirds of organizations use specialized EA tools" figure, IT spending statistics and product examples (Java EE, Oracle RDBMS, AngularJS) date the material most visibly.
- **Deliberately descriptive, not prescriptive.** The book supplies no step-by-step methodology, and Kotusev insists practices are idiosyncratic and must be established in-house. Sample artifacts in Chapters 9–14 are "characteristic, typical or 'average' samples", explicitly not ideal specimens to copy.
- **Strongly opinionated on frameworks.** The critique of TOGAF, Zachman, FEAF, DoDAF and EA certification is the author's argued position, presented here as he presents it. Users needing a balanced comparison of framework literature should look elsewhere.
- **Covered here**: all 19 chapters at study depth. **Not covered**: the Preface, Afterword, **Appendix A** (The Origin of EA and Modern EA Best Practices — the detailed historical analysis of BSP through TOGAF), **Appendix B** (Enterprise Architecture Teaching Pack), the ~1,000 endnotes, the References and the Index. Appendix A carries much of the evidentiary backing for the framework critique summarized in Chapter 3; consult the book directly for it.
- **Not a technical manual.** No code, no configuration, no notation reference. ArchiMate, UML, BPMN and ARIS are described at the level of what they cover and where they apply, not how to draw in them.
- **Terminology caution**: several subtype names ("Visions", "Outlines", "Considerations") are Kotusev's coinages for recurring artifact classes the industry names inconsistently; the chapter files list the common industry synonyms for each.
- The source text was extracted from a complete PDF with a clean text layer. Figure images are not reproducible in text, so figure content is reconstructed from captions, surrounding prose and the OCR-legible labels; a small number of figures (e.g. 18.9, 19.4, the "EA on a Page" poster) are described rather than tabulated.
