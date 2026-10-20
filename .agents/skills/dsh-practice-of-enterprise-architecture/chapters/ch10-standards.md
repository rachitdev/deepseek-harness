# Chapter 10 — Standards

## Core Idea
**Standards are IT-focused rules EA artifacts.** They describe global technical rules, standards, patterns and best practices relevant to IT systems, representing **proven reusable means for IT systems implementation**. They are "backend" artifacts — developed largely by architects for architects, with little or no stakeholders outside the IT department. By shaping individual solutions, the prolonged use of Standards eventually shapes the entire IT landscape.

## Frameworks Introduced

### Questions Standards answer
What technologies and products should be used in IT solutions? · How exactly should available technologies be used? · What implementation approaches should be followed? · What components should be reused? · How should IT systems be organized and integrated? · What protocols should be used for interaction? · How should main data entities be stored? · What legislative prescriptions should be followed?

### Five subtypes
| Subtype | Prevalence | Definition |
|---|---|---|
| **Technology Reference Models** (TRMs; technology standards, technical reference models, technology reference architectures; or split into infrastructure/application reference models) | Essential | Structured graphical representations of all technologies used in an organization |
| **Guidelines** (also standards, principles) | Essential | IT-specific implementation-level prescriptions applicable in narrow technology-specific areas or domains |
| **Patterns** (also reference architectures) | Common | Generic reusable solutions to commonly occurring problems in the design of IT systems |
| **IT Principles** (often just "principles") | Common | Global high-level IT-specific guidelines influencing all IT-related decisions and plans |
| **Logical Data Models** (logical information models, canonical data models, data schemas) | Uncommon | Logical or even physical platform-specific definitions of common data entities and their relationship |

Plus two **less popular** subtypes: **technology inventories** (tabular/textual catalogs of all technologies — "a mixture of regular Inventories and Technology Reference Models") and **interface definitions** (formal specifications of key APIs: functions, parameters with names, ordinal numbers, data types and logical meaning; invocation protocols such as REST or SOAP; exceptions, result and error codes — useful for SOA-style landscapes).

## Key Concepts
- **Development.** Part of **Technology Optimization**, collaboratively by architects and technical subject-matter experts (senior developers, team leads). Approved by senior IT managers and signed off by the CIO as part of technology governance. Introduced **reactively, bottom-up**, as they become necessary for particular initiatives, often alongside projects pioneering a new technology. Usually **not developed from scratch** — architects study available industry standards, assess applicability, adapt and introduce them. Some (security, encryption, retention) derive from Policies.
- **Adherence** achieved via formal architectural peer-reviews of Outlines and Designs in investment and project governance, plus supervision during implementation. Reasonable justified deviations are tolerated; the decision may be informed by the estimated architecture debt.
- **Permanent, reviewed yearly** to reflect technology evolution; obsolete Standards discarded.
- **Domain focus.** Seldom the business domain; predominantly applications, data, integration, infrastructure, security.
- **Guidelines vs. Technology Reference Models.** TRMs specify *what* technologies should be used; Guidelines specify *how* those technologies should be used. TRMs may be used only by architects; Guidelines are relevant to both architects and project teams. Some Guidelines (e.g. secure coding standards) cannot be enforced at planning time and are enforced by reviewing the actual program code.
- **IT Principles vs. Guidelines.** IT Principles are broad, abstract, technology-agnostic imperatives with far-reaching impact shaping the overall landscape structure; Guidelines are narrow technology-specific rules shaping the internal details of separate solutions. The practical border "can be somewhat blurred and a strict border between them is arguably impossible to delineate." Organizations often establish **about 10–20 or more** IT Principles, grouped by EA domain (applications, data, integration, infrastructure, security), with a hierarchy in large companies.
- **Patterns structure**: description (what problem it solves and how), applicability (when it should and should not be applied), rationale (why it is the preferred solution), solution (the detailed recommended structure). Patterns are intangible — they exist only via instantiation in real solutions. Particularly often they deal with information exchange and integration.
- **Logical Data Models** use entity-relationship notations (e.g. UML), include primary and foreign keys, may define physical platform-specific representations (XSD for integration middleware, SQL DDL for Oracle and DB2). They are more detailed, IT-specific versions of Conceptual Data Models and updated synchronously with them. Combine decisions (the desirable standard structure) and facts (documenting existing structures). More useful in information-dependent organizations with **coordination and unification** operating models.

## Mental Models
- **Standards are the "interior", Landscapes are the "exterior".** Standards prescribe the internal structure of new solutions; Landscapes help fit them into the external environment.
- **Business executives control Standards only indirectly** — by establishing Considerations that influence Standards selection, and by hiring trustworthy architects who reflect genuine business needs.
- **Technology Reference Models are decisions *and* facts.** "The set of utilized technologies represents objective facts, whereas their color-coding represents planning decisions." Adding a newly discovered technology with an unknown status can be done by a single architect; deciding to reuse, support or decommission it requires collective consensus of architects, senior IT managers and subject-matter experts.

## Anti-patterns
**The single biggest threat: overly strict and inflexible Standards.** They "virtually paralyze the delivery of new IT initiatives with endless negotiations, bureaucracy and red tape." Standardization for its own sake proliferates superfluous Standards trying to regulate everything, and architecture functions that stop projects for minute deviations become known as **"architecture police"**. "In relation to Standards, 'the more, the better' is an inappropriate and detrimental attitude."

Also: Standards established by architects alone without hands-on IT specialists (lead developers, senior infrastructure engineers) "are often found too clumsy by project teams and eventually end up in 'ivory towers'."

**The remedy** — a hierarchy of Standards (Figure 10.6):
- Large organizations: only a list of core strategic vendors standardized organization-wide.
- Medium-sized companies: also a kit of concrete vendor products for all business units.
- Business-unit level: unit-specific technical Standards reflecting process-specific requirements.
- IT-department subunits: technology-specific Standards.
- Project teams: local implementation-level Standards (directory structures, variable naming and other coding standards) — often informal, undocumented, agreed verbally or learned from the existing codebase.

More abstract Standards (IT Principles, Technology Reference Models) sit higher; more detailed ones (Patterns, Guidelines) sit lower. **Operating model shapes the hierarchy**: diversification gives high or unlimited autonomy and a wide tree; replication and unification imply little technical independence and narrow hierarchies. Review, revise and clean up yearly; tolerate documented, analysed deviations (possibly as architecture debts) to enable organizational learning.

## Reference Tables

### Benefits of Standards, with mechanisms
| Benefit | Mechanisms |
|---|---|
| **Faster initiative delivery** | Accumulate and leverage existing technical expertise; establish reusable components/building blocks; avoid learning curves of untried technologies |
| **Reduced costs** | Limit the number of supported technologies, products and vendors; minimize proprietary license fees; streamline IT staff skill sets and optimize the workforce |
| **Mitigated risks** | Reuse proven implementation approaches (delivery risk); reuse proven technologies (stability and security); adhere to regulatory acts (compliance risk) |
| **Lowered complexity** | Restrain technological diversity; control the diversity of implementation approaches; minimize interaction patterns and avoid the "spaghetti" of connections |
| **Improved interoperability** | Eliminate technological disparity; leverage common integration approaches and protocols; achieve logical data compatibility via common data types, structures and formats |

### Technology Reference Model status color-coding
| Status | Meaning |
|---|---|
| **Current** | Up-to-date, in use, not planned to be retired, can be safely reused |
| **Emerging** | Cutting-edge, not actively used now, planned for future adoption, can be used in new solutions |
| **Unsupported** | Outdated, in use, no longer vendor-supported, should not be reused |
| **Retiring** | Legacy, in use, planned for retirement soon, should not be reused |
| **Prohibited** | Known critical defects (licensing issues, security vulnerabilities); must be eliminated immediately, urgent action required |
| **Retired** | Presumably completely removed recently, temporarily left in the model for communication and status clarification |
| **Unknown** | Status unclear; decisions on future and reuse prospects not yet made |

### Standards vs. adjacent types
| Compared with | Difference |
|---|---|
| **Considerations** | Considerations' rules carry business importance and represent critical business decisions; they are abstract, in business language, influencing both business and IT decision-making. Standards are narrow, IT-specific, reflect no business interests directly, and offer more specific prescriptions applicable in particular projects. Business leaders do not understand Standards and cannot use them to manage IT. |
| **Landscapes** | Landscapes refer to particular instances of IT assets and their connections; Standards do not distinguish individual assets but give rules for all assets of a type. Standards recommend how to develop new systems but do not describe what systems exist or how they relate. |

## Worked Example
**Technology Reference Model structure and content.** Highest level either aligns with the stack of technical EA domains (applications, databases, middleware, networks, servers, security) or is arbitrary (end-user application services such as communication and office tools; system services such as virtualization and storage platforms; infrastructure services such as telephony and hosting). Structured hierarchically: the databases category may comprise relational databases, NoSQL databases and cloud-based storage; the software servers category may subclassify into web servers, application servers and servlet containers. Leaves show actual technologies, often with version numbers. In packaged-solution organizations TRMs stay high-level and focus on products, providers and vendors; in organizations writing their own software they get very detailed — concrete versions of programming languages, major libraries and frameworks (Spring, Hibernate, AngularJS) and development instruments (build automation tools, continuous integration servers, SCM and version control). They depict technologies end users never notice: web browsers, document viewers, email clients, instant messengers.

**Kotusev's sample Guidelines (Figure 10.2).** Server Deployment Standards: "Run Applications as OS Services", "Store Deployment Packages in VCS". Network Protocol Standards: "Avoid Using UDP Multicast", "Prefer REST Over SOAP". Data Encryption Standards: "Use 256-Bit Encryption Keys", "Store MD5 Hashes of Passwords". Interface Design Guidelines: "Use Web-Safe Colours", "Place Menu in the Top Right Corner". Secure Coding Guidelines: "Initialize Variables to Safe Defaults", "Validate All Incoming Data". Other examples in the text: "All data backups should be encrypted and password-protected to prevent unauthorized access", "Each user request should be assigned a unique transaction ID to enable traceability", "All interactions with external systems should be logged to monitor their SLAs".

**Kotusev's sample IT Principles (Figure 10.4).** Applications: "Prefer Open Source Solutions", "Log All Main Operations". Data: "Use Scalable Storage", "Backup All Permanent Data". Integration: "Use Middleware for Integration", "Avoid Binary Integration Protocols". Infrastructure: "Host in the Cloud", "Dedicated Server for Each System". Security: "Place Public Systems in DMZ", "Secure by Default".

**A fully specified IT Principle.** **"Two-Vendor Policy"** —
- *Statement*: "We work and build long-lasting relationships with two strategic vendors simultaneously without giving preference to either of them"
- *Rationale*: "Working with two different vendors allows balancing consolidation and flexibility, leveraging economies of scale but avoiding vendor lock-in"
- *Implication*: "In every situation products and services of both strategic vendors should be equally considered and the best of the two offers should be preferred"

**Policy → Guideline derivation.** "If regulatory Policies require handling personal information and credit card data in encrypted formats, then corresponding Guidelines may be developed to specify exactly how these types of data must be encrypted, transmitted, stored and protected in all corporate IT systems." Likewise, if Policies require preserving audit trails, access logs and electronic documentation for a set number of years, specific Patterns or other Standards provide the recommended way of meeting that across the landscape.

## Key Takeaways
- Standards are architect-to-architect reference material with no direct business stakeholders.
- TRMs and Guidelines are essential; Patterns and IT Principles common; Logical Data Models uncommon.
- Standardize only what genuinely matters, at the right organizational level, with hands-on practitioners involved.
- TRMs blend facts (what is used) and decisions (what status it has) — update each part with the right process.

## Connects To
- [Chapter 9 — Considerations](ch09-considerations.md) — Policies derive Standards; Conceptual Data Models derive Logical Data Models
- [Chapter 12 — Landscapes](ch12-landscapes.md) — the complementary IT-focused structures
- [Chapter 14 — Designs](ch14-designs.md) — where Standards are enforced
- [Chapter 18 — Instruments for Enterprise Architecture](ch18-instruments-for-enterprise-architecture.md) — architecture debt from Standards deviations; agility and standardization
