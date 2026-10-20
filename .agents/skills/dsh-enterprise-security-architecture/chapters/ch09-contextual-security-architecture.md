# Chapter 9 — Contextual Security Architecture

## Core Idea
The contextual layer is the **analysis and description of the business itself** — strategy, objectives, relationships, risks, constraints and enablers — because *"information security has no intrinsic value of its own. Its only possible value is that it protects something that has explicit value to the business."* This chapter is a structured checklist of what to investigate, plus the **SABSA Risk Assessment Method** and the **Business Risk Model** it produces, which is the single most important contextual deliverable.

Every section in the chapter is explicitly cross-referenced to a SABSA Matrix cell, so the chapter is effectively a walk across the contextual row: The Business (What) → Business Risk Model (Why) → Business Process Model (How) → Business Organisation and Relationships (Who) → Business Geography (Where) → Business Time Dependencies (When).

## Frameworks Introduced

### The chain of derivation
Business drivers (from this chapter's investigation) → Business Attributes → **Business Risk Model**, **Business Attributes Profile** and **Control Objectives**. Business drivers are the primary input to the entire process.

### Basel II definition of operational risk (adopted by the book)
> "The risk of loss resulting from inadequate or failed internal processes, people and systems or from external events."
(Basel Committee on Banking Supervision, "Working Paper on the Regulatory Treatment of Operational Risk," September 2001.)

### The risk model vocabulary
- **Assets** — things of value to your business that you want to protect.
- **Threats** — potential damaging events that put your assets in danger.
- **Impacts** — the potential outcome of a threat materialising and causing damage.
- **Vulnerabilities** — weaknesses in operational business procedures or systems that allow a threat to materialise and exploit an asset, causing an impact.
- **Likelihood** is a complex combination of *level of threat* (likelihood the threat event materialises in a given period) and *level of vulnerability* (likelihood the threat event succeeds in exploiting your assets).

### Why the method is qualitative and threat assessment is binary
- **Quantitative threat assessment is unrealistic.** Threats exist outside your span of control. Without reliable, consistent, complete data on previous loss events, statistical analysis gives little guidance on the probability of a threat materialising — and "observation of past events is not necessarily a good guide to how the future will be."
- **Intelligence gathering is unrealistic** for commercial organisations, and in any case only applies to threats posed by criminals, terrorists and the like — not to the wide range of non-malicious operational risks.
- **Vulnerability and impact are within your span of control**, so commercial risk assessment focuses there, usually qualitatively (low/medium/high). Threat analysis is limited to *identifying* threats without quantification.

### The generic six-step commercial risk assessment
1. **What are your business assets?** Identify and value them.
2. **What possible threats put them at risk?** Identify the possible threats.
3. **For each threat, if it materialised, what would be the business impact?** Identify and quantify by relating back to the asset list.
4. **If the impact is significant enough to trouble you, what vulnerabilities or weaknesses could allow the threat to exploit your assets?** Identify and quantify them.
5. **Can you reduce those weaknesses with additional controls?** Identify possible control strategies and quantify their total cost of ownership.
6. **What is the cost/benefit analysis** — reduction in potential business impact (benefit) weighed against cost of additional control? Quantify both.

Purpose: understand the risk profile in detail, and make well-informed risk management decisions.

### Two-dimensional threat classification: domains × categories
The four **threat domains** come directly from the Basel II definition. The **threat categories** are "entirely arbitrary and based upon practical experience only" — they could be changed without violating any theoretical principle, and new ones added. Each category maps to one or more domains, with no specific logical mapping. Together, domains × categories form the **Threats Database** referenced in Figure 7-4.

### Time-Based Security
| Symbol | Meaning (all measured in time) |
|---|---|
| **P** | Protection Value — the time the system will resist attack, i.e. the time it takes to break into it |
| **D** | Detection Value — the time it takes for the system to raise the alarm |
| **R** | Reaction Value — the time it takes for system management to react to an alarm |
| **E** | Exposure Value — the time during which the system is exposed and the attack can cause damage |

**IF P > D + R THEN the system is secure and the attack will fail.**
**IF P < D + R THEN the system is exposed and the attack will succeed.**
**E = D + R − P**

The design consequence: by measuring these parameters you can design systems that are more secure and less exposed, and in particular aim to **reduce both D and R**.

## Reference Tables

### Table 9-1: Threat Domains and Threat Agents
| Threat Domain | Description of domain — losses caused by… | Threat agents |
|---|---|---|
| **People** | Malicious violation of internal policies; negligent violation of internal policies; human errors | Current employees; past employees; people under consideration for employment |
| **Processes** | *Unintentional* losses caused by: deficiency in an existing procedure; absence of a suitable procedure; failure to follow a defined procedure | Employees; customers; suppliers; service providers; agents; partners; members of the public |
| **Systems** | *Unintentional* losses caused by: unforeseen breakdown of technical systems; insufficient resilience in technical systems | Technical failure through "fair wear-and-tear"; technical failure through inadequate design or poor implementation |
| **External** | Natural disasters; man-made disasters (unintentional); malicious actions of third parties; negligent actions of third parties; legitimate actions of third parties | Natural events; accidents; malicious third parties; negligent third parties; legitimate third parties whose business interests conflict with ours |

### Table 9-2: Taxonomy of Threats — the Threats Database (threat categories)
Each row shows the category, its description, and the domains it maps to (P = People, Pr = Processes, S = Systems, E = External).
| Threat category | Domains | Description | Representative threat examples → information security mapping |
|---|---|---|---|
| **Facilities and Operating Environment** | Pr, S, E | Loss or damage to operational capabilities caused by problems with premises, facilities, services or equipment | Business interruption from one of many possible threat scenarios → inadequate business continuity management and ICT disaster recovery; data processing service interruption; communications interruption → ICT systems outages through inadequate disaster recovery |
| **Health & Safety** | P, Pr, S, E | Threats to the personal health and safety of staff, contractors, suppliers, agents, customers and members of the public | Unsafe operating environments with safety hazards unmanaged → poor installations and operational management in ICT operations rooms; criminal or terrorist attacks on individuals → weak logical and physical security leading to unauthorised disclosure of private address details, travel itineraries; safety-critical systems failure → inadequate protection for process control systems for automated factory processes, machine tools, power generation, automated secure vaults and other specialised machinery; infectious diseases spread in the workplace → non-availability of key staff due to local epidemic |
| **Information Security** | P, Pr, S, E | Unauthorised disclosure or modification of information, or loss of availability of information, or inappropriate use of information | Unauthorised disclosure of corporate-confidential or customer-private information; unauthorised modification, deletion, duplication or replay of corporate or customer information; unauthorised or inappropriate use of corporate, customer or personal employee information → weak logical security, weak physical security and weak operating procedures. Loss of availability of internal corporate or customer information → inadequate resilience, backup and recovery in ICT systems. Malicious denial of service attacks → inadequate physical and logical security. Inability to hold people accountable → inadequate policies and guidelines and inadequate audit trails. False repudiation of completed transactions → lack of non-repudiation services and mechanisms |
| **Control Frameworks** (of the existing risk management infrastructure and processes) | — | Weaknesses in the control framework itself | Lack of adequate cost accounting → inadequate cost accounting in ICT projects, inadequate design; lack of measurement of efficiency → inadequate measurement and reporting of systems efficiency; theft and fraud → inadequate control of unauthorised activity on ICT systems |
| **Legal & Regulatory Compliance** | P, Pr, S, E | Failure to comply with the laws of the countries in which business operations are carried out, or with regulatory, reporting and taxation standards, or with contracts, or failure of contracts to protect business interests | Failure to comply with EU data protection legislation or similar → inadequate attention to privacy laws; a change in law or industry regulations in one or more countries leads to breach elsewhere → inadequate architecture to enable upgrade/extension/change of ICT systems to deal with regulatory change, and inadequate early-warning research; inconsistencies in the legal and regulatory framework → inadequate research into the legal environment for this business initiative; operating rules for a service found in breach → inadequate research and drafting of operating rules; contractual disputes → weak or inadequate contracts; deliberate, wilful or grossly negligent breach → weaknesses in operating procedures and inability to control staff behaviour; failure to comply with the taxation regime → inadequate reporting of tax details |
| **Business Strategy** | P/Pr, E | A strategic business plan fails to meet its expected targets | Expected market fails to emerge as forecast → poor market research and weak forecasting; competitors more successful, inadequate marketing → poor marketing of ICT services; technology in breach of patents → poor patent research and inadequate liability contracts; key technology suppliers suffer business collapse → dependence on single-source suppliers and lack of multi-sourcing; future technologies fail to emerge → poor information research and forecasting; interoperability failures with other key players → misjudgement of standards selection; long-term operational service failure destroys market confidence → inability to manage and solve operational problems with poor leadership; a merger fails because integration proves too difficult and expensive → failure of due diligence to assess integration costs and difficulties; consortia or joint ventures fail → weak buy-in from participants and inadequate contracts on liabilities and responsibilities |
| **Corporate Governance** | P, Pr, S | Failure of directors to fulfil their personal statutory obligations in managing the company and protecting shareholders' interests | Lack of internal policy → inadequate leadership and policy setting from senior levels; failure to take timely action when a major strategic project gets into difficulties → lack of due and timely diligence, inadequate reporting to senior management, lack of leadership in solving operational problems; failure to comply with internal policies → poor implementation of systems and processes, inadequate internal audit |
| **Public Relations** | P, Pr | The negative effects of public opinion, customer opinion, market reputation and damage to the brand caused by failure to manage public relations | Loss or interruption of services to customers → inadequate resilience, disaster recovery, continuity and crisis management; failure to meet customers' service delivery expectations → inability to manage and solve operational problems; failure to meet the market's expectations for rate of roll-out following launch → inadequate project management and strategic planning; failure to meet market expectations of what a warranty means in practice → inadequate marketing raising false expectations; loss of market and customer confidence from media reports of operational failures or of serious security breaches → inadequate crisis management including public relations management; hostile reporting by investigative journalists → inadequate protection of internal confidential information from access by hostile external parties |
| **Processing and Transactions** | P, Pr, S | Problems with service or product delivery caused by failure of internal controls, information systems, or weaknesses in operating procedures | Poor integration of processes → need to maximise integration of software applications and eliminate "islands" of functionality; lack of conceptual thinking in systems planning → inadequate ICT systems architecture; transaction failure → inadequate exception reporting and transaction recovery; reconciliation failure → inadequate exception reporting; unauthorised trading → inadequate authorisation, identification, authentication and access control |
| **Product Liability** | P, Pr | A product or service sold and delivered fails to meet the required standards for suitability for the client's needs | Poor advice given to clients; mis-selling of financial products → inadequate information content management; failure to meet contractual service levels → inadequate attention to fulfilling the SLA for ICT services |
| **Behavioural** | P | Problems with service or product delivery caused by lack of employee integrity, or by errors and mistakes | Data input errors → inadequate data input validation; business operations mistakes → inadequate ICT operations procedures and operations management |
| **Technology** | P, Pr, S | Failure to plan, manage and monitor the performance of technology-related projects, products, services, processes, staff and delivery channels | Failure to meet business requirements → inadequate business requirements specification on ICT development projects; failure to integrate with business processes → need for ICT projects to support and integrate with business processes; technical integration failure → must successfully integrate chosen ICT product components; inadequate technical architecture → inadequate ICT systems architecture to ensure flexibility in response to changing business requirements; lack of technical standards → badly designed and constructed ICT systems |
| **Ethical** *(social/environmental conduct)* | P, Pr, E | Damage caused by the organisation's or its business partners' conduct — racial and religious discrimination, exploitation of child labour, pollution, environmental and "green" issues, behaviour to disadvantaged groups, sexual harassment | e.g. inappropriate content on web sites |
| **Geo-Political** | P, S | Loss or damage in some countries caused by political instability, poor quality of infrastructure in developing regions, or cultural differences and misunderstandings | Breakdown of normal working life → inadequate resilience and disaster recovery to prevent ICT operations being interrupted; breakdown of national or local infrastructure → inadequate plan for dealing with power outages and telecommunications outages |
| **Cultural** | P, Pr, S | Failure to deal with cultural issues affecting employees, customers or other stakeholders — language, religion, morality, dress codes and other community customs and practices | Sexual harassment → inadequate policies and procedures to discourage and if possible prevent inappropriate or pornographic content in e-mails; language problems → lack of flexible language modules for all ICT applications |
| **Climate, Weather, Environment and Geology** | E | Loss or damage caused by unusual climate conditions — drought, heat, flood, cold, ice, storm, winds — or by geological instability such as earthquakes and subsidence | Loss or damage to operational facilities → inadequate site selection for data centres |

### Table 9-3: Framework for Describing a Threat Scenario
*(Concepts drawn from work by Andy Jones, University of Glamorgan, restructured by the authors.)*
| Parameter | Description | Worked example |
|---|---|---|
| **Specific threat** | A threat selected from Table 9-2 | Unauthorised code inserted into an application to either defraud the organisation or sabotage the organisation |
| **Threat agent** | An agent suggested by Table 9-1, defined in more detail as in Table 9-4 | Disaffected employee working in the systems development team |
| **Capability** | Level of resources expected to be under the agent's control (Table 9-5) | Full skill set and tool set required for the task |
| **Motivation** | What motivates the agent (Table 9-6) | (1) Personal gain; (2) Revenge |
| **Opportunity** | Description of the opportunity or level of access available | Full access to development code and development environment |
| **Catalysts** | Events or changes in circumstances that make the agent decide to act (Table 9-7) | Redundancy of employee; redundancy of a friend of the employee; employee runs up debts |
| **Inhibitors** | Factors that may deter the agent (Table 9-7) | Fear of being detected, losing job and gaining a criminal record |
| **Amplifiers** | Factors that may encourage the agent (Table 9-7) | Belief that the rogue code can be hidden and covered and not attributed to an individual |

### Table 9-4: Threat Agents
| Group | Specific threat agents |
|---|---|
| **Natural Events** | Wind; electrical storm; earthquake; flooding from rainwater, rivers, tidal surges, storms; volcanic eruptions |
| **Accidental Events** | Fire; flooding from burst water pipes or tanks; explosions caused by malfunction of processes or services; structural collapse or damage from external impact by aircraft or heavy vehicles; structural collapse or damage from other assorted causes |
| **Technical Failures** | Equipment failure from fair wear and tear; equipment failure from poor design or implementation |
| **Individuals** | Human errors by our employees; human errors by employees of other organisations (customers, suppliers, partners, agents); human errors by members of the public; gross negligence by our employees past, present and future; malicious actions by disaffected employees past, present and future; malicious actions by individuals belonging to external third-party organisations; malicious actions by external individuals including criminals, hackers, terrorists |
| **External Organisations** | Malicious actions by third-party organisations including organised crime syndicates, terrorist groups, state-sponsored action groups, competitive commercial organisations, political pressure groups |

### Table 9-5: Threat Agent Capabilities
**Finance** (money to finance the threat activities) · **Technical equipment** (computers, specialised networking equipment) · **Software** (tools to perform detailed analysis, probing and penetration) · **Facilities** (buildings, services, general support) · **Expertise** (people educated and trained in the techniques) · **Literature** (books, manuals and other documentation with details of how to mount the activities) · **Experience** (people with previous experience of mounting them).

### Table 9-6: Threat Agent Motivations
| Class | Motivations |
|---|---|
| **Personal Gain** | Financial gain; revenge; gaining knowledge or information; exerting power; gaining peer recognition and respect; satisfying curiosity; satisfying antisocial personality traits; terrorising certain target groups or individuals; enhancing personal status within one of the groups below |
| **Group Gain** | Furthering the aims of political groups; furthering the aims of criminal organisations; furthering the aims of religious organisations; terrorising certain target groups; gaining competitive advantage |

### Table 9-7: Catalysts, Inhibitors and Amplifiers
| Inhibitors | Catalysts | Amplifiers |
|---|---|---|
| Fear of capture | External events that trigger a response | Peer pressure |
| Fear of failure | Changes in personal circumstances creating a "need" | Fame |
| Insufficient access limiting the opportunity | Step changes in level of access increasing the opportunity | Easy access providing high level of opportunity |
| High level of technical difficulty | Step changes in level of difficulty through new technologies and tools | Ease of execution because of low level of technical difficulty |
| High cost of participation | Step changes in level of cost | Low cost of participation |
| Sensitivity to adverse public opinion | Dramatic changes in public opinion and cultural values | Belief in sympathetic public opinion |

### Rating scales
**Impact (column 7):** H = High impact, could potentially do great damage to the business · M = Medium impact, could do significant damage · L = Low impact, could do only minimal damage.
**Vulnerability (columns 9 and 12):** H = High vulnerability, easily exploited by the threat · M = Medium vulnerability, possible for the threat to exploit · L = Low vulnerability, very difficult to exploit by the threat.

### Table 9-9: Risk Categories (from Figure 9-3, impact × vulnerability)
| Category | Colour | Description | Required actions |
|---|---|---|---|
| **A** | Red | Severe risk | Immediate corrective actions are required either to reduce the vulnerability or to reduce the impact level, or both. These risks are of the highest priority. |
| **B** | Yellow / Amber | Significant risk | Appropriate corrective actions should be planned and executed so as to reduce the vulnerability or the impact level. |
| **C** | Green | Acceptable risk | These risks are acceptable, because either the vulnerability is at its lowest possible or the impact is minor, but they should be monitored to ensure they do not creep into the B category. |
| **D** | Blue | Negligible risk | No action needed. |

### The 13 columns of the Business Risk Model (Table 9-8)
1 ID · 2 Business Driver · 3 Business Attributes · 4 Business Requirement · 5 High-Level Threat · 6 Business Impact · 7 Impact Value · 8 Potential High-Level Vulnerability · 9 Green Field Vulnerability Value · 10 Green Field Risk Category · 11 High-Level Control Objectives · 12 Target Vulnerability Value · 13 Mitigated Risk Category.

**Columns 1–10 are contextual (this chapter); columns 11, 12 and 13 are conceptual (Chapter 10).** Columns 10–13 together are "one of the key interfaces between the contextual security architecture and the conceptual security architecture."

### Contextual checklists across the remaining matrix cells
**Business Process Model (How).** *Business interactions* need entity **identification** (unique identity, usually via a naming scheme guaranteeing uniqueness), entity **authentication** (proving a *claimant* is who it claims, to the satisfaction of the *verifier*), and entity **authorisation** (restricting actions to those authorised). Entities may be individual human entities (users), corporate entities (whole businesses, divisions or departments) or logical entities (applications acting on behalf of individuals or corporates).
*Communications methods:* point-to-point telephone calls; telephone conference calls; mobile telephones; video conferencing; fax; dial-up data; leased line data; broadband over ADSL; cable television; satellite television; LANs; WANs.
*Communications applications:* home banking; corporate office banking; home shopping; Internet and web access; e-mail; on-line chat; corporate networking and distributed applications; file transfer; on-line transaction processing; remote database access; on-demand entertainment.
*Transaction types:* contracts negotiation and agreement; distribution of catalogues of goods and services; specifications; orders; invoices; payments; transfers of ownership; information delivery; electronic publishing for product support.

**Business Organisation and Relationships (Who).** Management hierarchies and their effect on authorisation, governance and control; integrating the supply chain (trusted interactions, the trust model representing them, the associated risk model); outsourcing ICT operations (managing security policymaking and its implementation, and the accompanying risk model); strategic partnerships (how close you get, how much you share, the implied liabilities); **joint ventures** ("JVs pose difficult problems where you collaborate on some fronts but compete on others — the development of a security domain model is essential"); mergers, acquisitions and divestments (again a **security domain model** makes this far easier than a monolithic architecture).

**Business Geography (Where).** The global village marketplace — the Internet removes location barriers but "you can no longer see, feel and touch the other parties," which hugely affects trust relationship management. Remote working — teleworkers/telecommuters, "road warriors" with a virtual office of laptop plus mobile, requiring secure remote processing and communications in hostile environments over long-distance third-party networks. Virtual teams spread across countries and time zones. Multi-site offices linked by corporate data networks into a single logical office. Some virtual companies have no office at all (typically small hi-tech knowledge-economy firms).

**Business Time Dependencies (When).**
- *Business transaction turnaround times* — security mechanisms must not slow response beyond business tolerances.
- *Business transaction lifetime* — security mechanisms must have **lifetimes exceeding those of the transactions they secure, with a tolerable margin of safety**.
- *Business deadlines* — banking cut-offs, stock-market closing times; secure time-stamps may be a fraud-prevention mechanism to detect attempts to manipulate cut-offs. The extreme analogy: placing a bet on a horse race after the race has finished and the result is known.
- *Record retention times* — data must be retrievable, readable and usable right to the end of the period, often a regulatory requirement. The main threat is **withdrawal of support for old storage media technologies** — archived documents on old tapes or disks in a vault, with no surviving sub-system to read them (consider the disappearance of 8-inch and 5-inch floppy diskette drives).
- *Response to customers* — within the time they expect.
- *Just-in-time operations* — minimum stock levels to maximise cash flow; security delays that cause JIT deadlines to be missed damage business continuity.
- *Time to market* — balancing the risk of shipping imperfect security against the risk of missing the opportunity. "Mitigating one risk will always increase at least one other risk. Always be aware of the risk of doing something versus the risk of not doing it."

### Contextual business drivers surveyed in the chapter
*Security as a business enabler:* electronic publishing (copyright protection, preventing unauthorised mass copying and redistribution); on-demand entertainment (music, movies, computer games — the industry's piracy crisis); value-added information services (restricting access to authorised subscribers makes the service a commercial proposition); **remote process control**; supply chain management; research and information gathering.
*Operational continuity and stability:* revenue generation; customer service; market reputation; management control; operating licences; employee confidence; shareholder confidence; other stakeholders (customers, suppliers, employees, investors, regulators, bankers, government agencies).
*Safety-critical dependencies:* remote communications to safety-critical systems; systems assurance.
*Business goals, success factors and operational risks:* brand protection; fraud prevention; loss prevention; business continuity; strategic business development; legal obligations; confidence of key stakeholders.

**On fraud:** "Computers do not commit fraud. They are merely one of the many tools that can be used by humans to commit fraud." Fraud usually happens where there is **opportunity (access, skill and time) combined with motivation (need, justification and possibly the challenge)**. Varieties range from large-scale bank funds transfer fraud down to fiddling a travel expenses claim, and include the **"salami fraud"** — systematic fraudulent collection of small unnoticed amounts, a thin slice at a time.

**On legal obligations:** compliance with criminal law; compliance with civil law; regulatory compliance relevant to the industry; compliance with contractual obligations; management and mitigation of legal liabilities; **civil liability caused by failure to maintain a reasonable level of security** — including failure to prevent certain events, failure to warn about the imminence of certain events, failure to notify after certain events have occurred, and failure to monitor certain events.

## Worked Example

### The SABSA Risk Assessment Method applied to IBFS — Table 9-8, the Business Risk Model

**Step 1 — Business Drivers and Business Attributes (Assets).** Assets are captured through the business drivers and Business Attributes and turned into a statement of business requirements for security (columns 1–4). The drivers used are drawn from the Chapter 4 interviews: **"The customer is king"** from Rosemary Brown (SVP eBusiness) and **"We must comply with the law"** from Ho Siew Luan (Director of Compliance).

**Step 2 — Threat Assessment.** Because detailed threat assessment is too difficult for a commercial organisation, the approach is **binary — does this threat affect you or not?** List the threats or threat scenarios you consider the relevant threat model. Use the Step 1 business requirements to frame the statement of a threat that will *prevent that requirement being met* (column 5). *If several threats put one requirement at risk, split the row.*

**Step 3 — Impact Assessment.** State the business impact descriptively (column 6), then rate it H/M/L (column 7).

**Step 4 — Vulnerability Assessment.** Assess the strengths and weaknesses of your systems, processes and culture (columns 8–9). **The trick: ignore any additional controls already in place and assess the vulnerability as if nothing special had been done — the "green field" vulnerability assessment.** This is difficult because people always want to explain how they have already done enough to prevent the threat, but it lets you assess the contribution of *all* planned controls over and above the greenfield situation. *Several vulnerabilities may map to each threat — split the table again to address each individually.*

*(Note: the same method can be applied to a security review of an existing system, but the approach differs subtly, because a review is an operational activity rather than an architectural development activity — see Chapter 15.)*

**Step 5 — Risk Category.** Calculated directly from impact rating × vulnerability rating using Figure 9-3, giving categories A–D (Table 9-9). Enter the greenfield risk category in column 10, and **colour-code the cell background to produce a traffic-light system of risk reporting that is easily understood.**

**Then (Chapter 10) — Risk Mitigation.** Column 11 states the high-level control objectives (part of the conceptual architecture), column 12 the target vulnerability after mitigating controls, column 13 the mitigated risk category calculated from that target vulnerability.

**The IBFS rows as printed:**

| 1 ID | 2 Business Driver | 3 Business Attributes | 4 Business Requirement | 5 High-Level Threat | 6 Business Impact | 7 Impact |
|---|---|---|---|---|---|---|
| **BD001** | "The customer is king" | | | | | |
| BD001-1 | Customer experience impacts competitive advantage or disadvantage | **Usability** | Security features of any customer-facing business system must not create difficulties in use | Customer becomes frustrated by difficult login processes and other security features | Many customers go somewhere else where the experience is easier | **H** |
| BD001-2 | Business in the future will be customer-driven | **Trustworthy, Private, Confidential** | Customers who provide private information must be confident that it will be protected from disclosure | Customer details disclosed to unauthorised parties, and this becomes generally known | Wide loss of customer confidence; censure or prosecution by the regulators; **eventual loss of operating licence** | **H** |
| **BD002** | "We must comply with the law" | | | | | |
| BD002-1 | Data protection legislation | **Compliant, Private, Confidential** | Must comply with data protection legislation | Customer details disclosed to unauthorised parties, and this becomes generally known | Wide loss of customer confidence; prosecution by the regulators | **H** |

| 8 Potential High-Level Vulnerability | 9 Green Field Vuln | 10 Green Field Risk Cat | 11 High-Level Control Objectives | 12 Target Vuln | 13 Mitigated Risk Cat |
|---|---|---|---|---|---|
| Multiple logins and authentications, each requiring a new password | **H** | **A (Red)** | Establish consistent, easy-to-use authentication and login procedures | **L** | **C (Green)** |
| Inadequate control over privacy of information | **H** | **A (Red)** | Establish strong **physical** security surrounding all customer data, in transit, during processing and in storage; establish strong **logical** security surrounding all customer data, in transit, during processing and in storage | **L** | **C (Green)** |
| Inadequate control over privacy of information | **H** | **A (Red)** | Establish strong physical security surrounding all customer data, in transit, during processing and in storage; establish strong logical security surrounding all customer data, in transit, during processing and in storage | **L** | **C (Green)** |

The point of the model: three separate red (category A) risks — one usability risk, two privacy risks arising from two different drivers — resolve into a small set of high-level control objectives that then drive detailed control design at the logical, physical, component and operational layers.

### Case Study: Remote Control of an Electrical Power Plant
The authors visited an electrical power plant in a small island economy with a single electricity company owning a single generating plant — "a friendly, family sort of place… not really exposed to some of the dirtier aspects of life in the bigger world." Staff proudly explained that they now had complete control of the entire plant from a computerised control room, and more: **Harry, the deputy power plant manager and "a bit of a whiz with ICT," could control the plant from his laptop in his living room at home, dialling up to the control room.** Asked what security protected that, they had never considered the question. *"The benefits were clear, but the risks were not."* The authors add the sting: "if we went to a similar company of a large size serving half of the USA, would we get a different answer?"

**Generalising it — you have a remote process control problem if you answer yes to any of:** Do you run a data communications network? Do you run remote computer systems? Do you have a network management centre and perhaps a systems management centre? Do you use standard network management protocols (such as **SNMP version 1**) for managing your network?

The SNMP analysis: three basic elements — **GET** messages (enquire the current status of a managed device), **SET** messages (set up and configure the parameters of the managed device, on-line), and **TRAP** messages (a managed device reporting an unexpected or asynchronous event, similar to an interrupt). The only mechanism in SNMPv1 to authenticate these messages is a fixed password, the **community name**, sent in cleartext and **common to a group of devices**. Anyone who can intercept the message path can change existing SNMP messages or insert their own and take over control of the network — for example, close down all or part of it.

### Case Study: A Major Healthcare Administration Agency
An administration agency within a national government department, used to show how strategic business development drives the security programme.
*Six strategic goals:* protect and improve citizen health and satisfaction; purchase the best value health care for citizens; promote the fiscal integrity of the agency programmes; provide leadership in the broader public interest to improve health; promote citizen understanding of the agency and its programmes; foster excellence in the design and administration of the agency's programmes.
*Strategic objectives — Customer Services:* improve citizen satisfaction with programmes, services and care; enhance citizen programme protection; increase the usefulness of communications with citizens; increase the usefulness of communications with business partners, service providers and other agencies; ensure programmes and services respond to the health care needs of citizens.
*Quality of Care:* improve health outcomes; improve access to services for under-served and vulnerable citizen populations; protect citizens from substandard care.
*Programme Administration:* build a high-quality, customer-focused team; enhance programme safeguards; maintain and improve the agency's position as a prudent programme administrator and accountable steward of public funds; increase public knowledge of the financing and delivery of health care; improve the agency's management of information systems and technology.

**The infrastructure these goals imply:** good operational risk management practices; protection of the trust relationships between the agency, beneficiaries, health care providers and other parties; protection of the confidentiality, integrity and authenticity of electronic communications and information transfers automating the health care business processes; protection of the privacy of individual beneficiaries' health records; maintaining continuous service availability; authenticating all parties in eHealthcare communications and holding all parties accountable for electronic transactions made in their name.

## Mental Models
- **Greenfield first.** Assess vulnerability as if no controls existed, so you can measure what the planned controls actually contribute.
- **Threats are binary; impact and vulnerability are graded.** Do not waste effort trying to quantify threat probability you cannot know.
- **Traffic lights on the risk model.** Colour-code A/B/C/D red/amber/green/blue so the report is instantly readable by non-specialists.
- **P > D + R.** Security is a race between how long you resist and how long it takes you to notice plus react.
- **Threat scenarios beat threat lists.** Agent, capability, motivation, opportunity, catalysts, inhibitors and amplifiers give a far richer basis for risk decisions.
- **Every "yes" to a remote-management question is a process-control risk** — network management by SNMPv1 is the same class of problem as remotely controlling a power station.

## Anti-patterns
- Letting people explain away vulnerabilities with controls they have already put in place, defeating the greenfield assessment.
- Attempting quantitative threat probability without reliable loss-event data.
- A one-row-per-requirement risk table when several threats or several vulnerabilities apply — the rows must be split.
- Treating past events as a reliable guide to future threat.
- Deploying remote process control (including standard network management) without asking the security question.
- Archiving records for a regulatory retention period without preserving the means to read the media.
- Security mechanisms whose lifetimes are shorter than the transactions they secure.

## Key Takeaways
- Contextual architecture = the description of the business, cell by cell across the contextual row.
- Operational risk (Basel II): loss from inadequate or failed internal processes, people and systems, or from external events.
- Threats live outside your control (identify only, binary); vulnerability and impact live inside it (rate H/M/L).
- Threats Database = four domains (People, Processes, Systems, External) × sixteen practice-derived categories.
- Threat scenarios add agent, capability, motivation, opportunity, catalysts, inhibitors and amplifiers.
- The Business Risk Model has 13 columns; 1–10 contextual, 11–13 conceptual, with the greenfield → mitigated risk category pair showing the value of the architecture.
- Risk categories A (red, severe) / B (amber, significant) / C (green, acceptable, monitor for creep) / D (blue, negligible).
- Time-Based Security: E = D + R − P; drive down detection and reaction time.

## Connects To
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — the contextual row cells this chapter populates.
- [Chapter 4 — Case Study: IBFS](ch04-case-study-ibfs.md) — the interviews supplying BD001 and BD002.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — the Business Attributes used in columns 3 and 4.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — Figure 7-4, the contextual development process and the Threats Database input.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — columns 11–13, control objectives and the domain and trust models the organisation/geography sections demand.
- [Chapter 15 — Operational Risk Management](ch15-operational-risk-management.md) — the operational treatment, and risk-based security reviews of existing systems.
