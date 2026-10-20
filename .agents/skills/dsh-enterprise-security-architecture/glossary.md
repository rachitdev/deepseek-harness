# Glossary — Enterprise Security Architecture: A Business-Driven Approach

Terms as the book defines and uses them. Chapter references point to where the term is introduced or best explained.

**Access control service** — A service that decides and enforces whether a subject may perform an action on an object; comes in three forms — physical mechanisms for physical domains, logical mechanisms for logical domains, and logical mechanisms for physical domains such as platforms and networks (Ch 11)

**Acceptable Use Policy** — A level-5 policy governing what employees may and may not do with corporate systems, e-mail and Internet access (Ch 14, Ch 16)

**Amplifier** — A factor that encourages a threat agent to act: peer pressure, fame, easy access, low cost, low technical difficulty, belief in sympathetic public opinion (Ch 9)

**Annual Loss Expectancy (ALE)** — Expected annual loss from a risk; the book treats it with suspicion because the underlying probabilities are usually unknowable (Ch 15)

**Architecture Board** — The governance body that approves architecture and enforces it through budget approval; the book warns a board whose remit is *security only* will lack power and teeth (Ch 8)

**AS/NZS 4360** — The Australian/New Zealand risk management standard whose process the book adopts; itself a feedback control loop with "Communicate & Consult" and "Monitor & Review" running alongside every step (Ch 15)

**ASN.1 (Abstract Syntax Notation One)** — ISO/IEC 8824 notation for defining data structures independently of encoding; the binary choice for embedded and infrastructure protocols, and it nests inside XML (Ch 13)

**Asset** — A thing of value in the business that you want to protect; at the logical layer, information assets are *secondary* assets representing the true primary asset, the business itself (Ch 1, Ch 9, Ch 11)

**Assurance** — Confidence that the Business Attributes are meeting their performance targets; delivered through audit, testing and review rather than through certification alone (Ch 16)

**Assurance level (registration)** — The strength band of a registration process, from self-registration through e-mail, credit card, telephone confirmation, postal, personal, transferred and delegated registration (Table 10-1) (Ch 10)

**Attribute type (directory)** — A schema component defined by a unique name, an object identifier (OID), single/multi-valued indicator, syntax and syntax rules, usage indicator, and restrictions on range and size (Ch 11)

**Audit trail** — A recording of the actions of authorised parties and the chain of events and outcomes, at a level of detail matching business needs; also the basis of the *Auditable* attribute (Ch 6, Ch 11, Ch 17)

**Authentication information (AI)** — ISO/IEC 10181-2 vocabulary for the data used in authentication: claim AI, verification AI, exchange AI, authority claim AI and authority verification AI (Ch 11)

**Balanced Scorecard** — Kaplan & Norton's 1993 framework (financial, customer, internal process, learning and growth perspectives) used by the book as a vehicle for security metrics reporting (Ch 6)

**Basel II / New Basel Accord** — The banking capital framework whose operational risk definition the book adopts: "the risk of loss resulting from inadequate or failed internal processes, people and systems or from external events"; structured in three pillars (Ch 9, Ch 15)

**Belbin team roles** — The eight roles (Chairman/Co-ordinator, Shaper, Plant, Monitor-Evaluator, Company Worker/Implementer, Resource Investigator, Team Worker, Completer-Finisher) used to compose the architecture team; the startling finding is that the "A-team" of all high performers loses (Ch 8)

**Benchmarking** — Measuring the organisation against an external standard: "how are we doing compared to everyone else?" then "where do we want to get to?"; always rests on self-assessment (Ch 6)

**Black box (modelling)** — Accepting a defined function with a defined interface now and opening it later; one of the three complexity-simplifying techniques (Ch 5)

**Black box testing** — Functional testing against the specification; proves the specified functions work but cannot prove nothing else is in there — presence is testable, absence is not (Ch 16)

**BS 7799-2:2002** — The specification against which an Information Security Management System is certified; distinct from ISO/IEC 17799, which is a code of practice and certifies nothing (Ch 16)

**Business Attribute** — A normalised, named business requirement with an enterprise-specific definition, a metric type, a measurement approach and a performance target; the conceptual layer's *What* (Ch 6)

**Business Attributes Database** — The generic attribute list, used twice in the process: at the contextual layer only to prompt thinking about business drivers, and at the conceptual layer to build the full profile (Ch 7)

**Business Attributes Profile** — "The conceptualisation of the real business strategy, drivers, assets, goals and objectives"; the primary tool for capturing requirements in normalised form and the anchor of two-way traceability (Ch 6, Ch 7, Ch 10)

**Business Attributes Taxonomy** — The 85 attributes in 7 classes of Table 6-2 (User, Management, Operational, Risk Management, Legal & Regulatory, Technical Strategy, Business Strategy); explicitly incomplete by the authors' own statement (Ch 6)

**Business driver** — A business goal, objective, pressure or constraint; the primary input to the entire SABSA process and the starting point of every metrics chain (Ch 6, Ch 9)

**Business Risk Model** — The contextual layer's *Why*; a 13-column risk assessment matrix running from business driver through threat, impact, greenfield vulnerability and risk category to control objectives and mitigated risk (Table 9-8) (Ch 9)

**Capability (threat agent)** — The resources expected to be under a threat agent's control: finance, technical equipment, software, facilities, expertise, literature and experience (Ch 9)

**Capability Maturity Model (CMM)** — Carnegie Mellon SEI's staged model (Initial, Repeatable, Defined, Managed, Optimising) used in the book to measure security process maturity — and, in the authors' own practice, mechanism *strength* (Ch 6)

**Catalyst** — An event or change in circumstances that makes a threat agent decide to act: redundancy, debts, a step change in access or difficulty (Ch 9)

**Certificate Policy (CP)** — A named set of rules indicating the applicability of a certificate to a class of application with common security requirements; states *what* is required (Ch 10, Ch 14)

**Certification Practices Statement (CPS)** — A statement of the practices a certification authority employs in issuing certificates; states *how* it is done. CP and CPS are orthogonal: one CP may be met by many CPSs and one CPS may serve many CPs (Ch 14)

**Change control** — A record: the process of logging, evaluating, approving and tracking individual changes to a production system (Ch 17)

**Change management** — A project: the programme addressing a major change's impact on people, process, culture and organisation. Confusing it with change control means either bureaucratising trivia or running a transformation off a ticket (Ch 17)

**Claimant** — The party who makes claims about itself and wants to be trusted; one half of the simple one-way trust model (Ch 10)

**CobiT** — Control Objectives for Information and related Technology, from the IT Governance Institute: 34 ICT processes in four domains with 318 detailed control objectives, plus a six-level maturity model (0–5) (Ch 6, Ch 16)

**Common Criteria** — ISO/IEC 15408, the international successor to TCSEC, ITSEC, CTCPEC and the Federal Criteria for evaluating product security (Ch 13)

**Component Security Architecture** — The tradesman's view: detailed data structures, security standards, products and tools, identities and ACLs, processes and protocols, step timing (Ch 3, Ch 13)

**Conceptual Security Architecture** — The architect's view: Business Attributes Profile, control objectives, security strategies and architectural layering, security entity model and trust framework, security domain model, lifetimes and deadlines (Ch 3, Ch 10)

**Conditionally trusted entity** — An entity that can violate policy, but not without the misbehaviour being detected (Ch 10)

**Containment** — The defensive strategy of limiting damage once prevention has failed; second tier in prevent → contain → detect → react → collect evidence → assure (Ch 10, Ch 11)

**Contextual Security Architecture** — The business view: the business itself, business risk model, business process model, organisation and relationships, geography, time dependencies. Shortcuts here are the commonest cause of failure (Ch 3, Ch 9)

**Control** — A technical or procedural protection introduced to reduce vulnerability (Ch 1)

**Control objective** — An abstract description of a business requirement for control; the output of risk assessment is a *prioritised* set of control objectives, not a shopping list. The conceptual layer's *Why* (Ch 1, Ch 9, Ch 10)

**Corrective control** — A control that reduces an **impact** (Ch 15)

**Credentials certification** — Wrapping an entity's privilege credentials in a certificate digitally signed by the issuing security authority; also called an authorisation certificate or privilege attribute certificate (PAC) (Ch 11)

**Custodian** — In the owner/custodian/user model, the party that enforces the owner's policy over data or a system it holds but does not own (Ch 14)

**Daisy model** — The application architecture in which applications share common infrastructure services (directory, authentication, access control) around a common centre; the opposite of the stovepipe (Ch 10)

**Defence in depth** — Multiple independent layers of control so that no single failure is fatal; modelled in the book as the insulated cable, with cryptographic controls closest to the asset and organisation and policy at the outer layer (Ch 10, Ch 14)

**Dependency tree modelling** — Modelling a system as a tree of AND and OR dependencies; AND nodes are fragile, OR nodes are resilient. The shape of the tree *is* the risk profile (Ch 5)

**Detective control** — A control that detects a problem and **triggers other controls** (Ch 15)

**Deterrent control** — A control that reduces a **threat** (Ch 15)

**Directory service** — The repository for entity information — distinguished names, aliases and all attributes including security attributes. "Without directory services almost no other service can remain operable" (Ch 11)

**Directory entropy** — The tendency of directory contents to drift towards chaos over time with constant use by many entities; a schema slows it down (Ch 11)

**Domain (security)** — "A set of security elements subject to a common security policy defined and enforced by a single security policy authority" (ISO/IEC 10181) (Ch 10)

**Domain boundary** — The line separating what is inside a domain from what is outside; protected by logical access control for logical domains, firewalls for physical network domains, and doors, locks and guards for sites (Ch 10, Ch 11)

**Dual control** — Requiring two people to complete a sensitive action, so that neither alone can effect it; distinct from segregation of duties (Ch 17)

**Egg-and-honey model** — The end state of the perimeter progression: a hard shell *and* internal cellular structure — perimeter defence plus internal domain segmentation (Ch 10)

**Eggshell model** — A single hard perimeter with a soft interior; the binary friend/foe trust model the book rejects (Ch 10)

**Enablement objective** — The authors' coined complement to *control objective*, forcing the positive framing. The car-brake argument: better brakes enable faster driving (Ch 1)

**Entity (security)** — Something or someone that can take actions in a business environment and requires authorisation control; three types — individual personal, corporate, and application/system entities. Equivalent to *subjects* in access control terms (Ch 10)

**Entity schema** — The logical layer's *Who*: entities, their inter-relationships, attributes, authorised roles and privilege profiles, expressed as a schema (Ch 3, Ch 11)

**Entropy (passphrase)** — The true unpredictability of a secret; English text carries roughly 1.3 bits per character, so a 128-bit key needs about a 99-character passphrase. Entropy, not character count, is what matters (Ch 12)

**Eternal Triangle of Conflicting Objectives** — Cost control ↔ Security ↔ Usability; you cannot maximise all three, so architecture is the art of the best fit (Ch 2)

**Fast Track™ Workshop** — The book's facilitated workshop technique for compressing information collection and consensus building in the architecture programme (Ch 8)

**Feedback control loop** — Control → state change → measure → decide → control. Every security management process in the book is an instance of it, including governance and AS/NZS 4360 (Ch 5, Ch 15)

**Finite State Machine (FSM) modelling** — Modelling an interactive protocol as states, events and transitions in an event/state table, so no unforeseen state is left without a planned action; the hardest of the three advanced modelling techniques (Ch 5)

**Greenfield vulnerability assessment** — Assessing vulnerability as if no controls existed, so the contribution of planned controls can be measured. People will try to explain vulnerabilities away with existing controls; do not let them (Ch 9)

**Guideline** — Advice on how something might be done, carrying no obligation; the weakest form in the rules/practices/procedures/guidelines hierarchy (Ch 12, Ch 14)

**Hard metric** — A good solid objective number: response time, error rate, percentage up-time, search tree depth (Ch 6)

**Honeycomb model** — Internal cellular segmentation of a network or application estate into many small domains, with no single hard perimeter; the middle stage of the eggshell → honeycomb → egg-and-honey progression. A role is a honeycomb cell (Ch 10)

**Impact** — The business damage suffered if an asset is damaged or destroyed; rated H/M/L in the book's qualitative scheme (Ch 1, Ch 9)

**Incident management** — Restoring service as fast as possible; designed to conflict with problem management, which wants the system kept down for investigation. Security incidents sit exactly on the fault line (Ch 17)

**Independent domains** — Domains with no data objects in common, each constrained only by its own policy, whose authorities are not obliged to harmonise; they may enter into agreements (Ch 10)

**Information classification** — Labelling documents by sensitivity; the book largely rejects it commercially, since it produces a 60-cell handling matrix and information drifts permanently upward (Ch 14)

**Inhibitor** — A factor that may deter a threat agent: fear of capture, fear of failure, insufficient access, technical difficulty, high cost, sensitivity to public opinion (Ch 9)

**Inspector's View** — The auditor's and systems QA view; deliberately **not** a seventh SABSA layer, because assurance is designed into the architecture and folded into the operational layer (Ch 3, Ch 16)

**In-line cryptographic service** — Cryptography placed directly in the data path, so all traffic passes through it; contrasted with on-line, where the application calls out to a cryptographic service (Ch 12)

**Isolated domains** — Domains with no data objects and no activities in common; they cannot interact (Ch 10)

**Key-encrypting key (KEK)** — A long-lived key used only to protect other keys in transit or storage; the upper tier of the key management hierarchy (Ch 12)

**Key exposure** — Samples × time. The two trade off, which is the entire logic of the key hierarchy: bulk data keys change often, key-encrypting keys live long (Ch 12)

**Key management hierarchy** — The layered structure of master keys, key-encrypting keys and session/data keys, each protecting the tier below (Ch 12)

**Least privilege** — Granting only the access needed for the task. The book's most contrarian argument: it is a *platform* principle, not a *knowledge* principle — need-to-know versus need-to-share, with the 9/11 Commission Report as the case against over-application (Ch 14)

**Local Registration Authority (LRA)** — A delegated sub-registration authority registering users within its own domain; overall registration strength depends on every level of the delegation hierarchy (Ch 10)

**Logical domain** — A domain mapping onto groupings of logical entities — those sharing an application, a business community or a privileged group; bounded by logical access control (Ch 10, Ch 11)

**Logical Security Architecture** — The designer's view: business information model, security policies, security services, entity schema and privilege profiles, domain definitions and associations, security processing cycle (Ch 3, Ch 11)

**MAC / MIC (Message Authentication Code / Message Integrity Check)** — A cryptographic seal over a message; if the contents include the originator's identifier it gives origin *and* content authentication, and with a sequence number held as state at both ends, resequencing and deletion become detectable (Ch 11, Ch 12)

**Managed security service** — Security operations delivered by a third party; the book permits outsourcing *operational implementation* of policy but never policymaking, risk assessment, requirements specification or authorisation granting (Ch 8, Ch 14, Ch 17)

**Mechanism (security)** — A physical-layer implementation of a logical service: encryption, access control, digital signature, virus scanning. One service, many candidate mechanisms; Table 12-1 is the join (Ch 12)

**Metric type** — Hard or soft; step 4 of the six-step metrics framework procedure (Ch 6)

**Multi-tiered incident handling** — The layered response model of Figure 10-2: prevent, contain, detect and notify, collect and track events, recover and restore, assure (Ch 10)

**Non-repudiation** — Preventing a sender from falsely denying having sent a message or falsely denying its contents; one of the four things cryptography does (Ch 6, Ch 11, Ch 12)

**Nonce** — A one-time value never reused, built from time stamps, random numbers, sequence numbers or combinations, used against replay. Nonces do not *prevent* replay; they allow it to be **detected**, implying a recovery action (Ch 10, Ch 12)

**OAT (Operational Acceptance Testing)** — Testing by operations staff that the system is operable — the nine-category verification checklist; a different question from UAT, asked by different parties with a separate sign-off (Ch 16)

**Object class (directory)** — A schema component defined by a unique name, an OID, mandatory attributes, optional attributes, and an object kind (structural, auxiliary or abstract); attributes on neither list may not be used (Ch 11)

**Operational risk** — Basel II: "the risk of loss resulting from inadequate or failed internal processes, people and systems or from external events." Explicitly not "everything that isn't credit or market risk" (Ch 9, Ch 15)

**Operational risk management** — Risk assessment + risk mitigation; in the book's policy hierarchy it sits **above** information security, not below it (Ch 1, Ch 14, Ch 15)

**Operational Security Architecture** — The facilities manager's view; properly drawn as a vertical bar cutting all five other layers, and expanded into its own 6×6 matrix in Table 3-4 (Ch 3, Ch 17)

**Owner** — The designated entity that is policymaker for all risk management aspects of a system and exerts ultimate authority over it; owner decides, custodian enforces, user consumes (Ch 6, Ch 14)

**Penetration testing** — Adversarial testing across twelve levels from internal network penetration to social engineering; the partial answer to the fact that absence of unwanted function is not testable by black box means (Ch 16)

**Physical domain** — A domain mapping onto groupings of physical entities — a site, a platform or a network; bounded by firewalls, doors, locks, guards, or tamper-resistant devices (Ch 10, Ch 12)

**Physical Security Architecture** — The builder's view: business data model, security rules/practices/procedures, security mechanisms, users and the security user interface, platform and network infrastructure, control structure execution (Ch 3, Ch 12)

**Policy authority (security)** — The body responsible for setting and implementing the security policy within a domain; in a domain of registered users, the registration authority is the domain policy authority (Ch 10)

**Policy lifetime** — Policy decays in effectiveness over time, but frequent changes confuse and undermine confidence; review perhaps every three or four years. Rules, by contrast, change often (Ch 10)

**Polyinstantiation** — Duplicate directory or database entries for the same real-world thing; a schema exists partly to reduce it. Also the modelling artefact that made SeaView unusable (Ch 11, Ch 12)

**Practice** — A statement of what is habitually done, portable across platforms; backup practices are the same everywhere, backup *procedures* are not (Ch 12)

**Preventive control** — A control that reduces a **vulnerability** (Ch 15)

**Privilege Attribute Certificate (PAC)** — See *credentials certification*; a signed assertion of an entity's privileges rather than its identity (Ch 11)

**Problem management** — Root-cause investigation of underlying faults; in structural conflict with incident management's restore-now imperative (Ch 17)

**Procedure** — A prescribed sequence of steps for a specific platform or context; not portable (Ch 12, Ch 17)

**Red team review** — An objective appraisal by an independent expert team briefed to think either like the user or like an opponent, whichever suits the review's objective; a soft-metric measurement approach (Ch 6)

**Registration** — The process of enrolling an entity into a business community. "An important security control in its own right, because if an entity can become registered, it can fairly easily obtain privileges" (Ch 10, Ch 11)

**Registration lifetime** — Typically one to three years, with expiry and renewal preventing build-up of dormant registrations. **Certificate life must never exceed registration life** (Ch 10)

**Relying party** — The party who must rely on a claimant's claims; always the customer for trust, and therefore the party who should contract with and pay a trust broker (Ch 10)

**Risk category (A/B/C/D)** — The book's qualitative bands from impact × vulnerability: A red (severe, immediate action), B amber (significant, planned action), C green (acceptable, monitor), D blue (negligible, no action) (Ch 9, Ch 15)

**Risk register** — The repository of all risks — operational, strategic, reputation, credit, market, liquidity — with ID, name, type, threat, impact, vulnerability, category, **owner** and dates; itself an information repository requiring its own authorisation and access control (Ch 15)

**Rule** — A physical-layer statement interpreting policy in definite terms that can be incorporated into mechanisms; rules change frequently, policies do not (Ch 12, Ch 14)

**SABSA® Development Process** — The layer-by-layer development sequence implied by the SABSA model, with the operational architecture developed in parallel (Ch 7)

**SABSA® Lifecycle** — Strategy and Concept → Design → Implement → Manage and Measure, looping because operational experience becomes new business requirements (Ch 7)

**SABSA® Matrix** — The 36-cell grid of six layers × six questions; populating it *is* the work of developing an enterprise security architecture (Ch 3)

**SAML (Security Assertion Markup Language)** — XML framework for exchanging authentication and authorisation assertions; the book notes its reliance on lower-layer SSL/TLS with no signalling channel as a flaw (Ch 13)

**Security association** — ISO/IEC 10745: a set of shared security information items and attributes describing a relationship between entities, implying secure interaction rules and consistent security-related state variables at both ends (Ch 10)

**Security domain model** — The conceptual layer's *Where*: domains (logical and physical), boundaries and security associations (Ch 3, Ch 10)

**Security entity model** — The conceptual layer's *Who*: entities, the policy authorities governing trust in each domain, and inter-domain trust relationships (Ch 3, Ch 10)

**Security policy** — A statement expressing security requirements for a domain in general terms; says *what*, not *how*, which is why requirements are expressed as generic security services (Ch 11, Ch 14)

**Security service** — A generic, reusable capability delivering a security effect — authentication, confidentiality protection, integrity protection, non-repudiation, system assurance — catalogued in Table 11-2 by defensive strategy (Ch 11)

**Security standard** — The component layer's *Why*: external standards (international, national, industry) and the internal standards you must write yourself (Ch 13, Ch 14)

**Segregation of duties** — Splitting a sensitive task so no one person holds both halves; expressible as mutually exclusive roles a user entity may not hold together. "Guards must be segregated from the guarded" (Ch 11, Ch 17)

**Session authentication** — Mutual authentication counteracting man-in-the-middle and session hijack; the best versions exchange secret keys in the handshake and authenticate every subsequent exchange. It does not prevent changes — it makes them immediately detectable (Ch 11)

**Soft metric** — A qualitative, subjective measurement: focus groups, satisfaction surveys, independent audit and review against a Security Architecture CMM (Ch 6)

**SSE-CMM** — System Security Engineering CMM from ISSEA, now ISO/IEC 21827: 22 process areas × capability levels. The authors call it "a little on the heavy side" (336 pages) and recommend a simplified internal version (Ch 6)

**Stovepipe model** — The legacy application pattern: private database, private user registry, private interfaces, no cross-flow; the thing the Daisy model replaces (Ch 4, Ch 10)

**Sub-domain / super-domain** — A is a sub-domain of B iff A's elements and activities are subsets of B's, A's jurisdiction is delegated from B's authority, and A's policy does not conflict with B's. A may add policy only if B permits (Ch 10)

**System classification** — The book's preferred alternative to information classification: classify systems by risk, which is objective, transparent to users, auditable, and yields nine policies instead of sixty handling regimes (Ch 14)

**Threat** — A potential damaging event that puts assets in danger; assessed **binarily** (present or not) because quantitative threat probability is beyond your span of control (Ch 1, Ch 9)

**Threat agent** — The actor or force behind a threat: natural events, accidental events, technical failures, individuals, external organisations (Table 9-4) (Ch 9)

**Threat domain** — One of the four domains from the Basel II definition: People, Processes, Systems, External (Ch 9)

**Threat scenario** — The richer alternative to a threat list: specific threat, agent, capability, motivation, opportunity, catalysts, inhibitors and amplifiers (Table 9-3) (Ch 9)

**Threats Database** — The maintained inventory of threat domains × threat categories used to prompt thinking in risk assessment; the categories are "entirely arbitrary and based upon practical experience only" (Ch 7, Ch 9)

**Ticket** — A transitory credential built with symmetric cryptography, lifetime "up to a few hours generally speaking"; store an enduring template of the transitory structure, not the ticket itself (Ch 11)

**Time-Based Security** — P (protection time) > D (detection time) + R (reaction time) means the attack fails; E = D + R − P is the exposure. Design to reduce D and R (Ch 9)

**Traffic flow confidentiality** — Hiding volumetric flow, usually by dummy messaging presenting a continuous smooth volume of traffic (Ch 11)

**Transitive trust** — Trust conveyed through an introducer: birth certificates, passports, driving licences, banker's references, and a trusted third party issuing certified credentials so strangers can transact (Ch 1, Ch 10)

**Trust** — An attribute of relationships between business entities, **not** a technical attribute. Technical systems do not create trust; they protect trust that already exists (Ch 1, Ch 10)

**Trust assertion** — The generalisation of a certificate from "this identity is trustworthy" to "this statement about any information or transaction is trustworthy" (Ch 13)

**Trust broker** — A third party selling trust; must contract with and be paid by the **relying party**, and must offer liability cover, or the trust service has no value. What it can actually sell is a trusted *transaction*, not a certificate (Ch 10)

**Trusted time** — A common infrastructure service letting devices synchronise to a trusted source over a trusted protocol; "a critical piece of security infrastructure," because an opponent who can tamper with your clock can make you accept an expired message (Ch 10, Ch 11)

**Two-way traceability** — Down the model = completeness (every requirement is met); up the model = justification (every component traces back to a business requirement). The audit answer to "why are we doing this?" (Ch 6)

**UAT (User Acceptance Testing)** — Testing by users that the system does what the business needs, with a six-point problem classification scale from Cosmetic to Complete Failure; a different question from OAT (Ch 16)

**Unconditionally trusted entity** — An entity that can misbehave without the violation being detected — the dangerous category (Ch 10)

**Unique naming** — Assigning each entity a globally unique name so there is never confusion about which entity is referenced; local aliases are permitted where unique within the local domain (Ch 10, Ch 11)

**VAR (Value at Risk)** — The statistical loss figure at a given confidence level; the book warns it breaks down in the catastrophic tail — "infinity times zero" — and that you cannot estimate a probability from a database containing zero occurrences (Ch 15)

**Vulnerability** — A weakness in operational procedures or systems that allows a threat to materialise and exploit an asset; inside your span of control, therefore graded H/M/L rather than treated binarily (Ch 1, Ch 9)

**Web services architecture** — The component-layer pattern of a generic listener plus a specific façade translating to business logic; secured by the XML security standards family (Ch 13)

**White box testing** — Testing against internal structure; feasible at unit level, but "once the integration process begins and complexity grows it would be an infinitely difficult task" (Ch 16)

**XACML** — XML Access Control Markup Language, resolving the expression and exchange of access control policy in XML environments (Ch 13)

**XML firewall** — A firewall that can parse and inspect XML, needed because XML tunnels straight through a conventional network-level firewall (Ch 13)

**Zachman Framework** — John A. Zachman's enterprise architecture framework, from which SABSA's layer/question grid is adapted to a security view (Ch 3)
