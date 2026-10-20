# Chapter 8 — The CSVLOD Model of Enterprise Architecture

## Core Idea
The **CSVLOD model** is the book's central conceptualization of enterprise architecture. Its heart is a **taxonomy with two orthogonal dimensions** — *what* artifacts describe (rules / structures / changes) and *how* they describe it (business-focused / IT-focused) — whose intersection produces **six general types of EA artifacts**: Considerations, Standards, Visions, Landscapes, Outlines and Designs. Beneath these six types sit **24 narrow subtypes**, classified by prevalence into **essential, common and uncommon**.

## Frameworks Introduced

### Dimension One — What?
### Dimension Two — How?
### Their intersection — the six-cell taxonomy (Figure 8.1)

| | **Rules** | **Structures** | **Changes** |
|---|---|---|---|
| **Business-focused** | **Considerations** | **Visions** | **Outlines** |
| **IT-focused** | **Standards** | **Landscapes** | **Designs** |

## Key Concepts
- Each general type's main properties are determined by its **column and row**, plus its own **type-specific properties**.
- Each general type represents a potentially infinite variety of artifacts, but a limited set of **concrete subtypes** recurs across organizations. Many useful artifacts are unique and organization-specific and belong to no subtype.
- **Successful EA practices typically use limited pragmatic sets of 10–15 different EA artifacts**, not all imaginable artifacts.
- Of the six general types, **only most Landscapes and some Standards are facts** EA artifacts; everything else — including most Standards and some Landscapes — is decisions EA artifacts.
- Subtype titles in the book are "either the most commonly used titles of respective EA artifacts, or the most descriptive titles accurately conveying their perceived practical meaning" — the industry has no standardized names.

## Mental Models
- **Prevalence bands.** Essential = used in more than 50% of established EA practices. Common = ~25–50%. Uncommon = ~10–25%. Eight subtypes in each band.
- **Business-focused artifacts are always dual.** They are the "communication interfaces between business and IT", always developed collaboratively with — or at minimum consciously approved by — business leaders. IT-focused artifacts are "internal IT tools invisible to business", developed predominantly by architects.

## Anti-patterns
- Treating popularity as importance — see Chapter 15 for the explicit caveat that essential artifacts are not mandatory.
- Building a comprehensive artifact set instead of a pragmatic one.

## Reference Tables

### Table 8.1 — Rules, structures and changes EA artifacts
| | Rules | Structures | Changes |
|---|---|---|---|
| Describe | General global rules defining an organization or its divisions | High-level structures of an organization or its parts | Specific proposed incremental changes to an organization |
| Scope | Very broad, often relate to an entire organization | Broad, often cover large areas of an organization | Narrow, limited to separate IT initiatives or projects |
| Format | Often textual | Usually graphical | Mix of textual and graphical |
| Question | How do we work or want to work? | What approximately do we have or want to have? | What exactly are we going to change right now? |
| Lifecycle | Permanent, created once and then periodically updated | Permanent, created once and then continuously updated | Temporary, created for specific purposes and then discarded |
| Role | Basis for all other planning decisions | High-level "maps" facilitating decision-making | Tactical plans of an organization |
| Purpose | Help achieve consistency and homogeneity of all planning decisions | Help understand what changes are desirable and how to implement them | Help plan separate changes in detail |

Additional distinctions: rules typically do not refer to specific instances but to all instances of a type, and are **intangible and uncountable** (a rule may prescribe Linux on all servers whether there are zero, one, 50 or 1000). Structures refer to specific but relatively abstract instances and their relationships, and are tangible and countable. Changes describe concrete instances with internal details (process steps, system components, functional specifications, data objects, communication interfaces) and are tangible.

### Table 8.2 — Business-focused and IT-focused EA artifacts
| | Business-focused | IT-focused |
|---|---|---|
| Language | Technology-neutral business language (money, customers, capabilities, business goals, competitive advantages) | Technical IT-specific language (systems, applications, databases, platforms, networks) |
| Domains | Business domain and often other relevant domains at a high level | Applications, data, integration, infrastructure, security domains and sometimes business domain |
| Format | Brief, intuitive, largely informal, only the most essential information | Can be voluminous, formal, use strict notations, include comprehensive details |
| Stakeholders | Business leaders and architects | Architects and other IT specialists |
| Role | Communication interfaces between business and IT | Internal IT tools invisible to business |
| Purpose | Help business leaders manage IT | Help architects organize IT |

### Figure 8.2 — The CSVLOD model, cell by cell

| | **Considerations** | **Standards** | **Visions** | **Landscapes** | **Outlines** | **Designs** |
|---|---|---|---|---|---|---|
| Content | Global conceptual rules and fundamental considerations important for business and relevant to IT | Global technical rules, standards, patterns and best practices relevant to IT systems | High-level conceptual descriptions of an organization from the business perspective | High-level technical descriptions of the organizational IT landscape | High-level descriptions of separate IT initiatives understandable to business leaders | Detailed technical and functional descriptions of separate IT projects actionable for project teams |
| Meaning | Decisions on how an organization needs to work from the business and IT perspective | Decisions on how all IT systems should be implemented + some facts on current approaches and technologies | Decisions on what IT should deliver to an organization in the long run | Facts on the current IT landscape + some decisions on its future evolution | Decisions on how approximately specific IT initiatives should be implemented | Decisions on how exactly specific IT projects should be implemented |
| Usage | Developed collaboratively by senior business leaders and architects, then used to influence all architectural decisions | Developed collaboratively by architects and technical subject-matter experts; used to shape the architectures of all IT initiatives | Developed collaboratively by senior business leaders and architects; used to guide IT investments, identify, prioritize and launch initiatives | Developed and maintained by architects; used to rationalize the landscape, manage asset lifecycle, plan initiatives | Developed collaboratively by architects and business leaders; used to evaluate, approve and fund specific initiatives | Developed collaboratively by architects, project teams and business representatives; used by project teams to implement projects |
| Purpose | Help achieve the agreement on basic principles, values, directions and aims | Help achieve technical consistency, technological homogeneity and regulatory compliance | Help achieve the alignment between IT investments and long-term business outcomes | Help understand, analyze and modify the structure of the IT landscape | Help estimate the overall business impact and value of proposed IT initiatives | Help implement approved IT projects according to business and architectural requirements |
| Benefits | Improved overall consistency between business and IT | Faster initiative delivery, reduced costs, risks and complexity | Improved strategic effectiveness of IT investments | Increased reuse and agility, reduced duplication and legacy | Improved efficiency and ROI of IT investments | Improved quality of project delivery |
| Time focus | Stateless or long-term future | Stateless or current state | Long-term future, 3–5 years | More often the current state | Mid-term future, up to 1–2 years | Short-term future, up to one year |
| Typical format | Simple intuitive formats, often brief written statements | Various formats, often strict notations | Brief informal formats, often simple one-page diagrams | Strict formats, often complex one-page diagrams using ArchiMate | Mix of textual descriptions and simple diagrams | Mix of text, tables and complex diagrams, often UML; can be voluminous |
| Lifecycle | Permanent | Permanent | Permanent | Permanent | Temporary | Temporary |

### The 24 subtypes with prevalence
| General type | Essential | Common | Uncommon |
|---|---|---|---|
| **Considerations** (5) | Principles | Policies | Conceptual Data Models, Analytical Reports, Direction Statements |
| **Standards** (5) | Technology Reference Models, Guidelines | Patterns, IT Principles | Logical Data Models |
| **Visions** (5) | Business Capability Models, Roadmaps | Target States | Value Chains, Context Diagrams |
| **Landscapes** (4) | Landscape Diagrams | Inventories, Enterprise System Portfolios, IT Roadmaps | — |
| **Outlines** (3) | Solution Overviews | Options Assessments | Initiative Proposals |
| **Designs** (2) | Solution Designs | — | Preliminary Solution Designs |

24 subtypes in total: **eight essential, eight common, eight uncommon**.

### Subtype one-line definitions (Kotusev's own)
- **Principles** — global high-level guidelines influencing all decision-making and planning in an organization
- **Policies** — overarching organizational norms typically of a restrictive nature providing compulsory prescriptions in certain areas
- **Conceptual Data Models** — abstract definitions of the main data entities critical for the business and their relationship
- **Analytical Reports** — executive-level analyses of relevant technology trends and their potential impact on the business
- **Direction Statements** — conceptual messages communicating major organization-wide decisions with far-reaching consequences
- **Technology Reference Models** — structured graphical representations of all technologies used in an organization
- **Guidelines** — IT-specific implementation-level prescriptions applicable in narrow technology-specific areas or domains
- **Patterns** — generic reusable solutions to commonly occurring problems in the design of IT systems
- **IT Principles** — global high-level IT-specific guidelines influencing all IT-related decisions and plans
- **Logical Data Models** — logical or even physical platform-specific definitions of common data entities and their relationship
- **Business Capability Models** — structured graphical representations of all organizational business capabilities, their relationship and hierarchy
- **Roadmaps** — structured graphical views of all planned IT initiatives in specific business areas having direct business value
- **Target States** — high-level graphical descriptions of the desired long-term future state of an organization
- **Value Chains** — structured graphical representations of the added value chain of an organization
- **Context Diagrams** — high-level graphical descriptions of the current operational flows of an organization
- **Landscape Diagrams** — technical "boxes and arrows" schemes of different scopes and granularities describing the IT landscape
- **Inventories** — structured catalogs of currently available IT assets describing their essential properties and features
- **Enterprise System Portfolios** — structured high-level mappings of all essential IT systems to relevant business capabilities
- **IT Roadmaps** — structured graphical views of all planned IT initiatives of a purely technical nature having no visible business impact
- **Solution Overviews** — high-level descriptions of specific proposed IT solutions understandable to business leaders
- **Options Assessments** — lists of available high-level implementation options for specific IT initiatives with their pros and cons
- **Initiative Proposals** — very early idea-level descriptions of proposed IT initiatives and their justifications
- **Solution Designs** — detailed technical and functional specifications of approved IT solutions actionable for project teams
- **Preliminary Solution Designs** — preliminary high-level technical and functional designs of specific approved IT solutions

## Worked Example
**Questions the CSVLOD model answers.** What types of EA artifacts constitute enterprise architecture? What are the conceptual differences between them? What are their essential properties? Who uses them? How are they used? What are their roles and purposes? What is the relationship between them (Figure 4.8)? What processes are supported by them (Figure 6.1)? How do they benefit organizations?

**How to read a cell.** All artifacts related to Considerations share the common properties of *both* rules and business-focused artifacts; all artifacts related to Designs share the common properties of *both* changes and IT-focused artifacts. Then each type adds its own type-specific properties defining its unique role.

## Key Takeaways
- Two dimensions, six types, 24 subtypes, three prevalence bands.
- Business-focused = dual, brief, intuitive, agreed with business. IT-focused = technical, detailed, architect-owned.
- Aim for a pragmatic set of 10–15 artifacts, not maximal coverage.

## Connects To
- [Chapter 4 — Enterprise Architecture and City Planning](ch04-enterprise-architecture-and-city-planning.md) — the intuitive introduction of the same six types
- [Chapter 9 — Considerations](ch09-considerations.md) through [Chapter 14 — Designs](ch14-designs.md) — one chapter per general type
- [Chapter 15 — The CSVLOD Model Revisited](ch15-the-csvlod-model-revisited.md) — continuous taxonomy, mappings, decision paths, exceptions
