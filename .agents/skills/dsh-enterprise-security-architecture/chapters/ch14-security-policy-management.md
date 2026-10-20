# Chapter 14 — Security Policy Management

## Core Idea
> "Security policy is the logical embodiment of the enterprise business requirements for security and control."

The *deliverable* (Security Policies) sits in the SABSA Matrix at the Logical layer, Motivation column. But the **process of policy making and policy management is an operational activity** (Chapter 3, Table 3-2, Operational Security Architecture at the Logical Layer) — which is why this chapter sits in Part 4, Operations.

The chapter's central and most quoted argument is **cultural, not documentary**:

> "On one level a security policy appears to be words on a piece of paper — a purely physical document that lives in the filing cabinet in your office or in electronic format on your Intranet. However, **if your security policy is simply pieces of paper or screens with words on them, it has no value to the organisation, and you may as well discard it.**
>
> "Security policy should be a living, breathing thing. It represents a culture that exists in the organisation. It describes the way in which people behave when doing their work. It is a mindset that has been accepted and bought into at all levels of the organisation."

And therefore: *"Security policy is not something that you can lift from a textbook, and so this book does not include sample security policies for you to copy… Your security policy should be unique to your business."*

## Frameworks Introduced

### The two questions that determine a policy's structure and content
1. **What is the purpose of this security policy?** — *"If your only purpose is to get a tick in the box when the auditor comes around, then it doesn't really matter what approach you take — you just want some shelf-ware. However, it is unlikely that collectors of shelf-ware will be readers of this book."*
2. **Whose behaviour are you trying to influence?** — *"If when you try to define this community it turns out to be too difficult, then you have chosen the wrong scope for your policy. Go right back to the beginning and ask again."*

The method: list the identifiable groups you want to influence in mindset and behaviour; for each group ask what sort of policy is needed; accept that some people are in more than one group and need more than one policy. Then, applying the Chapter 8 **Rules for Influencing Opinion or Behaviour**: *"confine your content to only that which you think is relevant to influencing the target audience. Do not include irrelevant material 'just because it's there'. Keep your message concise and to the point."*

### Figure 14-1: The hierarchical policy architecture, and its SABSA alignment
| Level | Documents | SABSA layer |
|---|---|---|
| 1 | **Over-arching Business Operational Risk Management Policy** — addressed to all employees enterprise-wide | Logical |
| 2 | Information Security Policy \| Physical Security Policy \| Business Continuity Policy (*"you probably also need other operational risk management policies such as Health and Safety Policy"*) | Logical |
| 3 | **CA and RA Security Policies** | Logical |
| 4 | Infrastructure Security Policies \| "Line of Business" Application Security Policies | Logical |
| 5a | Anti-virus, Remote Access, Network Security, Platform Security, Acceptable Use \| Application 1–5 Security | Logical |
| 5b | **Security Rules, Practices and Procedures** | **Physical** |
| 6 | **Security Standards** (external — international, national, industry sector — *and* internal) | **Component** |
| 7 | **Implementation Guidelines** — *"likely to be sparsely populated because these guidelines are needed only in certain circumstances"* | **Operational** (at the Component layer of Table 3-2) |

**Why the top level is operational risk management and not information security:** *"There are policy statements that are applicable to all these related disciplines, and it is better not to repeat these same statements under different policy headings but to assemble them together into an integrated, overarching top-level policy."*

**How to split level 4:** *"The decision as to how to split up these policies will be driven by the granularity and variability of business risk across these sub-domains."*

### The corporate security policy — what the CEO is actually saying
ISO/IEC 17799's objective for a corporate information security policy: *"To provide management direction and support for information security. Top management should set a clear direction and demonstrate their support for and commitment to information security through the issue of an information security policy across the organisation."*

The book renders the same thing in the CEO's own voice:
> "Listen up everyone and pay attention! If you want to work here, this is the sort of outfit that we are. This is the way we behave. This is how we do business. If you really want to be one of us, do it this way. If not, you'd probably better move on."

**Eleven points a corporate-level policy should address:**
- It should come from the most senior level of management — **preferably personally signed and issued by the CEO**.
- To get that sponsorship, **invest many months in developing it**, mostly in **wide consultation across the organisation** to ensure it represents a consensus view and not the personal opinions of an individual.
- It should refer to **business risk management as the primary driver** and state senior management's expectations for limiting risk exposure; key business risks might be specifically mentioned.
- The need for security and risk management should be **related to the overall business goals**.
- It should provide a **strong mandate**, instructing management at all levels and employees at large to behave in certain ways and holding them accountable.
- It should **formally delegate responsibility** — "divisional directors are responsible for…", "line managers are responsible for…", "all staff are responsible for…"
- It might mention **ownership and custody**.
- It might specifically mention **compliance with laws, regulations and contracts**.
- It should refer to a **reporting process for suspected security incidents**.
- It should mention **education and training and the provision of a centre of expertise**.
- It should **refer to other documents** where more detail can be found.

### The nine policy principles
Offered *"not as something to which you must slavishly adhere. Indeed some of these principles may be counter to your intentions in some circumstances. Look on them merely as a resource from which you can draw inspiration."*

| Principle | Statement |
|---|---|
| **Least privilege** | Users and system processes should be given the least authority and minimum access to resources required to accomplish a given task |
| **Accountability** | All significant system and process events should be traceable to the initiator |
| **Minimum dependence on secrecy** | Controls should still be effective even if an opponent knows of their existence and knows their mode of operation |
| **Control automation** | Wherever possible, automatic controls should be used rather than controls that depend on human vigilance and human behaviour |
| **Resiliency** | Systems should be designed and managed so that in the event of breakdown or compromise the least possible damage and inconvenience is caused |
| **Defence in depth** | Controls should be layered such that if one layer fails, there is another *different type* of control at the next layer that will prevent a breach |
| **Approved exception** | Policy exceptions should always have management approval |
| **Secure emergency override** | Controls must only be bypassed in predetermined and secure ways. *"Systems are at their most vulnerable when normal controls are removed for emergency maintenance"* |
| **Auditability** | It must be possible for an independent expert to verify that the system conforms to the security policy. *"A necessary, but not totally sufficient condition for this is that the system must be able to record security-related events in a tamper-resistant audit log"* |

### The least-privilege challenge — the book's most contrarian argument
Least privilege *"works very well"* at the level of platform security. **But at the application level — dealing with true business information — the authors have a serious concern that it is "counter-productive to the goals of an organisation leveraging the power of the knowledge economy."**

The reasoning uses the Table 11-1 distinction: an organisation must be able to use existing business *information* to create new business *knowledge* — *"the whole basis of data mining is developed from this idea. Thus, for a user to be able to create new business knowledge, access to all business information may be required."*

**The reconciliation offered:** treat those whose job function is **knowledge engineer** (or similar) as requiring access to all business information all of the time — *"just a special case of the least-privilege principle."* Either way: *"ensure that your approach to information security policy does not inhibit genuine business activity related to knowledge."*

### Information classification — and why the book largely rejects it commercially
Classification is *"a concept developed over many centuries in military and government organisations,"* traditionally focused on **secrecy**: top secret, secret, confidential, restricted, with everything else unclassified by default. Objects are classified and **protectively marked**; subjects are **security-cleared** to a level. Matching subject clearance to object classification is the heart of **multi-level secure systems (MLS)**.

| Model | Domain | Rules |
|---|---|---|
| **Bell-LaPadula** (Bell & LaPadula, 1975, ESD-TR-75-306, Mitre) | Confidentiality | **"No read from a higher level" and "no write to a lower level"** |
| **Biba** (Biba, 1975, MTR-3153, Mitre) | Integrity — the issue is *"not unauthorised disclosure of information but unauthorised error-contamination of high-integrity information"* | **"No write to a higher level" and "no read from a lower level"** — the opposite of Bell-LaPadula |

Integrity classes given as an example: 'sensitive for personal health and safety'; 'sensitive for business mission'; 'sensitive for business function'; everything else 'non-sensitive'. Availability classes: highly critical, critical, non-critical by default.

**The combinatorial explosion.** *"Now you potentially have a three-dimensional matrix containing 60 different cells (5 for secrecy × 4 for integrity × 3 for criticality). That implies 60 different ways of handling information."*

> "It is at this point that you start to ask the question: 'Why are we doing this?' If the answer is: 'Because we can', then it is time to throw the whole lot in the trash can. If the answer is: 'Because it fulfils a useful business purpose', then you might keep it. However, in the experience of the authors, application of these classification schemes outside of their original military and government area of application usually falls into the 'Because we can' category and rarely (never?) do they fulfil a useful business purpose."

**Operational problems even if justified:** each category needs rules determining how something gets classified at that level; every creator of documents must know and rigorously apply them; all documents need protective marking; recipients must know the handling procedures; someone cleared high may have to manage several levels simultaneously. *"This is all a lot of hard work. It has an adverse effect on efficiency and upon the flow of information in the business, and it has a high cost."*

**The drift problem:** *"Because it is unusual for items to be downgraded in their classification, information tends to drift upwards in the hierarchy, usually attaining a higher classification than it really needs. This over-restrictive regime tends to inhibit real business activity."*

And a pointed observation: *"ISO/IEC 17799 mentions information classification as an approach but does not enter into any of the discussion you have seen above about the value or difficulty of classifying objects… one suspects that the advice falls into the category of 'because you can', since no real business value proposition is offered."*

### System classification — the book's preferred alternative
Classify **systems or applications** by the level of business risk (high / medium / low) revealed through a risk assessment, then set a security policy for each level and associate each system with a **control regime** — a standard set of controls that must be applied.

**Add granularity by attribute:** state the risk level for each of a small number of **Business Attributes** (*confidential*, *integrity-protected*, *available*), because the standard control regime for each differs. *"Thus a system may be classified as high-risk for available, medium-risk for integrity-protected and low-risk for confidential."*

**Five benefits:**
- A consistent, standardised approach to enforcing policy across an organisation with potentially hundreds or thousands of application systems.
- **Only a limited number of security policies** (one per risk-and-attribute category) are maintained.
- The level of security on each system is matched to its perceived business risk.
- **It is completely transparent to the user community** — no user has to decide how to handle a specific document, because system managers apply the control regime at the system level, not the user level.
- **It makes security auditing much simpler** by removing subjective judgement and giving the audit team objective criteria.

**The four audit questions it enables:**
1. Has the business risk assessment for the application system been carried out properly?
2. Have the business risk categories for the application system been correctly stated?
3. Does the control regime for the application system comply with the security policy and the standard controls associated with the risk categories allocated to it?
4. If gaps are found between the control standards and the actual system, is there a good reason? Are there other mitigating circumstances? What remedial actions are required?

*(Footnote worth keeping: "The security policy document should point to procedures, practices and standards documents specified in detail at lower levels of the policy architecture. The policy itself does not contain details of individual controls.")*

### CP vs CPS — the orthogonal relationship
| | **Certificate Policy (CP)** | **Certificate Practices Statement (CPS)** |
|---|---|---|
| PKIX (RFC 2527) definition | *"A named set of rules that indicates the applicability of a certificate to a particular community and/or class of application with common security requirements"* | *"A statement of the practices which a certification authority employs in issuing certificates"* |
| Aligned with | **A certificate type**; may apply across multiple CAs | **An individual CA**; may apply across multiple certificate types, each with its own CP |
| Purpose | **Inter-operability** — the basis of inter-operable services between multiple CAs for a specific certificate type in a specific application | **Trust** — informs subscribers and relying parties about operational practices; promotes the trustworthiness of the CA and its service offering |
| Depth | Less detailed | *"A detailed, comprehensive document that describes not only what is done but also how it is executed… almost certainly unique and proprietary"* |

**The CP alone is insufficient.** The CA's critical message when issuing a certificate is that the relying party can be assured **the public key embedded in the certificate is bound to the entity named as the certificate subject (the subscriber)**. But the level of that assurance depends on the whole set of processes and controls applied in registering the subject and issuing the certificate — so *"the CPS is a much more informative document in this respect."*

**Legal binding.** The CPS is usually itself legally binding on the CA, or referred to in support of another binding contract, regulation or statute. *"For the CPS to be legally binding on the relying party… there needs to be a clear legally binding contract between the relying party and the CA, which in many circumstances may be difficult to establish. If there is to be any chance of the relying party being bound in this way, it is essential for the certificate itself to embed a direct reference to both the CP and the CPS."*
**How to publish and reference them:** give each an **ISO standard object identifier (OID)** for a globally unique identifier, and use **X.509 version 3 extension fields** to hold reference pointers — to the OIDs or to the URLs where the documents live. *"In future it is also likely that XML-based policy documents will be used as a means to make a policy available online in a way that can be used to drive automated implementation of the policy and practices."*

**Seven issues CA and RA security policies must cover:** processes used to register entities and issue certificates; security management for the CA and RA ICT systems; validity periods for an entity **encryption** public key certificate; validity periods for an entity **signing** key; validity periods for an entity **verification** public key certificate (**which must be greater than or equal to the lifetime of the entity-signing key**); validity periods for lower-level CA signing (certification) keys; lifetimes for **certificate revocation lists (CRLs)** after which they must be discarded and new versions obtained.

## Reference Tables

### The four-document worked example — Data Backup
| Document type | SABSA layer | Content |
|---|---|---|
| **Policy Statement** | **Logical** | "All application systems must have a regular weekly full data backup, with a daily incremental backup on other days." |
| **Procedure** | **Physical** | "This is how you backup Application ABC hosted on Platform PQR: Step 1… Step 2… Etc." *(Note: "the procedure itself is a security mechanism at the physical security architecture layer, but executing the procedure is an operational activity")* |
| **Internal Standard** | **Component** | "Backup tapes must be of minimum quality 'X' in accordance with ISO YYYYY and must be retired after 'Z' uses. Labelling and indexing standards are… etc." |
| **Implementation Guideline** | **Operational** (at the Component layer) | "There are two software packages you can use to perform backups on Platform PQR, but Tool A is recommended over Tool B because you will find it easier to use. When you install it, the following configuration parameters are recommended… etc." |

### Figure 14-2: The Application Systems Security Policy Matrix
Nine policies covering any number of applications.
| | Low Risk | Medium Risk | High Risk |
|---|---|---|---|
| **Confidential** | Policy 1 | Policy 2 | Policy 3 |
| **Integrity-protected** | Policy 4 | Policy 5 | Policy 6 |
| **Available** | Policy 7 | Policy 8 | Policy 9 |

### Figure 14-3: The Line-of-Business Security Policy Matrix
An alternative that pairs risk level with line of business.
| | Low Risk | Medium Risk | High Risk |
|---|---|---|---|
| **Line of Business A** | Policy 1 | Policy 2 | Policy 3 |
| **Line of Business B** | Policy 4 | Policy 5 | Policy 6 |
| **Line of Business C** | Policy 7 | Policy 8 | Policy 9 |
| **Line of Business D** | Policy 10 | Policy 11 | Policy 12 |
| **Line of Business E** | Policy 13 | Policy 14 | Policy 15 |

*"You may then be able to make further efficiency gains by combining certain lines of business together because the differences in their security policy requirements are minor."*

**The escape hatch if this is too coarse:** *"you can of course have an individual risk assessment for each application, based upon the entire Business Attribute Profile for the line of business supported by that application. This will be less efficient in terms of effort but will provide a much more finely tuned security policy."*

### What each type of policy must address
| Policy type | Issues to address |
|---|---|
| **Application system security policy** | The **Six As** of application security (Chapter 10): Authorisation (granting a privilege); Authentication (verifying identity); Access control (making access decisions by checking authorisations and authenticating identity); Audit (writing, storing and reviewing records of all access attempts, decisions and outcomes); Administration (administering privileges and all associated activities); Application-to-application communications security |
| **Platform security policy** | Four strategic principles: reduce vulnerabilities in the information processing platforms and infrastructure; **segregate and isolate production platforms and environments from those used for development and testing**; provide and maintain highly trusted execution environments for highly sensitive data processing; provide secure storage environments for highly sensitive non-volatile stored data |
| **Network security policy** | One policy governing the entire enterprise network domain, plus sub-domain policies as required. **VPNs need a policy — perhaps one for all VPNs. Firewalls need a policy that gets converted into rules at the physical layer** for configuration |
| **General infrastructure policies** | Anti-virus and other malware policy; remote access policy; acceptable use policy; user authentication policy |
| **Specific infrastructure policies** | Middleware security policy; data management security policy; security management services security policy; directory service security policy |

**The platform-policy question the book poses without prescribing an answer:** *"Should the security policy for the platform be independent of the risk level and security policy for the application being hosted? If application risk level is to be taken into account, in the case of multiple applications being hosted on the same platform, will the platform security policy be **'system high'** (that is, appropriate to the highest-risk application)?"* And the governing trade-off: *"The more granularity you introduce, the more complex and the less efficient becomes the process of managing the security policies."*
Practical grouping advice: categorise platforms into reasonably homogenous types (*"such as servers running Solaris 3.x or laptops running Windows 2000 Pro"*) and write a policy plus standards per type.

### Figure 14-4: The governance structure for information security management
| Body | Responsible for |
|---|---|
| **Company Board** | Overall corporate governance; setting goals and expectations for risk management |
| **Executive Management Team** | Ratifying policies approved by the Steering Group; approving major budgets for information security initiatives and programmes |
| **Information Security Management Steering Group** | Reviewing and approving corporate security policy and subordinate policies as developed and proposed by the CISO after wide consultation; approving and supporting major initiatives to develop the enterprise information security programme, culture and architecture; developing and submitting major budgets to the Executive Team; **monitoring major information security-related threats to the enterprise business**; approving specific methodologies and processes; promoting the visibility of business support for information security activities; monitoring and reviewing significant security incidents |
| **Information Security Liaison Group** | Providing a channel to **listen** to views, ideas and inputs from all those with operational security responsibilities; providing a channel to **disseminate** new policies, processes and methodologies; finding resolution for operational problems or escalating them |
| **Line management** | **Owning the corporate information assets on behalf of the enterprise**; playing the role of owner, custodian or user as the department's function requires; as owners, authorising access by users, and as custodians, enforcing the authorisations created by owners; ensuring compliance with policies, practices, procedures and standards throughout the span of control; developing awareness and a strong culture throughout the span of control |

### The people: who does what, and one appointment to avoid
| Role | The book's guidance |
|---|---|
| **The director in charge** | ISO/IEC 17799: *"one member of the management team should take lead responsibility for co-ordinating information security policy."* **Almost certainly a part-time role** within an existing portfolio. Should be *"someone who is genuinely the flag-bearer for the information security cause… someone who will fight for the cause at meetings and who will proactively promote information security cultural values at every opportunity"* |
| **The one director to avoid** | **The IT director.** *"If this responsibility is placed with IT then it sends the wrong message. It is essential that information security be understood by everyone to be a business issue and not an ICT issue. The more involvement there is from the IT department, the less people will understand the true business emphasis"* |
| **The steering group** | Meets **three or four times a year**, chaired by the director in charge, with wide business representation and an agenda prepared in advance. Its terms of reference and power are critical: *"If it is not vested with any real power, then the senior people who are supposed to attend will always be busy doing something else and will always delegate someone else… You will then finish up with a junior committee that is a talking shop."* **The recommended fix: "give the steering group the power to award financial budget. Nothing gets attention more quickly than serious money"** |
| **The CISO** | A **full-time job**. Two rules: **(1) don't call it "IT Security Manager"** — *"this immediately sends the message that this is an IT issue (a technical issue) and that business people need not concern themselves with it."* **(2) The post-holder is NOT responsible for the security of information in the business** — they are *"a centre of expertise to help and advise others, to develop methods and processes that will support others… and to provide leadership in shaping the information security agenda."* Responsibility for securing business information *"falls to everyone in the enterprise, starting at the chairman and chief executive officer and cascading downwards through the line management"* |
| **The liaison group** | Informal, probably led and chaired by the CISO. *"The meetings are an opportunity to listen to their problems, their moans and their successes."* **It is not a policy-making or decision-making group**, although it may feed suggestions upward |
| **Other roles to fill** | Security administration (setting up and managing user access privileges); user support (a help desk for security-related problems); user and management awareness development; specific education, training and skills development; security audit, possibly within internal audit; system security administration (configuring platforms and applications to comply); **departmental security champions** — people within business units acting informally as a focal point and communications channel; and contracts with third parties, ensuring security is addressed in commercial contracts and SLAs |

*(Terminology footnote worth preserving: "There can be confusion between different cultures… We (the authors) are UK folk, and to us the term 'director' is definitely senior management, whereas in the USA it has less weight. There you would expect a vice-president or senior vice-president to be taking this lead role.")*

### Security culture development
- **A corporate policy statement signed off by the CEO** is the first key component — *"the lynchpin of the information security stance of the organisation."*
- **Awareness decays.** *"Even if you raise awareness to a high level at a given point in time, if you neglect it after that, the awareness decays and dies. People forget. People leave and new people arrive. So you need a campaign that will constantly renew the awareness level by a continual series of reminders that never stops."*
- **The security brand.** Borrow the retailer's technique: build something easy to identify and easily remembered. *"Think about the 'Intel Inside' brand on computers. You know what it looks like, and you can hear the little jingle in your head."* Then repeat it **on login screens, on notice boards, on internal newsletters, on documents, on intranet pages, on training materials, on presentations, on e-mail signatures, on letterheads, on mouse mats, on coffee mugs, on desk tidies, on calendars — everywhere!** *"Because the message is totally consistent it builds up a powerful and memorable impact. If on the other hand you have a different message each time, with different colours and images, nothing hangs together, and it all gets lost in the noise."*
- **Management example.** Managers at all levels must abide by the policy at all times and point out **compliant behaviour as well as non-compliant behaviour**, *"never letting the issue of security slip, especially when things have gone wrong."*
- **A no-blame culture.** *"A culture of blame will only encourage people to cover up mistakes."* Better: everyone is encouraged to admit mistakes, **own them on a team basis even though an individual may have made the mistake**, learn lessons, and pursue continuous improvement — which builds team values and keeps morale high. **But** blame and retribution are appropriate where an individual, *"either with deliberate malice or with reckless negligence, has caused a major loss event which should have been prevented"* — so a disciplinary process is essential within HR management.
- **Four types of education and training:** brief **induction** training for all new employees; **specific technical training** for anyone whose job includes a technical activity relevant to the enterprise security stance (IT development and IT operations); **professional career development** for those with specific responsibilities — an MSc in Information Security, or preparing for CISSP examinations; and **short courses** on new issues as they arise — new threats, solutions, products, technologies.

### Two traps in using incident counts as a metric
1. **A successful awareness campaign makes reported incidents rise.** *"Awareness rises, people are more likely to notice security incidents that they previously ignored, and they are more likely to report them in their newly aware state — so the number of reported security incidents shows a sharp rise! You must anticipate this and manage expectations accordingly."*
2. **Targets tied to bonuses suppress reporting.** *"If you have a target for numbers of reported incidents that is linked into any kind of bonus or reward scheme… then you encourage non-reporting. There are numerous examples of 'league tables', especially imposed by governments… in which it seems that those at the top of the table are more skilful at manipulating the reported events than those at the bottom."*

### Outsourcing — the ten principles
The three outsourcing issues: the security of outsourced IT services; the security of business information handled by a provider to which **non-IT business process operations** are outsourced; and the outsourcing of **operational security management services** themselves.

- **"You should NEVER outsource any type of security policy making activity. Setting policy is a business issue that you MUST keep in house."**
- For policies already made in house, **their implementation is well suited to being outsourced**, provided you maintain and manage the relationship.
- The contract **must explicitly address security management and how it is to be executed**. *"Far too many outsourcing deals are put together without due regard for this aspect of the relationship, and hence in the post-contract phase when there are problems, there is no adequate remedy."*
- The contract should refer to a **specific security management process** — points of contact on each side, and a protocol for raising issues, handling security incidents, escalating persistent problems and managing the relationship.
- There should be a **security management organisational structure on both sides** with clear responsibilities per post-holder.
- There should be a **security target document** agreed between the parties, forming an integral part of the **SLA**, specifying *"what the parties agree to be the meaning of 'secure' and 'secure operations'"* and setting metrics by which the provider's performance can be judged inside or outside an acceptable level. **The Business Attributes Profile (Chapter 6) should help with this.**
- **Responsibilities and liabilities of each party must be spelled out in detail** so there are no hidden surprises post-contract.
- **Retained in-house by the business folks:** assessing business risk, specifying business security requirements, granting authorisations, setting policy. *"Do not attempt to outsource any of these activities, although you may seek help from third-party consultants."*
- **Outsourceable:** implementing decisions and policies handed over by the business people — *"all of the operational and administrative work."*
- **"The best way to ensure that these principles are all observed is to make security management a key topic for discussion during the pre-contract phase. Get all the issues out on the table, explore them fully and resolve the differences BEFORE you sign the contract."**

### The owner / custodian / user model (Figure 14-5)
| Role | Responsibility |
|---|---|
| **Service owner** | Authorises users of the service and grants a **privilege profile** allowing access to certain functionality |
| **Data owner** | Authorises **services** (not users) with privilege profiles granting database access on the basis of what a particular service needs to carry out its functions |
| **Authorisers / stewards** | The people on an owner's team who actually do the work, because *"service owners are usually senior managers who do not themselves sit at a terminal entering authorisations into the computer system"* |
| **Custodian** (the service manager) | Manages the service — or looks after the database — on behalf of the owner, **enforcing all rules, policies and authorisations set by the owner** |
| **Users** | Consume the service under the privilege profile granted |

*"This model works for an internally provided service management function, but it works equally well for an outsourced service provider relationship. In fact once you adopt this conceptual model for how the relationships work, understanding the respective roles, responsibilities, and liabilities in an outsourcing contract is made very easy."*
*(Footnote: "Although these notions of ownership and custody are described as useful for managing an outsourced service, they are equally useful for internal relationships.")*

### Figure 14-6: Policy setting and compliance monitoring for outsourced services
The model assumes **both parties have an internal audit department and an internal corporate security team** — *"if these teams do not formally exist on one or both sides of the relationship, then these functions will still need to be fulfilled by some other functional unit."*
| Party | Does |
|---|---|
| **Corporate security team (both sides)** | Creates and supports security standards and methods, such as a risk assessment method |
| **Customer organisation** | Carries out a **business risk assessment**, uses it to set policy and to write the **security target document** expressing its requirements for security |
| **Service provider organisation** | Carries out a **technical risk assessment** to ensure it can continue to meet the requirements in the security target document |
| **Internal audit (both sides)** | Checks the correct operation of these internal processes; the two audit groups **liaise and share information** to keep the relationship harmonious |
| **The formal contract points of contact** | Carry the main interactions between the two parties |

## Worked Example

### The 9/11 Commission Report as a case against least privilege
> "One of the key findings of the 9/11 Commission was that the **need-to-know principle — the principle of least privilege — was central to the failure of the various intelligence agencies to piece together knowledge from the vast amounts of information potentially at their disposal.**"

The recommendation quoted, from the report's Executive Summary under *Unity of Effort: Sharing Information*:
> "The US Government has access to a vast amount of information. But it has a weak system for processing and using what it has. **The system of 'need to know' should be replaced by a system of 'need to share'.**"

The book's comment: *"Could there be a more powerful example?"*

### Separate ownership of service and data — the IBFS Retail Banking Company
The chapter's worked ownership scenario, deliberately constructed so that service and data have **different owners**:

- **The customer database.** IBFS Retail Banking has **10 million retail customers**, all registered in a single central customer database with full details — names, addresses, account details, credit history. **The Senior Vice-President, Global Customer Services, owns the customer database and the data in it.**
- **The card service.** Many retail customers are issued plastic cards, debit and credit. The **Card Services Division** deals with everything to do with plastic cards; its **Vice-President owns the service** that handles customer requests for cards, authorises the requests and issues the cards, and also deals with lost cards, card fraud incidents, blacklisting and hot-listing.
- **The split.** *"The VP, Card Services owns the card services applications and the services that they deliver to customers. The SVP, Global Customer Services owns the customer data that these applications use."*
- **The custodian.** Both the running of the main customer database and its immediate applications, **and** the physical creation and distribution of plastic cards **have been outsourced to a service provider called Intergalactic Data Services**, which *"manages all the operational services and data management on behalf of both bank owners."*

**How the authorisation flows in this scenario:**
- The **VP Card Services** (service owner) authorises *users* of the card service and grants their privilege profiles; the people on the team who actually enter them are the **authorisers/stewards**; the profile is handed to the **service manager at Intergalactic Data Services**, the **custodian**, who enforces it.
- The **SVP Global Customer Services** (data owner) authorises *services* — granting the card services applications access to the customer database on the basis of what those applications need to do their job; again the service manager at Intergalactic is the **custodian** enforcing the owner's rules and policies.
- **Neither owner outsources the decision.** Intergalactic implements; IBFS decides.

This is precisely the model the outsourcing principles rest on: policy making and authorisation stay with the owner inside the business; implementation and enforcement move to the custodian, internal or external.

## Mental Models
- **Policy is culture, not paper.** If it lives only in the filing cabinet, discard it.
- **Start from the audience, not from the topic.** "Whose behaviour are you trying to influence?" — and if you can't name the group, your scope is wrong.
- **The CEO's message is a membership test:** this is the sort of outfit we are; if not, you'd probably better move on.
- **Operational risk management sits above information security**, not the other way round.
- **Granularity trades against efficiency.** Every extra dimension of policy multiplies the management load — the 60-cell classification matrix is the reductio.
- **Classify systems, not documents.** It's objective, transparent to users, auditable, and yields nine policies instead of sixty handling regimes.
- **Least privilege is a platform principle, not a knowledge principle.** Need-to-know versus need-to-share.
- **Owner decides, custodian enforces, user consumes.** The three-role model makes outsourcing contracts legible.
- **Brand the security message.** One consistent mark everywhere beats a different message every time.
- **Money buys attendance.** Give the steering group budget authority or it becomes a talking shop.

## Anti-patterns
- Shelf-ware policy written to satisfy an auditor's tick-box.
- Copying a sample security policy out of a textbook.
- Padding a policy with material irrelevant to its target audience — *"do not include irrelevant material 'just because it's there'."*
- Repeating the same operational-risk statements under three different discipline headings instead of hoisting them into an over-arching policy.
- Making the **IT director** the director in charge of information security.
- Calling the role **IT Security Manager**.
- Letting the CISO be held responsible for the security of business information.
- A steering group with no real power — junior delegates, no decisions, no value.
- A military-style information classification scheme adopted commercially "because we can", with a 60-cell handling matrix and information drifting permanently upward.
- Applying least privilege so tightly at the application layer that knowledge work becomes impossible (the 9/11 pattern).
- Raising awareness once and then neglecting it; running a different awareness message and visual identity each time.
- A blame culture, which drives mistakes underground.
- Tying incident-report counts to bonuses or league tables.
- Failing to anticipate the reporting spike that follows a successful awareness campaign.
- Outsourcing policy making, risk assessment, requirements specification or authorisation granting.
- Signing an outsourcing contract before security management has been fully discussed and resolved.
- An outsourcing deal with no security target document in the SLA and no agreed definition of "secure".

## Key Takeaways
- Security policy is the logical embodiment of business requirements for security and control — a Logical-layer deliverable managed as an Operational-layer activity.
- Write for a defined audience whose behaviour you intend to change; if you can't define the audience, the scope is wrong.
- The policy hierarchy runs: over-arching operational risk management → the three discipline policies → CA/RA policies → infrastructure and line-of-business application policies → specific policies → **rules, practices and procedures (Physical)** → **standards (Component)** → **implementation guidelines (Operational)**.
- The Data Backup example is the canonical illustration of policy vs procedure vs standard vs guideline.
- Nine policy principles — least privilege, accountability, minimum dependence on secrecy, control automation, resiliency, defence in depth, approved exception, secure emergency override, auditability — offered as inspiration, not commandments.
- Least privilege is challenged at the application/knowledge layer; the 9/11 Commission's "need to share" is the exhibit.
- Information classification is judged largely unworkable commercially; **system classification by business risk × Business Attribute is the recommended substitute**, yielding a nine-policy matrix and four objective audit questions.
- CP and CPS are orthogonal: CP per certificate type for interoperability; CPS per CA for trust and liability.
- Governance runs Board → Executive Team → Steering Group → Liaison Group → line management, with a senior (non-IT) director in charge and a full-time CISO acting as a centre of expertise rather than an accountable owner.
- Culture is built by CEO sponsorship, a consistently branded never-ending awareness campaign, managerial example, a no-blame reporting culture backed by a real disciplinary process, and four tiers of training.
- Outsourcing: never outsource policy making, risk assessment, requirements or authorisation; do outsource implementation and enforcement; anchor it all in owner / custodian / user roles, a security target document in the SLA, and pre-contract negotiation.

## Connects To
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — Table 3-2, which places policy making in the Operational architecture at the Logical layer.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — the Business Attributes Profile used to write the security target document for an outsourcing SLA.
- [Chapter 8 — Managing the Security Architecture Programme](ch08-managing-the-security-architecture-programme.md) — the Rules for Influencing Opinion or Behaviour that govern how a policy should be written.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — the business risk assessment method that drives system classification.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — security domains and security policy authorities, the Six As, registration assurance levels, and the layered service bundles that supply policy content.
- [Chapter 11 — Logical Security Architecture](ch11-logical-security-architecture.md) — the theoretical view of security policy and the same hierarchical policy architecture (Figure 11-1).
- [Chapter 12 — Physical Security Architecture](ch12-physical-security-architecture.md) — rules, practices, procedures and guidelines in detail.
- [Chapter 13 — Component Security Architecture](ch13-component-security-architecture.md) — the security standards layer, internal and external, and XML-based policy documents.
- [Chapter 15 — Operational Risk Management](ch15-operational-risk-management.md) — the integration of operational risk disciplines that justifies the top policy layer.
- [Chapter 16 — Assurance Management](ch16-assurance-management.md) — the audit function that verifies policy compliance on both sides of an outsourcing relationship.
- [Chapter 17 — Security Administration and Operations](ch17-security-administration-and-operations.md) — the administration and support roles named in the governance structure.
