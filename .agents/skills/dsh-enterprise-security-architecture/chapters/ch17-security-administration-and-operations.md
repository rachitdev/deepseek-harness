# Chapter 17 — Security Administration and Operations

## Core Idea
> "The foregoing chapters of this book have focused heavily on the strategic and tactical aspects of managing an enterprise information security programme. In this final chapter the emphasis is **wholly upon the day-to-day operations** that keep that programme moving forward."

This chapter is unusual in the book because it fills **four** SABSA Matrix cells at once — the whole remainder of the Operational architecture row (Chapter 3, Figure 3-3; Chapter 7, Figure 7-8):

| Column | Operational-row cell filled here |
|---|---|
| **Process (How)** | Security Service Management and Support |
| **People (Who)** | Application and User Management and Support |
| **Location (Where)** | Security of Sites, Networks and Platforms |
| **Time (When)** | Security Operations Schedule |

(The other two Operational cells are taken by Chapter 16 — Assurance of Operational Continuity, Assets/What — and Chapter 15 — Operational Risk Management, Motivation/Why.)

The chapter must be read alongside **Chapter 11's *Security Management Services*** section: *"The emphasis here in this chapter is different from that in Chapter 11, focusing on the **operational procedures that run parallel with the logical security architecture** and which are a part of the overall operational security architecture."* Chapter 11 gives you the logical service; Chapter 17 gives you the human procedure that runs it every day.

Its closing claim is that this operational layer *"consists of a variety of operational and administrative processes that are performed on a day-to-day basis to maintain enterprise security… They cover **the bulk of the material contained in ISO/IEC 17799:2000**."*

## Frameworks Introduced

### The Operational Prevention Process (Figure 17-1)
Prevention is not a control; it is the **co-ordination of seven threads** that have each been developed elsewhere in the book and are here brought into one continuous operational loop:

**Monitoring & Detection · Intrusion Detection · Risk Assessment · Policy & Standards · Intelligence Gathering · Penetration Testing · Follow-up Actions from Incidents**

> "Many of the threads that it draws together have already been discussed, especially in Chapters 14, 15 and 16. What is added here in this chapter is a discussion of the **routine, ongoing operations and administrative functions** that keep your information security management fully operational on a day-to-day basis."

### ISO/IEC 17799:2000 as the operational skeleton
The standard is used a fourth time in this book — Chapter 14 for policy/organisation/asset classification, Chapter 15 for risk management, Chapter 16 as an audit framework, and here as the operations framework. The relevant sections:

| Section | Title |
|---|---|
| **6** | Personnel security |
| **7** | Physical and environmental security |
| **8** | Communications and operations management |
| **9** | Access control |
| **10** | Systems development and maintenance |
| **11** | Business continuity management |
| **12** | Compliance |

### Change control vs change management — a distinction the chapter insists upon
> "It is easy to confuse the two terms 'change control' and 'change management' because they sound similar, but they have **very different meanings**."

- **Change control** = the operational discipline over changes to equipment configurations, software configurations, operational data and *the operating procedures themselves*.
- **Change management** = *"the management of a **major change across the entire enterprise**."* The book's worked instance: an organisation with no formal security framework deciding to implement a **BS 7799-compliant ISMS** — that is a change *project*, not a change *record*.

### Incident management vs problem management — a conflict, not a hierarchy
> "The incident manager wants the system brought back up while the problem manager needs it to stay down for investigation."

Frameworks such as **ITIL, BS 15000 and AS 8018** deliberately segregate these duties — incident management optimises restoration of service levels, problem management optimises understanding of causes. *"This is of particular sensitivity in the area of security where **many incidents are not known problems but the result of new threats**, which may be very difficult for help desk staff to identify."*

Problem management divides into **reactive** (resolve identified problems within agreed service times) and **proactive** (analyse incidents to prevent recurrence — fed by service reports and incident reports).

### Three-level help desk support
| Level | Who | What they do |
|---|---|---|
| **Level 1** | Help desk agents with broad expertise, in direct contact with callers | Basic diagnostic tools, information lookup, asset databases and configuration data; immediate resolution of common problems and FAQs; capture of complex problems for referral |
| **Level 2** | Expert analysts with specific product and system knowledge | Detailed research, problem simulation, fault replication on test systems; **relieves Level 1 of protracted investigations so call-handling-time variance stays predictable for capacity forecasting** |
| **Level 3** | The development team that designed and built the product — *"often at a third-party vendor"* | Major intervention, usually a patch; often new hardware or software shipped and installed at the user location |

> "Help desk support, especially at Level 1, is extremely important in the management and administration of information systems security, since in most organisations **the majority of user calls are security-related**. Forgotten passwords, locked user accounts and incorrect privileges make up a huge number of the problems that users encounter."

### The business-process-based approach to BCM (Figures 17-4, 17-5, 17-6)
> "The most logical must be to start with the business processes, since BCM is essentially all about maintaining the continued operation of these processes."

Prerequisite: documented, hierarchical business process models. A large enterprise has roughly **10 Level-1 meta-processes** — *Develop product offerings · Bring product offerings to market · Acquire customer orders · Fulfil customer orders · Manage and administer the business* — each decomposing into ~10 sub-processes, each into ~10 sub-sub-processes. *"Potentially the total number of detailed processes grows exponentially at each level… you will need a **professional automated tool** to carry out this analysis, to store all the detail, keep the model up to date, and delegate process ownership at each level."*

**The seven-step BCM process:**

| Step | Activity | Content |
|---|---|---|
| **1** | Business Process Impact Assessment | Identify and map business processes; assess business impact of loss of each; classify into bands — **A: Critical** (loss will destroy the business) · **B: Severe** (persistent, severe damage) · **3: Significant** (optional band — significant damage) · **4: Other** (damage can be absorbed) |
| **2** | Functional Analysis of Business Processes | Take only Band A and Band B; analyse sub-processes down to **single functional steps** |
| **3** | Resource Analysis of Functions | For each function: what resources, how much? (people, ICT services, accommodation, equipment, communications, raw materials) |
| **4** | Threat Scenario Synthesis | For each resource, what high-level threat scenarios put it at risk? **"Focus here on effects, not causes."** |
| **5** | Resilience Analysis | For each resource/scenario pair, is current resilience sufficient? **Are there single points of failure?** |
| **6** | Business Continuity Planning | Additional resource protection: preventive measures · containment measures · redundancy · incident management plans · recovery plans · crisis management plans · training and awareness |
| **7** | Risk Financing for Cost of Recovery | Insurance and related services |

Recommended further reading: **BSI PAS 56, *Guide to Business Continuity Management***.

### Managed security services — the outsourcing frame
Candidate services: managed VPNs · managed firewalls · managed authentication service · **managed S-NOC** (secure network operating centre) operations · managed intrusion detection and prevention · managed anti-virus screening · managed content screening · penetration testing · vulnerability scanning.

**Benefits:** cost reduction through the provider's economies of scale · focus on core business rather than *"dissipating your efforts on peripheral activities"* · expert attention from a provider to whom security is a core business · **"Herd immunity — by joining a large community of customers being serviced by the same provider, ensuring that any new attack will affect only one or a few of those customers, the remainder being protected by the knowledge and skill gained from the early attacks."**

**The three questions that must be answered:** *"Will you lose control? Who will own the risk? What is the split of responsibilities?"*

**The split the book recommends:**

| The customer is responsible for | The service provider is responsible for |
|---|---|
| Business security risk assessment and **policy making** | Security infrastructure management |
| Management of business processes and systems using the outsourced services | **Policy implementation** |
| **Authorisation** of business users | **Administration** of users |
| — | Meeting the terms of the SLA |

Both parties monitor compliance with the SLA; an **independent third-party audit** ensures compliance with best practice. Required documents: a legally binding outsourcing agreement with defined responsibilities and liabilities, an organisational structure formalising ownership on both sides, a **Security Target document** describing the customer's primary security requirements, and supporting implementation documents.

## Reference Tables

### Making security everybody's responsibility
> "There is an unfortunate perception in many organisations that security is the responsibility of a few people who have job titles that include the word 'security'."

Operational counter-measures: mention security responsibility in **every job description and every contract of employment**, with confidentiality agreements · relate the responsibility to real corporate risks · reference it directly in the corporate information security policy · introduce it **at recruitment interviews and staff induction** · train people both in awareness and in the techniques they need · monitor each individual's compliance stance and **use appraisal reviews** to note both shortcomings and successes · mandatory incident-reporting procedures everyone knows how to use · *"if necessary… an **anonymous whistle-blowing facility** to protect an employee from intimidation by a more senior and powerful person who may be abusing that power."*

### Personnel management as a security control
**Screening and vetting** should include: character references — **one professional and one personal** · verification of the CV or application form *"ensuring that all periods of time are accounted for and checked"* · confirmation of all claimed academic or vocational qualifications · independent verification of identity · possibly personal credit checks where large sums of money are handled — *"although **being in debt is not a crime and not an indicator of criminal intent**."*

**Supervision** watches not only work quality but *"personal family life, financial difficulties, psychological difficulties or changes in lifestyle"* — with the warning to *"tread carefully and beware of being cavalier in making accusations that will lead to industrial relations difficulties."*

**The five kinds of leaver, each needing its own tailored approach:** voluntary resignation to another job (*"Why are they leaving — unhappy? Career move? More money? Following their spouse or partner?"*) · voluntary normal or early retirement · compulsory retirement by age against the person's wishes · dismissal for misconduct or under-performance · redundancy, possibly as part of wider downsizing.

### Segregation of duties and dual control
The banking origin: a vault with **two locks, two keys, two keyholders**. Modern instances — a high-value transaction entered by a clerk and released by a manager; a system change authorised by a manager and implemented by an engineering technician; **raising a purchase order segregated from signing off receipt of goods** (which blocks collusion with an outsider over fraudulent or duplicated invoices).

Three qualifications the book adds:
1. **Deputies must exist** so productivity is not blocked — *"a deputy cannot be a person authorised to perform the other part of the segregated task."*
2. **The monitor/auditor variant** leaves most room for proper segregation: one person simply checks the other's activity and reports unexpected behaviour.
3. **Collusion and conspiracy.** Exclude members of the same family and close friends; segregate the two people *"as much as possible both in their work interaction and in their social and domestic interaction."* Where that is impractical (small organisations), buy back the risk with *"enhanced audit trails and supervisor monitoring."*

**Enforced annual leave** — the chapter's sharpest single control:
> "If a member of staff is running an ongoing fraud against the employer, keeping that fraud hidden often depends upon that person maintaining continuous control over some activity or over a set of records. These are the people who **never take leave, never take sick leave (even when they are ill) and always work overtime — often alone in the premises**."

The rule: everyone at every level of seniority **MUST** take at least one continuous week of annual leave (preferably two) in each leave year, and **their job MUST be done by someone else during that period**.

### Physical and environmental security
Four perimeter types: **site perimeters · building perimeters · internal perimeters of secure areas** where sensitive processing or secure storage takes place · **locked cabinets, storage cupboards, 'dark' equipment rooms and storerooms** (*"the term 'dark' is often used to describe an unmanned equipment operations area"*).

Controlled access points may be automated mechanisms or staffed by guards/reception. **The duties of the guards must be segregated from those who are guarded.** Badges shown at the access point and worn visibly inside; visitors escorted and supervised at all times, including on-site contractors and temporary staff; delivery and loading areas need special arrangements. Equipment, power supplies, communications connections and cabling runs need protection against malicious *and* accidental tampering plus fire, water, smoke and physical damage — *"the safety of personnel who handle or come into contact with equipment is also of paramount importance."* Maintenance under documented procedures; authorised-removal procedures for offsite use or repair; all configuration changes under change control; **equipment authorised for disposal or re-use cleansed of residual data before removal from site**. Theft and loss controlled by clear desk and clear screen policies plus locked storage, with inventory control and logging of every item stored or removed.

### Operating procedures — the six documentation requirements
The test is that *"even if the staff members who are familiar with these tasks are unavailable, a new member of staff has access to a full description of each and every task."*

Detailed step-by-step processing task descriptions · scheduling requirements and constraints, **including interdependencies between systems and earliest or latest start times to harmonise with business deadlines and cut-offs** · instructions for handling errors and exceptions during processing · support contacts for unexpected difficulty · handling instructions for special output media · recovery and restart procedures in case of system failure.

### The change control process — seven points
Identification of significant proposed changes and capture of details · **impact assessment** for each · formal evaluation and approval · documentation with communication to all affected parties and updates to all records **including the configuration database and the business continuity plans** · clear responsibilities for implementation *including responsibility and procedures for reversing out and recovering from failed changes* · quality assurance of the implementation · **post-implementation testing**.

### The change management programme — what a major change must address
Building executive-level sponsorship and championship · defining goals and objectives · planning the investment · business process development · cultural development · infrastructure development · managing interfaces with customers, suppliers and regulators · identifying success factors · identifying obstacles and barriers (adopting a methodology, overcoming resistance, **estimating organisational change capacity and speed**, employee resistance, manager resistance) · project team structure and membership · communications to management and staff — messages and methods of delivery · training for new skills · dealing with redundant skills and the whole HR/industrial relations scenario · **retaining valued employees** · rewarding positive and discouraging negative behaviour · use of external consultants and change management models.

### Incident handling
**Incident types:** system failures and loss of service · denial of service · errors from incomplete or inaccurate business data · breaches of confidentiality · unauthorised changes.

**The handling procedure:** receive the alarm · identify and analyse the cause · **potential business impact assessment and prioritisation (triage)** · review remedial options · plan and implement remedies both to remove the cause and to recover the effects *"and also to prevent its recurrence"* · **collect and preserve audit trails and forensic evidence** for possible legal proceedings and contractual negotiations · communicate with affected parties · report the incident and responses to an appropriate authority.

**Security-specific additions where an ITIL/BS 15000/AS 8018 framework is in place:** identification and classification of a security incident · reporting and escalating · investigation · **criteria for management decisions balancing restore-service against contain-problem against investigate** · handling information relevant to the incident — *"if the incident is inappropriate or illegal use of computer resources it would be inappropriate for the help desk or incident-management systems to openly display and record the sensitive details of who was involved."*

### Segregating development from production — five rules
1. Development, test and live production facilities **completely segregated** from one another.
2. Transfer rules from development/test into production clearly defined and documented, **with an authorisation process involving a responsible and accountable manager**.
3. No development activity on a live production system, *"with the possible exception of emergency fixes that are carried out under strictly supervised special procedures."*
4. On a live production system the only permitted software is the application software plus the system software needed to run it. **All software development tools, editors, compilers, linkers, loaders, system utilities** and similar code-generating tools *"should be completely banned from all live production machines."*
5. Even for an emergency fix: **diagnose on the live system, fix on the development system, test on the test system, then promote and install the code patch on the live system.** *"There should be no need for software tools to be executed, or indeed reside, on the live system."*

### Capacity planning
**Eight capacity types:** processing power · memory size · disk (and other media) storage capacity · communications bandwidth · input and output volumes · **staffing levels** · **accommodation requirements** (according to the footprints of equipment items and staff working space) · **ancillary services** (electrical power, air conditioning).

**Four activities:** monitor utilisation of current capacity *"including the statistical variability in terms of peaks and troughs using **mean and standard deviation** parameters"* · monitor the rate at which spare capacity is being consumed and plot trends · make predictions and forecasts against timelines · plan ahead so the capacity is there when the time arrives.

### Data backup, media and logging
**Backup strategy** scales with business need: conventional **weekly full backups with daily incrementals**, or, *"for a more demanding set of business requirements, **real-time online data mirroring**."* Whatever the mechanism, it must be tested regularly — both that copies are actually being created **and** that restoration works — and *"these restoration tests should include verification that the restoration can be accomplished **within the window of time available in the business day**."*

**Operator logs** are the manual counterpart to automatic system logs. Each entry: name of the operator making the entry, date and time, nature of the entry — system starting and finishing times · system errors and other events and corrective actions taken · confirmation of completion and correctness of input and output runs.

**Media in scope:** magnetic (disks, diskettes, tapes and tape cassettes, *including voice recordings*) · optical (CDs and other optical media) · printed paper output and specialised printed media · **carbon paper, one-time printer ribbons and other quasi-media**.

**Media handling procedures:** secure permanent erasure of data no longer required · authorisation for removal of any media from the premises, **whether or not for disposal** · a complete audit record of every item — creation, use, storage, erasure, removal, disposal · secure storage including off-site for backup media · standards for media quality and acquisition, naming/labelling/indexing, secure storage and retrieval, environmental conditions (**temperature, humidity, electromagnetic fields, physical stability**), secure transport, care and maintenance, recycling and lifetime management, secure disposal and destruction.

Plus the readability trap: *"you also need to ensure that old, archived, stored media can still be read back using the correct media read/write sub-systems… **(When did you last see an 8-inch or even a 5.25-inch floppy drive?)**"*

### Event log and audit trail management
**Security-significant event types:** exceptions — *"events which are unusual and beyond the pattern normally expected"* · failed login attempts · successful logins and subsequent logoffs · access to especially sensitive information resources flagged for event logging · **use of privileged resources such as administrator or root identities** · errors or failures in any logical or physical component · security alerts from anti-virus software, firewalls and intrusion detection systems · **expiry of security credentials and authorisations**.

**Event log record fields:** date and time of the event · user identifiers associated with the event · logical or physical location, or both · event type (*"may be encoded to economise on log storage space"*) · any other context information needed to explain the full nature of the event.

**Storage models:** short-retention logs in a finite area on a rolling **FIFO** basis, oldest overwritten by newest; or multi-year retention with recent logs online and older material archived monthly.

**Tool capabilities:** search by event type or identifier · search combinations and patterns using **AND, OR, NOT, XOR** query functions over record fields · statistical analysis of event patterns, frequencies and severity · archiving, indexing and retrieving.

### Forensic investigations — four goals, four process elements
| Goals | Process |
|---|---|
| Maintaining the **continuity of the chain of evidence** | **Planning** — goals, objectives and scope; establishing the principles; defining the protocols |
| Preserving the **integrity of the evidence** — preventing contamination during or after collection | **Seizing the evidence** — tools required; conduct of the seizure process |
| Ensuring evidence will be **admissible** under the rules of the court and relevant laws | **Analysis** — tools; logical analysis processes; establishing the history of captured digital evidence; **detecting and overcoming data hiding tools and techniques**; handling encrypted forensic material |
| Building the appropriate **weight of evidence** — quality and completeness | **Reporting** — written reports; expert witness statements and court appearances |

> "This is not a job for amateur first-timers. Inappropriate actions can jeopardise potential criminal prosecutions or internal disciplinary hearings. The priority is to **preserve the 'crime scene' as completely as possible… by leaving it undisturbed and calling in expert assistance at the earliest opportunity**."

### Access control management — the operational layer
**Considerations drawn forward from Chapters 10 and 14 into the access control policy:** information classification · system classification · identity management and **federated identity management** (*"a system that allows individuals to use the same user name, password or other personal identification to sign on to the networks of more than one enterprise"*) · mandatory vs discretionary access control · consistency of rules across systems and networks forming **different policy domains** · compliance with legislation and contractual arrangements · policy principles · **the possible limitations of applying the least-privilege principle** · role-based access control to simplify administration · **context-based access control** (different rules by time of day, day of week, end-user location, and *"for an end user as an employee and as a customer of the same organisation"*) · strength of the authentication mechanisms · the authorisation process and the ownership/custody/use model · **the access-related parts of the SABSA Business Attributes Profile**.

**User access management operations:** registration and deregistration · unique identification (user IDs or distinguished names) so actions link unambiguously to a user for accountability · creating **system** authorisations from **business-level** authorisations (job functions and responsibilities) · administering the changing pattern for **'joiners', 'movers' and 'leavers'** · authenticating users and checking authorisations at login · recording all login and access events, successful and failed, **and reviewing the logs** · periodic review of privileges *"to check that they reflect real business access requirements and that the configuration of privileges **has not decayed over time**"* · periodic removal of redundant accounts · **removing or re-passwording vendor default accounts**, including vendor maintenance accounts.

### System-level privileges — the encapsulation rule
> "Much safer is the practice of creating utility tools that run with the privileges of these accounts but which **strictly control the functionality available (often known as encapsulation)**… Thus the highly privileged user is decoupled from the raw system, and any errors or mistakes will not have a direct impact on the system itself — the tool prevents incorrect actions from being made."

Raw privileged account passwords are **kept secret and locked in a safe**, released only in genuine emergencies when the routine tools cannot fix the problem.

And the cultural obstacle, named plainly:
> "There are some operational environments where the culture is that '**real men (and women) are not afraid of raw command-line access**'. It is a culture of the geeky, eccentric techno-nerd… Such system managers tend to regard the computers they manage as their personal playground, and their special knowledge maintains their powerful control over the system. It might be great fun, and in an academic environment it may be acceptable, but it is **no way to run a professional operational production shop**."

### Third-party access
**Two types:** physical (offices, store rooms, equipment rooms, computer rooms, filing cabinets) and logical (information systems and databases).

**Legitimate third parties:** service providers — telecommunications engineers, hardware/software maintenance engineers and support personnel, utilities engineers (electrical power, water, gas), cleaning staff, catering staff, security guards, consultants · trading partners — customers, suppliers, joint venture partners · short-term temporary contract staff and students on work-placements · **regulators** · **external auditors**.

**The nine access-management issues for on-site contractors:** access privileges to be granted · entry passes and passwords, on whose authority and against what criteria · degree of supervision and escorting · times and modes of access · **procedures for withdrawal and termination of access rights** · codes of behaviour with reporting and disciplinary processes · liabilities of the contractor for their employees' behaviour · contract terms covering compliance with the organisation's security policies · specific contractor employee confidentiality agreements.

The chapter notes that a **third-party access policy** belongs in the hierarchical policy architecture of Chapter 14, Figure 14-1, *"alongside the Remote Access policy, Anti-Virus policy and others in that group."*

### Outsourced ICT operations — the thirteen contract issues
Clear unambiguous definition of the services outsourced **plus a risk assessment of the associated information security risks** · compliance with legal requirements for information security · contractual agreement of security responsibilities, liabilities and obligations, *"ensuring that all the contractor's employees are aware of these"* · maintaining **and testing** confidentiality and integrity of the customer's business assets · protecting the IPR of both parties · selection, deployment and operation of physical and logical controls · processes for transferring staff · **the minimum service levels to be maintained even under disaster recovery conditions** · methods, processes and criteria for monitoring and reporting performance against SLA targets · levels of physical security for the operations centre and its equipment · **the customer's rights to audit** how the provider meets information security requirements · responsibilities and procedures for **shared** security management · processes for identifying, reporting, escalating and handling security incidents.

### Asset and configuration management
**Four asset classes for the register:** **information assets** — databases and data files (including backup copies), system documentation, user manuals, training materials, operational and support procedures, continuity plans, fallback arrangements, archived information · **software assets** — application software, system software, development tools and utilities · **physical assets** — computer equipment and peripherals, communications equipment, magnetic media, ancillary equipment, accommodation, furniture, office equipment · **services** — communications services and general utilities (heating, lighting, electrical power, water, air conditioning).

Each entry carries location, identification data such as serial number, date of acquisition, original value, present value, ownership and **security classification**.

> "Whereas asset inventories record lists of hardware and software items, **they do not record the relationships between them**. These relationships between the various ICT components are the objective of configuration management."

Configuration management = *"the unique identification, recording and reporting of components, their version, constituent components and relationships."* Items under configuration control: **hardware, software, data structures, licences, documentation and people.**

**Ten benefits:** support for change control and change management by making the impact of configuration changes traceable · effective deployment of software and hardware · improved resource planning · improved financial planning · improved risk management · maintaining stability and security of the operational environment · improved business continuity and disaster recovery · improved control over legal and contractual responsibilities such as licence management and copyright · **reduced complexity of configurations** · improved problem resolution and hence higher service delivery.

### The SLA — ten security clauses
> "It should be formalised **whether or not the service provider and customer are separate commercial entities**. It is just as applicable in the case where the service provider is another department or business unit within the same commercial organisation."

Security obligations, responsibilities and liabilities between the parties · points of contact, communication channels and methods, and the management process for security issues · escalation procedures · complaints process · **rights of the service customer to audit the service provider** · security service performance targets · reporting against those targets · security incident identification, handling, escalation and resolution · **remedies and penalties** for failure to reach agreed levels · disaster recovery and priorities following a service failure.

The SLA is itself under change control — *"the key word is 'agreement' — between the two parties — and hence **any changes must be renegotiated and subject to agreement**."*

### Business relationship management — the layer above the SLA
> "In many ways these formal frameworks represent only a **safety net — a fallback position for when the relationship sours**… the investment of effort on business relationship management will bring much greater returns than much of the effort that is necessarily put into keeping in place that safety net of contracts and SLAs."

**Three goals:** know your customer, understand the business, be aware of the business drivers and constraints · use that understanding to **interpret technical requirements more accurately by understanding the true business requirements that lie behind them** · understand the subtle cultural differences between different organisations in your portfolio — *"on the surface they may seem to be very similar, but in cultural terms can be very different."*

**Four techniques:** regular customer liaison meetings **at the business management level (not the technical management level)** · periodic service reviews focused around the SLA, past performance and future requirements, again at business management level · objective reporting of costs, service levels and workloads *"to try to head off subjective judgements supported by anecdotal evidence"* · customer satisfaction surveys **with visible, proactive follow-up**.

### Service monitoring and reporting
**Five report types:** workload management · problem management · financial · asset and configuration management · change control.

**Two categories:** **reactive reports** — past events, trend comparison of error rates, outages and cost overruns; **proactive reports** — early warning of significant events enabling preventive action.

> "**Raw metrics are rarely helpful to a manager.** The presentation of metrics needs to be tailored to the individual needs of the manager, and presented in a clear concise form that tells that manager what he wants to know — no more and no less."

The **management information dashboard** of Chapter 6 is the vehicle.

### Provisioning
*"The dynamic process of setting up, loading, configuring, co-ordinating and launching new components in a complex distributed-processing environment"* — most often used by network service providers, but equally applicable to a large distributed population of desktop PCs and workstations.

**General provisioning:** select remotely the hardware item or customer account · load appropriate software (operating system, device drivers, middleware, applications) · customise and configure the remote system, software, networking devices and storage resources · start newly loaded software · make the system ready for operation.

**Security provisioning as a sub-set:** creating, amending and deleting user IDs and accounts · setting and resetting passwords · administering access privileges and authorisations · loading and updating anti-virus software · loading and configuring firewall rules · configuring VPNs · **all operational activities involved in cryptographic key management**.

### Security-specific operations
| Level | Operational processes |
|---|---|
| **Security service management** | Refer back to Chapter 10 (*Security Service Management Strategy*) and Chapter 11 (*Security Management Services*) |
| **Security mechanism management** | Cryptographic key management · security parameter communication and synchronisation to support **security associations** · maintenance of ACLs and user privilege profiles · managing data backup and restoration · media management (labelling, indexing, transport, off-site storage, retrieval, recycling, lifetime control) · **virus signature maintenance and distribution** · **intrusion signature maintenance and distribution** · firewall rule maintenance · event log file management and archiving |
| **Security component management** | Product evaluation, selection and procurement · supplier relationship management · supplier contract management · project management for developments, implementations and roll-outs · **trusted build process management** — ensuring that every hardware and software component built into a system is authentic, comes from a known source, and is traceable · component lifecycle management (scheduled maintenance, replacement of worn parts and consumables, repair, retirement and disposal) |
| **User management** | Registration of new users · **authorisation of user privileges by the business** · implementation of authorised privileges · adding/amending/deleting registrations and credentials on system security databases · user support via help desk · problem tracking and management · password management, updating and changing · system account management of high-privilege accounts for applications, operators, managers and auditors |

### Compliance management — eight elements
Identify comprehensively all relevant laws and regulations **and extend the identification to your third-party contracts** · protect intellectual property rights, both yours and others' (copyright of documents and software, preventing unauthorised copying and distribution) · retain and safeguard organisational records that must be archived for legal periods (**always corporate financial accounting records**, sometimes sector-specific ones) · protect the privacy of personal information — **EU data protection legislation, the US Gramm-Leach-Bliley Act** · prevent misuse of corporate information-processing facilities · comply with cryptographic control regulations · ensure evidence collection complies with the rules of evidence and will be admissible · provide assurance of compliance with internal policies and standards (Chapter 16).

### Cryptographic regulations
> "Most countries treat cryptographic technology in **exactly the same way as munitions or armaments**."

Restrictions may fall on: export of cryptographic products · import of cryptographic products · manufacture of cryptographic products · **domestic use of cryptography in information systems** · export and import of **encrypted data** across national borders.

> "It is particularly complicated by the fact that every country is unique and has its own version of cryptographic control… It is even more complicated by the fact that the details are not necessarily made crystal clear by every government, some choosing to reserve judgement and **generally make it up as they go along**."

**The three international agreements (as at the book's stated time of writing, January 2005):**

| Agreement | Detail |
|---|---|
| **The Wassenaar Arrangement on Export Controls for Conventional Arms and Dual-Use Goods and Technologies** | Signed 1996 by **31 countries**, last revised December 1998. *"Each member state has to implement the provisions… in national legislation for them to have effect."* |
| **Council of Europe Convention on Cybercrime** | Adopted November 2001 by a body of **43 member states**; signed by **26** of them plus Canada, Japan, South Africa and the United States. Focus is law-enforcement access to information, **including the provision of decryption keys** for encrypted data. National legislation needed to implement |
| **EU Council Regulation (EC) No 1334/2000** | European Community regime for control of exports of dual-use items and technology (Official Journal L159, 30 January 2000), in force since **29 September 2000**. Follows Wassenaar but liberalises export between EU member countries |

**The six-point code of practice:** make a register of all countries where you have business operations, noting operational use of information in terms relevant to these regulations · set up a process that updates the register **every time you consider opening a new operation or significantly changing an existing one** · identify the current legal and regulatory situation per country · identify the government department or agency controlling the regulations in each country **and establish a relationship and dialogue with it** · keep liaison going so you see significant changes coming in advance · keep the register up to date and use it as a decision-support tool. *"Good advice is to be proactive — **never put your head in the sand and hope it will go away — it won't**."*

### Procurement — ITT, RFI, RFP
> "Procuring security products and tools is no different than doing so for other items. However, it is surprising how many times one encounters **sloppy practices** in making such choices. The contract and pricing is usually dealt with by a specialised purchasing department… **Where the deficiencies usually arise is in the technical evaluation and selection process**."

The **ITT problem**: government departments favour formal tendering to eliminate nepotism, fraud and back-door deals — *"all of which is laudable"* — but the rigid rules and explicit exclusion of negotiation on the technical specification break down when *"those who create the tender documents do not really understand what it is they are procuring."* The vendor must respond to the ITT as issued *"but who can see instantly what the customer should have been asking for. At this stage of the tendering process it is too late to put forward that advice."*

**The fix, in order:**
1. Research the area first and hold **informal discussions with all the leading vendors** before writing anything.
2. *"Stick rigidly to specifying their **requirements**, and let the vendors suggest **solutions**, which may differ greatly from one to another."*
3. If you lack the skills, hire an independent consulting firm to help write the ITT — *"it will be money well spent"* — and to help evaluate the responses. Experienced consultants add value *"through their ability to ask the awkward questions and expose all the issues."* As one client put it: **"I want you to bowl some googlies."** (Footnoted: *"In cricket the googly is an off-break ball bowled with a leg-break action. The batsman is taken by surprise by the unexpected direction of the spin."*)
4. In the commercial world, use the more flexible **RFI → meetings and presentations → RFP** sequence. *"Only when you understand fully exactly what it is you want to procure do you commit to a formal RFP… This still leaves headroom for subsequent negotiation of both the specification and the contractual terms."*
5. **Set out the selection criteria clearly in advance** and publish them in the ITT or RFP. *"In the end the overriding criterion for a decision is that you have the greatest confidence in the selected vendor to fulfil your requirements."*

### Software licence management
A published corporate software licence compliance policy defining legal use and the organisation's stance on copyright · standard procedures for software acquisition and deployment · staff awareness plus *"a robust disciplinary process for dealing with those who violate the policy"* · a comprehensive asset register of all commercial software products acquired and in use — **number of run-time copies, location of the machines, location of the master copies (CD or diskettes), and documentary proof of possession of the requisite licences** · ensuring site licence maximum run-time user counts are not exceeded · ensuring unauthorised unlicensed software is not in use or installed, **especially on end-user PCs**.

### Information exchange
**Three controls:** risk assessment revealing threats, impacts and vulnerabilities of the exchange · procedures and standards to protect information in transit, *"including specific procedures for sending and receiving"* · formal agreements and contracts defining roles, responsibilities and liabilities of each party.

**Ten mechanisms, each needing its own threat and vulnerability assessment:** physical transport of media (disks, tapes, paper documents) · web-based electronic commerce · file-transfer-based **EDI** · e-mail and file attachments · voice telephony and voice mail · video conferencing · web conferencing and web-casting · fax transmissions · **SMS text messaging** · IRC-based instant messaging *"and the constantly emerging multimedia versions of this type of service."*

### Network operations — three additional controls
Segregation of responsibilities and activities for **network** operations from those for **computer systems** operations · clear responsibilities and operating procedures for **remote networking equipment** · clear responsibilities and operating procedures for **cryptographic key management where cryptographic networking equipment has been deployed**.

## Worked Example

### Vendor Selection for a Secure Banking System — the Business Attributes Profile as an RFP
This is the chapter's flagship artifact, and it closes the loop the book opened in Chapter 6: the Business Attributes Profile, invented as a requirements-engineering and metrics device, is used here as a **procurement instrument**.

**Why it works, as the book argues it:**
> "A Business Attributes Profile as described in Chapter 10 can be used as the framework for developing the RFP. This is particularly attractive to the business team since **they need not become involved in any technical specifications — the Business Attributes Profile is a specification of business requirements, not technical solutions**."

**The case, step by step:**
1. Consultants were engaged to assist with the specification of a new secure banking system, and in the course of the work **helped the client build a Business Attributes Profile**.
2. *"The client embraced this methodology enthusiastically, especially those people in the business who **for the first time began to see a real linkage between specifying business requirements and specifying a technical system to meet those requirements**."*
3. Once agreed, the Profile became the main basis for an **RFP issued to a short-list of two major systems vendors, both with globally recognised names**.
4. The vendors were asked to respond to a set of requirements built from **each and every Attribute in the Profile** — one requirement per attribute, no technical specification imposed.
5. Each response was assessed for level of compliance with each requirement, on a **five-point scale: 0%, 25%, 50%, 75%, 100%**.
6. The client produced **a pair of charts similar to Figure 6-3 in Chapter 6** — the Business Attributes taxonomy diagram — but covering only the attributes in this Profile, **colour-coded by compliance level per attribute**. Reproduced in the book as **Figures 17-2 (Vendor A) and 17-3 (Vendor B), pages 556 and 557, "exactly as the client used them."**
7. The two charts were presented as **two slides** to the Architecture Board.

**The outcome:**
> "This was the **shortest meeting the Architecture Board has ever had**, because the visual result of this approach was instantaneous and stunning. Vendor A has a '**cool**' chart with lots of greens and blues, whereas the chart for Vendor B is '**hot**', covered in red and orange. **Vendor A was awarded the contract**."

**Generalisation the book offers:** *"You may not choose to use this exact approach, although it has many merits to recommend it. However, the techniques of scoring and colour-coding applied in any format are extremely useful to help you get the appropriate vendor ranking… If you have criteria at different levels of importance, then **weighted scores** can be used so that the weights are appropriate to the level of importance for each selection criterion."* And the same colour-coded-attribute chart *"is also commonly used for reporting service levels in an outsourced service delivery environment."*

### Case Study: Not Counting on the Capacity of Census Interest
The chapter's capacity-planning failure, with its published numbers.

- In 2002 the **UK Government Public Records Office (PRO)** put the **1901 UK census** data online.
- The site launched **2 January 2002**, designed for a maximum of **1.2 million users in a 24-hour period**.
- **By noon on the first day — three hours after launch — 1.2 million users *per hour* were trying to access the site**, and that level of demand continued for days.
- The site was **withdrawn five days after launch**. *"There was severe embarrassment for the PRO."*
- It reopened on a limited basis in **August 2002**, was made fully available in **November 2002**, and now serves **8,000–10,000 web users every day**.
- The **UK National Audit Office (NAO)** found the PRO had *"developed a pre-launch strategy based on a low key launch"*, was taken by surprise by press interest, and had launched **during a holiday period**, meaning more users could reach it from home.

**The NAO's five principal lessons, as the book reproduces them:**
1. Transfer commercial risk to the service provider, to be funded where possible from revenues earned from the service, and secure an interest in further revenues thereafter.
2. **Recognise the distinction between commercial risk, which can be transferred to third parties, and reputation risk, which usually remains with the contracting party.**
3. Make a realistic assessment of usage and put in place a capacity management strategy that will successfully divert unexpected overloads experienced in practice.
4. **Carefully select both the rate and timing of the launch** of new services to maximise the opportunity to resolve unforeseen problems before peak demand has built up.
5. Before the service goes live, develop an **agreed post-launch disaster recovery strategy**.

**And three recommendations for the National Archives:** monitor the website's ongoing financial performance · work with the contractor to maximise marketing opportunities and revenues · include in the corporate plan an early appreciation of how the completion of the **10-year contract** with the contractor will be managed.

### The three backup case studies — one failure mode each
| Case | What happened | The missing step |
|---|---|---|
| **The Phantom Backups** | A small software house whose assets were *"largely in the code that it produced"* installed a fully automated high-speed tape backup system, rented secure off-site storage, ran incrementals every evening and a full backup every Friday, paid a member of staff overtime to run them after hours, recycled tapes monthly so a full month of depth was always available, and retired every tape after 10 uses. After ~18 months of running *"as smoothly as clockwork"*, a major disk crash destroyed the development disk beyond recovery. **Every tape in the entire month was completely blank — including the recycled ones brought back.** *"It was impossible to tell when the backup writing system had failed (had it ever worked?)."* | **Verification that data was actually written to the tapes and that a restoration could be achieved** |
| **Time Travelling** | A credit card processing firm's mainframe production facility — millions of cardholders, 24×7, *"production never stopped"* — had a solid tape backup regime for data and current software versions. Management commissioned a study on how long a full business restoration would take after a complete system failure. **The answer was 27 hours — more than a whole business day, "so you could never catch up."** | **Restoration time inside the available business window.** The fix was architectural: real-time mirroring logically, plus upgrades to processing power and channel bandwidths physically |
| **Banking on the Backups** | A major European bank, legally required to retain account information for **seven years** and to disclose on request, tested tape content at backup time, tested the restore process at the same time, and stored duplicates on site in a fireproof safe and off site. In **1999** a court ordered disclosure of **1993** transactions. The tapes were retrieved — and **could not be read**: the physical architecture had changed beyond recognition and the tape drives used to write and read them had been disposed of as redundant. The bank *"had to engage the expensive services of a specialist forensic recovery company."* | **Media-readability lifecycle management** — retaining the read/write sub-systems for the full retention period |

### Case Study: Default Accounts Give Default Access
A small business with little computer expertise bought a system from a well-known vendor, whose **field service engineers installed, configured and commissioned it**. Only authorised finance department staff were permitted to use it; all had individual accounts and had been told to choose strong passwords and not share them. Yet other people were evidently reading company records, and the finance director *"was at a loss as to how this was happening."*

The cause: **two preconfigured default accounts shipped with the system** —

| Account | Password | Privilege |
|---|---|---|
| `System` | `Manager` | Highest — access to all parts of the system |
| `Field` | `Service` | Highest — access to all parts of the system |

> "Anyone who knew about these default accounts could access anything they wanted."

This is the case that motivates the user-access-management rule that vendor default accounts — *"including accounts to be owned and used by the vendor for maintenance purposes (whether remote or local) during the operational lifetime of the system"* — must be **completely removed or re-passworded to a strong value**.

## Mental Models
- **The Operational row is where four columns collapse into one job.** Process, People, Location and Time all resolve, on any given Tuesday, into the same set of administrative procedures run by the same team.
- **Chapter 11 designs the service; Chapter 17 staffs it.** Every logical security management service has an operational procedure running in parallel with it. A service with no procedure is not deployed.
- **Security is everybody's responsibility, and the proof is in the HR documents.** If it is not in the job description, the contract of employment, the induction and the appraisal, it is not anyone's responsibility.
- **Personnel management *is* a security control.** Screening, supervision, disciplinary process, leaver process and enforced annual leave are security mechanisms that happen to live in HR.
- **The person who never takes leave is a finding.** Continuous control over records is what an ongoing fraud requires.
- **Guards must be segregated from the guarded** — dual control applies to the physical layer exactly as it applies to payments.
- **Change control is a record; change management is a project.** Confusing them means either bureaucratising trivia or running an enterprise transformation off a ticket.
- **Incident management and problem management are designed to conflict.** Restore-now versus keep-it-down-to-investigate is a genuine trade, and security incidents — *"not known problems but the result of new threats"* — sit exactly on the fault line. You need pre-agreed decision criteria, not a winner.
- **Encapsulate privilege; lock the raw password in a safe.** The privileged human should meet the system through a tool that cannot express the wrong action.
- **A backup you have not restored is a rumour.** Verify content, verify restore, verify restore time, verify the drive still exists in seven years.
- **Requirements, not solutions, in the ITT.** Specifying the technical solution before you understand the market means the tender can only find someone to build the wrong thing.
- **The Business Attributes Profile is a universal adapter.** Requirements engineering (Ch 9–10), metrics and ROI (Ch 6), risk normalisation (Ch 15), assurance targets (Ch 16), and — here — vendor scoring and service-level reporting, all off the same taxonomy.
- **Colour beats numbers in the boardroom.** Cool chart versus hot chart decided a banking system in the shortest Architecture Board meeting on record.
- **BCM starts from processes, not from the data centre.** Impact bands → functional decomposition → resources → threat scenarios (effects, not causes) → resilience → plans → risk financing.
- **The SLA is the safety net, not the relationship.** Effort spent on business relationship management returns more than effort spent on the contract that catches you when it fails.

## Anti-patterns
- Believing security belongs to the people whose job titles contain the word "security".
- Screening that leaves gaps in the employment history unaccounted for, or that treats being in debt as evidence of criminal intent.
- One leaver process for five very different kinds of leaver.
- A dual control procedure with no deputies — so the control is bypassed the first time someone is unavailable.
- Dual control between family members, close friends, or people who socialise together; ignoring conspiracy along a chain of control.
- Nobody ever takes their annual leave, and nobody treats that as a signal.
- Guarding arrangements where the guards are not segregated from the guarded.
- Equipment disposed of or re-used without cleansing residual data.
- Procedures that live only in the heads of the staff who normally do the job.
- Confusing change control with change management — or running a BS 7799 ISMS implementation as though it were a configuration change.
- Change records with no defined reversal and recovery procedure, and no post-implementation testing.
- Letting a security incident's sensitive details — who was involved in suspected illegal use — be openly displayed in the help desk trouble-ticketing system.
- Compilers, editors, linkers, loaders and system utilities resident on live production machines; developers holding production access; emergency fixes coded on the live box.
- Capacity planned from a hoped-for launch profile rather than measured means and standard deviations — and launched at the moment of maximum unmanaged demand.
- Backups never restore-tested; restore time never measured against the business window; read/write hardware retired while the media it wrote is still under a legal retention obligation.
- Archived media with no environmental controls, no naming/labelling/indexing, no audit record of removal.
- Vendor default accounts left in place after commissioning — `System`/`Manager`, `Field`/`Service`.
- Access privileges never reviewed, so *"the configuration of privileges has decayed over time"*; redundant accounts never removed; joiners/movers/leavers handled ad hoc.
- Routine administration performed with raw root or administrator privilege because *"real men (and women) are not afraid of raw command-line access"*.
- Third-party access granted without risk assessment, time limits, supervision terms, revocation procedures or confidentiality agreements.
- Outsourcing ICT operations without defining minimum service levels **under disaster recovery conditions**, or without retaining the right to audit.
- An asset inventory with no configuration model — lists of items with no record of the relationships between them.
- Reporting raw metrics to managers instead of tailoring the presentation to what that manager needs to decide.
- Treating the SLA and the contract as the relationship rather than as its fallback.
- Writing an ITT or RFP without first talking to the vendor community; specifying the technical solution instead of the requirement; leaving selection criteria undefined until the responses are in.
- Ignoring cryptographic import/export/use regulation, or assuming one country's position holds elsewhere — *"never put your head in the sand and hope it will go away — it won't."*
- Handling a forensic scene yourself: *"this is not a job for amateur first-timers."*
- Outsourcing security services without settling who owns the risk, who makes policy and who administers it.
- BCM planning that starts from technology recovery rather than from a banded business process impact assessment; a plan built without asking where the single points of failure are.

## Key Takeaways
- Chapter 17 fills the remaining four Operational-row cells of the SABSA Matrix — Security Service Management and Support (Process), Application and User Management and Support (People), Security of Sites, Networks and Platforms (Location), and Security Operations Schedule (Time) — and is the operational parallel to Chapter 11's logical Security Management Services.
- The Operational Prevention Process (Figure 17-1) co-ordinates monitoring and detection, intrusion detection, risk assessment, policy and standards, intelligence gathering, penetration testing and follow-up actions from incidents into one continuous loop.
- ISO/IEC 17799:2000 sections 6–12 supply the operational skeleton: personnel security, physical and environmental security, communications and operations management, access control, systems development and maintenance, business continuity management, compliance.
- People controls carry the load: security in every job description and contract, awareness from recruitment onward, screening with two references and full history verification, supervision that notices lifestyle change, five distinct leaver processes, a formal disciplinary process — and **enforced annual leave of at least one continuous week with the job done by someone else**.
- Dual control needs deputies who cannot perform the other half, exclusion of family and close friends, and enhanced audit trails plus supervisor monitoring where segregation is impractical.
- Change control (seven-point process over configurations, data and procedures, including reversal and post-implementation testing) is not change management (an enterprise-wide change project, of which a BS 7799 ISMS implementation is the worked instance).
- Incident management and problem management are deliberately conflicting duties under ITIL/BS 15000/AS 8018; security incidents need their own classification, escalation, investigation, decision criteria and information-handling rules.
- Development, test and production must be completely segregated, with no code-generating tools of any kind on live machines and emergency fixes diagnosed live but built and tested elsewhere.
- Capacity covers eight dimensions including staffing, accommodation footprint and ancillary services, and is planned from measured mean and standard deviation, trend of spare-capacity consumption, and forecasts against timelines — the 1901 census site under-provisioned by a factor of 24 on demand rate.
- Backups must be verified for content, for restorability, for restore time inside the business day, and for media readability across the full legal retention period — three case studies, three distinct failure modes.
- Event logs need defined event types, five record fields, a FIFO or archive retention model, and tools supporting AND/OR/NOT/XOR pattern search plus statistical analysis.
- Forensics has four goals (chain of evidence, integrity, admissibility, weight) and four process elements (plan, seize, analyse, report) — and should be handed to trained experts with the scene left undisturbed.
- Access control operations turn business authorisations into system authorisations for joiners, movers and leavers; review privileges for decay; remove redundant and vendor default accounts; and encapsulate system-level privilege in tools, with the raw passwords locked in a safe.
- Configuration management adds what an asset register lacks — the relationships between hardware, software, data structures, licences, documentation and people.
- The SLA carries ten security clauses and is itself under change control, but it is only the safety net; business relationship management at business-management level is where the returns are.
- Cryptography is regulated like munitions across export, import, manufacture, domestic use and cross-border movement of ciphertext; Wassenaar (31 countries), the Convention on Cybercrime (43 CoE members, 26 signatories plus four others) and EC 1334/2000 are the co-ordinating instruments, and the operational answer is a per-country register kept proactively current.
- Procurement fails in technical evaluation, not in contracting: run RFI → vendor meetings → RFP, specify requirements not solutions, publish selection criteria in advance, and score them — the Business Attributes Profile turned into an RFP with 0/25/50/75/100% compliance scoring and colour-coded attribute charts decided a bank's system in one glance.
- BCM is business-process-centric in seven steps — impact bands A/B/3/4, functional analysis of Band A and B only, resource analysis, threat scenarios by effect, resilience and single points of failure, continuity planning, risk financing — with PAS 56 as the reference guide.

## Connects To
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — Figure 3-3, the SABSA Matrix whose four remaining Operational-row cells this chapter fills.
- [Chapter 5 — A Systems Approach](ch05-a-systems-approach.md) — the feedback control loop that the Operational Prevention Process, the change control cycle and the problem management cycle all instantiate.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — Figure 6-3, the Business Attributes taxonomy chart reused as the colour-coded vendor evaluation; and the management information dashboard applied to service reporting.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — Figure 7-8, the populated matrix that names these operational cells and their deliverables.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — the Business Attributes Profile used here as an RFP framework; Security Service Management Strategy; Outsourcing Strategy; access control concepts.
- [Chapter 11 — Logical Security Architecture](ch11-logical-security-architecture.md) — Security Management Services and the incident management service (detection and response), whose operational procedures this chapter specifies; Software Integrity Protection.
- [Chapter 12 — Physical Security Architecture](ch12-physical-security-architecture.md) — the Password Management sub-section covering both design and operational issues, referenced directly here.
- [Chapter 13 — Component Security Architecture](ch13-component-security-architecture.md) — the products, tools and standards that the product evaluation and selection process procures.
- [Chapter 14 — Security Policy Management](ch14-security-policy-management.md) — Figure 14-1's hierarchical policy architecture, where a third-party access policy belongs alongside Remote Access and Anti-Virus; access policy management by information and system classification; least privilege and its limitations.
- [Chapter 15 — Operational Risk Management](ch15-operational-risk-management.md) — the broad legal and regulatory compliance context, Gramm-Leach-Bliley, and risk financing, which reappears here as BCM Step 7.
- [Chapter 16 — Assurance Management](ch16-assurance-management.md) — UAT and OAT, the System Assurance Strategy behind development/production segregation, the audit programme that verifies these operations, and BS 7799-2 ISMS implementation as the change management worked instance.
