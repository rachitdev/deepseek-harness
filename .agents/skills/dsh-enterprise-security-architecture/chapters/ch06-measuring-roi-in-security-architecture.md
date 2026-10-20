# Chapter 6 — Measuring Return on Investment in Security Architecture

## Core Idea
Security has a measurement problem baked into it: **if security is good, nothing happens** — there are no incidents — so there is no event to point at as the return. The chapter's answer is to stop trying to measure "security" and instead measure **business virtues**. Since each business risk is the opposite of a business virtue worth protecting, and risk is easier to measure than security, you define the virtues explicitly. In SABSA those virtues are the **85 Business Attributes**, each with a definition, a metric type (hard or soft) and a suggested measurement approach. This makes the enterprise security architecture measurable end to end and gives **two-way traceability** between business drivers and delivered components.

## Frameworks Introduced

### "Return of value", not "return on investment"
"Investment" = spending resources now (money, time, effort) for a later payback. "Return" = the payback. The security paradox: invest, security works, and *nothing happens*. The authors recommend the phrase **return of value** as more honest, while insisting that you must still demonstrate and **measure** the added value if you want to convince senior management.

**The insurance analogy is rejected.** Insurance is not a preventative measure — you buy life insurance precisely because death cannot be avoided, and the policy contributes nothing to the quality or length of your life. Security professionals who use it "set an expectation that nothing of value will happen and thus contribute significantly to the perception of security as a business inhibitor."

### Who needs metrics, and for what
| Group | Their questions |
|---|---|
| Information security group manager | What has the team achieved? Does the team add value? How can I demonstrate how valuable we are? How can I justify the department's budget? How can I motivate the team to achieve more? |
| Individual team members | Are we getting anywhere? Can I feel a sense of achievement to boost morale? Can I see my career developing? Can I measure my achievements at my next staff appraisal? |
| Senior managers | What assurance can be demonstrated that our systems' security is adequate? Can I show due diligence was exerted? Are we ahead, behind, or comfortably mid-pack? Are we fulfilling corporate governance responsibilities? What return am I getting from my security investment? |

Quotations the chapter marshals: Lord Kelvin ("if you cannot measure it, your knowledge is meagre and unsatisfactory"), F. D. Rossini, Galileo ("measure what is measurable and render measurable that which is not yet measurable") and, treated as the most germane, **Peter Drucker: "If you can't measure it, you can't manage it."**

### The Security Management Dashboard (Figure 6-1)
An instrument panel for security management, directly instantiating the Chapter 5 feedback control loop: the panel reports measurements → the manager decides → the manager acts through controls → the controls change the measured parameters → repeat. The metrics you choose *are* the dials.

The dashboard is organised by the **Harvard Business School three domains**, and a successful business operates simultaneously on all three — you cannot afford to neglect any one:
- **Strategic** planning and management — long-term goals and plans; goals take a year or several years; a strategic programme "might have a beginning but never ends"; strategy-focused organisations are in constant change and succeed by managing that change programme.
- **Tactical** planning and management — medium-term goals, several months; tactical programmes have a beginning **and an end**, and you know when one is complete. Projects are typically tactical initiatives moving you toward strategic goals. Sometimes purely tactical action is needed with no strategy, because there is an immediate problem current strategy does not address — the gap is closed later by new strategy.
- **Operational** planning and management — the day-to-day job of keeping the business running, based on repetitive procedures making up business processes, which were themselves most likely developed through tactical projects driven by strategies.

### The Balanced Scorecard (Kaplan & Norton, 1993)
Kaplan (Harvard Business School) and Norton's thesis: traditional financial measures such as return on investment or earnings per share are good only for **reporting results** and are not useful for making the strategic decisions that produce those results. BSC combines financial and operational measures into one framework that "translates a company's strategic objectives into a coherent set of performance measures."

Four perspectives, arranged as a layered architecture model (Figure 6-2), each expressed as **Goal (intention) / Measure (outcome)** pairs:
| Perspective | Question | Typical measures |
|---|---|---|
| **Financial** | What financial results are we looking for? | Cost per unit, revenue growth, profitability, gearing |
| **Customer** | What level of customer service and quality do we need to meet our financial targets? | Subjective (satisfaction surveys) and objective (customer acquisition rates, retention rates) |
| **Internal Process** | What processes do we need to deliver these customer service targets? | Efficiency — time to answer a customer query, fulfil an order, open a new account |
| **Innovation and Learning** | What growth, evolution and change is needed in our organisation, people and processes? | Investment in training, innovation processes |

Benefits claimed: clarify the vision throughout the organisation; gain consensus and ownership by the executive team; provide a framework to align the organisation; provide structure for multiple initiatives; drive the capital and resource allocation process; integrate the strategic management process across the organisation; focus teams and individuals on strategic priorities.

**The five principles of the Strategy-Focused Organisation:** translate strategy into operational terms and performance objectives; align the organisation to the strategy; motivate the people by making strategy everyone's job; adapt to make strategy a continual process of change; mobilise the resources for ongoing change through executive leadership.

### Business drivers, Business Attributes and two-way traceability (Figure 6-3)
- Start by defining the **business drivers** — the things the business needs in terms of security. These are the opening statement of the contextual security architecture.
- Each business risk can be expressed as **the opposite of a business virtue**. In SABSA those virtues are **Business Attributes** (85 at time of writing), each a distillation of practical experience in assessing business risk and classifying it under useful headings.
- Business Attributes can be used **either** as a *pick-list* to prompt your thinking on business drivers, **or** as a *crosscheck for completeness* against drivers derived some other way.
- Mapping need **not** be one-to-one: one driver may map to several attributes and one attribute may serve several drivers.
- The cross-mapping delivers the two directions of traceability:
  - **Justification** — every business driver in the contextual architecture is valid because it is shown to support or increase a business asset or value (the attribute). *"Why are we doing it this way?"* is always answered by tracing back to business requirements.
  - **Completeness** — the attribute list is shown to be complete because each attribute is the conceptualised desired return from at least one active business driver.
- The layered framework "ensures that every component and every operational procedure is there because at the top of the model there is a business driver that eventually is satisfied by these elements."

**Hard vs soft metrics.** *Hard* metrics are good solid objective numbers; *soft* metrics are qualitative and subjective. Not all Business Attributes lend themselves to hard quantitative measurement — many are best measured by "independent audit and review against a Security Architecture Capability Maturity Model," focus groups, satisfaction surveys, or a **red team review** (an objective appraisal by an independent team of experts briefed to think either like the user or like an opponent/attacker, whichever suits the review's objectives).

### The metrics framework procedure (Figure 6-5)
Six steps, applied per business driver:
1. **Business Driver** → 2. **Select Business Attribute(s)** → 3. **Define Business Attribute** (customise the generic definition to your business) → 4. **Define Metric Type** (hard/soft) → 5. **Define Measurement Approach** → 6. **Define Performance Target** → then **Collect & Evaluate Metrics**.

**Reporting independence.** Give careful thought to who receives the metrics. If metrics show failings on the part of a specific manager, delivering them to that manager gives them the opportunity to cover up the failings. Treat the reporting line the way internal audit reports are treated.

### Capability Maturity Models
Developed at the **Carnegie Mellon Software Engineering Institute**. An organisation's expertise in any domain begins immature and passes through levels of maturity as experience accrues. Each level decomposes into **key process areas (PAs)**.

CMM variants supported by Carnegie Mellon SEI: SW-CMM (Software), P-CMM (People), SA-CMM (Software Acquisition), SE-CMM (Systems Engineering), IPD-CMM (Integrated Product Development). SW-CMM level 2 process areas: Requirements Management; Software Project Planning; Software Project Tracking and Oversight; Software Subcontract Management; Software Quality Assurance; Software Configuration Management.

**SSE-CMM** (System Security Engineering CMM) from the International Systems Security Engineering Association (ISSEA), now **ISO/IEC 21827**. It uses a two-dimensional matrix: 22 Process Areas subdivided into Base Practices (the *domain* dimension) crossed with Capability Levels subdivided into Common Features and Generic Practices (the *capability* dimension). Cells are assessed to track progress visually. The authors' honest warning: the model is "a little on the heavy side" — the SSE-CMM Model Description PDF runs to 336 pages — but the CMM concept is good and **you can easily create your own simplified version for internal use**, something they have helped clients do. They also warn that the SSE-CMM's use of "risk" and "exposure" is "very confusing and shows poor clarity of thinking," and that readers should be careful reconciling it with this book's terminology.

**CobiT** (Control Objectives for Information and related Technology), an open standard from the IT Governance Institute under ISACA, identifies **34 ICT processes**, a high-level control approach, and **318 detailed control objectives** plus audit guidelines. Its 34 process areas are classified under four domains: Planning and Organisation; Acquisition and Implementation; Delivery and Support; Monitoring. The CobiT Management Guidelines add a Maturity Model, Critical Success Factors (CSFs), Key Goal Indicators (KGIs) and Key Performance Indicators (KPIs) — the KGIs and KPIs being defined using the Balanced Scorecard approach.

**Applying CMMs to ROI.** The authors report personal experience of applying the CMM structure to measure the **level of strength** (in place of maturity) of **technical security mechanisms** (in place of process areas). Two caveats: it requires senior management sophistication — they must buy into the maturity model as a valid measurement tool, understand what the levels mean, and accept that reaching a given level in a given domain is a valid business goal. And **reaching the top of the scale is not always the right goal**; the skill is using the levels to determine where you want to aim, and why.

### Benchmarking
Measuring the organisation against an external standard: *"How are we doing compared to everyone else?"*, followed by *"Where do we want to get to?"* You measure the external benchmark, your current position, and set a target (Figure 6-7). Benchmarking always depends on a **self-assessment** against your chosen standard.

Benefits, tied directly to the ROI problem: demonstrates added value; builds the business case; justifies expenditure; sets objectives to motivate the team; helps you understand what you are achieving.

## Reference Tables

### Table 6-2: The SABSA Business Attributes Taxonomy — 85 attributes in 7 classes
*(Attribute — definition — metric type — suggested measurement approach. "SA-CMM review" below is shorthand for the book's repeated "Independent audit and review against Security Architecture Capability Maturity Model".)*

**1. User Attributes** — "related to the user's experience of interacting with the business system." (16)
| Attribute | Definition | Type | Suggested measurement |
|---|---|---|---|
| Accessible | Information the user is entitled to access should be easily found and accessed by that user | Soft | Search tree depth necessary to find the info |
| Accurate | Information provided to users should be accurate within a pre-agreed range applicable to the service delivered | Hard | Acceptance testing on key data to demonstrate compliance with design rules |
| Anonymous | For certain specialised services the anonymity of the user should be protected | Hard / Soft | Rigorous proof of system functionality; red team review |
| Consistent | Login, navigation and target services should be presented consistently across different times, locations and channels of access | Hard / Soft | Conformance with design style guides; red team review |
| Current | Information provided should be current and kept up to date within a pre-agreed range | Hard | Refresh rates at the data source and replication of refreshed data to the destination |
| Duty-segregated | For certain sensitive tasks duties should be segregated so no user has access to both aspects of the task | Hard | Functional testing |
| Educated and aware | Users should be educated and trained so they embrace the security culture and have sufficient awareness that their behaviour is policy-compliant | Soft | Competence surveys |
| Informed | The user should be kept fully informed about services, operating procedures, operational schedules, planned outages and so on | Soft | Focus groups or satisfaction surveys |
| Motivated | Interaction with the system should add positive motivation to the user to complete the business tasks in hand | Soft | Focus groups or satisfaction surveys |
| Protected | The user's information and access privileges should be protected against abuse by other users or by intruders | Soft | Penetration test — *could be regarded as hard, but only if a penetration is achieved; failure to penetrate does not mean penetration is impossible* |
| Reliable | Services should be delivered at a reliable level of quality | Soft | A definition of "quality" is needed against which to compare |
| Responsive | Users obtain a response within a satisfactory period that meets their expectations | Hard | Response time |
| Supported | When a user has problems there should be a means to receive advice and support so problems are resolved to the user's satisfaction | Soft | Focus groups or satisfaction surveys; SA-CMM review |
| Timely | Information is delivered or made accessible at the appropriate time or within the appropriate period | Hard | Refresh rates at the data source and replication of refreshed data to the destination |
| Transparent | Full visibility to the user of the logical process while hiding the physical structure (as a URL hides the actual physical locations of web servers) | Soft | Focus groups or satisfaction surveys; SA-CMM review |
| Usable | Easy-to-use interfaces navigable intuitively by a user of average intelligence and training for the given system; the experience should be at best interesting and at worst neutral | Soft | Numbers of clicks or keystrokes required; conformance with industry standards (e.g. colour palettes); feedback from focus groups |

**2. Management Attributes** — "the ease and effectiveness with which the business system and its services can be managed." (8)
| Attribute | Definition | Type | Suggested measurement |
|---|---|---|---|
| Automated | Wherever possible, and depending on cost/benefit, management and operation should be automated | Soft | Independent design review |
| Change-managed | Changes should be properly managed so the impact of every change is evaluated and changes are approved in advance of implementation | Soft | Documented change management system with change management history, evaluated by independent audit |
| Controlled | The system should at all times remain in the control of its managers — observe operation and behaviour, decide how to control based on those observations, implement actions to exert that control | Soft | SA-CMM review |
| Cost-effective | Design, acquisition, implementation and operation at a cost the business finds acceptable judged against the benefits derived | Hard | Individual budgets for the phases of development and for ongoing operation, maintenance and support |
| Efficient | Deliver target services with optimum efficiency, avoiding wastage of resources | Hard | A target efficiency ratio based on (INPUT VALUE)/(OUTPUT VALUE) |
| Maintainable | Capable of being maintained in a state of good repair and effective, efficient operation, with the required actions feasible within normal operational conditions | Soft | Documented execution of a preventive maintenance schedule for hardware and software, correlated against continuity-of-service targets such as MTBF |
| Measured | Performance against a variety of desirable targets should be measured to provide feedback supporting management and control | Hard | Documented tracking and reporting of a portfolio of conventional system performance parameters, together with other Business Attributes from this list |
| Supportable | Capable of being supported for both users and operations staff so all types of problems and operational difficulties can be resolved | Hard | Fault-tracking system providing MTBF, MTTR and maximum time to repair, with targets for each parameter |

**3. Operational Attributes** — "the ease and effectiveness with which the business system and its services can be operated." (8)
| Attribute | Definition | Type | Suggested measurement |
|---|---|---|---|
| Available | Information and services available according to the requirements specified in the SLA | Hard | As specified in the SLA |
| Continuous | The system should offer "continuous service"; the exact definition is always subject to an SLA | Hard | Percentage up-time correlated versus scheduled and/or unscheduled downtime; or MTBF, or MTTR |
| Detectable | Important events must be detected and reported | Hard | Functional testing |
| Error-free | The system should operate without producing errors | Hard | Percentage or absolute error rates (per transaction, per batch, per time period) |
| Inter-operable | Should inter-operate with other similar systems, both immediately and in future | Hard | Specific inter-operability requirements |
| Monitored | Operational performance continuously monitored to ensure other attribute specifications are met; deviations from acceptable limits notified to systems management | Soft | SA-CMM review |
| Productive | Operate so as to sustain and enhance user productivity with regard to the business processes users are engaged in | Hard | User output targets related to specific business activities |
| Recoverable | Able to be recovered to full operational status after a breakdown or disaster in accordance with the SLA | Hard | As specified in the SLA |

**4. Risk Management Attributes** — "the business requirements for mitigating operational risk… most closely relates to the security requirements for protecting the business." (19)
| Attribute | Definition | Type | Suggested measurement |
|---|---|---|---|
| Access-controlled | Access to information and functions controlled per the authorised privileges of the requesting party; unauthorised access prevented | Hard | Reporting of all unauthorised access attempts — number per period, severity, and result (did it succeed?) |
| Accountable | All parties with authorised access held accountable for their actions | Soft | SA-CMM review with respect to the ability to hold all authorised parties accountable |
| Assurable | A means to provide assurance that the system operates as expected and all controls are correctly implemented and operated | Hard / Soft | Documented standards exist against which to audit; SA-CMM review |
| Assuring honesty | Employees protected against false accusations of dishonesty or malpractice | Soft | SA-CMM review with respect to the ability to prevent false accusations that are difficult to repudiate |
| Auditable | Actions of all authorised parties and the complete chain of events and outcomes recorded so the history can be reviewed, at a level of detail matching business needs; the actual configuration must also be auditable against a target configuration representing the governing security policy | Soft / Hard | SA-CMM review; documented target configuration under change control with capability to check current configuration against it |
| Authenticated | Every party claiming a unique identity (a claimant) subject to a procedure verifying it is the authentic owner of the claimed identity | Soft | SA-CMM review with respect to the ability to successfully authenticate every claim of identity |
| Authorised | The system should allow only actions that have been explicitly authorised | Hard / Soft | Reporting of all unauthorised actions — number per period, severity, result; SA-CMM review of the ability to detect unauthorised actions |
| Capturing new risks | New risks emerge over time; management and operational environment should provide a means to identify and assess new risks (new threats, impacts or vulnerabilities) | Hard / Soft | Percentage of vendor-published patches and upgrades actually installed; SA-CMM review of a documented risk assessment process and risk assessment history |
| Confidential | Confidentiality of corporate information protected per security policy; unauthorised disclosure prevented | Hard | Reporting of all disclosure incidents — number per period, severity, type of disclosure |
| Crime-free | Cyber-crime of all types should be prevented | Hard | Reporting of all incidents of crime — number per period, severity, type of crime |
| Flexibly secure | Security can be provided at various levels per business need; the system should secure information according to those needs and may need different levels for different information types (per security classification) | Soft | SA-CMM review |
| Identified | Each entity granted access and each object that is itself a resource uniquely identified (named) so there can never be confusion about which entity or object is referenced | Hard | Proof of uniqueness of naming schemes |
| Independently secure | Security should not rely on the security of any other system outside this system's direct span of control | Soft | SA-CMM review of technical security architecture at conceptual, logical and physical layers |
| In our sole possession | Information of value should be in the possession of the business, stored and protected against loss (no longer available) or theft (disclosed to an unauthorised party), including intellectual property | Soft | SA-CMM review |
| Integrity-assured | Integrity protected to assure no unauthorised modification, duplication or deletion | Hard / Soft | Reporting of all compromise incidents — number per period, severity, type; SA-CMM review of the ability to detect integrity compromise incidents |
| Non-repudiable | When one party sends a message to another it should NOT be possible for the sender to falsely deny sending it or falsely deny its contents | Hard / Soft | Reporting of all incidents of unresolved repudiations — number per period, severity, type; SA-CMM review of the ability to prevent repudiations that cannot be easily resolved |
| Owned | Every system should have a designated owner entity — the policymaker for all risk management aspects who exerts ultimate authority for controlling the system | Soft | SA-CMM review of the ownership arrangements, of the management processes by which owners fulfil their responsibilities, and of their diligence in doing so |
| Private | Privacy of personal information protected per relevant privacy or data protection legislation and so as to meet citizens' reasonable expectation of privacy; unauthorised disclosure prevented | Hard | Reporting of all disclosure incidents — number per period, severity, type of disclosure |
| Trustworthy | Able to be trusted to behave in the ways specified in its functional specification and to protect against a wide range of potential abuses | Soft | Focus groups or satisfaction surveys researching around the question "Do you trust the service?" |

**5. Legal & Regulatory Attributes** — "mitigating operational risks that have a specific legal or regulatory connection." (9)
| Attribute | Definition | Type | Suggested measurement |
|---|---|---|---|
| Admissible | Provide forensic records (audit trails and so on) deemed admissible in a court of law should evidence be needed for criminal prosecution or civil litigation | Soft | SA-CMM review by a computer forensics expert |
| Compliant | Comply with all applicable regulations, laws, contracts, policies and mandatory standards, internal and external | Soft | Independent compliance audit with respect to the inventories of regulations, laws, policies, etc. |
| Enforceable | Designed, implemented and operated such that all applicable contracts, policies, regulations and laws can be enforced by the system | Soft | Independent review of (1) inventory of contracts, policies, regulations and laws for completeness, (2) enforceability of those on the inventory |
| Insurable | Risk-managed so an insurer can offer reasonable commercial terms against a standard range of insurable risks | Hard | Verify against insurance quotations |
| Legal | Designed, implemented and operated per applicable legislation — e.g. data protection laws, laws controlling use of cryptographic technology, laws controlling insider dealing on the stock market, laws governing racist, seditious or pornographic information | Soft | SA-CMM review; verification of the inventory of applicable laws for completeness and suitability |
| Liability-managed | Designed, implemented and operated to manage the organisation's liability regarding errors, fraud, malfunction; responsibilities and liabilities of each party clearly defined | Soft | Independent legal expert review of all applicable contracts, SLAs, etc. |
| Regulated | Designed, implemented and operated per applicable regulations, general (e.g. safety) or industry-specific (e.g. banking) | Soft | SA-CMM review; verification of the inventory of applicable regulations for completeness and suitability |
| Resolvable | Designed, implemented and operated so disputes can be resolved with reasonable ease and without undue impact on time, cost or other valuable resources | Soft | SA-CMM review by a legal expert |
| Time-bound | Meeting requirements for maximum or minimum periods of time — e.g. a minimum records retention period or a maximum period within which something must be completed | Hard | Independent functional design review against specified functional requirements |

**6. Technical Strategy Attributes** — "the needs for fitting into an overall technology strategy." (13)
| Attribute | Definition | Type | Suggested measurement |
|---|---|---|---|
| Architecturally open | Wherever possible not locked into specific vendor interface standards; allows flexibility in choice of vendors and products, initially and in future | Soft | SA-CMM review of technical architecture (conceptual, logical, physical) |
| COTS and GOTS compliant | Wherever possible use commercial off-the-shelf or government off-the-shelf components, as appropriate | Soft | SA-CMM review of technical architecture |
| Extendable | Capable of being extended to incorporate new functional modules as required by the business | Soft | SA-CMM review of technical architecture |
| Flexible and Adaptable | Flexible and adaptable to meet new business requirements as they emerge | Soft | SA-CMM review of technical architecture |
| Future-proof | Architecture designed as much as possible to accommodate future changes in both business requirements and technical solutions | Soft | SA-CMM review of technical architecture |
| Legacy-sensitive | A new system should work with any legacy systems or databases with which it must inter-operate or integrate | Soft | SA-CMM review of technical architecture |
| Migrateable | A feasible, manageable migration path acceptable to business users, from an old system to a new one or one released version to the next | Soft | SA-CMM review of technical architecture |
| Multi-sourced | Critical components obtainable from more than one source, protecting against withdrawal of a single source of supply and support | Soft | SA-CMM review of technical architecture at the component level |
| Scaleable | Scaleable to the size of user community, data storage requirements, processing throughput and so on that might emerge over the system's lifetime | Soft | SA-CMM review of technical architecture |
| Simple | As simple as possible, since complexity only adds further risk | Soft | SA-CMM review of technical architecture |
| Standards-compliant | Designed, implemented and operated to comply with appropriate technical and operational standards | Soft | Independent audit and review of (1) the inventory of standards for completeness and appropriateness, (2) compliance with standards on the inventory |
| Traceable | Development and implementation documented to provide complete two-way traceability: every implemented component justifiable by tracing back to the business requirement that led to its inclusion, and every business requirement reviewable to show which components meet it | Soft | Independent expert review of documented traceability matrices and trees |
| Upgradeable | Capable of being upgraded with ease to incorporate new releases of hardware and software | Soft | SA-CMM review of technical architecture |

**7. Business Strategy Attributes** — "the needs for fitting into an overall business strategy." (12)
| Attribute | Definition | Type | Suggested measurement |
|---|---|---|---|
| Brand-enhancing | Should help establish, build and support the brand of the products or services based on this system | Soft | Market surveys |
| Business-enabled | Enabling the business and fulfilling business objectives should be the primary driver for the system design | Soft | Business management focus group |
| Competent | Protect the organisation's reputation as being competent in its industry sector | Soft | Independent audit, or focus groups, or satisfaction surveys |
| Confident | Behave so as to safeguard confidence placed in the organisation by customers, suppliers, shareholders, regulators, financiers, the marketplace and the general public | Soft | Independent audit, or focus groups, or satisfaction surveys |
| Credible | Behave so as to safeguard the credibility of the organisation | Soft | Independent audit, or focus groups, or satisfaction surveys |
| Culture-sensitive | Designed, built and operated with due care and attention to cultural issues for anyone who will experience the system in any way — religion, gender, race, nationality, language, dress code, social customs, ethics, politics and the environment; the objective is to avoid or minimise offence or distress caused to others | Soft | Independent audit and review of (1) the inventory of requirements in this area for completeness and appropriateness, (2) compliance of system functionality with that set of requirements |
| Enabling time-to-market | Architecture and design should allow new business initiatives to be delivered to the market with minimum delay | Soft | Business management focus group |
| Governable | Enable owners and executive managers to control the business and discharge their responsibilities for governance | Soft | Senior management focus group; SA-CMM review for governance |
| Providing good stewardship and custody | Protect other parties with whom we do business from abuse or loss of business or personal information of value to them through inadequate stewardship on our part | Soft | Independent audit, or focus groups, or satisfaction surveys |
| Providing investment re-use | As much as possible re-use previous investments and ensure new investments are re-usable in future | Soft | SA-CMM review of technical architecture (conceptual, logical, physical and component) |
| Providing return on investment | Provide a return of value to the business to justify the investment made in creating and operating the system | Hard / Soft | Financial returns and ROI indices selected in consultation with the CFO; qualitative value propositions tested by opinion surveys at senior management and boardroom level |
| Reputable | Behave so as to safeguard the business reputation of the organisation | Soft / Hard | Independent audit, or focus groups, or satisfaction surveys; correlation of the organisation's stock value versus publicity of system event history |

### Table 6-6: The Carnegie Mellon Capability Maturity Model
| Maturity stage | Description |
|---|---|
| **Initial** | The process is characterised as ad hoc, and occasionally even chaotic. Few processes are defined, and success depends on individual effort and heroics. |
| **Repeatable** | Basic project management processes are established to track cost, schedule and functionality. The necessary process discipline is in place to repeat earlier successes on projects with similar applications. |
| **Defined** | The process for both management and engineering activities is documented, standardised and integrated into a standard process for the organisation. All projects use an approved, tailored version of the organisation's standard process for development and maintenance. |
| **Managed** | Detailed measures of the process and product quality are collected. Both the process and products are quantitatively understood and controlled. |
| **Optimising** | Continuous process improvement is enabled by quantitative feedback from the process and from piloting innovative ideas and technologies. |

### Table 6-7: SSE-CMM Process Areas
**Security Engineering:** PA01 Administer Security Controls · PA02 Assess Impact · PA03 Assess Security Risk · PA04 Assess Threat · PA05 Assess Vulnerability · PA06 Build Assurance Argument · PA07 Coordinate Security · PA08 Monitor Security Posture · PA09 Provide Security Input · PA10 Specify Security Needs · PA11 Verify and Validate Security
**Project and Organisational:** PA12 Ensure Quality · PA13 Manage Configuration · PA14 Manage Project Risk · PA15 Monitor and Control Technical Effort · PA16 Plan Technical Effort · PA17 Define Organization's Systems Engineering Process · PA18 Improve Organization's Systems Engineering Process · PA19 Manage Product Line Evolution · PA20 Manage Systems Engineering Support Environment · PA21 Provide Ongoing Skills and Knowledge · PA22 Coordinate with Suppliers

### Table 6-8: SSE-CMM Base Practices example — PA03 Assess Security Risk
Goals: (1) an understanding of the security risk associated with operating the system within a defined environment is achieved; (2) risks are prioritised according to a defined methodology.
- BP.03.01 Select the methods, techniques and criteria by which security risks, for the system in a defined environment, are analysed, assessed and compared
- BP.03.02 Identify threat/vulnerability/impact triples (exposures)
- BP.03.03 Assess the risk associated with the occurrence of an exposure
- BP.03.04 Assess the total uncertainty associated with the risk for the exposure
- BP.03.05 Order risks by priority
- BP.03.06 Monitor ongoing changes in the risk spectrum and changes to their characteristics

*A Process Area:* assembles related activities in one area for ease of use; relates to valuable security engineering services; applies across the enterprise lifecycle; can be implemented in multiple organisation and product contexts; can be improved as a distinct process; can be improved by a group with similar interests in the process; includes all base practices required to meet its goals.
*A Base Practice:* applies across the enterprise lifecycle; does not overlap with other base practices; represents a best practice of the security community; does not simply reflect a state-of-the-art technique; is applicable using multiple methods in multiple business contexts; does not specify a particular method or tool.

### Table 6-9: SSE-CMM Capability Levels, Common Features and Generic Practices
| Level | Description | Common Features / Generic Practices |
|---|---|---|
| 1 | Performed Informally | 1.1 Base Practices are Performed — GP 1.1.1 Perform the Process |
| 2 | Planned and Tracked | 2.1 Planning Performance — GP 2.1.1 Allocate Resources, 2.1.2 Assign Responsibilities, 2.1.3 Document the Process, 2.1.4 Provide Tools, 2.1.5 Ensure Training, 2.1.6 Plan the Process · 2.2 Disciplined Performance — GP 2.2.1 Use Plans, Standards and Procedures, 2.2.2 Do Configuration Management · 2.3 Verifying Performance — GP 2.3.1 Verify Process Compliance, 2.3.2 Audit Work Products · 2.4 Tracking Performance — GP 2.4.1 Track with Measurement, 2.4.2 Take Corrective Action |
| 3 | Well-Defined | 3.1 Defining a Standard Process — GP 3.1.1 Standardize the Process, 3.1.2 Tailor the Standard Process · 3.2 Perform the Defined Process — GP 3.2.1 Use a Well-Defined Process, 3.2.2 Perform Defect Reviews, 3.2.3 Use Well-Defined Data · 3.3 Coordinate the Process — GP 3.3.1 Perform Intra-Group Coordination, 3.3.2 Perform Inter-Group Coordination, 3.3.3 Perform External Coordination |
| 4 | Quantitatively Controlled | 4.1 Establishing Measurable Quality Goals — GP 4.1.1 Establish Quality Goals · 4.2 Objectively Managing Performance — GP 4.2.1 Determine Process Capability, 4.2.2 Use Process Capability |
| 5 | Continuously Improving | 5.1 Improving Organizational Capability — GP 5.1.1 Establish Process Effectiveness Goals, 5.1.2 Continuously Improve the Standard Process · 5.2 Improving Organisational Effectiveness — GP 5.2.1 Perform Causal Analysis |

### Table 6-11: The Maturity Levels in the CobiT CMM (six levels, 0–5)
| Level | Title | Description |
|---|---|---|
| 0 | Non-Existent | Complete lack of any recognisable processes. The organisation has not even recognised that there is an issue to be addressed. |
| 1 | Initial | Evidence the organisation has recognised the issues exist and need addressing. No standardised processes; ad hoc approaches applied individually or case by case. The overall approach to management is disorganised. |
| 2 | Repeatable | Processes have developed to the stage where different people undertaking the same task follow similar procedures. No formal training or communication of standard procedures; responsibility left to the individual. High reliance on individuals' knowledge, therefore errors are likely. |
| 3 | Defined | Procedures standardised, documented and communicated through training. It is left to the individual to follow them and unlikely deviations will be detected. The procedures are not sophisticated but formalise existing practices. |
| 4 | Managed | Possible to monitor and measure compliance with procedures and act where processes appear not to be working effectively. Processes under constant improvement and provide good practice. Automation and tools used in a limited or fragmented way. |
| 5 | Optimised | Processes refined to best-practice level based on continuous improvement and maturity modelling with other organisations. ICT used in an integrated way to automate workflow, providing tools to improve quality and effectiveness, making the enterprise quick to adapt. |

**The CobiT four benchmarking measurements** (Figure 6-6), assessed under each of the 34 process areas: the current status of the organisation (where you are today); the current status of best-in-class in the industry (comparison benchmark); the current status of international standard guidelines (additional benchmark); the organisation's strategy for improvement (where you want to be).

### External benchmarks the chapter names
- **Best/good practice baseline standards:** ISO/IEC 17799 (Code of Practice for Information Security Management); ISO/IEC 21827 (SSE-CMM); CobiT.
- **Legislative and regulatory:** Government Information Security Reform Act (US); Directive on Data Protection and aligned national legislation (EU); Directive on Privacy and Electronic Communications and aligned national legislation (EU); Health Insurance Portability and Accountability Act 1996 (US); Gramm Leach Bliley Act (US); Sarbanes-Oxley Act (Public Company Accounting Reform and Investor Protection Act of 2002, US); Basel II New Basel Capital Accord (banking).
- **Surveys and member publications:** ISF (Information Security Forum) Survey; I-4 (International Information Integrity Institute) publications and reports; CSI (Computer Security Institute) annual survey and IPAK Self-Assessment Kit.

## Worked Example — IBFS Real-Time Settlement System (RTSS)

IBFS is introducing a new back-office computer system to carry out **real-time settlement of front-office transactions**. Several brainstorming workshops collected all business drivers for security in this system, collated as **Table 6-1: 43 business drivers, BD1–BD43**.

**Table 6-1 — the 43 IBFS RTSS business drivers (verbatim in substance):**
| # | Business driver |
|---|---|
| BD1 | Protecting the reputation of IBFS, ensuring that it is perceived as competent in its sector |
| BD2 | Providing support to the claims made by IBFS about its competence to carry out its intended functions |
| BD3 | Protecting the trust that exists in business relationships and propagating that trust across remote electronic business communications links and distributed information systems |
| BD4 | Maintaining the confidence of other key parties in their relationships with IBFS |
| BD5 | Maintaining the operational capability of IBFS's systems |
| BD6 | Maintaining the continuity of service delivery, including the ability to meet the requirements of service level agreements |
| BD7 | Maintaining the accuracy of information |
| BD8 | Maintaining the ability to govern |
| BD9 | Preventing losses through financial fraud |
| BD10 | Detecting attempted financial fraud |
| BD11 | Providing the ability to prosecute those who attempt to defraud IBFS |
| BD12 | Ensuring that solutions for securing electronic business services include a clear and unambiguous definition of responsibilities and liabilities for all parties at every stage of the transaction |
| BD13 | Providing and maintaining the ability to resolve disputes between IBFS and any other parties, quickly, efficiently and with minimum cost |
| BD14 | Ensuring information processed in IBFS's systems can be brought to a court of law as evidence in criminal and civil proceedings, that the court will admit it, and that it will withstand hostile criticism by the other side's expert witnesses |
| BD15 | Ensuring the information security approaches used directly support compliance by IBFS with commercial contracts to which IBFS is a party |
| BD16 | Ensuring IBFS is at all times compliant with laws and industry sector regulations, and that the information security approach directly and indirectly supports legal compliance |
| BD17 | Maintaining the privacy of personal and business information stored, processed and communicated by IBFS's systems |
| BD18 | Protecting against deliberate, accidental or negligent corruption of personal and business information stored, processed and communicated |
| BD19 | Ensuring an entity that makes a business transaction cannot later deny having made it, and will be bound by the contractual obligations associated with it |
| BD20 | Ensuring all users can be held accountable for the actions they take in making use of their access privileges |
| BD21 | Ensuring access privileges are designed and implemented so as to minimise the risk of a single individual having excessive power that could be abused without easily being detected |
| BD22 | Providing a means by which IBFS can monitor compliance with its information security policies and can detect, investigate and remedy attempted or actual violations |
| BD23 | Providing assurance of the correct functioning of IBFS's systems and sub-systems |
| BD24 | Providing for policy setting and the control and monitoring of policy compliance by the authorities vested with responsibility for corporate governance in the system environment |
| BD25 | Protecting other parties with whom IBFS has business dealings from abuse, loss of business or personal information |
| BD26 | Ensuring employees using the system are only granted authorised access within need-to-know and need-to-use privileges |
| BD27 | Ensuring the system security solution is cost-effective and provides good value for money |
| BD28 | Ensuring the security of IBFS's information depends only on its own system security measures and not on the security competence of any other organisation |
| BD29 | Ensuring the granularity of system security services is appropriate to business need |
| BD30 | Preserving the ability of authorised business users to maintain a high level of productivity |
| BD31 | Ensuring information security interfaces are easy and simple to use |
| BD32 | Utilising, where possible, commercial off-the-shelf products to build information security solutions |
| BD33 | Ensuring security services can be extended to all user locations, all interface types and across all network types used to support delivery |
| BD34 | Maximising the economic advantage of the enterprise security architecture |
| BD35 | Supporting security services through electronic communications, without the need for physical transfer of documents or storage media |
| BD36 | Ensuring system security solutions comply as far as possible with internal and external standards and best practices |
| BD37 | Ensuring the security architecture is independent of any specific vendor or product and is capable of supporting multiple products from multiple vendors |
| BD38 | Ensuring the security architecture remains compatible with new technical solutions as they evolve and with new business requirements as they emerge, with a minimum of redesign |
| BD39 | Adapting the security architecture to counter new threats and vulnerabilities as they are discovered |
| BD40 | Ensuring the required internal and external cultural shift is achieved to support the security architecture |
| BD41 | Ensuring accurate information is available when needed |
| BD42 | Minimising the risk of loss of key customer relationships |
| BD43 | Minimising the risk of excessive loading on insurance premiums due to negligence on IBFS's behalf or lack of due diligence |

**Table 6-3 — drivers mapped to attributes (justification direction):**
| Business driver | Supporting Business Attributes |
|---|---|
| BD1 | Credible, Reputable |
| BD8 | Controlled, Governable |
| BD17 | Access Controlled, Authenticated, Confidential, Identified, Private |

**Table 6-4 — attributes mapped back to drivers (completeness direction):**
| Business Attribute | Associated business drivers |
|---|---|
| Private | BD17 |
| Informed | BD5, BD30, BD31 |
| Non-repudiable | BD3, BD4, BD13, BD14, BD19 |

**Figure 6-5 — the six-step metrics framework worked through for "Private":**
1. *Business driver:* "Maintaining the privacy of personal and business information that is stored, processed and communicated by IBFS's systems." (BD17)
2. *Select Business Attribute(s):* **Private**.
3. *Define Business Attribute* (customised from the generic Table 6-2 definition): "The privacy of customer information should be protected in accordance with relevant privacy or 'Data Protection' legislation in each country where IBFS operates, and so as to meet the reasonable expectations of the customers for privacy of their information. Unauthorised disclosure should be prevented **and attempted unauthorised disclosures should be reported**."
4. *Define metric type:* **Hard** — based on the number of reported incidents involving unauthorised disclosure of customer information, **including unsuccessful attempts**.
5. *Define measurement approach:* measure the number of incidents per period and classify each incident by type and severity; also measure the ability to report incidents.
6. *Define performance targets:*
   - Target 1: set maximum number of allowable disclosures (**= 0**)
   - Target 2: set maximum elapsed time (in minutes) for an attempted incident to be reported
   - Target 3: set a regular reporting cycle for summaries of incidents by type and severity
7. *Collect & evaluate metrics:* number of actual disclosures; maximum, minimum and average reporting time for incidents; periodic summaries and analyses of incidents.

**Table 6-5 — the resulting IBFS RTSS metrics framework entries:**
| Attribute | Driver(s) | Type | Measurement approach | Performance target |
|---|---|---|---|---|
| Private | BD17 | Hard | Reporting of all unauthorised disclosure incidents — number per period, severity and type of disclosure | Zero successful attempts at unauthorised disclosure. Alerts of unauthorised access attempts produced and delivered to `<systems manager name>` and `<business owner name>` within `<X>` minutes. Summary reports of number, severity and type of unauthorised access attempts to private data produced and delivered to those two roles monthly. |
| Private | BD17 | Soft | Independent audit and review with respect to the prevention of unauthorised disclosure of private information | System passes review by `<independent legal and forensic authority>` to a degree deemed acceptable by `<Group Legal>` to prevent prosecution under EU Data Protection legislation |
| Non-repudiable | BD5, BD30, BD31 *(as printed in Table 6-5)* | Hard | Reporting of all incidents of unresolved repudiations — number per period, severity, type | Exception report detailing all incidents of repudiation produced and delivered to `<business owner name>` for validation. Audit trails recording the detail of all transaction-based information required to provide non-repudiable proof and accountability, available to `<business owner name>` on demand. |
| Non-repudiable | as above | Soft | Independent audit and review with respect to the ability to prevent repudiations that cannot be easily resolved | Independent audit and review by `<Group Internal Audit / Group Legal>` with respect to the ability to provide proof of the transaction flow and individual accountability on all transactions |
| Informed | BD3, BD4, BD13, BD14, BD19 *(as printed in Table 6-5)* | Hard | Awareness programme | Adherence to quarterly awareness programme plan produced by `<ebusiness operations manager name>` and agreed with `<ebusiness owner name>` |
| Informed | as above | Soft | Focus groups or satisfaction surveys | Monthly report on all customer feedback relating to level of awareness produced and delivered to `<ebusiness owner name>` and `<ebusiness operations manager name>`. Report from quarterly customer and non-customer focus groups delivered to `<ebusiness owner name>`. |

*Note on the source text: Tables 6-4 and 6-5 print the driver mappings for "Non-repudiable" and "Informed" in opposite order to one another. Table 6-4 (Non-repudiable → BD3, BD4, BD13, BD14, BD19; Informed → BD5, BD30, BD31) is the mapping consistent with the wording of those drivers; Table 6-5's swap appears to be a typesetting transposition in the printed book. Both are reproduced above as printed.*

## Mental Models
- **Measure the virtue, not the absence of the vice.** Risk is the negative of a Business Attribute; attributes are measurable, "security" is not.
- **The dashboard, not the report.** Metrics exist to close the feedback control loop, not to be filed.
- **Two-way traceability is the audit answer.** Down the model = completeness; up the model = justification. Any "why are we doing this?" is answered by walking upward.
- **Hard where you can, soft where you must** — and be honest about which. The Protected attribute footnote is the model of this honesty: a penetration test is only a hard metric if a penetration is achieved; *failure to penetrate does not mean penetration is impossible.*
- **You do not have to reach level 5.** Use maturity levels to decide where on the scale you want to be, and why.

## Anti-patterns
- Selling security as insurance — it establishes the expectation that nothing of value will happen.
- Presenting security metrics with no defined performance targets.
- Sending metrics that expose a manager's failings to that same manager.
- Adopting the full SSE-CMM (336 pages) when a simplified internal version would serve.
- Aiming for the top maturity level by default.
- Treating the 85-attribute taxonomy as closed — the authors state it is "almost certainly not complete," that one or two attributes are typically added per client assignment, and that you should define your own where you find a gap.

## Key Takeaways
- Prefer "return of value" to "return on investment," but still measure it.
- 85 Business Attributes in 7 classes: User (16), Management (8), Operational (8), Risk Management (19), Legal & Regulatory (9), Technical Strategy (13), Business Strategy (12).
- Use the taxonomy either as a pick-list for drivers or as a completeness crosscheck; mapping is many-to-many.
- The metrics framework: driver → attribute(s) → customised definition → metric type → measurement approach → performance target → collect and evaluate.
- Balanced Scorecard (4 perspectives, goal/measure pairs), CMMs (Carnegie Mellon 5-level, SSE-CMM/ISO 21827 22 PAs, CobiT 6-level 0–5 with 34 processes and 318 control objectives) and external benchmarking are the complementary approaches.
- Keep the metrics reporting line independent, as with internal audit.

## Connects To
- [Chapter 5 — A Systems Approach](ch05-a-systems-approach.md) — the feedback control loop the dashboard implements, and abstract system attributes as normalised objectives.
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — the Business Attributes Profile is the Conceptual/Assets cell.
- [Chapter 4 — Case Study: IBFS](ch04-case-study-ibfs.md) — the company behind the RTSS.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — the Manage and Measure phase where these metrics are consumed.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — business drivers as the opening statement of the contextual layer.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — building the Business Attributes Profile with control objectives.
- [Chapter 15 — Operational Risk Management](ch15-operational-risk-management.md) — the risk management dashboard.
