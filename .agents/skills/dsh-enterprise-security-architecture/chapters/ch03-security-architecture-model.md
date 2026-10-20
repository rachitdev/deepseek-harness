# Chapter 3 — Security Architecture Model

## Core Idea
The SABSA® Model is a **six-layer** architecture model, adapted from John A. Zachman's enterprise architecture framework to a security view of the world. Each layer is the view of a different player in specifying, designing, constructing and using a building. Each layer is then cut vertically by **six questions** (What, Why, How, Who, Where, When). The result is the **36-cell SABSA® Matrix**, and *developing an enterprise security architecture is the process of populating all 36 cells*.

## Frameworks Introduced

### Table 3-1: The SABSA® Model Layered Architecture Views
| Player's view | Architecture layer |
|---|---|
| The Business View | **Contextual** Security Architecture |
| The Architect's View | **Conceptual** Security Architecture |
| The Designer's View | **Logical** Security Architecture |
| The Builder's View | **Physical** Security Architecture |
| The Tradesman's View | **Component** Security Architecture |
| The Facilities Manager's View | **Operational** Security Architecture |

In Figure 3-1 the Operational layer is redrawn as a **vertical bar across the other five**, because operational security issues arise at every one of them and have a distinct meaning in the context of each.

### The six questions (from Kipling's "I Keep Six Honest Serving-Men")
| Question | What it asks of a layer |
|---|---|
| **What?** (Assets) | The assets to be protected by your security architecture at this layer |
| **Why?** (Motivation) | The motivation for applying security, expressed in this layer's terms |
| **How?** (Process) | The functions needed to achieve security at this layer |
| **Who?** (People) | The people and organisational aspects of security at this layer |
| **Where?** (Location) | The locations where you apply security, relevant to this layer |
| **When?** (Time) | The time-related aspects of security relevant to this layer |

### The Inspector's View — deliberately *not* a seventh layer
The inspector (computer auditor, systems QA) provides assurance that the architecture is complete, consistent, robust and fit-for-purpose. SABSA does **not** give this a separate layer: the architecture as a whole supports assurance needs, the framework itself structures the audit process, and security audit and review appear as a major strategic programme in the operational architecture at the conceptual layer.

## Key Concepts — the six layers in detail

**Contextual (Business View).** A description of the business context in which secure systems must be designed, built and operated. Established by asking the six questions of the enterprise the same way a building owner is asked what sort of building is needed. Shortcuts here are the commonest cause of failure.
- *What?* The business, its assets to be protected (brand, reputation) and the business needs for information security — security as a business enabler, secure electronic business, operational continuity and stability, compliance with the law.
- *Why?* Business risks expressed as assets, goals, success factors, and the threats/impacts/vulnerabilities putting them at risk — brand protection, fraud prevention, loss prevention, legal obligations, business continuity.
- *How?* Business processes requiring security — business interactions and transactions, business communications.
- *Who?* Organisational aspects — management structures, supply chain structures, outsourcing relationships, strategic partnerships.
- *Where?* Business geography — the global village marketplace, distributed corporate sites, remote working.
- *When?* Business time dependencies, in performance *and* sequence — transaction throughput, lifetimes and deadlines, just-in-time operations, time-to-market.

**Conceptual (Architect's View).** "An architect is a visionary." Impressionistic drawings, broad brushes; defines principles and fundamental concepts that guide the selection and organisation of logical and physical elements below.
- *What?* **SABSA® Business Attributes Profile** — the primary tool for capturing business requirements in a *normalised* form (Ch 6).
- *Why?* **Control objectives** — derived directly from analysis of business operational risks; a conceptualisation of business motivation for security.
- *How?* **Major security strategies and architectural layering** — applications security strategy, network security strategy, cryptographic infrastructure strategy, RBAC strategy, etc. Every major area of contextual business requirement has a supporting strategy or strategy group.
- *Who?* **Security entity model and trust framework** — the policy authorities governing trust in a domain, the policies they set to govern entity behaviour in each domain, and inter-domain trust relationships.
- *Where?* **Security domain model** — security domains (logical and physical), domain boundaries, security associations.
- *When?* **Security-related lifetimes and deadlines** — expiry of keys, certificates, passwords, sessions; trusted time for time-stamping and time-sensitive transactions; time-related performance criteria.

**Logical (Designer's View).** The designer is an engineer, not an artist: interprets the architect's vision into a logical structure that can be engineered. Models the business as a system of sub-systems. Also called systems engineering.
- *What?* Business information model — the logical representation of the real business, and the thing to be secured.
- *Why?* Security policy requirements — high-level security policy, registration authority policy, certification authority policy, physical and logical domain policies.
- *How?* Logical security services — entity authentication, confidentiality protection, integrity protection, non-repudiation, system assurance — and how they fit together as common re-usable building blocks.
- *Who?* Entities (users, security administrators, auditors), their inter-relationships, attributes, authorised roles and privilege profiles, expressed as a schema.
- *Where?* Security domain definitions and inter-domain relationships (logical domains, physical domains, security associations).
- *When?* Security processing cycle — registration, certification, login, session management.

**Physical (Builder's View).** Turns logical abstractions into a technology model, specifying functional requirements of system components. Logical *services* are now expressed as physical *mechanisms and machines*.
- *What?* Business data model and security-related data structures — tables, messages, pointers, certificates, signatures.
- *Why?* Rules driving logical decision-making in the system — conditions, practices, procedures and actions.
- *How?* Security mechanisms — encryption, access control, digital signatures, virus scanning — and the physical machines hosting them.
- *Who?* People dependency — the users, the applications they use, and the security user interface (screen formats and user interactions).
- *Where?* Security technology infrastructure — physical layout of hardware, software and communications lines.
- *When?* Execution control structures — sequences, events, lifetimes, time intervals.

**Component (Tradesman's View).** The bricklayer, plasterer, electrician, plumber, carpenter — each bringing specific production skills and specific products. In ICT: specialist vendor products plus integration skills.
- *What?* Data field specifications, address specifications, other detailed data structure specifications.
- *Why?* Security standards.
- *How?* Products and tools, hardware and software.
- *Who?* User identities, privileges, functions, actions and access control lists (ACLs).
- *Where?* Computer processes, node addresses, inter-process protocols.
- *When?* Security step timings and sequencing.

**Operational (Facilities Manager's View).** Runs the building over its lifetime, keeps it in working order, monitors how well it meets requirements.
- *What?* Operational continuity of business systems and information processing; maintaining security of operational business data (confidentiality, integrity, availability, auditability, accountability).
- *Why?* To manage operational risks and minimise operational failures and disruptions.
- *How?* Specialised security-related operations — user security administration, system security administration, data back-ups, security monitoring, emergency response procedures.
- *Who?* Operational support for the security needs of all users and applications — business users, operators, administrators.
- *Where?* Maintaining system integrity and security of all operational platforms and networks, by applying operational security standards and auditing configuration against them.
- *When?* Scheduling and executing a timetable of security-related operations.

## Reference Tables

### Table 3-3: The 36-Cell SABSA® Matrix
| Layer | Assets (What) | Motivation (Why) | Process (How) | People (Who) | Location (Where) | Time (When) |
|---|---|---|---|---|---|---|
| **Contextual** | The Business | Business Risk Model | Business Process Model | Business Organisation and Relationships | Business Geography | Business Time Dependencies |
| **Conceptual** | Business Attributes Profile | Control Objectives | Security Strategies and Architectural Layering | Security Entity Model and Trust Framework | Security Domain Model | Security-Related Lifetimes and Deadlines |
| **Logical** | Business Information Model | Security Policies | Security Services | Entity Schema and Privilege Profiles | Security Domain Definitions and Associations | Security Processing Cycle |
| **Physical** | Business Data Model | Security Rules, Practices and Procedures | Security Mechanisms | Users, Applications and the User Interface | Platform and Network Infrastructure | Control Structure Execution |
| **Component** | Detailed Data Structures | Security Standards | Security Products and Tools | Identities, Functions, Actions and ACLs | Processes, Nodes, Addresses and Protocols | Security Step Timing and Sequencing |
| **Operational** | Assurance of Operational Continuity | Operational Risk Management | Security Service Management and Support | Application and User Management and Support | Security of Sites, Networks and Platforms | Security Operations Schedule |

### Table 3-2: The Operational Security Architecture, layer by layer
| At the… layer | Operational activity implied |
|---|---|
| Contextual | Business policymaking, business risk assessment process, business requirements collection and specification, organisational and cultural development |
| Conceptual | Major programmes for training and awareness, business continuity management, audit and review, process development for registration/authorisation/administration/incident handling, development of standards and procedures |
| Logical | Security policymaking, information classification, system classification, management of security services, security of service management, negotiation of inter-operable standards for security services, audit trail monitoring and invocation of actions |
| Physical | Development and execution of security rules, practices and procedures: cryptographic key management, communication of security parameters between parties, synchronisation between parties, ACL maintenance and distribution of ACEs (access control entries), backup management (storing, labelling, indexing), virus pattern search maintenance, event log file management and archiving |
| Component | Products, technology, evaluation and selection of standards and tools, project management, implementation management, operation and administration of individual components |

### Table 3-4: The Operational Security Architecture Matrix (the operational layer expanded 6×6)
| Layer | Assets (What) | Motivation (Why) | Process (How) | People (Who) | Location (Where) | Time (When) |
|---|---|---|---|---|---|---|
| **Contextual** | Business Requirements Collection; Information Classification | Business Risk Assessment; Corporate Policy Making | Business-Driven Information Security Management Programme | Business Security Organisation | Business Field Operations Management | Business Calendar and Timetable Management |
| **Conceptual** | Business Continuity Management | Security Audit & Assurance Levels; Measurement, Metrics & Benchmarking | Incident Response; Disaster Recovery; Change Control Programme | Security Training, Awareness and Culture Development | Security Domain Management | Security Operation Schedule Management |
| **Logical** | Information Security; System Integrity; Compliance Monitoring; Intelligence Gathering | Detailed Security Policy Making; Policy Security Service Management; System Development Controls; Configuration Management | Intrusion Detection; Event Monitoring; Process Development | Access Control & Privilege Profile Administration | Application Security Administration & Management | Managing Application Deadlines & Cut-off |
| **Physical** | Database Security; Software Integrity | Vulnerability Assessment; Penetration Testing; Threat Assessment | Rule Definition; Key Management; ACL Maintenance; Back-Up Admin; Computer Forensics; Event Log Admin; Anti-Virus Admin | User Support and Help Desk | Network Security Management; Site Security Management | User Account Aging; Password Aging; Crypto Key Aging; Administering Time Windows for Access Control |
| **Component** | Product & Tool Security & Integrity | CERT Notifications; Research on Threats & Vulnerabilities | Product Procurement; Project Management; Operations Management | Personnel Vetting; User Administration | Platform, Workstation and Equipment Security Management | Time-out configuration; Detailed operation sequence |

## Mental Models
- **Six players, one building.** Owner → architect → designer → builder → tradesman → facilities manager. Each has a legitimate, different, complete view of the same thing.
- **Artist then engineer.** The architect (conceptual) is a visionary with broad brushes; the designer (logical) is an engineer who makes the vision buildable. Do not merge the roles.
- **The matrix as a document structure.** A security architecture document might be structured with a chapter per row and a section per cell. Do that and you have high assurance of comprehensiveness.
- **Populate, don't necessarily sequence.** Each cell must be addressed in turn "although not necessarily in strict sequential order."

## Anti-patterns
- Beginning systems architecture from a technical perspective, looking at technologies and solutions while ignoring requirements.
- Technologists assuming they already know the requirements, despite a poor relationship with the people who express them.
- Flipping ahead to the solutions chapters because you are in a hurry — costs more, delivers less, and destroys business confidence in ICT.
- Treating audit as an external, separate architectural view instead of designing assurance into the architecture.
- Assuming a completed 36-cell matrix guarantees correctness — the authors footnote that "there may remain a potential issue regarding consistency and lack of conflict between all the various cells."

## Worked Example
**Tracing one business concern down the matrix column by column.**

Take the *Why* (Motivation) column and follow a single requirement down all six layers, which is exactly the traceability the matrix is designed to give:

| Layer | Cell name | Instantiated for "we must not leak customer account data" |
|---|---|---|
| Contextual | Business Risk Model | The business risk: disclosure of customer financial data → brand and reputation damage, regulatory penalty |
| Conceptual | Control Objectives | Control objective: customer financial data is disclosed only to the authenticated data subject |
| Logical | Security Policies | A high-level security policy plus the registration authority and certification authority policies and the logical domain policy that bind it |
| Physical | Security Rules, Practices and Procedures | The conditions, practices, procedures and actions that drive the runtime decision |
| Component | Security Standards | The specific standards the products must meet to implement that rule |
| Operational | Operational Risk Management | Ongoing management of the residual operational risk, plus (Table 3-4) vulnerability assessment, penetration testing and threat assessment at the physical row |

The same walk can be done down any of the six columns; the matrix's value is that every cell has a named deliverable, so a gap is visible rather than implicit.

## Key Takeaways
- Six layers: Contextual, Conceptual, Logical, Physical, Component, Operational — business, architect, designer, builder, tradesman, facilities manager.
- Operational is best drawn as a vertical bar cutting all five others, and expands into its own 6×6 matrix (Table 3-4).
- Six questions: What (Assets), Why (Motivation), How (Process), Who (People), Where (Location), When (Time).
- 6 × 6 = the 36-cell SABSA Matrix; architecture development = populating those cells.
- The inspector's/auditor's view is folded into the operational architecture, not given its own layer.
- Business Attributes Profile is the conceptual layer's *What*; control objectives are its *Why*.

## Connects To
- [Chapter 2 — The Meaning of Architecture](ch02-the-meaning-of-architecture.md) — why the layered approach exists.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — the Business Attributes taxonomy that fills the Conceptual/Assets cell.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — turns the matrix into a development process and lifecycle.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — populating the contextual row.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — populating the conceptual row.
- [Chapter 11 — Logical Security Architecture](ch11-logical-security-architecture.md) — populating the logical row.
- [Chapter 12 — Physical Security Architecture](ch12-physical-security-architecture.md) — populating the physical row.
- [Chapter 13 — Component Security Architecture](ch13-component-security-architecture.md) — populating the component row.
