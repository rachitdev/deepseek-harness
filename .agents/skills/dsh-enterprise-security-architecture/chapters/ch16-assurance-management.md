# Chapter 16 — Assurance Management

## Core Idea
This chapter fills the SABSA Matrix cell **Assurance of Operational Continuity — Operational layer, Assets (What) column** (Chapter 3, Figure 3-3; Chapter 7, Figure 7-8), and "Framework for Assurance of Operational Continuity" appears as a key deliverable of the operational security architecture in Chapter 7, Figure 7-12.

The definition of assurance is deliberately narrow and mechanical:

> "Within the SABSA methodology that means that you need assurance that **all of the Business Attributes in your SABSA Business Attributes Profile are being provided at a level compatible with the performance targets that you have set for each one** (see Chapter 6)."

Assurance is therefore the operational counterpart of the **Inspector's View** introduced in Chapter 3 — *"concerned with providing assurance that the architecture is complete, consistent, robust and fit-for-purpose in every way."* And assurance is always delivered by **inspection of some sort**; the chapter's structure is the taxonomy of inspection activities.

## Frameworks Introduced

### The inspection taxonomy — the whole chapter in one list
| Inspection activity | Against what |
|---|---|
| **Security audits and reviews of an organisational unit** | A suitable code of practice, such as ISO/IEC 17799 |
| **Security audits and reviews of a system** | A set of pre-determined security standards deemed appropriate under the security policy (Chapter 14) |
| **System assurance through controls** | Systems development · systems operations (especially data-centre production facilities) · systems integrity protection against malicious software · systems use by the wider community of business users |
| **Functional testing in the development lifecycle** | Unit testing · integration testing · system validation testing · user acceptance testing · operational acceptance testing |
| **Quality assurance of application systems** | Code reviews · coverage validation for each phase of testing |
| **Penetration testing** | Systems under test before release into production · operational production systems |

### The audit programme — thirteen things it must address
Establishing the business drivers for security and for security audit · defining the general approaches and strategy for security audit · levels of assurance required as a result of an audit · setting out pre-audit work · **defining security targets (how much security is needed)** · establishing the audit baseline · designing the audit work program · system assessment and analysis using both automated tools and interviews with key staff · penetration testing (probing for weaknesses, including by the use of social engineering) · assessing the systems and software development environment · assessing the operational environment · security audit reporting · **responding to audit reports and tracking progress**.

### CobiT as an audit framework
CobiT (Control Objectives for Information and Related Technology), from the **IT Governance Institute**, associated with **ISACA** (Information Systems Audit and Control Association) and **ISACF** (Information Systems Audit and Control Foundation).

> "CobiT is not just about the security of information systems. It has a much broader scope, dealing with all aspects of governance of information systems. For some organisations this will be a more appropriate approach to ICT audit than, for example, ISO/IEC 17799, which is specifically security-focused."

Its structural core:
- **Four domains**
- **34 high-level control objectives** — one for each ICT process
- **318 specific, detailed control objectives** (control activities)
- **Seven information criteria**: effectiveness, efficiency, confidentiality, integrity, availability, compliance, reliability
- **ICT resources**: people, applications, technology, facilities, data

### ISO/IEC 17799:2000 and BS 7799-2:2002 as an audit framework
> "It is important to understand that ISO/IEC 17799:2000 addresses topics in terms of policies and general good practices. The document specifically identifies itself as **'a starting point for developing organisation-specific guidance.'** … It is not intended to give definitive details on how to do things — rather it is a general guide to what you should do… **ISO/IEC 17799:2000 is a management standard, not a technical standard.**"

Some people pedantically argue that as a code of practice it is not really a standard — *"but what is important is that it is a common internationally recognised framework."*

**The two-part British Standard:**
- **BS 7799-1:1999** — Code of Practice for Information Security Management (same content as ISO/IEC 17799:2000)
- **BS 7799-2:2002** — Information Security Management Systems: Specification with Guidance for Use (a model process for setting up and managing an ISMS)

*"It seems that at the time of writing there is no intention on the part of ISO/IEC JTC1 to generate the ISO/IEC equivalent of BS 7799-2."* The chapter's closing footnote drives the point home: *"ISO/IEC 17799:2000 covers only BS 7799-1:1999 and thus technically is not suited to the assurance of the Information Security Management System (ISMS) as described in BS 7799-2:2002."*

**The PDCA model (Figure 16-1)** — BS 7799-2:2002's Plan-Do-Check-Act process approach, which emphasises: understanding business information security requirements and the need to establish policy and objectives · implementing and operating controls **in the context of managing overall business risk** · monitoring and reviewing performance and effectiveness of the ISMS · continual improvement based on objective measurement.

**The BS 7799-2 annexes:**
| Annex | Contents |
|---|---|
| **A** | Comprehensive list of control objectives, all cross-referenced to sections of ISO/IEC 17799:2000, each with identified controls. *"No such list can ever be complete, but this one does provide a great amount of material from which to draw ideas."* |
| **B** | Practical guidance on implementing the PDCA process; cross-references to the OECD *Guidelines for the Security of Information Systems and Networks* (July 2002) |
| **C** | Cross-references to **BS EN ISO 9001:2000** (quality management system) and **BS EN ISO 14001:1996** (environmental management system); also cross-references **ISO/IEC 13335** *Guidelines for the Management of IT Security* |

### The multi-phased testing model (Figures 16-2 and 16-3)
The software development lifecycle: the software engineer designs, constructs and tests code modules → the development manager organises **independent** testing of modules, integration, and testing of integrated sub-systems and the system, delegating to *other* software engineers (*"It is impossible to test your own programs"* — Rakitin, *Software Verification and Validation*, Artech House, 1997) → handover to the quality assurance team → delivery to user sponsors → user acceptance testing → handover to the operations team for live production.

> "If at any stage any part of the system fails, then the relevant modules are **demoted all the way back to the beginning** for rework, retest and reintegration."

Emergency fixes bypass this, and the chapter is explicit that they are a debt: *"It is dangerous in the long term to rely on live systems that are running with emergency patches in place, since there may be other faults (bugs) related to these emergency patches that the lack of full testing has not revealed."* Every emergency fix should be followed immediately by a formal change request through the full rework process, and after patching, **regression testing** returns to the beginning of the lifecycle.

**The vendor security patch dilemma, posed and left open:** *"You must make a decision as to whether you regard all vendor security patches as emergency fixes to be applied immediately or whether they should be tested and evaluated before they are applied, with the potential outcome that you may decide not to apply a specific fix."* Under RAD, application prototyping and extreme programming this tension sharpens: *"you cannot simultaneously mitigate them all."*

### Black box vs white box
- **Functional testing = black box testing.** *"A black box is something into which you cannot see… It has inputs, which you can see, and outputs, which you can also see, but you cannot see its internal mechanisms… **It is a logical description, not a physical one.**"* Functional tests are written against a functional test specification derived from a **Functional Requirements Specification (FRS)** — *"a specification of a system at the logical architecture level."*
- **White box / glass box testing** is carried out in full knowledge of how the mechanisms inside work — used by software engineers doing module testing.

**The FRS warning, from the chapter's footnote:** many FRS documents wrongly go into physical mechanism detail. *"The pitfall of this sloppy approach is that a physical architecture is designed before the wider range of functional requirements are captured and understood, and the resulting design fails to deliver fully the business requirements — often by being constrained by ill-informed design decisions taken too early in the process."*

**The asymmetry that defines the limits of testing:**
> "This provides you with assurance that the system performs all the functions that it should. However, **it is much more difficult to verify that the system does not have any unwanted functionality.** This is especially important for safety-critical systems."

Key black-box tests sit **at the boundaries of the expected inputs — just inside the boundary, on the boundary, just outside the boundary.**

## Reference Tables

### The three types of security audit activity
| Type | Frequency | How it is best achieved |
|---|---|---|
| **Daily event monitoring and follow-up** looking for violations of policy, procedure and standards | Daily | Automated log-scanning tools creating exception reports for manual follow-up |
| **Routine periodic inspections of system parameters** for continued compliance with security standards | Periodic | Automated tools scanning all system files and directories, comparing parameter settings with preferred settings **stored in a policy file**, creating exception reports for manual follow-up |
| **Periodic reviews of system construction and operations** for compliance with policies, procedures and standards | Periodic | Manual information gathering and reporting by audit experts, possibly assisted by checklists |

*"Your security audit strategy should preferably include all three of the above approaches since they all contribute differently to the overall process of assurance."*

**The six points a security audit strategy must settle:** Definition (what do you mean by 'security audit'?) · Expertise (who audits, with what training and CPD?) · Tools and techniques (the standard toolkit) · Audit work planning (scope and resources) · Execution (methodology) · Reporting (style, content, consumers, objectives, and **what value reporting will add**).

### Why actual configuration never matches the standard
> "The audit process, whether it be manual or automated, is one of comparing the actual configuration parameters found in the operational system against the theoretical configurations described in the policy and standards documents. In theory they should match perfectly. **In practice they rarely do so.**"

Five causes named:
1. **Entropy** — *"the laws of thermodynamics that tell us that entropy (chaos) tends to increase over time as orderly patterns and symmetry decay through a wide range of natural processes (have you ever tried keeping a garden?)"*
2. Mistakes are made in setting up systems;
3. Some people think they know a better way to do it;
4. New releases of software bring configuration changes, often to make the new software "work properly";
5. Inexperienced or ignorant personnel make changes to make things easier without understanding the ramifications.

### The four grades of ISMS auditor, and CISA
| Regime | Qualification | Requirements |
|---|---|---|
| **CobiT** | **CISA** (Certified Information Systems Auditor) | Pass the CISA exam — *"offered annually at nearly 200 sites around the world in 10 languages during the month of June"* · satisfy the IS auditing/control/security work experience requirement (education waivers available) · adhere to ISACA's Code of Professional Ethics · comply with annual continuing education requirements. CobiT is not specifically tested, but its control objectives reflect the CISA Practice Analysis, and ISACA recommends a thorough review of CobiT as exam preparation |
| **BS 7799** | **IRCA ISMS auditor grades**: ISMS Provisional Auditor → ISMS Auditor → ISMS Lead Auditor → **ISMS Principal Auditor** | Evaluation criteria specify the education, work experience, auditor training and auditing experience needed at each grade. Organisations are certified for BS 7799 compliance by a **UKAS-accredited body** employing qualified ISMS auditors |

**The ISMS auditor checks three things:** **Completeness** — have all parts of BS 7799 been covered? · **Relevance** — is the interpretation of BS 7799 relevant for the organisation? · **Implementation** — is the ISMS being followed?

**The Statement of Applicability (SOA)** — a document listing all requirements in BS 7799 Part 2 with an explanation of how the organisation complies with them, and *"an explanation and justification of any deviations from them."*

### Development controls (the strategy list)
- An **end-to-end development process with a complete audit trail and sign-off at each significant stage**, from business requirements capture to production — *"providing assurance that there are no holes in the process whereby sub-standard code could be accidentally or maliciously incorporated."*
- **Traceability** — *"being able to trace the entire development path of a specific application from concept through to production, with documented authorisations at each key stage."*
- Periodic **independent audits of the development environment** and activity.
- An audit trail per system, so any individual system can be audited for its history from the very beginning.
- **Complete segregation of development and production environments.** No development work on a production machine; **no system developer with any access to a production machine.**
- Emergency maintenance may require limited, time-boxed, strictly supervised developer access to production — *"but this should be minimised, and the definition of an 'emergency fix' needs to be defined carefully in business terms."*
- Comprehensive **version controls and version archiving** under an appointed **software librarian** who is one of the development team.
- **Only approved, authorised tools** — no private tools, scripts, macros or programs, even shared informally. *"If these tools are needed, they should be officially developed, tested and released. If not, they should be banned."*
- Development standards and approved methodologies so staff can take over or verify each other's work.
- **Peer review of critical work packages.**
- Module, integration, system and regression testing **carried out by people other than those who cut the code.**
- User acceptance testing in a **separate environment segregated from both development and production.**
- **Test data and live data completely segregated.**
- **Technical risk assessment** for: new software development language · new development tools · new methodologies · new delivery technology · new code · inexperienced development staff · hostile delivery environment.

**The unavoidable conflict this chapter names:** *"To mitigate the risk of missing a business opportunity you must take additional risks with lack of assurance. Conversely, to achieve a high level of assurance you increase the risk of delaying the release into production and thus missing a valuable business opportunity."* And a caveat: segregation controls *"are also difficult to achieve in a small ICT department where the development and production activities are necessarily blurred by lack of staff and lack of physical resources."*

### Production controls
- Production environment **completely segregated** from development and from any testing environments.
- **Segregation of duties** preventing development staff from accessing production systems.
- **A complete ban on the storage of software development tools and utilities** (editors, compilers, linkers) on live production systems. *"The only executable code available should be the applications and the system software required to run these applications. All other tools and utilities should be removed."*
- A formal, rigorous **change-control process** — no change without approval, hardware or software, with approval responsibilities assigned to people with suitable knowledge and appropriate authority.
- A formal **configuration-management process** embracing: requirements management · inventory management · change management · release management · data management · records management · document control · library management.
- New releases through the full systems-development lifecycle including the entire cycle of testing and acceptance.
- **A reversal procedure for every planned release.**
- Periodic **logical scans of software and configuration data files** in live systems to detect unauthorised change.
- **Prevention of operator access to maximum privilege** (e.g. UNIX root) by encapsulating all routine operations tasks in scripts or utilities. Root privilege reserved for emergency fixes only, independently supervised, requiring authorisation from a senior operations manager.
- Physical security over machine rooms — only authorised operations personnel.
- Documented and tested **incident management procedures**.
- **Master backups** of all current releases of production software and configuration data, and regular backup of all production data.

### Software integrity and anti-virus strategy
Three sources of malicious software:
1. **Self-replicating malicious software from outside** — viruses, certain Trojan horses, worms. *"The destructiveness of the payload varies greatly."*
2. **Malicious mobile code** — Java applets, ActiveX, mobile script, remote database queries using SQL, OQL and other query mechanisms.
3. **Code maliciously inserted** by a member of the development or operations team, or an external hacker with unauthorised access — Trojan horses, logic bombs, time bombs.

The strategic objective is *"simply a microcosm of the overall strategic model described earlier in Chapter 10 and shown in Figure 10-2"* — **prevent contamination; if that fails, detect as soon as possible, contain, and remove.**

| Technical control techniques | Strategic controls |
|---|---|
| Scanning stored files and directories for known malicious code | Enterprise policy requiring **authorised, licensed software only**, distributed through corporate procurement — including open source and public domain software, which must be *"acquired, tested and officially distributed by the enterprise"*; individual introduction of public domain software *"should constitute a disciplinary offence"* |
| Scanning stored files and directories for unexpected changes | An enterprise policy making all employees aware of malicious software issues and mandating acceptable behaviour |
| Real-time monitoring of the execution environment for unauthorised execution paths | A **corporate standard for the configuration of e-mail clients and web browsers** |
| Filtering and scanning imported digital materials at the enterprise perimeter — diskettes, CDs, other media, plus real-time filtering in a firewall or security gateway | An ongoing awareness programme — suspicion of certain e-mail attachments and web downloads |
| Control rules on web browsers to prevent or monitor execution of mobile code, possibly via digitally signed objects from trusted sources | Anti-virus detection software from a reputable supplier, **with regular frequent updates**, scanning all corporate systems and all incoming files by media, e-mail attachment, FTP or any other import route |
| Software object authentication **on installation** | Reinstallation of damaged software from original installation files or a known good backup |
| Software object authentication **on boot-up** | Anti-virus repair software **as a last resort** if original installation files have been lost |
| Secure coding practices in the development standards for in-house software | Backup and restoration housekeeping — software from masters, data from recent operational copies |
| **Code audit** of in-house software to check for Trojan code before release — *"both difficult and expensive and should be considered only for extremely high-assurance systems"* | An incident management process for malicious software contamination |

### Acceptable use
*"'Acceptable use' refers to the use of corporate information systems for acceptable purposes. Unacceptable purposes are those that are illegal, unauthorised or so grossly anti-social as to cause offence to people both inside and outside the enterprise."*

Unacceptable use examples given: browsing pornographic sites from corporate computers, and downloading/storing such material · browsing gambling sites and gambling on corporate time · sexual harassment of colleagues through corporate e-mail · racially or religiously abusive material sent to colleagues · wasting corporate time browsing sites irrelevant to one's job · incurring unacceptable communications costs via unauthorised sites or premium-rate numbers · administering private business interests on corporate systems · using corporate facilities to hack other systems inside or outside the enterprise · loading and playing unauthorised games.

Three strategic controls: an **Acceptable Use Policy** (noting that what counts as acceptable *"varies from one organisation to another"*) · **content-filtering software** to detect the most offensive and the illegal items · **a disciplinary process** by which violators are *"'brought to book' both as a punishment to them and a deterrent to others."*

### The five testing phases
| Phase | Who | Box | Objective |
|---|---|---|---|
| **Unit / module testing** | The software engineers writing the code | White box, exhaustive | Find bugs in logic, data (local and global) and algorithms within a single module. All independent paths identified and exercised; imported/exported data specifications checked against the interface specification on the other side; boundary conditions specified and tested; error handling tested so all possible errors are trapped and handled. *"Often not really regarded as a separate testing activity but carried out as an integral part of the coding process — a debugging activity"*, hence relatively informal with little or no test documentation. Requires stubs and scaffolding **prepared in advance so it is available as soon as coding begins** |
| **Integration testing** | Other software engineers, delegated by the development manager | Black box | Needed even though modules passed unit tests, because: modules may interact adversely in ways not obvious independently · **errors and imprecise calculations can become magnified as they are passed from one module to another** · modules may contend for shared resources, at worst causing **deadlock** · in real-time systems timing is critical and only visible across modules · interface problems may remain. *"It is important to design the integration process and the integration testing methods together"* — top-down, bottom-up or outside-in |
| **System validation testing** | **A separate quality assurance team that has not been involved in the development in any way** | Black box | Verify the system meets all requirements in the system requirements specification — **including performance**, since the requirements specification will specify it. Where exhaustive input/output combination testing is infeasible, prioritise using the **act-like-a-customer (ALAC)** approach, *"since the tests are based upon what an end user is most likely to do with the system"* |
| **User acceptance testing (UAT)** | The end users | Black box | Ensure the delivered system meets the **user requirements specification**. Ends with formal sign-off by the person with authority on the user side, *"possibly with a number of conditions specified as part of the acceptance"* |
| **Operational acceptance testing (OAT)** | The ICT operations group — often a third party under an outsourcing agreement | Black box | *"OAT is concerned only with operability, not business functionality."* The users test whether they are getting the right business functionality; the operators are concerned only with ease of operation under the SLA and supportability over the system's operational lifetime |

Three documents carry system validation testing forward to future releases: the **system validation test plan** (goals, objectives, resource estimates, schedules), the **system validation test procedure** with detailed test scripts, and the **system validation test report**.

**Seven elements of a testing strategy:** a test plan (what, why, how, where, when, by whom) · testing methods (white box, black box, ALAC for systems testing, top-down/bottom-up/outside-in for incremental integration) · testing personnel · **acceptance criteria** (what level of failures can be tolerated, and at what level of assurance is a test passed) · test schedule · test environments (stubs, simulators, emulators, scaffolding). And: *"It is essential as part of the testing strategy to specify the exit criteria for each phase and the entry criteria for each phase."*

### The six-point UAT problem classification scale
| # | Category | Rule |
|---|---|---|
| 1 | **Complete Failure** | Impossible to continue with the testing because of the severity of this problem |
| 2 | **Critical Problem** | Testing can continue but the system cannot go into live production with this problem |
| 3 | **Major Problem** | Testing can continue but this feature will cause severe disruption to business processes in live operation |
| 4 | **Medium Problem** | Testing can continue and the system is likely to go live with only minimal changes to the agreed business processes |
| 5 | **Minor Problem** | Both testing and live operations may progress; should be corrected, but little or no business process change envisaged |
| 6 | **Cosmetic Problem** | Colours, fonts, pitch size — *"however if such features are key to the business requirements (such as the readability of the screen) they will warrant a higher severity level"* |

*"Three- or four-point scales are often used."* The UAT plan must also set out the expected responses from users, project development team, vendors, consultants and contractors when issues are identified, and the resolution process including responsibilities.

### The OAT verification checklist (nine categories)
| Category | What must be verified |
|---|---|
| **Stability and robustness** | Continuous operation over an extended period; services restart correctly on reboot of any server **or** documented procedures exist for service restarts |
| **Event capture** | A user error message is generated after an incomplete or failed user transaction; all failures, errors and significant events are logged — including **exceeding high- and low-water marks for pre-emptive capacity management** |
| **Resilience** | Continuous operation through failure or controlled shutdown of individual components; sufficient redundancy to maintain **committed availability targets under the SLA**; application-specific data mirroring and failover working as expected |
| **Systems monitoring** | Comprehensive error reporting and alerting supports troubleshooting and problem diagnosis; **pre-emptive alerting** (approaching a capacity threshold or preventive maintenance cycle) allows pro-active management; operator procedures documented for all required control interventions |
| **Performance management and service level reporting** | Performance monitoring data supports detection of performance issues and measurement against agreed service levels; automated reports meet service level reporting requirements; observed performance is consistent with the SLA; **all application failures logged with time stamps to enable downtime reporting** |
| **Software deployment** | All software deployable within the available deployment window; meets the operations group's quality standards; **all changes and releases reversible within the deployment window** in case of deployment failure |
| **Systems operations** | Support and maintenance processes operable; automated housekeeping archives log files and prevents disk space filling; all routine housekeeping automated; start/stop and routine operations procedures fully documented; capacity management tools installed and configured to produce the expected output |
| **Impact on other systems and regression testing of other systems** | No disruption to other applications in the same **client**, **mid-tier server** or **mainframe** environment |
| **Security manageability** | All hardware platforms **"hardened" according to the operations group's internal security standards**; any user support provided by the business user community meets the operations group's internal standards; the system adheres to all relevant internal operational security standards; system account information and passwords can be changed and managed according to documented procedures |
| **Backup and recovery** | A reliable automated backup solution including **notifications of backup failures**; restoration from backup completes successfully **within the available business window** |

Ends with formal sign-off by the person with authority on the operational side.

### Software quality assurance
*"Software quality assurance is concerned with monitoring and continuously improving the software development process, making sure that any agreed standards and procedures are followed and ensuring the problems are identified and resolved."*

**Four characteristics of good quality software:** reasonably bug-free (within the context of the application) · delivered on time and within budget · meets the user requirements and expectations · **is operable, supportable and maintainable**.

Remaining QA activities: source code reviews and inspections (peer review as a regular procedure; verification of adherence to internal coding standards) and reviews of testing strategies and plans (**coverage validation for each phase — do the planned tests cover everything that they should cover?**). The maturity benchmark: **CMMI** from the Carnegie Mellon Software Engineering Institute (see Chapter 6 for capability maturity models generally).

### Twelve levels of penetration testing
| Level | What it does |
|---|---|
| **Internal network penetration** | Attacking the network from inside the enterprise |
| **Static analysis** | Of computer-generated logs and firewall rule sets |
| **On-host auditing** | Ensuring security policies and standards have been rigorously applied on host platforms |
| **Vulnerability assessment** | Automated scanning of networks and systems for known vulnerabilities — *"new ones emerge continually"* |
| **External network penetration** | Attacking from external locations, including from the Internet |
| **Application testing** | Subverting applications by **manipulating data inputs, looking for unwanted functionality** |
| **Source code analysis** | Poor coding practices or suspicious routines |
| **Wireless and mobile telephony penetration testing** | For systems employing these technologies |
| **PABX auditing and testing** | Poor configuration that could lead to penetration of corporate voice and data systems |
| **War dialling** | Automatically dialling specified ranges of telephone numbers to look for useful services an opponent could exploit |
| **Remote access server (RAS) testing** | Vulnerabilities an opponent could use to penetrate your network |
| **Social engineering** | Testing whether your people can be socially engineered into revealing passwords or allowing unauthorised personnel into control areas of your buildings |

## Worked Example

### Penetration Testing of a Web Casino — the negative stake
A casino operator set up a web-based casino offering blackjack, poker, roulette and other games of chance. *"Using modern graphics the site was made to look very attractive, dynamic and interesting — with all the action of a real casino."* Players registered with a username and password and a credit card authorisation to cover their stakes; they purchased chips with the card, played with the chips, and cashed winning chips back to the card.

The operator's reasoning about risk was, on its own terms, sound:
> "Obviously fraud is an issue in casinos, so the credit card interface was robust. Another key threat in the world of casinos is cheating at the game itself, but in a web interface the casino has ultimate control. The player cannot touch anything because there is no physical presence, and the possibility of socially engineering or colluding with casino staff is also eliminated, so **the web casino seemed much easier to secure than the real thing.**"

Before going live the operator nevertheless called in specialist penetration testers. *"It was a wise decision."*

> "The mindset of the casino operator had been conditioned by years of experience of running real casinos. Thus some of the things that can be done on a web interface had not been considered. **The exploit that they found was so simple that it defies belief**, and yet you can see how conventional thinking led to the problem.
>
> "The penetration testers started to manipulate the data inputs into the gambling application, and the key input here is how much stake you want to gamble. **By entering negative numbers into the stake field it was possible to force a win every time.** Of course nobody ever bet a negative stake in a real casino."

The problem was fixed before it went live. This is the chapter's exhibit for two of its own claims: that penetration testing exists because *"functional testing is relatively static in its nature… in reality systems are dynamic in their behaviour"*, and that the hard part of testing is verifying the **absence** of unwanted functionality — no functional requirement was violated here; a boundary that nobody thought to specify simply was not there.

### The Network Engineers — why audit finds drift
> "It has happened in real life many times.
>
> "The network team is monitoring network performance. Something is wrong — the throughput simply is not what it should be.
>
> "Someone on the team asks: 'Is it those damned encryptors that are playing up again?'
>
> "'Try turning them off and see what happens.'
>
> "'There! That's much better!'
>
> "**Different people have different priorities.**"

This is the concrete case behind the audit premise: the audit compares the operational system's actual parameters against the policy and standards documents, and *"in theory they should match perfectly. In practice they rarely do so."*

### The BS 7799-2:2002 Annex B §6.4.5 internal audit specification
The chapter reproduces the standard's own internal-audit clause as the reference model for an internal ISMS audit programme:

> "The overall objective is to check over a specified regular audit period (**which should last no more than one year**) that all aspects of the ISMS are functioning as intended. A sufficient number of audits should be planned so that the audit task is **spread uniformly over the chosen period**. Management should ensure that there is evidence that confirms that:
> a) The information security policy is still an accurate reflection of the business requirements;
> b) An appropriate risk assessment methodology is being used;
> c) The documented procedures are being followed (i.e. within the scope of the ISMS) and are meeting their desired objectives;
> d) Technical controls (e.g. firewalls, physical access controls) are in place, are correctly configured and working as intended;
> e) The residual risks have been assessed correctly and are still acceptable to the management of the organisation;
> f) The agreed actions from previous audits and reviews have been implemented;
> g) The ISMS is compliant with this standard.
>
> "The audits will need samples of current documents and records and involve interviews with management and staff."

### The penetration testing strategy — eight rules
1. **Use external experts — "this is not a game for amateurs."**
2. Choose an expert service supplier with depth of expertise and multiple testing tools.
3. **Be wary of ex-hackers and convicted criminals.** *"You are potentially opening your systems to people with dangerous skills, so trust in the integrity of penetration testers is a major issue."*
4. **Use more than one firm and rotate them.** *"They have different tools and different experts, so that what one team misses might be found by another."*
5. Run a regular programme of tests for existing systems and applications — new vulnerabilities and exploits emerge all the time.
6. Test new systems, new applications and new releases before they go live.
7. Make the contract clear about responsibilities and liabilities on each side. *"You will have to give them authorisation to work on live system interfaces, but you must restrict what they can do next if they discover an exploit. The risk is that live production systems will be brought down by a test that succeeds in penetrating the system."*
8. If testing a separate environment rather than live, replicate the live environment as closely as possible — and accept the residual risk: *"If the environment changes then an exploit that failed may now be successful. **Thus a test system that was impenetrable may not guarantee the same properties for the live system.**"*

## Mental Models
- **Assurance = the Business Attributes are meeting their performance targets.** Everything in this chapter is instrumentation for that one claim.
- **The Inspector's View made operational.** Chapter 3's sixth view is not a diagram; it is an audit programme, a testing strategy and a penetration testing contract.
- **You cannot test your own programs.** Independence rises with each testing phase — engineer, other engineers, QA team, users, operations.
- **Entropy is the default.** Configuration drifts away from standard by natural process; audit is the restoring force, not a one-off certification event.
- **Speed versus assurance is a risk trade, not a quality failure.** Two operational risks in direct conflict — the Chapter 15 principle applied to the SDLC.
- **Presence is testable; absence is not.** Black box testing proves the specified functions work; it cannot prove there is nothing else in there. Penetration testing is the partial answer.
- **Management standard ≠ technical standard.** ISO/IEC 17799 tells you what to do; it will not tell you how, and it will not certify your ISMS (that is BS 7799-2).
- **The users test functionality; the operators test operability.** UAT and OAT are different questions asked by different parties with different sign-offs.
- **The tester's mindset must not be the operator's mindset.** Nobody who had run a physical casino would think to bet a negative stake.

## Anti-patterns
- Treating certification as assurance — the ISMS auditor's three questions are completeness, relevance **and implementation**.
- Auditing against nothing: conducting a security audit without a written definition of what the auditors expect to find, embodied in policies and standards.
- Turning off the encryptors to fix a throughput problem — and no audit process that would notice.
- Relying on ISO/IEC 17799 to assure an ISMS when only BS 7799-2:2002 specifies one.
- Development staff with access to production machines; development tools, editors, compilers and linkers left on live production systems.
- Test data mixed with live data; UAT run in the development or production environment.
- Private developer tools, scripts and macros used on real work and informally shared.
- Letting emergency patches accumulate in live systems without a follow-up change request and regression test.
- Routine operations performed with root privilege instead of encapsulated scripts.
- A planned release with no reversal procedure.
- Testing phases with no defined entry and exit criteria.
- Attempting exhaustive white box testing beyond the unit level — *"once the integration process begins and complexity grows it would be an infinitely difficult task."*
- Writing a Functional Requirements Specification full of physical mechanism, freezing the physical architecture before the functional requirements are understood.
- Employees installing public domain or open source software on their own initiative rather than through corporate procurement.
- Amateur penetration testing; a single testing firm used forever; a testing contract that does not restrict what testers may do after a successful penetration.
- Assuming an impenetrable test environment guarantees an impenetrable live one.

## Key Takeaways
- Assurance of Operational Continuity sits at the Operational layer, Assets column of the SABSA Matrix, and means: every Business Attribute is being delivered at or above its performance target.
- Six families of inspection deliver it: organisational audits, system audits, system assurance controls (development, operations, integrity, use), functional testing, software QA, and penetration testing.
- CobiT (4 domains, 34 high-level control objectives, 318 detailed ones, 7 information criteria) is the broader ICT governance audit framework; ISO/IEC 17799 / BS 7799 is the security-specific one, with BS 7799-2:2002's PDCA model as the ISMS process and its Annex B §6.4.5 as the internal audit specification.
- Qualified auditor status: **CISA** in the CobiT universe; **IRCA's four ISMS auditor grades** and UKAS-accredited certification bodies under BS 7799, with a Statement of Applicability required.
- Three types of system security audit — daily automated event monitoring, periodic automated parameter scans against a policy file, periodic manual construction and operations reviews — and a good strategy uses all three.
- Development and production must be completely segregated, with traceability end to end, an appointed software librarian, authorised tools only, peer review, and testing performed by people other than the coders.
- Production adds change control, configuration management across eight disciplines, reversal procedures, logical scans for unauthorised change, root privilege reserved for supervised emergencies, and master backups.
- Software integrity strategy is Chapter 10's prevent → detect → contain → remove model applied to malicious code, with licensed-software-only policy and disciplinary consequences for unauthorised installs.
- Testing is five phases with entry and exit criteria: unit (white box, exhaustive, by the coder) → integration → system validation (independent QA, includes performance, ALAC prioritisation) → UAT (against user requirements, six-point problem scale, user sign-off) → OAT (operability only, nine verification categories, operations sign-off).
- Functional testing proves the specified functions work; it cannot prove the absence of unwanted functionality, which is why penetration testing exists — twelve levels of it, from war dialling to social engineering.
- The web casino's negative stake is the chapter's proof that the gap is in the mindset, not the specification.

## Connects To
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — the Inspector's View, of which this whole chapter is the operational implementation.
- [Chapter 5 — A Systems Approach](ch05-a-systems-approach.md) — the feedback control loop that PDCA and the audit-report-and-track cycle instantiate.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — Business Attributes, performance targets and metrics, which define what "assured" means; and capability maturity models, of which CMMI here is the software development instance.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — Figure 7-8 (the populated matrix) and Figure 7-12, where Framework for Assurance of Operational Continuity is listed as an operational architecture deliverable.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — safety-critical systems, where verifying the absence of unwanted functionality matters most.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — Figure 10-2's prevent/detect/contain/recover model, of which the anti-virus strategy is a microcosm.
- [Chapter 14 — Security Policy Management](ch14-security-policy-management.md) — the policies and standards that supply the "ideal world" definition against which every audit compares reality.
- [Chapter 15 — Operational Risk Management](ch15-operational-risk-management.md) — the speed-versus-assurance conflict as an instance of risk trading against risk; penetration testing and vulnerability scanning as inputs to Step 6 of the ITEC risk review.
- [Chapter 17 — Security Administration and Operations](ch17-security-administration-and-operations.md) — the operational functions whose day-to-day discipline these audits and acceptance tests verify.
