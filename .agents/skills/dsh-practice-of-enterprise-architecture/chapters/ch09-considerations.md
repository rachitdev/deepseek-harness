# Chapter 9 — Considerations

## Core Idea
**Considerations are business-focused rules EA artifacts.** They describe global conceptual rules and fundamental considerations important for business and relevant to IT — the "overarching organizational context for information systems planning". They are the **global compass**: they say which general direction is right or wrong and what decisions are desirable or unacceptable, without saying where specifically the organization must go.

## Frameworks Introduced

### Questions Considerations answer
How should an entire organization work? · Which operating model is desirable? · Which business processes should be standardized across business units? · Which types of data should be standardized and shared organization-wide? · What is the general role and purpose of IT? · How should, and should not, an organization use information systems? · What technology trends may be disruptive for the business? · What IT-driven innovations may be strategic?

### Five subtypes
| Subtype | Prevalence | Definition |
|---|---|---|
| **Principles** (also maxims, drivers) | Essential | Global high-level guidelines influencing all decision-making and planning |
| **Policies** (security, cloud, access, information exchange policies) | Common | Overarching organizational norms of a restrictive nature providing compulsory prescriptions |
| **Conceptual Data Models** (corporate/enterprise data models, information models, data architectures) | Uncommon | Abstract definitions of the main data entities critical for the business and their relationship |
| **Analytical Reports** (whitepapers, position papers, strategy papers) | Uncommon | Executive-level analyses of relevant technology trends and their potential impact |
| **Direction Statements** (architecture strategies, strategic/position/governance papers, or area-specific: digital channels strategies, ERP strategies, data warehousing strategies) | Uncommon | Conceptual messages communicating major organization-wide decisions with far-reaching consequences |

## Key Concepts
- **Development and usage.** Developed as part of **Strategic Planning**, collaboratively by senior business executives and architects, based on how the organization should operate to achieve long-term goals. Discussions cover the target operating model: desired synergies between units, global process standardization and data integration opportunities. Often explicitly signed off by both parties as part of strategic governance. Physically created by architects, but "their real meaning is based mostly on the desire of business executives to work in a particular way."
- **Permanent, reviewed yearly** after re-approval of the business strategy; may be revised or discarded as irrelevant. Exception: most **Direction Statements** and some **Analytical Reports** are temporary.
- **Deviations tolerated** — reasonable, substantiated deviations from Considerations are usually acceptable, **except for Policies**. Deviation decisions may be informed by the volume of the associated architecture debt.
- **Scope.** Often organization-wide; large decentralized companies also develop unit-level Considerations consistent with global ones.
- **Not everything strategic becomes a Consideration.** HR strategies (candidate qualities, promotion criteria, salary ranges), legal and financial strategies typically have no articulate IT implications and are seldom represented.

### Principles in detail
Definition of a single Principle usually includes **title, statement, rationale and implications**. Alternative organization-specific structures exist: description, reasoning, prerequisites, context of usage, notes for usage. Organizations usually establish **up to 5–10 global guiding Principles**, sometimes formally endorsed by the technology committee of the board; grouped into categories (e.g. business, data, systems); stored in a simple MS Word file. Large companies establish a **hierarchy**: organization-wide overarching Principles plus more specific local Principles for business units. Principles often provide the basis for developing IT Principles.

### Policies in detail
Formulated as textual descriptions usually specifying what an organization **must not** do. More precise, definite and unambiguous than Principles; not a subject of debate. Principles give "positive" guidance (what to do); Policies give "negative" guidance (what not to do). Policies define the boundaries and limits of possible planning decisions and **cannot be deviated from**. Often derived from external regulation: GDPR (EU), Sarbanes-Oxley, Gramm-Leach-Bliley, HIPAA (US), PCI DSS (international). More common in organizations handling sensitive data (financial, personal) and regulated industries (banking, healthcare). Policies often provide the basis for developing more detailed IT-specific Standards.

### Conceptual Data Models in detail
Simple intuitive data diagrams describing essential information entities in a business-oriented manner, organized around subject areas (finance, customer, facilities, human resources). Higher-level models define major data entities, groupings and dependencies; lower-level models describe data objects, attributes and relationships, sometimes with a glossary. Relatively stable, may look similar across an industry — e.g. insurers can base theirs on the industry-standard **ACORD Information Model**. They let executives specify what information matters, propose naming conventions and agree the semantic meaning of core entities (exactly what counts as "customer" or "product"). They provide the basis for more detailed **Logical Data Models**. More useful for heavily information-dependent organizations and those with **coordination or unification** operating models.

### Analytical Reports in detail
Forms: **hype cycles**, **technology radars**, **SWOT analyses**, **vendor analyses**.
- Hype cycles assess emerging technologies by five maturity phases: technology trigger, peak of inflated expectations, trough of disillusionment, slope of enlightenment, plateau of productivity.
- Technology radars assess by four stages of adoption readiness: hold, assess, trial, adopt.
- SWOT analyses cover technology strengths, weaknesses, opportunities and threats.
- Vendor analyses evaluate and rank market offerings, often using techniques similar to the Gartner Magic Quadrant and Forrester Wave.

Questions they answer: What new technologies may be relevant? Which should be adopted or ignored? What are our technology strengths and weaknesses? What opportunities and threats does the technology environment provide? Which product vendors or service providers should be selected for strategically important technologies?

### Direction Statements in detail
The **most action-oriented of all Considerations** — they point to a direction and explain the rationale, but not how. Examples of directives: migrate all non-business-critical applications into the cloud; consolidate all corporate systems on a single global ERP platform; or, conversely, develop custom systems tailored to individual business units. Often treated as the architectural constituent of a broader IT strategy (which also covers solution delivery, system maintenance, sourcing, staff education) and refreshed yearly with it. Complementary to Analytical Reports: reports analyse positioning, statements propose strategies addressing them. They provide the basis for developing more specific **Visions**. Mostly **temporary** artifacts with a limited (though possibly long) lifespan.

## Mental Models
- **The "smoke" test for Considerations.** Meaningful Considerations "always imply certain trade-offs, can be potentially disagreed with and stay meaningful when negated." Useful Considerations let you clearly distinguish appropriate planning decisions from inappropriate ones.
- **Considerations set guardrails, Visions set the destination.** "Considerations typically describe how an organization wants to work, rather than what an organization wants to do."

## Anti-patterns
**The single biggest threat: trivial and universal Considerations** that apply equally to every organization, "only distract attention, but add little or no real value".

Useless Principles Kotusev names: "All IT systems should be effective and efficient" · "All IT systems should be developed for specific purposes" · "All IT systems should be driven by the essential business needs" · the ubiquitous industry-standard "Reuse Before Buy, Buy Before Build" and "Data Is a Corporate Asset".

Useless Analytical Reports: recommending adoption of the latest release of Windows on Windows computers, or that web applications embrace the latest browser versions. Valuable ones instead assess the readiness, maturity, suitability, adoption opportunities and non-adoption risks of potentially disruptive technologies — cloud, big data, RFID, blockchain.

## Reference Tables

### Considerations vs. adjacent types
| Compared with | Difference |
|---|---|
| **Standards** (rules, IT-focused) | Standards' rules are purely technical, incomprehensible and useless to executives; their influence is limited to narrow IT decisions (how best to implement an initiative). Considerations are organization-wide rules influencing both business and IT decisions at the **portfolio** level (what initiatives are desirable). Considerations let business leaders govern IT indirectly, without understanding how IT works. |
| **Visions** (business-focused, structures) | Visions' descriptions are more specific, situational and volatile. Considerations focus on the most profound, stable business aspects less dependent on the latest priorities. Considerations influence Visions but do not direct future investments or provide real guidance for launching initiatives. |

### Consideration → downstream effects
| Consideration | Ripple effects |
|---|---|
| Requirement for a single customer view | Selection of reliable secure storage for a shared customer repository; deployment of integration infrastructure to reach it from all locations; **cancellation** of current or planned initiatives contradicting it; modification of the designs of all new projects to use the same customer database |
| Decision to adopt a new strategic technology | Development of guidelines regulating its usage; initiation of pilot projects; upgrade of the current landscape for compatibility |
| Policy restricting offshore storage of commercially sensitive data | Considerable implications at both organization-wide and project levels |

## Worked Example
**A fully specified Principle (Kotusev's own).** Principle **"Business Continuity"** —
- *Statement*: "Critical business operations should not be interrupted even in case of severe emergency"
- *Rationale*: "Uninterrupted 24/7 business operations are required to provide flawless customer experience and establish the reputation of the most trusted service provider according to the business strategy"
- *Implication*: "All critical information systems and databases should have geographically dispersed, readily available reserve copies in place"

**Deceptively trivial but genuinely meaningful Principles.** "All IT systems should be designed for high availability and business continuity" and "All IT systems should be highly secure" *seem* universal, but in reality "typically indicate that an organization needs IT systems with increased levels of fault tolerance and security exceeding some reasonable default expectations (e.g. guaranteed service uptime of 99.99% or even higher) and its business executives are ready to invest additional amounts of money in the necessary IT infrastructure to achieve these desired levels." Hot standby servers, real-time data backups and MFA imply "an inevitable trade-off between business continuity, information security, user convenience, development timelines and costs."

**Genuinely organization-specific Principles.** "All IT systems deployed in retail outlets should work correctly with the intermittent Internet connectivity" · "All customer-facing web interfaces should be accessible via a single sign-on mechanism".

**Kotusev's sample Policy set (Figure 9.2).** External — National Privacy Policies: "Personal Data Must Be Stored Onshore", "Destroy Personal Data When Not Needed"; Sarbanes-Oxley Policies: "Log All Accesses to Accounting Systems", "Retain Audit Trails and Emails for 5 Years". Internal — Data Security Policies: "No Sensitive Data on Mobile Devices", "Store Credit Cards in Encrypted Formats"; Data Exchange Policies: "Do Not Share Key Data with Third Parties", "Share Client Data with Trusted Partners"; Cloud Hosting Policies: "Use Only the PCI DSS Compliant Cloud", "Do Not Store Health Data in the Cloud".

**Conceptual Data Model in action.** "If Conceptual Data Models define the customer data entity as a set of name, date of birth, location and phone properties, then every IT system capturing customer data should capture name, date of birth, location and phone properties, while all customer databases should store these properties in appropriate formats." Business value follows: dates of birth for all customers enable birthday marketing campaigns; home addresses enable location-specific offerings.

## Key Takeaways
- Considerations = the decision-making framework, not the destination.
- Principles (essential) and Policies (common) do most of the work; the other three subtypes are uncommon.
- Apply the smoke test: if negating it is meaningless, it is not a Consideration.
- Deviations tolerated everywhere except Policies.

## Connects To
- [Chapter 8 — The CSVLOD Model of Enterprise Architecture](ch08-the-csvlod-model.md) — where Considerations sit in the taxonomy
- [Chapter 10 — Standards](ch10-standards.md) — the IT-focused rules Considerations shape
- [Chapter 11 — Visions](ch11-visions.md) — the business-focused structures Considerations influence
- [Chapter 5 — The Dialog Between Business and IT](ch05-the-dialog-between-business-and-it.md) — the operating model that Principles typically encode
