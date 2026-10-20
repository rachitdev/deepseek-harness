# Chapter 11 — Logical Security Architecture

## Core Idea
The logical layer *"develops more detail to flesh out the bones of the conceptual framework."* It is **the designer's view and it is almost entirely functional**: it defines a comprehensive set of *functional requirements* — what security must do — and deliberately says nothing about the mechanisms that will deliver those functions, which belong to the physical layer below.

> "Security services are logical services — specified independently of what physical mechanism might be used to deliver them. They are driven from the layer above, most specifically from the Business Attributes Profile, the control objectives and the security strategies."

The chapter fills the logical row: Business Information Model (What), Security Policies (Why), Security Services (How), Entity Schema and Privilege Profiles (Who), Security Domain Definitions and Associations (Where), Security Processing Cycle (When) — plus the **Security Improvements Programme** as a methodology deliverable.

## Frameworks Introduced

### Table 11-1: Abstraction Levels of the Real Business (the information architecture)
The knowledge / information / data distinction is mapped directly onto SABSA layers.
| SABSA Layer | Abstraction level | Explanation |
|---|---|---|
| Contextual | **Business** | The real business context |
| Conceptual | **Business Knowledge** | Information that has been given business value and related to business context through interpretive or reflective intellectual activity of a human |
| Logical | **Business Information** | Data that has been transformed and structured to have business meaning and relevance through intelligent analysis and synthesis |
| Physical | **Business Data** | Raw facts and quantities that form the inputs and outputs of business processes and that are processed and stored during process execution |

Note: *"It is not the task of a security architecture team to develop the Business Information Architecture"* — it is assumed to be a pre-existing model.

**The seven properties of information:** it is a logical representation of something real (a "real customer" is represented by "customer information"); it is **structured data** — organised into fields, records, files, tables, databases; information structures relate hierarchically *and* peer-to-peer; it is **time-related** (currency or historical context matters); it is **independent of location**, though *access* to it may be location-dependent; its quality depends on the structure used to present it and the analytical tools applied, not only the underlying data ("all the relevant data may be there, but the information needed may not be apparent"); and its success is best measured by the **user experience** of using it. Critically: *"When one talks of information assets, these are **secondary assets** — representing the real primary business assets that you want to protect."*

### Static vs dynamic information — different service needs
| Class | Definition | Examples |
|---|---|---|
| **Static** | Does not move or change in the short term | Master records and files (e.g. customer information); executable object code; configuration information for systems and applications; historical information — all stored audit trails, historical transaction records, historical message records |
| **Dynamic** | Changes and moves in the short term; may have a short lifetime between creation and destruction | Real-time free-format messages (e-mail); real-time structured application messages (SQL database queries); real-time transaction information; system and service management real-time information exchanges |

Both need **confidentiality protection, integrity protection, availability protection**. Dynamic information additionally needs **authenticity of source** and **non-repudiation**. (Note the deliberate placement: *stored* transaction and message information counts as static.)

### The seven security services for business transactions
Business transactions are a special case of dynamic information and imply: **business user identification; business user authentication; business user authorisation; business entity authentication; business transaction integrity protection** (completed as expected, protected from unauthorised modification, duplication, replay, delay or deletion); **business transaction authentication**; **business transaction non-repudiation**.

### Security policy — the theoretical view (from ISO/IEC 10181)
> "Security policies are statements of what type of security and how much should be applied to protect the business in various ways."

A security policy defines what is meant by security *within a security domain*, the high-level rules for achieving it, the activities authorised to achieve security objectives, and **how entities outside the domain are allowed to interact with entities inside it**.
- The **domain owner** sets the policy. The owner may delegate implementation to a lower security authority acting on the owner's behalf — **that delegated authority is effectively the custodian of the domain**.
- A **security authority** is an entity responsible for the implementation of a security policy on behalf of the owner (and may also be the owner); it may further delegate *enforcement* to other entities within the domain.
- **Policy states what is to be done, not how.** *"The security policy states what should be done but as far as possible avoids any reference to particular technical solutions. For this reason security requirements are expressed in terms of generic security services."*

### Figure 11-1: The hierarchical security policy architecture
The distinguishing feature: *"this is different because it does not have as its top layer the corporate information security policy — this appears on the second layer."* Information security management is only one of several disciplines involved in corporate operational risk management and must integrate with the others, especially **physical security management** and **business continuity management**. Statements applicable to all of them are gathered into an integrated over-arching top-level policy rather than repeated under each heading.

| Level | Contents | Architecture layer |
|---|---|---|
| 1 | **Over-arching Business Operational Risk Management Policy** (addressed to all employees enterprise-wide) | Logical |
| 2 | Information Security Policy \| Physical Security Policy \| Business Continuity Policy | Logical |
| 3 | CA and RA Security Policies | Logical |
| 4 | Infrastructure Security Policies \| Business Application Security Policies | Logical |
| 5 | Anti-virus, Remote Access, Network Security, Platform Security, Acceptable Use \| Application 1..5 Security | Logical |
| 6 | Security Rules, Practices and Procedures | **Physical** |
| 7 | Security Standards | **Component** |
| 8 | Implementation Guidelines | **Operational** |

### The directory service's four models
1. **Information model** — defines the types of data and basic units you can store. The basic unit is a **directory entry**, a collection of information about an object. Objects of a type form an **object class** (`person`, `role`, `device`, `site`, `building`, `top`). Classes can be **sub-classes of a super-class** (`building` is a sub-class of `site`); the single root super-class is **`top`**. An entry is a set of **attributes**, each with a type and one or more values.
   *(The book footnotes a caution: "attribute" in the directory sense is quite different from its usage in the SABSA Business Attributes Profile.)*
2. **Naming model** — arranges objects in a hierarchical logical structure producing the **distinguished name**.
3. **Functional model** — the operations you can perform, implemented through protocols. X.500 defines four; **DAP** provides directory access but "has been found to be complex in implementation and in operation" and has been overtaken by **LDAP**, a client-server protocol providing all access and management functions in one. LDAP function groups: **interrogation, update, authentication and control, extended operations** (LDAPv3 only).
4. **Security model** — see below.

### Entity authentication information (ISO/IEC 10181-2 vocabulary)
| Term | Meaning |
|---|---|
| **Authentication information (AI)** | Information used for authentication purposes — passwords, cryptographic keys, random numbers |
| **Exchange AI** | Information exchanged between claimant and verifier during authentication — e.g. a one-time password |
| **Claim AI** | Information used by a claimant to generate exchange AI — e.g. a password or cryptographic key |
| **Verification AI** | Information used by a verifier to verify an identity claimed through exchange AI — e.g. a previously registered password or key |
| **Authority verification AI** | Verification AI used specifically and exclusively by the authority |
| **Authority claim AI** | Claim AI used specifically and exclusively by the authority |

**The four authentication scenarios (Figures 11-6 to 11-9):**
| Scenario | Description | Real-world instance |
|---|---|---|
| **Direct, no trusted third party** (11-6) | Claimant and verifier interact directly | Direct password verification |
| **Indirect in-line** (11-7) | A TTP intercepts the exchange; the verifier trusts the intermediary to have verified the claimant | The initial exchange in an authentication-server architecture; also the user→PC exchange |
| **On-line** (11-8) | The TTP sets up an initial exchange with each party, after which claimant and verifier exchange AI directly | Cryptographic key distribution servers |
| **Off-line** (11-9) | The verifier obtains AI in advance from the TTP, allowing real-time verification later | Public key certificate authorities |

### Logical architecture of security management services (Figure 11-10)
A **security management centre** distributes policy to **security management agents** — a software sub-system embedded in each managed end-system that handles the security management protocol and implements instructions from the centre. The protocol has primitives such as **GET, SET and TRAP**, as found in most systems and network management protocols. All the technical security management services (policy management, operations management, provisioning, administration, monitoring, alarm management, intrusion detection) run on this one architecture. Note: *"The use of the term 'security policy' in this schematic is a loose one. It means all of the policies and standards that drive the configuration and management of the managed security objects."*

### The extended application domain (Figure 11-16)
Each application is a logical domain; **its sub-domains are best mapped onto the roles, each role being a logical sub-domain**. Users live in a separate **people domain** and resources in an **information domain**, with mappings across. The extended application domain therefore comprises: roles (and the functions and information associated with each, and the user-to-role mappings); functions per role; users; **high-privilege users** — administrators, managers, auditors, operators, maintenance staff; user groups; information resources accessed by each role. *(Footnote: application domains and sub-domains tend to be always logical, rarely physical.)*

The **security service management domain** is a special case of this (Figure 11-17): managed security objects and their agents; the security management information database; the security service management functions; and the security management personnel.

## Reference Tables

### Table 11-2: Security Services by Defensive Strategy (the security service catalogue)
The six defensive strategies come from the multi-tiered model of Figure 10-2. *"No list of security services will ever be complete, and you should not take this one to be so. It is a useful guide and is reasonably comprehensive."*

| Defensive strategy | Security services |
|---|---|
| **Prevention** — *Entity Security Services* | Entity unique naming; entity registration; entity public key certification; entity credentials certification; directory service; entity authorisation; entity authentication; user authentication; device authentication |
| **Prevention** — *Communications Security Services* | Session authentication; message origin authentication; message integrity protection; message content confidentiality; non-repudiation; message replay protection; traffic flow confidentiality |
| **Prevention** — *Application and System Security Services* | Entity authorisation; logical access control; audit trails; stored data integrity protection; stored data confidentiality; software integrity protection; software licensing management; system configuration protection; data replication and backup; software replication and backup; trusted time; user interface for security |
| **Prevention** — *Security Management Services* | Security policy management; security training and awareness; security operations management; security provisioning; security monitoring; security measurement and metrics; security administration (privilege management); user support; physical security devices; environmental security services |
| **Containment** | Entity authorisation; stored data confidentiality; software integrity protection; physical security; environmental security; security training and awareness |
| **Detection and Notification** | Message integrity protection; stored data integrity protection; security monitoring; intrusion detection; security alarm management; security training and awareness; security measurement and metrics |
| **Event Collection and Event Tracking** | Audit trails; security operations management; security monitoring; security measurement and metrics |
| **Recovery and Restoration** | Incident response; data replication and backup; software replication and backup; disaster recovery; crisis management |
| **Assurance** | Audit trails; security audit; security monitoring; security measurement and metrics |

Note how several services appear under multiple strategies — a service is classified by the *effect* it delivers, so one service can serve several defensive tiers.

### Selected service definitions worth quoting
| Service | Definition / distinguishing point |
|---|---|
| **Unique naming** | A unique name per entity to prevent confusion over which entity is referenced; syntax depends on entity type; there must be a service that creates and registers names |
| **Registration** | *"Registration is an important security control in its own right, because if an entity can become registered, it can fairly easily obtain privileges"* |
| **Public key certification** | Prevents an unauthorised, unregistered entity becoming a fraudulent participant in the business community |
| **Credentials certification** | The target server trusts the Central Access Manager (CAM) but must be assured what it receives is what the CAM sent — so credentials are wrapped in a certificate digitally signed by the CAM security authority. Known as **authorisation certificates** or **privilege attribute certificates (PACs)** |
| **Session authentication** | Counters the **man-in-the-middle**/session-hijack threat via a mutual authentication exchange; the best versions exchange secret cryptographic keys during the handshake and then authenticate every subsequent exchange. *This does not prevent an opponent making changes — it means changes are immediately detected* |
| **Message integrity protection** | MAC or MIC per message; if the contents include the originator's identifier the same mechanism gives **both** origin and contents authentication; if they include a **sequence number held as a state variable at both ends**, resequencing and deletion are also detectable |
| **Message replay protection** | A **nonce** value built from time stamps, random numbers, sequence numbers or combinations. If time stamps are used the time service must be trusted |
| **Traffic flow confidentiality** | Hides volumetric flow — usually by **dummy messaging** to present a continuous smooth volume of traffic |
| **Software integrity protection** | Anti-virus scanning; change-detection checksums; **quarantine environments for testing newly imported software** before release; controlled acquisition from reputable sources; controlled release/publishing of own software |
| **Software replication and backup** | *"Software should NOT be backed up from the system, since this may incorporate alterations into the backups"* — back up from a **master copy of the latest release** |
| **Trusted time** | If an opponent cannot tamper with the message but *can* tamper with your clock, he may still persuade you to accept an out-of-time message. Trusted time is *"a critical piece of security infrastructure"* |
| **User interface for security** | Easy-to-use strong authentication; single sign-on to all applications; GUI-based login screens and operational messages; easy navigation through hierarchical menus and hypertext. Otherwise the security services fall *"into disrepute"* |

### The three types of access control service
| Domain type controlled | Mechanism type | Examples |
|---|---|---|
| Physical domains (sites, buildings) | Physical | Gates, doors, locks, guards |
| Logical domains (systems, applications, files, records, databases) | Logical | Permissions and software decision-making functions |
| Physical domains (hardware platforms, networks) | **Logical** | Permissions and software decision-making functions |

### Real-time role association depends on
The static role(s) stored in the directory as an entity attribute; **inheritance of role attributes under the directory schema**; **pre-condition rules** stored with the roles object (preventing illegal role combination assignment, preventing illegal security association assignment, controlling transitivity within allowed limits, configuring delegation controls, checking user conditions and constraints); **post-condition rules** (controlling real-time interactions between roles and between roles and other directory objects, defining logical security domains by role group membership, setting real-time limits on transitivity and delegation); and **context-based access control rules** — time of day, day of week, location of client workstation, remote access method (leased line vs dial-up).

### Directory schema components and their definitions
| Component | Definition includes |
|---|---|
| **Attribute type** | Unique name; an **object identifier (OID)**; single- or multi-valued indicator; associated syntax and syntax rules; usage indicator (which applications use it); restrictions on range and size of values |
| **Object class** | A uniquely identifying name; an OID; a set of **mandatory** attributes; a set of **optional** attributes (attributes on neither list may not be used); an object kind (structural, auxiliary or abstract) |

Purpose of a schema: maintain integrity and quality of stored data; **reduce duplication (polyinstantiation)**; impose constraints on size, range and format; provide a documented, predictable access method for directory-enabled applications; and *"help to slow down the effects of directory entropy, in which over a period of time with constant use by many entities, the contents of the directory tend to move towards chaos."*

### Role types and a worked role set
Main role types: **user roles; business manager roles; system manager roles; operations management roles; administrator roles; auditor roles.** For a financial accounting department: **Financial director; Financial controller; Accounting supervisor; Accounting clerk; Financial auditor.**
Design rule: *"aim to minimise the number of roles whilst still maintaining sufficient granularity to achieve segregation of job types and their access privilege profiles."* Roles may be defined as **mutually exclusive to segregate duties** — a user entity may then not hold both.

### Certificates vs tickets
| | Cryptography used | Lifetime | Storage guidance |
|---|---|---|---|
| **Certificate** | Asymmetric | Some are enduring | A digital certificate is usually stored as an **attribute of the directory object** representing the entity to which it was issued |
| **Ticket** | Symmetric | Always transitory — *"up to a few hours generally speaking"* | Define an object class and store **an enduring template** of the transitory structure |

### Indicators of an intrusion incident
Multiple instances of the same user; failed logon attempts; attempted access to unauthorised resources; unusual network loading conditions; components failing integrity tests; unknown source addresses; detection of certain attack signatures by specialised intrusion-monitoring software. Agents are deployed to monitor **both host platforms and network components**.

### The eight logical steps of incident response
Data collection → data normalisation and collation → data analysis → incident assessment and conclusions → presentation → response alternatives → response decision → response action management.
Two decisions govern the flow: **(1) automated or manual?** — decidable by error type or number of errors, controlled by scripts or logic engines (one instance of an error type may be handled automatically, multiple instances may require human action). **(2) Are the error conditions correlated?** — *"In most cases of system failure, it becomes progressive and may require rapid action to contain damage."*
The recommended preparation: develop worst-case scenarios and analyse network and application resources so a **minimum sustainable configuration** is known and can be quickly implemented — including physical domain segmentation, human intervention, firewalls at critical domain boundaries and alternate communications links.

### Physical, personnel and environmental security services (logical definitions)
| Group | Services |
|---|---|
| **Physical security** | Site design and construction including buildings and site perimeters; guarding perimeters of controlled areas; authorisation of personnel for physical access; authentication of personnel at physical access points; visitor handling; contractor handling; maintenance activity handling |
| **Personnel security** | Hiring policies on employment history, criminal records, qualifications; background checks, vetting and reference checking; training and awareness for all personnel; disciplinary processes |
| **Environmental security** | Site selection; fire prevention; fire detection; fire quenching; flood prevention; climate control; electrical power protection; other critical services protection |

## Worked Example

### The logical network domain architecture of a large corporate (Figure 11-13)
This is the chapter's flagship artifact and it is explicitly *logical*: *"When you look at the diagram in Figure 11-13, you will find no servers — only services. You will find no computers, no firewalls, no routers, no VPNs, no equipment boxes of any sort… the lines that join the boxes are not cables — they are logical channel connections that may have any type of physicality, both in media type and topology."*

The requirements, and the domain each one creates:

| Business fact | Logical domain created | Rationale in the book's words |
|---|---|---|
| Many business units, each potentially subject to different sharing/disclosure policy; businesses acquired and sold; some are joint ventures not wholly owned | **One logical domain per business unit** | "So that domains can be added or removed painlessly according to changes in business structure, and so that potential differences in security policy between one business unit and another are easily implemented" |
| Business partners — suppliers, major customers, service providers — given access to certain corporate information and services | **Partner services domain** | "A buffer zone to prevent any direct external access to corporate applications and to ensure that only the desired application functionality and information is externalised" |
| Each partner has its own domain and its own policies | **Extranet domain** | A networking domain allowing parties outside the corporate network domain to connect into services inside it. "An extranet is a logical definition. At this stage no assumptions are made about how that extranet will be physically implemented" — VPN technology, the Internet, direct dial-up, or all of these |
| Home workers, and road warriors on laptops with mobile connections from hotel rooms, business centres, airport lounges — but policy forbids direct external access | **External services domain** | Another buffer zone through which externalised application functionality and information is delivered securely to externally located employees; reached through the extranet |
| Those employees also need office automation and information services | **Intranet services** package | Available both internally and externally to all employees; externally via external services, connected through the extranet |
| Road warriors manage the external partner relationships | Their extranet access **also reaches partner services** | So they can monitor activity with the partners whose relationships they manage |
| Only data centre operations staff may touch production applications hands-on | **Production domain** — "an inner sanctum" | Business users reach it through an inter-domain multi-layered architecture like Figure 10-24 |
| The public at large | **The Internet** → web services | Public web access |

**The lesson the chapter draws:** *"Once you have this logical domain definition it is easy to move forward to a physical design. If you miss out this logical step, believing that you can move straight to the physical design and thus cut a corner to save time, beware! It is actually very difficult to design the physical architecture without this intermediate step, and you will almost certainly design a physical network that does not fully meet the requirements. If you doubt this, try it!"*

**The plan-and-elevation analogy.** Figure 11-13 (network domains) is the *plan view of the building, looking down from above*. Figure 11-14 (the middleware domain model — Central Access Management Service, Directory Service, Certification Services, Registration Services, Data Management Services, Other Common Services, sitting above platform domains and beside the corporate network / extranet / Internet domains) is the *side elevation, looking horizontally from the side*. Same infrastructure, two views. Each service domain can then be exploded into the next level of detail (Figure 11-15 does this for directory services).

### Figure 11-18: Policy interaction between domains, worked with Alice, Bob, Xavier and Yvonne
Domain 1 and Domain 2 are **independent** domains but both are **sub-domains of Domain 3**.
- Alice (A) and Bob (B) are registered members of Domain 1, registered by security authority **SA1**, subject to SA1's policy.
- Xavier (X) and Yvonne (Y) are members of Domain 2, registered by **SA2**, subject to SA2's policy.
- SA1 and SA2 are both registered by **SA3**, the security authority governing Domain 3. SA3's policy applies throughout both sub-domains; the sub-domains add their own supplementary policies, which **cannot by definition conflict** with the super-domain's.

The resulting interaction rules:
| Interaction | Governing policy |
|---|---|
| Alice ↔ Bob | SA1's policy |
| Xavier ↔ Yvonne | SA2's policy |
| **Bob ↔ Xavier** | **Essentially the policy of Domain 3, constrained by any additional requirements placed on them by both SA1 and SA2.** "They cannot behave as they might if only operating within their home domain" |

### Case study: premature release of prisoners (Los Angeles County)
> "The Los Angeles County Board of Supervisors ordered the Sheriff's Department and the district attorney's office Tuesday to investigate two recent incidents in which suspected killers were mistakenly released from county jails." — Timothy Williams, *The Los Angeles Times*, 30 August 1995, p.3

The investigation suggested the computer systems had been interfered with and the data records for the two prisoners altered, causing automatic generation of release papers. **But it was impossible to be certain**: the system had a very poor access control regime, and large numbers of court officials and other administrators had read-and-write access to these records. **There was also no audit trail of the transactions made, and hence no accountability for any information changes.** Someone had obviously tampered with the records, *"but there was no evidence that could prove this had happened, and no evidence that could begin to identify the perpetrator."*
The lesson for the audit trail service: it needs not only mechanisms for capture and storage of event information but also **mechanisms to protect the integrity of that stored information**, since tampering with an audit trail may cover up unauthorised activity.

### The Security Improvements Programme (methodology deliverable)
A series of **short-term projects aimed at achieving quick wins**, planned from a **gap analysis** performed after the logical security architecture has been designed, comparing that design against the **current status assessment generated during development of the conceptual security architecture** (Chapter 10). Purpose: maintain momentum while the longer-term architectural work proceeds, and *"ensure that senior management see a frequent flow of deliverables resulting from the investment in the security architecture development programme."*

## Mental Models
- **Information assets are secondary assets.** They logically represent the true primary assets — the business itself.
- **Knowledge (conceptual) → information (logical) → data (physical).**
- **Policy says what, not how.** That is exactly why requirements are expressed as generic security services.
- **Operational risk management is the top-level policy**, not information security — infosec is one discipline among physical security and business continuity.
- **Plan view and elevation view.** Network domains and middleware domains are two orthogonal views of the same infrastructure.
- **A role is a logical sub-domain of an application domain.**
- **The directory is pivotal.** *"Without directory services almost no other service can remain operable"* — protect its integrity and availability "almost at all costs."
- **Detection, not prevention, is what integrity mechanisms buy you.** Session authentication and MACs do not stop changes; they make changes immediately detectable, so action can be taken.

## Anti-patterns
- Skipping the logical domain model and jumping straight from concept to physical design — *"you will almost certainly design a physical network that does not fully meet the requirements."*
- Putting physical elements (servers, firewalls, routers, VPNs, cables) into a logical architecture diagram.
- Accepting a vendor's claim to provide a service as evidence of a strong mechanism. **The named example: the "authenticator" in SNMPv1 is the plaintext "community name" shared by every device in the community — "Not a strong mechanism!"** (SNMPv3 adds strong cryptographic authentication.)
- Backing up software *from the running system*, thereby baking alterations into the backups.
- Relying on nonce time stamps without a trusted time service, so an opponent who can tamper with your clock can still make you accept an expired message.
- Uncontrolled inheritance and transitivity in the directory, *"in which everything gains access to everything else."*
- Read-and-write access for large numbers of administrators, with no audit trail — the LA County prisoner-release pattern.
- Creating too many roles, driving up role administration cost at target servers.
- Expecting a silver bullet against malicious code. **Fred Cohen proved *"precise determination of a virus by its appearance is undecidable"*** unless it is code you have seen before — one of the undecidable problems in computing like the halting problem. Dorothy Denning: *"The problem of protecting against malicious code would always be a 'learn as we go' business."*
- A security user interface that obstructs legitimate business activity, bringing the security services into disrepute.

## Key Takeaways
- The logical layer is the **functional** view: a complete set of functional requirements, mechanism-free.
- Business information is the logical representation of the real business; information assets are secondary assets.
- Security policy is the logical encapsulation of business requirements, arranged in a **hierarchical policy architecture whose top layer is over-arching operational risk management policy**, not information security policy.
- The security service catalogue (Table 11-2) is organised under the six defensive strategies: prevention, containment, detection and notification, event collection and event tracking, recovery and restoration, assurance.
- Prevention services split into four families: entity, communications, application-and-system, and security-management services.
- The directory service is the pivot — four models (information, naming, functional, security), LDAP over DAP, and a schema of attribute types, syntax rules and object classes.
- Authorisation has three parts: off-line grant of privileges by a registration authority into the directory; off-line local association of privileges to roles at target servers; and real-time on-line authorisation by the CAM plus the target server.
- Four authentication topologies: direct, indirect in-line, on-line, off-line.
- Define network domains, middleware domains, application domains (roles as sub-domains) and security service management domains before designing anything physical.
- The security processing cycle (the When cell) is a loose finite state machine: list your security processes, then for each ask what event initiates it, what event closes it, what intermediate states exist, and what events trigger each transition.
- Run a Security Improvements Programme of quick wins from the logical-vs-current gap analysis to keep senior management engaged.

## Connects To
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — the logical row of the SABSA Matrix this chapter populates.
- [Chapter 5 — A Systems Approach](ch05-a-systems-approach.md) — finite state machine modelling, the rigorous version of the security processing cycle.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — the Business Attributes Profile that drives the security services, and security metrics.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — Figures 7-8 and 7-9, the deliverables of the logical layer.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — multi-tiered security (Figure 10-2), security domains, trust and registration strength.
- [Chapter 12 — Physical Security Architecture](ch12-physical-security-architecture.md) — the mechanisms and the physical version of the Figure 11-13 network.
- [Chapter 14 — Security Policy Management](ch14-security-policy-management.md) — how the policies in the hierarchy are actually written, and ownership vs custody.
- [Chapter 15 — Operational Risk Management](ch15-operational-risk-management.md) — the integration of infosec, physical security and BCM that explains the top policy layer.
- [Chapter 16 — Assurance Management](ch16-assurance-management.md) — the security audit service in detail.
- [Chapter 17 — Security Administration and Operations](ch17-security-administration-and-operations.md) — the procedural side of provisioning, monitoring, administration, BCM and crisis management.
