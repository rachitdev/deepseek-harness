# Chapter 7 — Using This Book as a Practical Guide

## Core Idea
The SABSA® **Development Process** is derived directly from the SABSA® Model: the layered structure *implies* a layer-by-layer development sequence. Wrapped around it is the four-phase **SABSA® Lifecycle** — Strategy and Concept, Design, Implement, Manage and Measure — which loops, because operational experience becomes new business requirements. This chapter is the single place where the whole process lives; the authors deliberately grouped it here rather than distributing it, accepting that some terms (trust model, domain model) appear before their explanations.

## Frameworks Introduced

### The SABSA Development Process (Figure 7-1)
Layers are developed **in sequence**, with one exception: **operational security architecture is developed in parallel**, though it cannot really begin until the conceptual security architecture is defined.

There is a **natural break** between the conceptual and logical layers. The conceptual architecture (the architect's vision) concludes the Strategy and Concept phase. *"It is essential to gain buy-in and sign-off to these strategic visions before heavily committing resources to what follows."* The end of the conceptual activity is a milestone at which to stop, draw breath and reach consensus.

### The SABSA Lifecycle (Figure 7-2) — four phases
| Phase | Contains | Ends with |
|---|---|---|
| **Strategy and Concept** | Contextual + Conceptual security architecture | Sign-off / buy-in milestone |
| **Design** | Logical + Physical + Component + Operational security architecture | Sign-off and agreement milestone |
| **Implement** | Construction of the designed systems and processes | Tested, documented implementations |
| **Manage and Measure** | Operational management, telling you whether the systems really meet the original business requirements | Feeds back into a new Strategy and Concept phase |

Off-page connector **25** on the process flowcharts runs all the way back from the final Manage and Measure flowchart into the gathering of current-status information — irrelevant on a first greenfield development, very important when building on previous foundations.

### Key to the process diagrams (Figure 7-3)
Symbols: process steps; off-page connector; milestone; standard input documents; a collection of output documents comprising a given architectural layer. The diagrams are laid out in columns: **inputs on the left**, **sub-process activities in the next column**, **deliverable documents on the right**, grouped inside a box labelled with the SABSA layer.

### Working documents vs deliverables
The SABSA Matrix format is for *thinking* — "there is no need for you actually to format the information like this." What you gather at the contextual stage is the substance of a series of **working documents**: field notes, pre-existing enterprise documents, interview notes, notes summarising several sources. They are ad hoc, have no recommended structure, and are **not formal deliverables**. The deliverables are the products of analysing and synthesising them.

### Two databases feed the process
- **Business Attributes Database** — used **twice**. First (Figure 7-4, contextual) *only to prompt your thinking* on business drivers and related assets. Second (Figure 7-5, conceptual) to develop the full Business Attributes Profile with enterprise-specific definitions, metric types, measurement approaches and performance targets. *"The Business Attributes Profile is the conceptualisation of the real business strategy, drivers, assets, goals and objectives."*
- **Threats Database** — an up-to-date list of threat types and threat agents to prompt thinking in the risk assessment (detailed in Chapter 9; operational view in Chapter 15).

### Prerequisites the security architecture programme does NOT create
Two models must **pre-exist** from the wider enterprise architecture programme:
- **The business process model** (contextual). If the organisation has no process model and no real concept of business processes in its culture, *some of the risk assessment work at the contextual layer may be faulty*, because it will not have been possible to identify critical functions that interrupt critical processes — the Missing Taxis failure from Chapter 5.
- **The business information model / information architecture** (logical). "You need it, and without it you will find it hard to proceed."
Also assumed pre-existing: the **business data model** (physical) and the **data dictionary** (component); and externally available: the **security mechanisms database** (Chapter 12) and **security products market information** from vendors and analysts.

### Where the functional specification lives
The functional requirements specification *is* part of the logical architecture — but at the enterprise-architecture stage you are creating a blueprint/road map guiding individual development projects, not defining every business project in detail, so you deliberately hold back on functional requirements. Each project defines its own at detailed design. If a project's functional requirements **conflict** with the overarching architecture, you have misjudged the architectural requirements — "it happens, not everything you do is perfect" — hence the need for an **architecture maintenance process** (Chapter 8).

### Starting in the middle
Sometimes you must build on work others did that may not fit the model. Introduce two extra process steps: **review and validate business requirements / contextual security architecture**, and **review and validate conceptual security architecture**. If the review shows inadequacies, propose re-work steps with a business case based on your review findings. The warning: *"If you, as the architect, build your house on sand instead of rock, then it will probably fall over later on, and you will take the blame! If you are ordered to proceed against your own better judgement, make sure that you carefully document your advice and lodge it where it can be retrieved and authenticated if (when?) the building starts to subside."*

## Reference Tables

### Deliverables by architecture layer

**Contextual Security Architecture (Figure 7-4)**
- The business model — business drivers, including business assets, goals and objectives, mapped to SABSA Business Attributes
- The SABSA Business Risk Model, in the form of a risk assessment matrix, driven from the SABSA Business Attributes
- The business process model
- The business organisation and relationships model
- The business geography model
- The business time-dependency model

*Contextual sub-processes:* Gather, Assess and Analyse Business Requirements (business strategy, drivers, goals and objectives; critical success factors, motivations and risks; business processes and functions; business people and organisations; business locations and time dependencies; budgets, technical issues and other constraints) · Gather, Assess and Analyse Current State (technology infrastructure; service and system management; security policy and practices; management processes) · Describe the Business (use the Business Attributes list as a prompt to state key business drivers) · Extract Other Requirements · Analyse Business Risks part 1 (assets in the form of Business Attributes with enterprise-specific descriptions; threats prompted by the Threats Database; impacts from business knowledge) · Analyse Business Risks part 2 (technical vulnerabilities; procedural vulnerabilities).

**Conceptual Security Architecture (Figure 7-5)**
- The SABSA Business Attributes Profile — selected attributes, their detailed definitions within the context of this enterprise and its business, the metric types and measurement approaches, and the performance targets for each metric
- The SABSA Business Risk Model extended to include a statement of the control objectives
- Assessment of the current status of security against the Business Attributes Profile and associated control objectives
- A description of the architectural layering to be employed, and the major security strategies and concepts mapped to the control objectives
- A series of **break-out documents**, each describing one major security strategy
- The security entity model and trust framework
- The security domain model
- A list of security-related lifetimes and deadlines to be addressed at lower layers

*Conceptual sub-processes:* Define Business Attributes Profile (select attributes mapped to business drivers; define enterprise-specific attributes, measurement approach, metrics and performance targets) · Synthesise Control Objectives (derive from the Business Risk Model and the Business Attributes Profile; integrated into the Business Risk Model) · Assess Current State of Security against the Business Risk Model and control objectives · Synthesise Conceptual Domain Model · Synthesise Conceptual Time Model · Synthesise Trust Model (entities internal and external; trust relationships) · Synthesise Major Security Strategies (mapped to control objectives and to the Business Attributes Profile) · Describe each Major Security Strategy in an individual break-out document · **Get Sign-Off and Buy-In to Conceptual Security Architecture**.

**Logical Security Architecture (Figure 7-9)**
- Security policy architecture — a hierarchical model of policy documentation and how it fits together
- The individual security policies, or at least templates and guidelines for their production
- A list and description of the logical security services, **mapped to the control objectives and major security strategies they are intended to satisfy**
- The entity schema for the enterprise-wide (logical) directory, with associated models for privilege profiles, authorisations, authentication attributes, etc.
- The specific security domains with their logical make-up, individual security policies, and the security associations existing both intra-domain and inter-domain
- A description of the logical security processing cycle
- **An improvements programme to gain short-term advantages and deliver early wins** from the security architecture programme

**Physical Security Architecture (Figure 7-10)**
- An updated business data model describing new data types required by the security architecture (passwords, usernames, certificates, etc.)
- A statement of the security rules, practices and procedures required — *details not written at this stage*; only which procedures and practices are needed to implement the logical-layer policies, plus possibly templates for creating them
- A list of the security mechanisms needed to implement the logical security services. Different mechanisms will be used in different contexts for the same service, and where each should be used can be indicated. **Minimise the number of mechanism types** to avoid complexity and provide generic, re-usable, modular building blocks.
- A list of applications and user communities with a **security user interface design for each type**. Again minimise the number of user interface types; *"a user interface module with a defined API would be a good architectural approach."*
- The physical layout of platforms and networks, probably diagrammatic — number of physical computer boxes, physical communications lines and physical networking equipment: how many, what type and where
- A statement of **capacity planning** — given device throughput, computer processing power and line bandwidth, how many of each are required to handle the expected load
- A description of the **resilience model** provided by redundancy of boxes and connections, integral to the physical layout model
- The **control structure execution model** needed to execute the logical security processing cycle from the layer above

**Component Security Architecture (Figure 7-11)**
- An updated **data dictionary** defining the syntax rules of all data structures required by the security architecture
- A framework for security standards and a list of all required security standards — *detailed content of individual standards probably not developed at this time*
- A list with descriptions and specifications of all strategic technologies, products and tools selected, with guidance for project teams as to **how, why, where and when** they should be used
- A **naming scheme** and a framework for defining roles, identities, access privilege profiles (permissions/authorisations), authorised functions and actions, plus guidance on building access control lists representing these parameters
- Detailed design of the security infrastructure: application processes to be run, platform nodes hosting them, handling of both logical and physical addressing schemes, and the protocols for inter-process and inter-node communication
- Detailed specification of procedural step timings and sequences implementing the control structure execution model

**Operational Security Architecture (Figure 7-12)** — based on the framework of Table 3-4. Note: *the design of this layer needs major participation and input from the teams that will have operational responsibility during the lifetime of the systems*; architects lay out the framework of what processes, procedures and activities are needed and how they relate, and the operational teams define detailed procedures.
- **Framework for assurance of operational continuity:** business requirements collection process; information classification scheme; business continuity management process/programme; information security management process; systems integrity management process; database security management process; software integrity management process; product and tool security and integrity management process.
- **Operational risk management framework:** business risk assessment process; corporate policymaking process; measurement, metrics and benchmarking process; security assurance framework; security audit process; detailed policymaking process; policy compliance monitoring; intelligence gathering; vulnerability assessment; threat assessment; penetration testing; ongoing research into threats and vulnerabilities; CERT notification management process.
- **Security service management and support framework:** business-driven information security management programme; incident response process; disaster recovery process; change control process; intrusion detection service; event monitoring service; security process development programme; security service management process; development controls programme; configuration management process; operational procedures (rule definition, key management, ACL maintenance, backup administration, information forensics, event log administration, anti-virus administration); product procurement process; operational management process; project management process.
- **Application and user management and support framework:** business security management organisation structure (roles, reporting structure, responsibilities); security training, awareness and culture development programme; access privileges (permissions) management framework (role-based profile definitions, user identity management and registration, user account and privilege administration, personnel vetting); user support and help desk framework.
- **Security management framework for sites, networks and platforms:** business field operations management; security domain management; application security administration and management; network security management; site security management; platform, workstation and equipment security management.
- **Framework for managing the security operations schedule:** business calendar and timetable management; security operations schedule management; management of application deadlines and cut-offs; time-dependency management (password lifecycle, account lifecycle, cryptographic key lifecycle, defining the time-dependent context for access control); operations sequencing.

**Implement phase deliverables (Figure 7-19)**
- Implementation plans: specifications for build; procurement plans; change management plans; project plans; quality plans; test plans
- Deliverables from the implementation itself: operating guides; support documentation; training materials; traceable change history
- The test reports

**Manage and Measure phase deliverables (Figure 7-20)**
- Operational reports; event reports; incident reports; penetration test reports; gap analysis reports; improvements programme plan

### How the Strategy and Concept phase fits together (Figure 7-7)
| Contextual (business level) | → | Conceptual representation |
|---|---|---|
| Business drivers — business strategy, goals, objectives; treated as the **business assets** in the risk assessment | → | **Business Attributes Profile** |
| What puts the assets at risk, plus the **critical success factors** (what influences success, as opposed to the risks that might bring failure) — all answering *why* you want security | → | **Control objectives** |
| The *when, how, who, where* questions — critical business processes and business models, bringing out criticality, dependency, responsibility and logistics | → | The conceptual **time model, trust model and domain model**, all feeding the conceptualisation of a series of **linked security strategies** |

## Worked Example — Directory Infrastructure Project at IBFS
CIO Ranjit Patel commissions a global directory service to support all IBFS applications, and the chapter walks it down all six layers.

**Contextual.** A study identifies all types of business entity: **people; corporate legal entities; organisational units; sites; equipment items; computer systems and applications.** The analysis of these business entities *is* the contextual architecture of the directory infrastructure.

**Conceptual.** The concept is that the entire set of business entities and their relationships will be logically represented through a directory infrastructure, built as a relational database compliant with an internationally recognised standard such as **X.500**. Access is via the enterprise-wide **middleware**, itself another company-wide infrastructure project (Figure 7-14). The directory is conceived as a **meta-directory — a directory of directories** (Figure 7-15) — because IBFS already has legacy directories that must be integrated into the global service but are still needed in their present form for legacy applications, and re-engineering them is too difficult and expensive. The meta-directory integrates them as they are while presenting a completely new directory interface to new applications.

**Logical.** Business entities are modelled as a **directory schema**. Each entity is a unique entity name plus a set of attributes describing it. Entities are related in a **branching tree**; the naming convention describes a path from the root through nodes to the entity, and entities may sit at intermediate nodes or at leaves. The full path name is the **distinguished name** and is unique. Some attributes are **alias names**, often used in a local context without ambiguity. Others include access rights, digital certificates, cryptographic keys (stored encrypted) and passwords (stored encrypted). Figure 7-16's simplified schema: Attribute 1 Distinguished Name; Attribute 2 Alias Name(s); Attribute 3 Digital Certificate; Attribute 4 Organisation(s) with pointer to Organisation sub-tree; Attribute 5 Role(s) with pointer to Role sub-tree; Attribute 6 etc.

Two hard problems flagged at this layer:
- **Inheritance of properties**, happening two ways: (a) an attribute is itself defined as an entity in another sub-tree, so the entity holding that attribute inherits the attribute's properties; (b) entities lower in a sub-tree inherit properties from an entity higher up.
- **Transitivity of trust between entities.** Together with inheritance this "can have a huge impact on the effectiveness of the trust models, because some entities can become endowed with trust that is not supposed to be there."

*(The authors footnote a terminology warning: "attribute" here means a descriptor of a directory entity and must not be confused with the Business Attributes Profile usage — "an entirely different usage.")*

**Physical (Figure 7-17).** Concerned with: the detailed naming standard to be applied; physical storage of the directory database; **resilience of the database by mirroring**; **replication** of directory data from masters to slaves to make it locally available without creating bottlenecks and bandwidth problems, providing the required performance, availability and response time; access protocols (such as **LDAP**); physical layout and location of equipment. The illustrated topology: two Unified Directory MASTERs in real-time mirroring, each feeding Replication Engines, feeding Local Directory Replicas, each supporting a Local Authorisation Server.

**Component.** "Whatever directory products have been selected."

**Operational.** Backup and recovery of directory data; directory security management (privacy, authentication, access control and auditing); synchronisation and replication strategy; directory administration, maintenance and support; monitoring and directory management.

**The layering summary (Figure 7-18)**, with the operational layer divided into five sub-layers, one per other layer:
| Layer | Directory content | Operational sub-layer |
|---|---|---|
| Contextual | Business entities: people, corporate entities, org units, sites, equipment items, systems | Business entity analysis |
| Conceptual | Directory concept: entity relationship model built as a relational database to X.500 | Directory strategy development |
| Logical | Directory schema: entity naming and attributes; directory services (functional spec) | Naming management, service management |
| Physical | Directory mechanisms: storage, replication, access, back-ups; physical layout/locations of equipment | Physical infrastructure management |
| Component | Directory products | Product procurement and deployment, directory use, administration, maintenance, support |

## Mental Models
- **The blueprint, not the building.** *"It is unlikely that a major strategic enterprise-wide security infrastructure will ever be implemented as a single project. You would be wise not even to try to take this route."* Infrastructure projects notoriously fail to attract business funding — business units pay for projects tied to a business initiative they conceived. So the architecture is a blueprint and road map guiding a series of separate, business-funded implementation projects. The architecture's job is to ensure that **fragmented implementation does not become piecemeal design** — which works only if the architecture is created, documented, referred to, and enforced through **architecture governance via the Architecture Board** (Chapter 8).
- **Ask the operating-cost question at design time.** *"When you design something, perhaps the single most important question you should address is: 'How much will it cost to operate this?' It will usually be far more than it ever cost to design and to build it."* Total cost of ownership is dominated by operations — user access privilege administration and help desk for tens or hundreds of thousands of users. Check the Business Attributes **scaleable, cost-effective and supportable**.
- **"Look boss, it's working!"** — the purpose of the measurement half of Manage and Measure is feedback to senior management against the design goals embodied in the Business Attributes Profile.
- **The architecture must live and breathe** — "not a document written once and then carved in tablets of stone." New business requirements, new technical solutions, and lessons learned all drive change; hence an architecture maintenance process.

## Anti-patterns
- Beginning detailed design work without signed-off contextual and conceptual architectures, i.e. without confirmed business support for the requirements and technology leadership support for the strategic approach.
- Building on someone else's unvalidated foundations without inserting review-and-validate steps.
- Proceeding against your own judgement without documenting and lodging your advice.
- Attempting the whole enterprise security infrastructure as one funded project.
- Proliferating security mechanism types or user interface types instead of minimising them.
- Running the contextual risk assessment in an organisation with no business process model or process culture — the resulting risk assessment may be faulty.
- Designing the operational layer without the operations teams.

## Key Takeaways
- SABSA Development Process = layer-by-layer, sequential, with operational in parallel after the conceptual layer exists.
- SABSA Lifecycle = Strategy and Concept → Design → Implement → Manage and Measure → back to Strategy and Concept.
- Two mandatory sign-off milestones: after the conceptual layer, and after the design phase.
- The matrix structures thinking; working documents hold the raw data; the deliverable lists per layer are the real output.
- Business Attributes Database is used twice (prompting drivers, then building the Profile); Threats Database prompts the risk assessment.
- Business process model, business information model, business data model and data dictionary are prerequisites you do not create.
- Implementation will be fragmented; the architecture plus architecture governance is what stops fragmented from becoming piecemeal.
- Operations dominates total cost of ownership — design for it.

## Connects To
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — the matrix rows this process populates.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — the Business Attributes Database and the metrics consumed in Manage and Measure.
- [Chapter 8 — Managing the Security Architecture Programme](ch08-managing-the-security-architecture-programme.md) — information gathering, design authority, the Architecture Board and architecture maintenance.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — the contextual deliverables in detail.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — the conceptual deliverables in detail.
- [Chapter 11 — Logical Security Architecture](ch11-logical-security-architecture.md) · [Chapter 12 — Physical](ch12-physical-security-architecture.md) · [Chapter 13 — Component](ch13-component-security-architecture.md) — the design-phase layers.
