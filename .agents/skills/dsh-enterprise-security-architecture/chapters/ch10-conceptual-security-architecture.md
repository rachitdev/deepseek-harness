# Chapter 10 — Conceptual Security Architecture

## Core Idea
The conceptual layer is where the architect adds value: *"a vision of the future — a conceptualisation of the types of solution that will satisfy the business needs. If you get this part right, then everything else should flow with ease. If you get this part wrong, nothing at the lower layers of the architecture will ever fix it."* Conceptual thinking means **standing back from the trees to see the forest** — the overall shape, size, mix of species, texture, firebreaks and planting density of the forest, not the individual trees.

The chapter fills the five remaining cells of the conceptual row: Business Attributes Profile (What), Control Objectives (Why), Security Strategies and Architectural Layering (How), Security Entity Model and Trust Framework (Who), Security Domain Model (Where), Security-Related Lifetimes and Deadlines (When).

A warning to software architects: *"this section has very little to do with software architecture… when you examine the various layered models of security architecture, you must resist the temptation to translate them into software designs — they are not software designs."*

## Frameworks Introduced

### Business Attributes Profile (What)
The **complete set of Business Attributes that you believe represents your business**, mapped to business drivers and business risks, each with a measurement approach for producing metrics and specific performance targets. It selects only the attributes that apply to this business (creating new ones where there are gaps), using the Chapter 6 taxonomy as a checklist. **Business analysts decide inclusion; senior executives usually sign off** on both the overall profile and the performance targets. It is the mechanism that binds the Manage and Measure phase back to the Strategy and Concept phase, because it supplies the target that answers "Did we hit the target?"

### Control Objectives (Why)
> "A control objective is a statement of a desired result or purpose to be achieved by implementing controls within a particular business activity."

Controls are implemented through **policies, organisational structures, processes, practices and procedures, and through technical systems**. A control objective can be stated in response to a specific business requirement, or be a generic "good practice" statement applicable to all businesses (the CobiT approach). They **normalise a unique real business into common terminology and abstract concepts** that can drive more detailed design work free of the confusion that unique business detail creates.

Sources of generic good-practice control objectives: ISO/IEC 17799 (Code of Practice for Information Security Management); ISO/IEC 21827 (SSE-CMM); CobiT; the ISF's Standard for Good Practice; the IT Baselines Protection Manual published by Bundesamt für Sicherheit in der Informationstechnik (BSI).

Control objectives go into **column 11 of the Business Risk Model**, with column 12 the target vulnerability after mitigation and column 13 the resulting mitigated risk category — the interface between contextual and conceptual.

### Multi-layered security (Figure 10-1)
The onion-skin idea, redrawn as an **electrical cable**: the conductor at the centre is the information assets; around it layer upon layer of insulation. Closest to the assets are controls acting directly on the information — **cryptographic controls**. Moving outwards the controls become more and more generic, until at the outer layer you have **responsibilities, organisation and policy**.

> "The primary reason for this multi-layered approach is to ensure that there is no single point of failure in the security measures. If one measure fails to stop a security incident, then there are others that do the job in a different way."

### Multi-tiered incident handling (Figure 10-2)
A conceptual set of tiers your mix of security services must cover:
**Prevent** → if that fails, **Contain** the effects → **Detect** the incident and raise the alarm → **React** to recover from its effects and restore business as usual → **Collect evidence** to track events, assist restoration and serve forensic purposes → and over the whole process, **Assure** that it all works correctly.

### The Six As of application security
Application security is fundamentally about **who is allowed to do what within the application, and how much** — that is, authorisation.
| A | Meaning |
|---|---|
| **Authorisation** | The process of granting a privilege |
| **Authentication** | The process of verifying identity |
| **Access control** | The process of making access decisions by checking authorisations and authenticating identity |
| **Audit** | The process of writing, storing and reviewing records of all access attempts, decisions and outcomes |
| **Administration** | Administering privileges and all associated activities |
| **Application-to-application communications security** | Confidentiality, integrity, authenticity and non-repudiation of files, messages and transactions between applications |

Privileges include: application functions the user may use; application data they may read, update and create; limits on application transactions (especially financial); dual control on sensitive transactions requiring a second authoriser; and sometimes **context-based rules** governing user location (head office desk vs hotel dial-up) or time of day / day of week.

**The holy grail** of application security architecture is integrating all applications under a single sub-system for the Six As via an integrated API. Benefits: **single window for user administration** (reduced training, support, higher productivity, lower administration cost); **single authorisation database** (one registration per user instead of many, lower overheads, better control over total user privilege, less risk of dormant accounts, and the ability to block or delete a user's access to all applications in one action for leavers); **single sign-on** for users, giving better password control. The architectural approach that delivers all three is **role-based access control (RBAC)**.

**Network security ≠ application communications security.** The authors reject the claim that application-to-application protection belongs to network security, for two reasons: (1) the network does not understand application data structures — it sees only a payload of unknown structure — and therefore cannot apply the structure-dependent mechanisms that three of the four services need; (2) the network is frequently owned and operated by a third party, and *"you cannot outsource the protection of applications data to someone who knows nothing about the applications and who has no way of controlling them."* **The rule: network security protects the network; application security protects the applications.**

### The generic access control sub-system (Figure 10-7)
Components: the **subject** (a party requesting access — a human user or an external system acting on behalf of a user); the **object** (the resource requested — a data structure such as a file or database record, an application function, a computer system, a peripheral); the **access control enforcement function** (intercepts all subject requests and implements grant/deny decisions); the **access control decision function** (makes the grant/deny decision); the **subject registry** (all registered subjects with their names, authentication data and access privileges); an **access control list (ACL)** per object listing the subjects or groups allowed access; and the **audit logging sub-system** storing comprehensive data on all access requests, successful or not. The decision is a sequence of questions beginning: Is the subject registered? Has the registered subject been successfully authenticated? …

### Stovepipe vs Daisy application architecture (Figures 10-5, 10-6)
- **Stovepipe model** — each legacy application sits in a vertical pipe with its own database, its own registry of users, its own transaction handlers, external interfaces, internal interfaces and business decision support engine. Information flows into and out of the pipe at its end but there is **no cross-flow between applications**. "This model was not planned — it just happened this way as a result of independent, uncoordinated developments." It makes digital business hard, because a web front end must integrate with several legacy back ends.
- **Daisy model** — a common infrastructure shared by all applications: a **central data repository (data warehouse)** at the centre, surrounded by **common services** shared because all applications need them (printing is the given example), the individual applications, and **common external interfaces** such as the web interface — all integrated through common APIs. Common *security* services simply take their place alongside the other common services.

### The security entity model
A **security entity** is something or someone that can take actions in a business environment, requiring control through authorisation processes and through technical and procedural controls that enforce those authorisations. Three types: **individual personal entities (people); corporate entities (organisations or organisational units, whether legally recognised or not); application or system entities (automated processes acting on behalf of personal or corporate entities).** Security entities are equivalent to **subjects** in access control terms.

**Naming:** each entity must have a **globally unique name** so there is never confusion about which entity is referenced; **local alias names** are permitted where the alias is unique within the local domain but not necessarily globally. The **directory** is the repository for entity information — distinguished name, alias names and all other attributes including security attributes.

**Three types of entity relationship:**
- **Unilateral** — one entity broadcasts or publishes information and others may receive it at their choice.
- **Bilateral** — two entities make a specific contract, formal or informal, to transact business and exchange information.
- **Multilateral** — a number of entities participate in a group relationship under an agreed set of rules.

### Trust modelling
**Trust is an attribute of relationships between business entities (people or corporate bodies); trust is not a technical attribute.** (Footnoted exception: you might call a *trusted system* one designed and tested to criteria that let you trust its operation — in that sense only.) Inter-corporate trust is heavily linked to the trust between the individual people representing those corporate bodies.

**The simple one-way trust model (Figure 10-12):** a **claimant** (the trusted party, who makes claims) and a **relying party** (the trusting party, who must rely on those claims). *"After analysis, most trust relationships will boil down to this simple one-way trust model."* (The authors footnote that they were tempted to say *all* but thought better of it, "although they cannot think of any" exceptions.)

**Hierarchical decomposition of trust (Figure 10-13):** a real business relationship is complex two-way trust comprising several individual types of trust, each possibly at a different level of assurance. Decompose top-down — Complex Two-Way Trust → Trust Type X, Y, Z → … — until at the bottom of the analysis tree there are only simple one-way trust relationships.

**Levels of trust vary enormously.** Buying a hot dog from a street stall: you might wonder if it is fit to eat, but you will not wonder whether the stallholder really owns it or will be there tomorrow for complaints. Buying a house: you employ a lawyer at significant cost to investigate ownership, planning regulations and prospects for future undisturbed residence, and the contract is filled to the brim with conditions and get-out clauses. **"Trust is not homogenous across all business relationships."**

**Trust brokers.** Ask "who is the customer for trust?" — the answer is always **the relying party**. The claimant has only a passing interest (he wants to be trusted); it is the relying party who must be convinced the trust is real, so it is the relying party who must contract with, and pay, the trust broker. A trust broker must also **offer liability cover** to add value: "when the wheels start coming off, the trust broker has to be responsible and liable, otherwise what was the value of the trust service?"

**The flaw in the original PKI business model (Figure 10-14).** In the late 1970s and early 1980s, cryptography textbooks assumed the business opportunity in running a certification service was **selling digital certificates**. Rigorous analysis shows this assumption — "that has contaminated thinking ever since" — is deeply flawed: the broker gives the certificate to the *claimant*, but the customer for trust services is the *relying party*, who "got nothing at all." **What a trust broker can actually sell is a trusted transaction**, on a model like conventional credit card transactions. Identrus, a consortium of very large banks, is named as the organisation rolling these concepts out in banking.

**Level of trust is determined by strength of registration (Figure 10-17).** The trust attachable to a transaction depends on trust in the digital certificate used to broker it, which depends directly on **the strength of the registration process** by which that certificate was granted. Birth certificates, passports, driver's licences, banker's references and personal referees are all examples of **transitive trust**.

### The security domain model
> "A **security domain** is a set of security elements subject to a common security policy defined and enforced by a single security policy authority." *(Formal definition from ISO/IEC 10181 — Security Frameworks for Open Systems, 1996.)*

- **Security element** — a security entity (a subject) or a security object (a data structure such as a field, record, file or database; or a physical system or sub-system such as a computer, disk drive or printer). Logical systems such as applications may be subjects or objects depending on context.
- **Security policy** expresses security requirements for a domain in general terms. **Security policy rules** are derived from it during security engineering, interpreting the policy in definite terms that can be incorporated into security mechanisms, which deliver security services that implement the policy.
- **Security policy authority** — responsible for setting and implementing the security policy within the domain. Example: in a domain of registered users, the registration authority is the domain security policy authority.

**Inter-domain relationships:**
| Relationship | Definition |
|---|---|
| **Isolated** | Two domains with no data objects in common and no activities in common; they cannot interact |
| **Independent** | No data objects in common; activities in each constrained only by their own policies; the two policy authorities are not constrained to coordinate or harmonise their policies. Independent domains may enter into **agreements** to coordinate sharing of information |
| **Sub-domain / super-domain** | A is a sub-domain of B (and B a super-domain of A) if and only if: the elements of A are a subset of or the same as those of B; the activities in A are a subset of or the same as those in B; **jurisdiction for A is delegated from B's security authority to A's**; and A's policy does not conflict with B's. A may introduce additional policy, but only if permitted under B's policy |

**Trust in domains.** An entity is **trusted** for some classes of activity, in the context of a security policy, if the means of enforcing the policy depend upon the entity behaving in a particular way. The policy defines which entities are trusted and, for each, the set of activities for which it is trusted — an entity trusted for one set is not necessarily trusted for all. A trusted entity that can misbehave **without the violation being detected** is **unconditionally trusted**; one that can violate policy but not without the misbehaviour being detected is **conditionally trusted**. Trust within a domain is **not necessarily mutual and not necessarily transitive**.

**Secure interaction between domains.** To exchange information, the domain policy authorities must agree **secure interaction rules**, which then become part of each domain's own policy rules. To implement them they must also negotiate a set of **common security services**, **common security mechanisms**, and the **security information items to be exchanged** (for example in a mutual authentication exchange protocol). If both interacting domains are sub-domains of the same super-domain, the super-domain authority may impose the rules or may allow the sub-domains to negotiate their own, depending on its own policy.

**Security association** *(ISO/IEC 10745: Upper Layers Security Model, 1993)* — a set of shared security information items and attributes (such as state information and rules) describing a relationship between two or more entities. It governs the provision of security services involving interaction between them, and implies the existence of secure interaction rules and the maintenance of **consistent security-related state variables for both entities** — passwords, sequence numbers, cryptographic keys and so on.

**Logical vs physical domains.** A **logical domain** maps onto groupings of logical entities — entities sharing an application, a business community, a privileged group. Its boundaries are usually protected by **logical access control**, role-based or otherwise. A **physical domain** maps onto groupings of physical entities — usually a site, a platform or a network. Physical network domain boundaries are usually protected by **firewalls**; physical site boundaries by doors, locks, gates and guards; and **tamper-resistant computing devices** are used where a physically trusted and secure domain is needed to protect sensitive code execution such as handling cryptographic keys.

**Multi-domain environments.** A given environment can support multiple overlaid domains of different types. *Example 1:* a distributed application is a single logical domain but may span several physical network domains including the Internet and several private networks. *Example 2:* a single host computer is a single physical domain but may simultaneously host several applications, each a separate logical domain.

**Guidelines for applying the domain concept:**
- Make sure your specification for a given domain is clear and explicit.
- In particular make the **domain boundary and domain interfaces** clear, so there is no confusion about which elements and activities are included and excluded.
- Identify the **domain security authority** for each domain, and check that it really does set and implement the governing policy.
- Be prepared to consider **sub-domains and super-domains** — real-world situations often reflect these structures.
- Many real environments involve multiple overlaid, overlapping domains; the modelling approach unravels that complexity into a set of well-defined simple domains, by top-down analysis like that used for trust.
- Cover **all domain boundaries and all domain interaction points**.
- **"Do not forget that there is always a surrounding domain called 'the universe' that is a super-domain of all domains that you will define and which should be regarded as completely hostile, with no security policy, no domain authority and containing untrusted security elements."**

## Reference Tables

### Table 10-1: Assurance Levels for Registration Processes
| Registration assurance level | Implications for level of trust |
|---|---|
| **Self-registration** | Very quick click-and-go sign-up. Suitable only for services publicly available within the user's environment (possibly a closed business environment) where the only service-provider interest is collecting general information on how many and which users register. **No attempt is made to ensure the registered user is the authentic owner of the name claimed.** May suit certain web-based services |
| **E-mail registration** | Quick sign-up authenticating the user on possession of an e-mail address with a given domain name embedded, indicating membership of that organisation. **Authentication is weak** and domain-name credentials will not cover all situations, especially staff with other addresses. Suitable for certain low-assurance applications |
| **Web-based credit card registration** | The user self-registers but proves more about identity by supplying a valid credit card number. Still relatively weak — it is easy to supply a fraudulent card number that may or may not exist — but suitable for low-assurance applications with small payments and tolerable fraud, e.g. low-value information services |
| **Telephone confirmation** | Augments any of the previous three. The user is telephoned at a number obtained independently of the initial registration (**"out of band"**) and certain details are verified in conversation. Eliminates certain types of impersonation |
| **Postal registration** | The entity posts an application enclosing documentary evidence of identity and membership of the business community. Strength is determined by the registration authority's policy — copies or originals, how many documents of what type. **In the USA there is additional strength: use of the postal service to defraud constitutes mail fraud, a federal offence, adding risk-mitigation potential** |
| **Personal registration** | The individual attends a registration office in person and presents credentials for verification — birth certificate, passport, membership certificate, driving licence, proof of address. Can be made as rigorous as required, entirely at the registration authority's discretion. **High-value transaction or order-execution systems will require strong registration processes and strong user authentication mechanisms such as smart cards or biometrics** |
| **Transferred registrations** | Existing client databases used as sources of registration data; previous registration details obtained for another purpose are accepted and transferred. **Strength depends entirely on the strength of the original registration and on the level of maintenance keeping the database up to date** |
| **Delegated or multi-tiered registration** | A registration authority registers a sub-registration authority delegated to register users within its own domain — often called **local registration authorities (LRAs)**. There may be several tiers of delegation. **The overall strength depends upon each and every one of the processes at each level in the hierarchy** |

Different levels of trust lead to different **classes of digital certificate**, whose intended purpose and associated trust and liability management are described in the **Certificate Policy (CP)** and **Certificate Practices Statement (CPS)**. The authors anticipate a gold/silver/bronze hierarchy and certificates restricted to certain activity groups within a vertical market sector, while noting the market response is hard to predict.

### The two ends of the registration scale, illustrated
*Low assurance:* go to a public web site, fill in a form with your name and other details ("Micky Mouse, George Washington… whatever"), click and go, and a certificate is automatically e-mailed to you. A slight improvement is checking that the name and the e-mail address are related, though this may not work in all cases.
*High assurance:* apply in writing, like getting your first passport — birth certificate, probably a social security number, written references independently procured from citizens of upstanding position who will testify they have known you for at least 10 years; for certain applications a banker's reference to testify to credit-worthiness and credit history over an extended period; plus independent checks for a criminal record. *"Now that's serious registration!"*

### Security lifetimes and deadlines (the When cell)
| Lifetime / deadline | Guidance from the book |
|---|---|
| **Registration lifetime** | Fixed period, then expiry and renewal, preventing build-up of dormant registrations. **Typically one to three years**; some are much longer (a social security number assigned at birth remains operative for the individual's entire life) |
| **Certification lifetime** | Digital certificates expire and cannot be used afterwards. **Certificate life must never exceed registration life**, else the holder would hold apparently valid credentials after registration expiry. All entities using others' certificates must check the expiry date as part of verification |
| **Cryptographic key lifetime** | Fixed operational lifetime by expiry date, limiting exposure to cryptanalysis. Some keys are also limited by **amount of usage** — expire on the expiry date or the usage threshold, whichever comes sooner. **Digital signature keys need two lifetimes**: one for the period over which signatures can be made, one for the period over which those signatures remain valid and verifiable; the former must expire before the latter, and archiving requirements determine the verification lifetime. Keys may also need archiving for data recovery, requiring a second lifetime as long as the data's expected archive lifetime — for most business data several years and potentially indefinite. Factors: type of usage, algorithm strength, key length, system architecture |
| **Policy lifetime** | Policy decays in effectiveness and appropriateness over time. **Frequent policy changes are a bad idea** — they cause confusion and undermine confidence — but there should be a review cycle, **perhaps every three or four years** |
| **Rule lifetime** | Distinguish policies (logical level) from rules (physical level). Policy stays relatively unchanged; **rules may need to change quite frequently** to keep up with rapid technology change, new attack modes and exploits |
| **Password lifetime** | Enforced expiry at the end of a specified lifetime, set by domain policy. Also a **minimum lifetime** before which a password cannot be expired, **usually one day (24 hours)** — this prevents an uncooperative user resetting new passwords immediately and cycling back to the previous value. More effective still: a **history file** of previously used passwords preventing reuse until several generations have expired |
| **Token lifetime** | Tokens must be individually programmed with user-specific data and distributed securely, with an initial password distributed through **another channel** to prevent token and password being stolen together. Redistribution is expensive, so tokens such as smart cards should be **renewed every two to three years**; they are mechanically capable of this, but any cryptographic keys stored unchanged over the whole lifetime must be strong enough to withstand that exposure. **Token issue should be staggered over the business year** to smooth the operational workload |
| **Message time to live** | Prevents denial of service from a build-up of undeliverable messages. On expiry, the options are: discard without notifying the sender; discard with a return notification (not guaranteed to be delivered); discard and send a **priority notification** giving high assurance the sender learns of non-delivery; or a **positive acknowledgement service** where all delivered messages are acknowledged explicitly or implicitly using **sliding windows**, with a NAK on failure and a time-out window at the sender in case the NAK is lost |
| **Stored data lifetime** | Analyse every application or service that stores data. There may be several phases: **memory-cached → on-line disk-stored → off-line archived**, plus archiving lifetimes for backup copies. Over decades the critical issue is **maintaining the technologies and products needed to retrieve the data** — magnetic tapes are useless if the machines to read them are unsupported — and **physical degradation of media** whose quality decays until unreadable |
| **Data secrecy lifetime** | Determined by the policy authority governing the business area. Example: in many jurisdictions privacy law applies to personal information during the human subjects' lifetime but not once they are dead. Impacts cryptographic algorithm and key-length selection, taking account of likely technology advances over the intended lifetime (the authors cite **differential power analysis** as an example of an entirely new approach to attacking cryptosystems) |
| **User session lifetime** | A maximum lifetime for an open session, limiting sessions kept alive after they are needed — waste of resources and possible abuse. **In the order of a few hours, to match the working-hours patterns of authorised personnel** |
| **System session lifetime** | Even continuous never-stopping processes (such as middleware messaging) benefit from a session concept: a session context of security associations lets fast, efficient **symmetric** cryptography be used, with session keys as key state variables. Keys must be renewed to limit cryptanalytic exposure. **Generally in the order of 24 hours**; for dial-up connections, considerably shorter, bounded by the call duration |
| **Response time-out** | Ensures a requesting process or entity is not left hanging indefinitely when a request or its response is lost. Values are local domain policy and depend on the business environment: *Who is waiting? What is the actual business activity? And how long will they be patient?* Care is needed where a transaction times out at the client but was actually serviced at the server — **the server must monitor requests to ensure an incoming request is not a repeat of a previous request already serviced** |
| **Inactivity time-out** | Handles users who leave a session logged in on the desktop while at a meeting, at lunch, at home or on vacation. Since statistically an incident is more likely to be launched from inside than outside, this is a serious vulnerability. Two actions: **completely close the session**, requiring full login to restart (usual for dial-up), or **suspend the session** holding the context — open files, active applications — with simple re-authentication restoring it (usual for LAN or permanent WAN connections). Thresholds are local policy but **periods of 10 to 15 minutes are frequently used** |
| **Context-based access control** | Uses context parameters — **time of day, day of week, location of terminal, type of network connection (LAN, WAN, dial-up)** — for real-time access decisions. Typically access is granted during normal working hours but not outside them. Context rules can attach to roles centrally or locally; the selection is a policy issue within each domain |
| **Replay protection** | Against an opponent capturing whole messages or exchanges and replaying them later — especially dangerous for authentication exchanges, and it still works even if the data is encrypted, provided the encrypted structure is reused. The countermeasure is a **nonce value** — a field with a one-time value never used again — constructed from: time stamps with a short acceptance lifetime; random numbers in challenge-response protocols; sequence numbers checked off by both parties; or a combination. Using a time stamp as a nonce is attractive because it needs no stored state variable, **but the receiving system's clock must be correct and must not be maliciously manipulable to accept a replay of an expired time stamp — hence the need for a trusted time service.** Crucially: **"nonce values do not prevent replays — they merely allow them to be detected — which implies some recovery action to follow"** |
| **Trusted time** | A common security infrastructure service letting any computing device synchronise its clock to a trusted time source using a trusted communications protocol |

## Worked Example

### The merchant–customer trust model, decomposed (Figures 10-11 to 10-13)
A merchant advertises goods on the web; a customer browses, selects, orders and (in the model's future state) makes an electronic payment by sending a **digitally signed authorisation message** to take money from their bank account. *(Footnoted as at least a generation beyond credit cards in cardholder-not-present mode — implemented perhaps by smart cards holding user private keys and capable of signing business transactions.)*

*"What needs to be protected here? … The answer can be summed up in a single word — TRUST."*

**The buyer must trust the merchant to:**
- offer goods of reasonable quality for the given price;
- actually dispatch the ordered goods once payment has been made;
- not repudiate the receipt of a payment that has been received;
- dispatch the correct number of units ordered for the agreed price;
- dispatch the same quality goods that were described on the web site;
- accept the return of the goods and refund the money if they fail to meet expectation once seen and handled;
- handle after-sales complaints about failure of the goods to live up to pre-sales claims about durability, fitness for purpose and so on.

**The merchant must trust the buyer to:**
- pay for the goods and have enough money in his bank account to cover the price;
- not make vexatious or false claims about the quality of the goods;
- not repudiate receipt of the goods delivered;
- not repudiate the order that was placed;
- not repudiate the payment authorisation.

**The decomposition.** Ask "who is the customer for trust?" per facet. For *quality of goods*, the buyer is the customer for trust — he relies on the merchant's claims. For *payment*, the merchant is the customer for trust — he must rely on the electronic payment turning into real cash. Each facet reduces to one claimant and one relying party. Repeat until the complex two-way relationship is nothing but an array of aggregated simple one-way trust models — which is what makes it tractable, and which tells you exactly who the trust broker's paying customer is in each case.

### The Eggshell → Honeycomb → Egg-and-Honey progression (Figures 10-18 to 10-21)
This is the chapter's signature domain-modelling walkthrough, motivated by the requirement of digital business to *let outsiders into your business computing systems*.

1. **The fortress / eggshell model (Figure 10-18).** Security models date to fortified castles and cities: a strong perimeter wall entirely surrounding the protected area, battlements to fire missiles from, strong gates closed against enemies but openable for authorised people and traffic. In computing: **a hard perimeter but a soft inside** — "once you break through the hard perimeter, everything is easily consumed." Most existing business computing systems are built this way, on the basis of allowing access only to those already inside and preventing any access from outside.
2. **The eggshell with a gateway (Figure 10-19).** Put a gate in the perimeter — a **security gateway** that can be opened for an authenticated, authorised friend and closed against foes.
3. **Why that is insufficient.** The friend/foe classification is **too crude**: "In real business life trust is not a binary quantity. Trust is measured on a continuum." A firewall tends to make a binary decision, so once through the gate the outsider can do pretty much anything an insider can. The binary model does not map onto the real business trust model.
4. **The honeycomb model (Figure 10-20).** Many cells, each with its own perimeter protecting it from intrusion. **Granting access to one cell does not imply granting access to any other.**
5. **The egg and honey combination model (Figure 10-21).** The practical enterprise model: **concentric eggshell-type perimeters, each with a gateway, and honeycomb cells within each concentric zone.** Each zone is a security domain with a **more stringent security policy than the one before**. To get through the outer gateway a party must be:
   - **Registered** — do you know who he is and have you got him on your books?
   - **Authenticated** — can he prove his identity?
   - **Authorised** — have you granted him privileges to enter this domain under the terms of the domain policy?

   To get through subsequent inner gateways the party must be authorised to a **higher level of privilege** and must have a **valid business reason** to reach the honeycomb at that level. Access to individual cells is granted or denied per specific business need — **hierarchical filtering**.

**The mapping that makes the model operational:**
| Model element | Domain type | Realised as | Access control |
|---|---|---|---|
| Eggshell (concentric zones) | Usually **physical** — network sub-domains or whole computer systems on the network | Network security device | **Firewall** (physical access control mechanism) |
| Honeycomb cells | Usually **logical** — applications within a physical computer, logical functions within an application program, logical data sets within an application database | Application security device | **Logical access control software sub-system** — in the application, the DBMS, or the operating system |

The two work together as a layered series of filters, mapping the trust model and authorisation model that represent the real business, achieving the granularity of control needed.

**The four questions to ask of your own business:** What are the domains and how do they relate to one another? How many concentric eggshells do you need? What are the honeycombs and how many cells does each need? What privileges are associated with each domain (each eggshell and each honeycomb cell)?

*"You can clearly see how many firewalls you need and where they should be positioned. You can also see how many levels of logical access control you need and how many roles you will need at each level — the role being equivalent to a single honeycomb cell. You will have represented what seemed at first to be a very complex environment by a model whose simplicity is beautiful."*

### VPN and firewall analysis through the domain lens (Figures 10-22 to 10-25)
**VPN.** Point-to-point encryption within the network layer creating secure pipes; because the application data stream is completely encrypted, an outsider also cannot change transmitted data undetected.

**The domain trap:** *"If you construct a secure pipe between two domains without regulating the flow at each end of the pipe, then all that the pipe does is to connect the two domains to form a single domain."* Figure 10-22 shows three physically remote domains interconnected by encrypted pipes forming **one logical domain**. This is acceptable only if the security policy and ownership of the remote domains are identical; otherwise **the secure pipe becomes a means by which one domain can attack the other**. You therefore need a **regulator (firewall) at each end of the pipe, in each domain, because each domain must be responsible for setting and enforcing its own security policy** (Figure 10-23).

Typically a VPN is built with encryption/decryption embedded in the firewalls that interface to a hostile network, standardly using **IPSec** — a secure version of IP providing encryption and/or authentication at the IP packet level. IPSec can be embedded in any end-system and so may tunnel *through* the firewall right up to an application server if the rules allow — but then **the firewall can no longer monitor packet contents because they are encrypted**, so VPNs are usually terminated at the firewall. For an extranet, the VPN client resides on the external user's PC and the VPN runs all the way to that platform (Figure 10-24).

**Firewall.** *"A firewall is a security gateway that sits on the boundary between two network domains, enforcing the security policy of one of those domains (usually the internal corporate domain), and regulating the flow of network traffic into and out of that domain."* Its jobs: preventing unauthorised traffic flows and detecting unauthorised attempts to penetrate the boundary.

**The limitation that is poorly understood (Figure 10-25).** The Internet is so pervasive that it effectively surrounds your enterprise domain — think of the private corporate domain as **a small island in a large ocean called the Internet**. Once that is clear, it is obvious that **the firewall alone will do little, because it can only regulate traffic directed through it**. If the rest of the boundary is leaky, the firewall "is ineffective and may as well not be there." The example: a member of staff buys a modem, installs it in her PC without authorisation and dials up an ISP — the firewall cannot intervene. So a firewall needs an **invisible, impenetrable barrier** around the rest of the perimeter, and *"this barrier is mostly not a technical thing — it is a combination of policy, procedure, behaviour, awareness, and so on, without which the firewall cannot be effective."*

### Assessing the current state (the quick-wins deliverable)
Figure 7-5 lists an additional deliverable, "Assessment of current security status of the enterprise." This is a **tactical** deliverable providing the backdrop for a programme of quick wins (Chapter 8). Take the contextual Business Risk Model plus the control objectives you added at the conceptual layer and ask: **what is the current state of the enterprise security architecture?** Document it in a form that enables a **gap analysis against the logical security services definition** at the next stage.

## Mental Models
- **See the forest, not the trees** — and be concerned with its shape, species mix, texture, firebreaks and planting density.
- **The insulated cable** — cryptographic controls closest to the asset, responsibilities/organisation/policy at the outer layer; no single point of failure.
- **Prevent → contain → detect → react → collect evidence → assure.**
- **Trust decomposes to claimant and relying party.** The relying party is always the customer for trust.
- **The universe is a hostile super-domain** with no policy, no authority and untrusted elements.
- **A secure pipe with no regulator merges two domains into one.**
- **Your corporate network is an island in the Internet sea.**
- **A role is a honeycomb cell.**

## Anti-patterns
- Translating conceptual layered security models into software designs.
- The stovepipe application model — private database, private user registry, no cross-flow.
- Binary friend/foe trust classification at a single hard perimeter.
- Treating application-to-application protection as a network security responsibility, or outsourcing it to a network provider who cannot see application structures.
- Selling digital certificates to claimants and calling it a trust business.
- A digital certificate lifetime that exceeds its registration lifetime.
- Frequent security policy changes (confusing, undermines confidence) — versus rules, which must change often.
- No minimum password lifetime, letting users cycle straight back to the old password.
- Relying on nonce values to *prevent* replay rather than to detect it.
- Assuming a firewall secures a perimeter that also contains unauthorised modems and other bypasses.

## Key Takeaways
- Conceptual = the big picture, the helicopter view, the strategic plan; get it wrong and no lower layer can fix it.
- Business Attributes Profile (What) normalises the business; control objectives (Why) normalise the mitigation strategy and fill columns 11–13 of the Business Risk Model.
- Layer security so there is no single point of failure; cover all six incident-handling tiers.
- Six As of application security: Authorisation, Authentication, Access control, Audit, Administration, Application-to-application communications security — integrated via RBAC.
- Security entities are people, corporate entities and application/system entities; relationships are unilateral, bilateral or multilateral.
- All trust decomposes into simple one-way claimant/relying-party relationships; the relying party pays; trust brokers sell trusted transactions with liability, not certificates.
- Level of trust follows strength of registration, from click-and-go self-registration to in-person credential checking, with eight named assurance levels.
- Security domain = elements under a common policy set by one policy authority; relationships are isolated, independent, or sub/super-domain; interaction requires agreed secure interaction rules and security associations.
- Egg and honey: concentric physical domains with firewall gateways plus logical honeycomb cells with software access control.
- The full catalogue of security lifetimes and deadlines is the conceptual When cell.

## Connects To
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — the Business Attributes taxonomy that the Profile draws on.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — Figure 7-5, the conceptual development process and its deliverables.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — the Business Risk Model whose columns 11–13 this chapter completes.
- [Chapter 11 — Logical Security Architecture](ch11-logical-security-architecture.md) — the full security services catalogue classified under the multi-tiered model, and logical domain definitions.
- [Chapter 12 — Physical Security Architecture](ch12-physical-security-architecture.md) — the mechanisms realising these concepts.
- [Chapter 14 — Security Policy Management](ch14-security-policy-management.md) — the policy hierarchy that domain policy authorities produce.
- [Chapter 8 — Managing the Security Architecture Programme](ch08-managing-the-security-architecture-programme.md) — the quick-wins programme the current-state assessment feeds.
