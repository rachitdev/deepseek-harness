---
name: dsh-enterprise-security-architecture
description: "Distillation of \"Enterprise Security Architecture: A Business-Driven Approach\" by Nicholas Sherwood (with Andrew Clark and David Lynas), CMP Books, 2005 — the definitive text on the SABSA® framework. Use when working on SABSA, enterprise or information security architecture, the six architecture layers (contextual, conceptual, logical, physical, component, operational), the SABSA matrix and its six questions, business attributes profiling and security metrics, business risk modelling and operational risk management, control objectives, trust and security domain modelling, security policy architecture and policy management, security service catalogues and mechanism selection, assurance management, audit and penetration testing strategy, or security administration and operations."
---

# Enterprise Security Architecture: A Business-Driven Approach

**Author**: Nicholas Sherwood, with Andrew Clark and David Lynas
**Pages**: 609
**Chapters**: 17
**Generated**: 2026-08-23

## How to Use This Skill

- **No arguments** — read this file for the SABSA matrix, the business attributes taxonomy and the service catalogue, then open the chapter for the layer you are working at.
- **With a topic** — check `## Topic Index` below for the chapter numbers, then read those chapter files. Use `glossary.md` first if the terminology is unfamiliar.
- **With a chapter** — open `chapters/chNN-slug.md` directly; each carries Core Idea, Frameworks, Key Concepts, Mental Models, Anti-patterns, Reference Tables, a Worked Example and cross-links.
- **Browsing** — `patterns.md` for the reusable moves by layer, `cheatsheet.md` for decision tables and procedures.

## Core Frameworks & Mental Models

### 1. The SABSA® Matrix — six layers × six questions = 36 cells

Each layer is a different player's view of the same building; each column is one of Kipling's six honest serving-men. **Developing an enterprise security architecture is the process of populating all 36 cells.**

| Layer (player's view) | Assets (What) | Motivation (Why) | Process (How) | People (Who) | Location (Where) | Time (When) |
|---|---|---|---|---|---|---|
| **Contextual** — *the Business View* | The Business | Business Risk Model | Business Process Model | Business Organisation and Relationships | Business Geography | Business Time Dependencies |
| **Conceptual** — *the Architect's View* | Business Attributes Profile | Control Objectives | Security Strategies and Architectural Layering | Security Entity Model and Trust Framework | Security Domain Model | Security-Related Lifetimes and Deadlines |
| **Logical** — *the Designer's View* | Business Information Model | Security Policies | Security Services | Entity Schema and Privilege Profiles | Security Domain Definitions and Associations | Security Processing Cycle |
| **Physical** — *the Builder's View* | Business Data Model | Security Rules, Practices and Procedures | Security Mechanisms | Users, Applications and the User Interface | Platform and Network Infrastructure | Control Structure Execution |
| **Component** — *the Tradesman's View* | Detailed Data Structures | Security Standards | Security Products and Tools | Identities, Functions, Actions and ACLs | Processes, Nodes, Addresses and Protocols | Security Step Timing and Sequencing |
| **Operational** — *the Facilities Manager's View* | Assurance of Operational Continuity | Operational Risk Management | Security Service Management and Support | Application and User Management and Support | Security of Sites, Networks and Platforms | Security Operations Schedule |

*(Table 3-3. The Operational layer is properly drawn as a **vertical bar cutting all five others**, because operational issues arise at every layer; Table 3-4 expands it into its own 6×6 matrix. The **Inspector's View** — auditor, systems QA — is deliberately *not* a seventh layer: assurance is designed into the architecture, not bolted on. See `chapters/ch03-security-architecture-model.md`.)*

**Column meanings**: What = assets to protect · Why = motivation for applying security · How = functions needed · Who = people and organisation · Where = locations · When = time-related aspects.

### 2. The Business Attributes Taxonomy — the conceptual *What*

The **Business Attributes Profile** is SABSA's primary tool for capturing business requirements in a **normalised** form: an abstraction layer between business drivers and technical design, and the anchor for **two-way traceability** (down = completeness, up = justification). Each attribute gets an enterprise-specific definition, a **metric type** (hard/soft), a **measurement approach** and a **performance target**. Table 6-2 gives 85 attributes in 7 classes; the authors state it is "almost certainly not complete" — add your own.

| Class | Attributes (the book's own names) |
|---|---|
| **User** (16) | Accessible · Accurate · Anonymous · Consistent · Current · Duty-segregated · Educated and aware · Informed · Motivated · Protected · Reliable · Responsive · Supported · Timely · Transparent · Usable |
| **Management** (8) | Automated · Change-managed · Controlled · Cost-effective · Efficient · Maintainable · Measured · Supportable |
| **Operational** (8) | Available · Continuous · Detectable · Error-free · Inter-operable · Monitored · Productive · Recoverable |
| **Risk Management** (19) | Access-controlled · Accountable · Assurable · Assuring honesty · Auditable · Authenticated · Authorised · Capturing new risks · Confidential · Crime-free · Flexibly secure · Identified · Independently secure · In our sole possession · Integrity-assured · Non-repudiable · Owned · Private · Trustworthy |
| **Legal & Regulatory** (9) | Admissible · Compliant · Enforceable · Insurable · Legal · Liability-managed · Regulated · Resolvable · Time-bound |
| **Technical Strategy** (13) | Architecturally open · COTS and GOTS compliant · Extendable · Flexible and Adaptable · Future-proof · Legacy-sensitive · Migrateable · Multi-sourced · Scaleable · Simple · Standards-compliant · Traceable · Upgradeable |
| **Business Strategy** (12) | Brand-enhancing · Business-enabled · Competent · Confident · Credible · Culture-sensitive · Enabling time-to-market · Governable · Providing good stewardship and custody · Providing investment re-use · Providing return on investment · Reputable |

**Decision rule**: *measure the virtue, not the absence of the vice.* Risk is the negative of a Business Attribute. "Security" is not measurable; *Available*, *Non-repudiable* and *Responsive* are. Full definitions and measurement approaches in `chapters/ch06-measuring-roi-in-security-architecture.md`.

### 3. The Security Service Catalogue — the logical *How*

Table 11-2 classifies services by **defensive strategy** (from the multi-tiered model of Figure 10-2: **prevent → contain → detect → react → collect evidence → assure**). A service is classified by the *effect* it delivers, so services legitimately appear under several strategies. *"No list of security services will ever be complete."*

| Defensive strategy | Security services |
|---|---|
| **Prevention** — *Entity* | Entity unique naming · entity registration · entity public key certification · entity credentials certification · directory service · entity authorisation · entity authentication · user authentication · device authentication |
| **Prevention** — *Communications* | Session authentication · message origin authentication · message integrity protection · message content confidentiality · non-repudiation · message replay protection · traffic flow confidentiality |
| **Prevention** — *Application and System* | Entity authorisation · logical access control · audit trails · stored data integrity protection · stored data confidentiality · software integrity protection · software licensing management · system configuration protection · data replication and backup · software replication and backup · trusted time · user interface for security |
| **Prevention** — *Security Management* | Security policy management · security training and awareness · security operations management · security provisioning · security monitoring · security measurement and metrics · security administration (privilege management) · user support · physical security devices · environmental security services |
| **Containment** | Entity authorisation · stored data confidentiality · software integrity protection · physical security · environmental security · security training and awareness |
| **Detection and Notification** | Message integrity protection · stored data integrity protection · security monitoring · intrusion detection · security alarm management · security training and awareness · security measurement and metrics |
| **Event Collection and Event Tracking** | Audit trails · security operations management · security monitoring · security measurement and metrics |
| **Recovery and Restoration** | Incident response · data replication and backup · software replication and backup · disaster recovery · crisis management |
| **Assurance** | Audit trails · security audit · security monitoring · security measurement and metrics |

### 4. The chain of derivation (the spine of the whole method)

**Business drivers → Business Attributes → Business Risk Model + Business Attributes Profile → Control Objectives → security strategies → logical security services → physical security mechanisms → component products, tools and standards → operational procedures.**

The **Business Risk Model** (Table 9-8) instantiates this in 13 columns: ID · Business Driver · Business Attributes · Business Requirement · High-Level Threat · Business Impact · Impact Value · Potential High-Level Vulnerability · Green Field Vulnerability Value · Green Field Risk Category · High-Level Control Objectives · Target Vulnerability Value · Mitigated Risk Category. Columns 1–10 are contextual; 11–13 are conceptual — the key interface between the two layers.

### 5. The SABSA Lifecycle and Development Process

| Phase | Layers | Ends with |
|---|---|---|
| **Strategy and Concept** | Contextual + Conceptual | Sign-off / buy-in milestone — *the natural break; do not commit design resource before it* |
| **Design** | Logical + Physical + Component + Operational | Sign-off and agreement milestone |
| **Implement** | Construction of designed systems and processes | Tested, documented implementations |
| **Manage and Measure** | Operational management and metrics | Feeds back as new business requirements |

Layers develop **in sequence**, with one exception: the **operational architecture develops in parallel** (though it cannot begin before the conceptual layer exists). The output is a **blueprint and road map** guiding many separately business-funded projects — never one monolithic infrastructure project.

### 6. Load-bearing decision rules

- **Never answer "is this system secure?"** Ask what *secure* means to them first. Everything downstream is invalid without it.
- **Every control has an enablement reading** (the brakes model). If you cannot state it, you have not understood the requirement.
- **Threats are binary; impact and vulnerability are graded.** Do not attempt quantitative threat probability — threats sit outside your span of control. Assess **greenfield** (as if no controls existed) so you can measure what planned controls contribute.
- **P > D + R** (Time-Based Security): Protection time must exceed Detection + Reaction time. Exposure E = D + R − P. Design to reduce D and R.
- **Threat × Vulnerability × Impact**: deterrent controls hit threat, preventive hit vulnerability, corrective hit impact, detective trigger the others.
- **Cryptography does exactly four things**: confidentiality, integrity, authenticity, non-repudiation. If your requirement is not one of these, cryptography is the wrong answer.
- **Integrity mechanisms detect; they do not prevent.** Nonces allow replay to be *detected*, not prevented — so plan the recovery action.
- **Trust is an attribute of business relationships, not a technical attribute.** Decompose complex two-way trust until only simple one-way claimant/relying-party relationships remain. The **customer for trust is always the relying party**.
- **A security domain** = a set of security elements under a common policy set by a single policy authority. There is always a surrounding super-domain called *the universe*: hostile, no policy, no authority, untrusted elements.
- **Operational risk management sits above information security**, not below it.

## Chapter Index

| # | Chapter | File |
|---|---|---|
| 1 | The Meaning of Security | [chapters/ch01-the-meaning-of-security.md](chapters/ch01-the-meaning-of-security.md) |
| 2 | The Meaning of Architecture | [chapters/ch02-the-meaning-of-architecture.md](chapters/ch02-the-meaning-of-architecture.md) |
| 3 | Security Architecture Model | [chapters/ch03-security-architecture-model.md](chapters/ch03-security-architecture-model.md) |
| 4 | Case Study: Intergalactic Banking and Financial Services Inc | [chapters/ch04-case-study-ibfs.md](chapters/ch04-case-study-ibfs.md) |
| 5 | A Systems Approach | [chapters/ch05-a-systems-approach.md](chapters/ch05-a-systems-approach.md) |
| 6 | Measuring Return on Investment in Security Architecture | [chapters/ch06-measuring-roi-in-security-architecture.md](chapters/ch06-measuring-roi-in-security-architecture.md) |
| 7 | Using This Book as a Practical Guide | [chapters/ch07-using-this-book-as-a-practical-guide.md](chapters/ch07-using-this-book-as-a-practical-guide.md) |
| 8 | Managing the Security Architecture Programme | [chapters/ch08-managing-the-security-architecture-programme.md](chapters/ch08-managing-the-security-architecture-programme.md) |
| 9 | Contextual Security Architecture | [chapters/ch09-contextual-security-architecture.md](chapters/ch09-contextual-security-architecture.md) |
| 10 | Conceptual Security Architecture | [chapters/ch10-conceptual-security-architecture.md](chapters/ch10-conceptual-security-architecture.md) |
| 11 | Logical Security Architecture | [chapters/ch11-logical-security-architecture.md](chapters/ch11-logical-security-architecture.md) |
| 12 | Physical Security Architecture | [chapters/ch12-physical-security-architecture.md](chapters/ch12-physical-security-architecture.md) |
| 13 | Component Security Architecture | [chapters/ch13-component-security-architecture.md](chapters/ch13-component-security-architecture.md) |
| 14 | Security Policy Management | [chapters/ch14-security-policy-management.md](chapters/ch14-security-policy-management.md) |
| 15 | Operational Risk Management | [chapters/ch15-operational-risk-management.md](chapters/ch15-operational-risk-management.md) |
| 16 | Assurance Management | [chapters/ch16-assurance-management.md](chapters/ch16-assurance-management.md) |
| 17 | Security Administration and Operations | [chapters/ch17-security-administration-and-operations.md](chapters/ch17-security-administration-and-operations.md) |

## Topic Index

| Topic | Chapters |
|---|---|
| SABSA matrix, layers, six questions | 3, 7 |
| Business attributes, profiling, metrics, ROI | 6, 7, 9, 15 |
| Capability maturity models (SSE-CMM, CobiT CMM) | 6, 16 |
| Business risk modelling, threat taxonomy, threat scenarios | 1, 9, 15 |
| Operational risk management, Basel II, VAR, risk register | 1, 14, 15 |
| Control objectives and enablement objectives | 1, 9, 10 |
| Systems thinking, decomposition, feedback loops, FSM modelling | 5 |
| Trust modelling, PKI business model, registration assurance | 10, 12, 14 |
| Security domain modelling, inter-domain relations, associations | 10, 11, 12 |
| Security entity model, roles, RBAC, directory schema | 10, 11 |
| Security services catalogue and service definitions | 11 |
| Security mechanisms, cryptography, key management, resilience | 12 |
| Components, standards bodies, ASN.1, XML, web services, SSL/IPSec | 13 |
| Security policy architecture and policy management | 11, 14 |
| Information and system classification | 14 |
| Least privilege, need-to-know vs need-to-share | 14 |
| Governance, steering groups, architecture board, team building | 8, 14, 15 |
| Assurance, audit, ISO 17799 / BS 7799, testing phases, penetration testing | 16 |
| Security administration, operations, BCM, change and incident management | 17 |
| Outsourcing, SLAs, managed security services | 8, 14, 17 |
| Time-Based Security (P > D + R), lifetimes and deadlines | 9, 10 |
| Case study material (IBFS, ITEC, and real-world cases) | 4, 5, 6, 8, 9, 11, 12, 14, 15, 16, 17 |

## Supporting Files

- [glossary.md](glossary.md) — 150+ SABSA and security-architecture terms, alphabetical, chapter-referenced.
- [patterns.md](patterns.md) — the book's reusable moves as When / How / Trade-offs, grouped by architecture layer.
- [cheatsheet.md](cheatsheet.md) — layer-by-layer decision tables, the attribute-profiling procedure, risk-modelling steps, tells and smells, and process templates.

## Scope & Limits

- **Edition**: the 2005 CMP Books edition (SABSA as published then). SABSA has since been developed further by the SABSA Institute; later material — SABSA attribute taxonomy revisions, the formal SABSA Blue Book, current certification syllabi — is not in this book and is not represented here.
- **Dated technical specifics.** Chapter 12's Table 12-2 (cryptographic strengths) treats Triple DES, MD5 and SHA-1 as "strong" and RSA-1024 as adequate; all are superseded. Chapter 13's standards inventory (SAML 1.x, WS-Security drafts, XKMS, Common Criteria versions) is a 2005 snapshot — the authors themselves warn printed standards lists "go out of date very quickly." Chapter 16's ISO/IEC 17799:2000 and BS 7799-2:2002 are now ISO/IEC 27002 and 27001. Use the *methods* from these chapters, not the *thresholds*.
- **Deliberately qualitative.** The book advocates qualitative risk assessment (H/M/L, A/B/C/D) and rejects quantitative threat probability for commercial organisations. If you need quantitative loss modelling, Chapter 15 explains why the authors distrust it rather than how to do it.
- **Not covered**: cloud architecture, DevSecOps, zero-trust as a named movement (though the domain model and honeycomb progression in Chapter 10 anticipate much of it), container and API security, machine learning, and modern identity federation beyond SAML 1.x.
- **Regulatory framing** is Basel II, the EU Data Protection Directive, HIPAA 1996, Gramm-Leach-Bliley and the UK Combined Code / Turnbull. GDPR, SOX-era refinements and Basel III are absent.
- **IBFS (Intergalactic Banking and Financial Services Inc)** and its personnel are explicitly fictitious composites created by the authors as a teaching device; the numbers in its worked examples are illustrative, not benchmarks.
