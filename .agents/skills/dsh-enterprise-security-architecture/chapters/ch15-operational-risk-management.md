# Chapter 15 — Operational Risk Management

## Core Idea
> "Operational risk kills organisations. It is difficult to define, and harder still to measure, but until you can measure it, you cannot control it."
> — *Risk Magazine*, November 1998

This chapter occupies the SABSA Matrix cell **Operational Risk Management — Operational row, Motivation (Why) column** (Chapter 3, Figure 3-3; Chapter 7, Figure 7-8). Its argument is that information security is *one silo inside* operational risk, not a discipline standing alongside it, and that:

> "Operational risk management is a trade-off — if there is a risk associated with taking a particular course of action, there is also a risk of not doing so. Furthermore, individual risks interact in complex ways, and **if you mitigate one risk you almost certainly increase at least one other risk in response.**"

The three aims of operational risk management:
- Understand the enterprise risk profile in detail;
- Make well-informed risk-mitigation **and risk-taking** decisions;
- Minimise the overall cost to the organisation, taking into account the widest possible view.

Ask three people what "operational risk management" means and you get three answers: **prevention of loss** (auditor), **cost-effective risk financing** (insurance manager), **managing volatility** (finance director).

## Frameworks Introduced

### Basel II's definition — and its exclusions
Basel II defines operational risk as *"the risk of losses resulting from inadequate or failed internal processes, people and systems, or external events."*

> "Some commentators often say that operational risk refers to all risks not included under credit risk or market risk, but **this is not true under this definition**."

**Included:** internal and external fraud; employment practices and workplace health and safety; clients, products and business practices; damage to physical assets; business disruption and system failures; legal risks; failure in execution, delivery and process management; sundry unspecified operational risks.
**Explicitly excluded:** reputation risk, strategic business risk, systemic risk. "Losses" also excludes indirect losses such as opportunity cost.

**Four Basel II domains: People, Processes, Systems, External Events.** The UK FSA's CP142, *Operational Risk Systems and Controls*, adds a **fifth domain: Outsourcing**.

**The book's cynical reading of the exclusions:**
> "A cynic might comment that the New Basel Accord requires a bank to measure certain operational risks 'because it can' and that those areas that are too difficult to measure are excluded, yet it is these very exclusions where the largest potential business impacts are to be found."

Reputation, systemic and strategic risk are **tail risks** — very low probability, very high impact. The counter-argument the book accepts: had they been included, *"the entire initiative would have failed through being too difficult to implement."*

### The three-pillar structure of the New Basel Accord
| Pillar | Content |
|---|---|
| Pillar 1 | Allocation of capital against measured value at risk (VAR) |
| Pillar 2 | Enhanced supervision of risk management in banks by national banking regulators |
| Pillar 3 | Improved public disclosure to market participants of each bank's risk exposure and risk management practices |

The incentive design: **reduced capital allocation traded against improved sophistication of risk measurement.**

### The VAR formula
```
VAR = Value of potential loss  ×  Probability of the event  ×  Probability of failure of controls
        Impact                 ×  Threat                    ×  Vulnerability
```

### The statistical modelling method (Figure 15-1)
**Frequency distribution:**
1. Collect historical data on frequency of loss events;
2. Analyse the sample — mean and standard deviation for frequency over a one-year period;
3. Estimate population statistics μ and σ;
4. Fit to a theoretical distribution — **the Poisson distribution is the most suitable**;
5. Goodness-of-fit test using **χ²**;
6. Use the fitted distribution as the assumed frequency distribution.

**Severity distribution:** the same six steps applied to *size* of losses, except **the Lognormal distribution is the most suitable**.

**Combination:** a **Monte Carlo simulation** takes the two theoretical distributions as inputs, randomly selecting sizes and frequencies, and generates the combined loss distribution.

**Banding the resulting curve:**
| Band | Range | Treatment |
|---|---|---|
| **Expected losses** — "the cost of doing business" | 0 < loss value < μ + σ | Operating expenses |
| **Unexpected losses** — severe | μ + 2σ | Capital financing |
| **Unexpected losses** — catastrophic | μ + 4σ | Hedging (transfer/insurance) or Accept |

Confidence intervals: ~95% at (μ + 2σ), 99% at (μ + 3σ), **99.9% at (μ + 4σ)**. *"The capital adequacy requirements of Basel II are that sufficient risk capital must be held to cover losses over a one-year period up to a 99.9% confidence interval"* — covering both expected and unexpected losses, all risk types and all lines of business.

**The goal of control** in these terms: *"push the mean (μ) as far to the left as possible … and to minimise the standard deviation (σ)"*, so that severe and catastrophic can be designated at four, five or even six standard deviations out.

### Why the statistics break down in the tail
- *"In the case of catastrophic tail risks the value of a potential loss is close to infinity in terms of the firm's ability to absorb it, and the probability of a loss event occurring is close to zero. **The multiplication of infinity by zero yields a mathematically indeterminate result**, and so the application of simple VAR computations in these cases is dubious."*
- *"One cannot accurately estimate a very low probability by looking at a loss database that contains zero occurrences of that event."*
- Even with perfect history: *"the past is not necessarily a good predictor of the future, since new threats are emerging all the time"* — the computer virus, unknown before the early 1980s; international terrorism in the early 21st century.

### The AS/NZS 4360 process (Figure 15-10)
*"There is one true national standard available"* — AS/NZS 4360:1999. Five steps, bound by **Communicate & Consult** across the top and **Monitor & Review** across the bottom, with a feedback loop from Step 5 back to Step 1.

| Step | What it requires |
|---|---|
| **1. Establish Context** | *Descriptive phase*: determine business objectives; define measurable success criteria; create risk measurements (qualitative or quantitative); distinguish clearly between likelihood and impact; take stakeholders' concerns into account. *Creative phase*: split the enterprise into key elements; focus thinking on them; stimulate creative thought through workshops and brainstorming; speak specific risk-oriented language |
| **2. Identify Risks** | What can happen and how; cover both threats and vulnerabilities; checklists may be used but **avoid preconditioning expectations** — use them to verify completeness; recommended approach is a structured brainstorming workshop; secondary approach is questionnaires, surveys or interviews; include a creative assessment of the future |
| **3. Analyse Risks** | Assign a significance rating using simple impact vs likelihood scales; custom modelling where complex; accept that first-pass ratings can be too high or too low |
| **4. Evaluate Risks** | Screen so the system does not get bogged down with too many items; prioritise relative to the complete set |
| **5. Treat Risks** | Decide preventive measures and contingency arrangements; **calculate and sign off the residual risk** — the business accepts this level |

*"The AS/NZS 4360 process bears many striking similarities to the SABSA approach, although both were developed independently of one another."* Figure 15-12 maps it onto the Chapter 5 feedback control loop (Figure 5-4). The AIRMIC/IRM/ALARM standard (Figure 15-11) *"differs only in minor details of description and language."* ISO/IEC Guide 73 supplies the vocabulary; there is no international equivalent of AS/NZS 4360.

### Eight risk management strategies
Reduce/mitigate · Transfer (insurance) · Avoid · Delay · Compensate (offset against benefits) · Spread · Accept · **Reject — "not a real risk at all!"**

### Four types of control (Figure 15-13)
| Control type | What it reduces |
|---|---|
| **Deterrent** | Reduces a **threat** |
| **Preventive** | Reduces a **vulnerability** |
| **Corrective** | Reduces an **impact** |
| **Detective** | Detects a problem and **triggers other controls** |

### The Enterprise Risk Management Framework cube (Figure 15-4)
Three dimensions of the board's view:
- **Risk dimension** — a hierarchical taxonomy of risks, classified and grouped as a framework for risk identification;
- **Enterprise organisation dimension** — risk understood and managed at every level of organisational granularity;
- **Process dimension** — all major process-related aspects of risk management addressed.

The process face breaks out into five sub-models: the External Environment (Figure 15-5), the Internal Environment (Figure 15-6, "conceptualised as a wheel with the corporate policies … as the hub"), the Governance Process (Figure 15-7), a Generic Risk Management Process (Figure 15-8), and an Operational Risk Management System architecture (Figure 15-9).

## Reference Tables

### Table 15-1: Operational Risk Categories (with the information/ICT mapping)
| Operational risk area | Description | Information or ICT mapping |
|---|---|---|
| **Facilities and Operating Environment Risk** | Loss or damage to operational capabilities caused by problems with premises, facilities, services or equipment | Business continuity management for ICT facilities |
| **Health & Safety Risk** | Threats to personal health and safety of staff, customers and public | Confidentiality of home addresses and travel schedules |
| **Information Security Risk** | Unauthorised disclosure or modification, loss of availability, or inappropriate use of information | All aspects of information and ICT security |
| **Control Frameworks Risk** | Inadequate design or performance of the existing risk management infrastructure | Business process analysis to identify critical information flows and control points |
| **Legal & Regulatory Compliance Risk** | Failure to comply with laws, regulatory/reporting/taxation standards or contracts; failure of contracts to protect business interests | Data protection legislation and cryptographic control regulations; accuracy and timeliness of information reported to regulators; content management of all information sent to other parties |
| **Corporate Governance Risk** | Failure of directors to fulfil their personal statutory obligations | Information security policy making, performance measurement and reporting |
| **Reputation Risk** | Negative effects of public/customer opinion and market reputation; damage to the brand; failure to manage PR | Controlling disclosure of confidential information; presenting a public image of a well-managed enterprise |
| **Strategic Risk** | Failure to meet long-term strategic goals, including dependence on outcomes controlled by third parties | Quality and granularity of information behind strategic decisions (mergers, acquisitions, disposals) |
| **Processing and Behavioural Risk** | Failure of internal controls, information systems, employee integrity, errors and mistakes, weak operating procedures | All aspects of information systems security and security-related behaviour of employees |
| **Technology Risk** | Failure to plan, manage and monitor technology-related projects, products, services, processes, staff and channels | Failure of ICT systems; need for business continuity management |
| **Project Management Risk** | Failure to plan and manage resources for tactical project goals — budget/time overruns, non-completion, technical failure, integration failure | Management of all security-related projects |
| **Criminal and Illicit Acts Risk** | Fraud, theft, wilful neglect, gross negligence, vandalism, sabotage, extortion | Security services and mechanisms to prevent all types of cyber crime |
| **Human Resources Risk** | Failure to recruit, develop or retain appropriately skilled employees, or to manage employee relations | Policies protecting employees from sexual harassment and racial abuse through corporate e-mail |
| **Supplier Risk** | Failure to evaluate supplier capabilities; supply-chain breakdowns; sub-standard delivery | Outsourced service delivery of ICT or other information-processing activities |
| **Management Information Risk** | Inadequate, inaccurate, incomplete or untimely information for decision making | Accuracy, integrity, currency, timeliness and quality of decision-support information |
| **Ethics Risk** | Unethical business practices, including those of partners — discrimination, child labour, pollution, green issues, disadvantaged groups | Ethical collection, storage and use of information; content management on web sites, Intranets, e-mail, IM |
| **Geopolitical Risk** | Political instability, poor infrastructure in developing regions, cultural differences and misunderstandings | Information and ICT security in regions with special geopolitical risks |
| **Cultural Risk** | Language, religion, morality, dress codes and other community customs affecting employees, customers, stakeholders | Content management on web sites, Intranets, e-mail and instant messaging |
| **Climate and Weather Risk** | Drought, heat, flood, cold, storm, winds | Business continuity management for ICT facilities |

### The regulatory drivers
| Regulation | Jurisdiction / sector | Key security-relevant requirements |
|---|---|---|
| **Sarbanes-Oxley Act 2002** (signed 30 July 2002, after Enron and WorldCom) | USA, publicly traded companies | Board oversight of financial accounting; **audit trail and financial records kept a minimum of seven years**; independent external audit not linked to consulting; confidential anonymous whistle-blowing; executive certification of quarterly/annual reports; management assessment of internal controls; accelerated real-time reporting; code of ethics; corporate and criminal fraud accountability; white-collar crime penalty enhancements |
| **USA Patriot Act 2001** (signed 26 October 2001; 342 pages, amends 15 statutes) | USA | Expanded government surveillance with reduced checks and balances; **requirement to gather and archive ALL documents and communications for seven years** — call logs, corporate e-mail, corporate instant messaging |
| **Basel II / New Basel Accord** | Banking, G10 (committee founded 1975 under the BIS; meets four times a year; ~30 working groups) | Capital allocation against operational risk VAR; three-pillar structure. The 1988 Capital Accord addressed credit risk; the January 1996 fifth amendment added market risk; operational risk appears in neither |
| **Gramm-Leach-Bliley Act** (enacted 12 November 1999; privacy regulations effective 13 November 2000) | USA, financial institutions | Privacy of consumer financial information for personal, family or household use |
| **HIPAA 1996** | USA, health care | Standards for technical capabilities of health record systems; cost/benefit evaluation of security measures; security training; audit trails; security policies and procedures; risk assessment against **"reasonably anticipated threats"**; integrity and confidentiality of health information; electronic signatures |
| **CAD3** | EU | Transposes the New Basel Accord into EU law, extending it to all credit institutions and investment firms across the (then) 25-member EU |
| **Combined Code / Turnbull / Higgs / Smith** | UK, LSE-listed companies | Principle D.2 — *"maintain a sound system of internal control"*; Provision D.2.1 — annual directors' review of effectiveness covering financial, operational and compliance controls; D.2.2 — companies without internal audit should review the need for one. Turnbull adds five board considerations. Higgs/Smith (both 20 January 2003) strengthened the June 2003 reissue: split chairman from CEO, stronger non-executive directors, boardroom transparency, auditor independence |
| **FSA Integrated Prudential Sourcebook** | UK, all financial services | Standards for credit, market, **operational**, insurance, liquidity and group risk; adequate financial resources (solvency, capital adequacy) **and appropriate systems and controls**. CP142 adds Outsourcing as a fifth operational-risk domain |
| **21 CFR Part 11** (effective 20 August 1997) | USA, food and drug / pharmaceuticals | Trustworthiness, privacy and reliability of electronic records and signatures. Does **not** mandate electronic recordkeeping — but if you keep records electronically, compliance is mandatory |
| **FAA / CAA and equivalents** | Civil aviation | Licensing of aircrew, mechanics, engineers, air traffic controllers, aircraft, airports, airlines — an electronic "paper trail" whose **integrity and availability** determine whether an aircraft can take off or a person can work |
| **EU Data Protection Directive** | EU member states | Defines personal data, processing, data subjects, data controller, data processor, recipient, third party — and eight principles |

**The eight data protection principles:** processed fairly and lawfully · obtained only for specified lawful purposes and not further processed incompatibly · adequate, relevant and not excessive · accurate and kept up to date · not kept longer than necessary · processed in accordance with data subjects' rights · appropriate technical and organisational measures against unauthorised or unlawful processing and against accidental loss, destruction or damage · not transferred outside the EEA unless that country ensures an adequate level of protection.

### Business impact categories (the checklist)
Loss of revenue income · loss of capital value · increased operating costs · opportunity costs · damage to reputation · bad publicity · loss of key customers · loss of key suppliers · loss of market share · loss of management control over the business · damage to the entire market · damage to customer confidence · damage to employee confidence and morale · damage to investor and shareholder confidence · breach of law or regulations leading to **fines, withdrawal of operating licence, custodial jail sentences for senior executives**.

Quantitative impact measures named: **Annual Loss Expectancy (ALE)**, **Value at Risk (VAR)**, **Risk-Adjusted Return on Capital (RAROC)**.
Qualitative bands (the SABSA Method, Chapter 9) use only three: **High** — could potentially do great damage; **Medium** — could do significant damage; **Low** — only minimal damage.

### Table 15-6: Risk Category Mapping Matrix
| Vulnerability ↓ / Business Impact → | Low | Med | High |
|---|---|---|---|
| **High** | C (green) | B (amber) | **A (red)** |
| **Med** | C (green) | B (amber) | B (amber) |
| **Low** | C (green) | C (green) | C (green) |

| Rating | Meaning |
|---|---|
| **A (red)** | Unacceptable — system fails the review |
| **B (amber)** | Business risk decision needed |
| **C (green)** | Acceptable without further consideration |

*"Rating A is a clear fail, and rating C is a clear pass, but rating B is more complex."*

Vulnerability rating definitions: **High** — current controls are poor and it is very likely they will fail; **Medium** — controls are fair, but may fail; **Low** — current controls are good and unlikely to fail.

### Table 15-2: The Risk Register structure
| Field | Notes |
|---|---|
| Risk ID | Unique identifier to avoid cross-referencing confusion |
| Risk Name | |
| Risk Type | Operational, strategic, reputation, credit, market, liquidity, etc. — **the register includes ALL risks, not just operational** |
| Threat Description | |
| Business Impact Description & Rating (H/M/L) | |
| Vulnerabilities & Overall Likelihood (H/M/L) | |
| Risk Category Rating (A/B/C/D) | |
| Owner | *"Assigning ownership of risk is critical to successful risk management"* |
| Date Identified / Last Updated / Date Closed | Closed items retained for historical analysis, possibly in a separate 'closed risks' table |
| Actions Planned (URL) / Current Status Report (URL) | Held as hyperlinks because full documents are too detailed for table cells |

### Table 15-3: Role management for risk register access control
| Role | Privileges |
|---|---|
| **Risk Manager** | Full access to the entire table and **the only role with Delete access** |
| **Risk Owner** | Read access to all entries; Create/Write/Update access to owned entries |
| **Risk User** | Read access to all entries |
| **Restricted Risk User** | Read access only to entries flagged as unrestricted |

Add a risk register transactions table recording all additions, changes and deletions to create an audit trail. Sensitive items are flagged with an extra field.

### Risk financing instruments
| Instrument | What it is |
|---|---|
| **Banker's Blanket Bond** (fidelity bond) | Direct theft of cash or funds in transit or on premises, employee infidelity, forgery |
| **Professional Indemnity (PI) / Errors & Omissions (E&O)** | Failures in human professional judgement |
| **Directors and Officers (D&O)** | Liabilities of those with formal responsibilities under company legislation |
| **Traditional property insurance** | Buildings, equipment, plant, finished goods, raw materials |
| **Traditional crime policies** | Theft |
| **Specialised computer crime policies** | Direct loss of tangible property — cash, negotiable instruments, securities |
| **Business interruption insurance** | Lost net profits and continuing expenses from temporary shutdown by fire or other insured peril, including cost of operating at a temporary location |
| **Single parent captive** | Insurance company owned and controlled by one company, insuring only it and its subsidiaries |
| **Group captive** | Owned by two or more non-affiliated organisations it insures; all mutuals are in theory group captives; intergovernmental pools are captives owned by the public agencies they insure |
| **Rental captive ("rent-a-captive")** | A front company's captive rented out to firms too small to form their own — *"perfectly legal and provides a viable alternative"* |
| **Finite insurance** | Coverage limit, time period and premium acknowledge the time value of money; most premiums go into a fund wholly owned by the insured, which accrues interest and pays losses, with the balance returned at end of period; the third party acts as reinsurance |
| **Capital markets products** | Hedge funds, bonds, options and derivatives linked to specific risk types |

**The three insurance failure modes for information risk:**
1. **The definition of property.** *"One of the most important of these with regard to information security risk is the exclusion of coverage for loss of confidential or proprietary information. It usually comes down to the definition of property, which is defined as something tangible under the terms of the insurance policy."*
2. **Indirect or consequential loss is excluded** — business interruption from fire is coverable; interruption from employee dishonesty or extortion generally is not. Fidelity bonds do not cover loss from unauthorised actions of an employee, customer or any party who had authorised access and abused those privileges.
3. **Aggregation.** Insurance works on spread; a single cyber attack (the book names **I Love You** and **Melissa**) could cause billions across an insurer's entire client population — potentially causing *"the financial demise of the insurance company, which is in no one's interests."* And there is no long loss history for actuarial analysis, so *"insurance products tend to lag behind the development of digital business by a considerable margin."* Against that, *"as much as 50 percent of the value of new businesses in the knowledge economy can consist of intangible assets in the form of information."*

## Worked Example

### The Interstellar Travel Engineering Company (ITEC) — an eleven-step risk-based security review
**The brief:** ITEC asked a team of specialist information risk consultants to review the risks in its main business information system and the security of that system. The company wanted a report highlighting key risk areas, **clear justified recommendations for additional controls where risk was judged unacceptable, and an assurance that once the new controls were applied the risks would be reduced to an acceptable level.**

The difference from the Chapter 9 assessment: *"in the enterprise architecture development one is providing for a generic set of systems, whereas in a risk-based system security review, the specifics of the system are already known."*

| Step | Who runs it | Activity | Output |
|---|---|---|---|
| **1** | Business analyst workshops with **business management** | Build the SABSA Business Attributes Profile — relevant attributes, their ITEC-specific definitions, metric types, measurement approach, performance targets | Business Attributes Profile |
| **2** | Business analyst workshops with **operational management** | Build detailed business process maps using a specialised process-mapping tool, decomposing to the granularity that gives full understanding — knowing which parts are automated and which manual | Business process maps |
| **3** | Risk analyst workshops with **operational management** | Walk each process step by step against the threats database: what is the requirement for security and control here, and what threatens it? | Working Table cols 1–4: Risk ID, Bus. Proc. Ref., Business Requirement, Threat Description |
| **4** | Risk analyst workshops with **business management** | Cross-reference each business requirement and threat to the Business Attributes Profile; rows split where a threat affects more than one attribute | Cols 5–6: Business Attributes, Performance Targets |
| **5** | Risk analyst workshops with **business management** | Brainstorm the business impact of each threat materialising, **based on potential failure to meet the Performance Targets** | Col 7: Business Impact + H/M/L rating |
| **6** | Risk analyst workshops with **operational management plus technical experts** | Describe current controls and residual vulnerabilities; feed in results of vulnerability scanning tools and a specialist penetration testing team | Cols 8–9: Current Controls, Current Vulnerabilities + H/M/L |
| **7** | Risk analyst activity | Apply the Table 15-6 mapping matrix to impact × vulnerability | Col 10: Risk Rating A/B/C, colour-coded |
| **8** | Risk analyst activity | Extract every instance of each Business Attribute; show each once, coloured by the **highest** risk recorded against it | **Risk-High Business Attributes Current Status** (Figure 15-16) |
| **9** | Risk analyst activity | Same extraction, but colour each attribute **proportionally** to the spread of its risk ratings | **Proportional-Risk Business Attributes Current Status** (Figure 15-17) |
| **10** | Risk analyst workshops with **operational management** (same team as Step 6) | For each red or amber threat, propose additional controls | Risk Mitigation Working Table (Table 15-5), cols 1–10 |
| **11** | Risk analyst activity | Redraw Steps 8 and 9 for the post-mitigation state | **Risk-High** and **Proportional-Risk Business Attributes Projected Status** |

**The Risk Mitigation Working Table's decision columns:**
| Col | Field | Note |
|---|---|---|
| 5 | Current Risk Rating | **Only red and amber ratings are selected for this table** |
| 6 | Proposed Improvements to Mitigate Risk | May instead record a management decision to accept an amber risk |
| 7 | Cost Factor | H/M/L |
| 8 | Difficulty Factor | H/M/L |
| 9 | Projected Vulnerability | *"The usual target will be L"* |
| 10 | Projected Risk Rating | Mapped from Table 15-6. *"The usual target will be C"* |

**Why Steps 4 and 5 route through the Business Attributes rather than going straight from threat to impact** — the chapter answers its own anticipated objection:
> "The reason for using the Business Attributes Profile is to **normalise all impact assessments against the performance targets agreed by senior management.** There is a potential problem if this is not done that individual operational managers will either underestimate or overestimate the true level of business impact… By referencing everything back to a Business Attributes Profile that has an enterprise-wide applicability, **the variability of opinion is eliminated from the process**, and the results are far more meaningful and reliable."

**The presentational payoff:** senior management sees the profile it already signed off, first with red and amber lights, then — after planned mitigations — mostly green. *"This is a powerful way to present the business case for the investment needed for the risk mitigation projects that are planned."*

### The Cost of Computer Virus Control — why unadjusted cost is illusory
A small company with a number of PCs. The ICT manager decided the computer virus problem was overrated and implemented **no anti-virus software, processes or procedures** — data interchange with the outside world was small, and in his opinion the risk was negligible.

> "All went well for two years, and the CEO of the company was very pleased that the ICT department was able to keep down its costs to such a low level. Then, about six months into the third year, the company suffered a major virus infection that was initially introduced through an e-mail… The entire business operations of the company were affected for several weeks. Productivity was hit and revenues were decimated. Customers were unable to be serviced, and many took their business elsewhere. **What a piece of bad luck!**"

The lesson, framed against Figure 15-2 (Optimising the Risk-Related Total Cost):
> "It is easy to keep your short-term costs down if you take enormous risks, but the longer-term view is different — because sooner or later the risks will bite and the costs of those events will be borne… In this case study over the first two years both the cost of control and the cost of losses were zero, but that cannot be sustained forever, **because the risk factor was missing from the calculation.**"

Total cost = cost of control + cost of losses. Figure 15-14 puts the same idea as a two-axis judgement: low investment in control against high risk means **over-exposure**; high investment against low risk means **excessive cost of control**; the answer is *"that middle ground where the optimum position is to be found — the appropriate level of control."*

### The three-view risk dashboard (Figure 15-18)
Using the Business Attributes taxonomy diagram (Figure 6-4) directly as the dashboard, with each attribute box a hyperlink drilling down to proportional risk status, measurement approach, metric types, performance targets and collected metrics:

| View | What it shows | Expected picture |
|---|---|---|
| **Green field** (pure risk) | Before any controls of any kind — the Chapter 9 concept | *"Many red lights showing, since there is no mitigation in place"* |
| **Post-baseline** | After the organisation's mandatory baseline control standard, applied regardless of risk assessment | *"Most of the red lights"* off, some amber remaining borderline |
| **Post-special treatments** | After treatments addressing the remaining red and amber | *"Mostly one of green lights"* |

The example fragment for a single attribute: Risk-High shows Admissible as red at green field, amber post-baseline, green post-treatment; Proportional-Risk shows the same attribute as 55% / 36% / 9% red-amber-green, then 32% / 58% / …, then **100% green**.

**Why baseline plus special treatment:** *"All controls in the baseline standard are mandatory and must be applied regardless of the risk assessment. However, applying the baseline controls may not mitigate all risks to an acceptable level, leaving some that require special treatment above and beyond the baseline."*

**And why the dashboard matures:** *"The first time you make the selection of controls it has to be done on a fairly subjective basis, but as the data in the risk management dashboard mature you accumulate real metrics on the effect of each individual control or group of controls. Later selections as part of new projects eventually become much more objective."*

## Mental Models
- **Every mitigation creates a risk.** The book's own footnoted example: easily opened fire escape doors versus doors locked against unauthorised access — *"there have been many tragic events, especially in nightclubs and similar venues, where this conflict has led to multiple deaths and injuries."*
- **Risk silos are a filing convenience, not a property of the world.** *"Just because you classify risks into silos for the convenience of identifying and managing them does not mean that they actually possess that discrete granularity."*
- **The exclusions are where the money is.** Basel II measures what it can measure; the tail is what kills you.
- **Threat × Vulnerability × Impact.** Deterrent hits threat, preventive hits vulnerability, corrective hits impact, detective triggers the others.
- **Risk-adjusted cost is the only real cost.** A zero-control, zero-loss year is a loan, not a saving.
- **Normalise impact through the Business Attributes Profile** so one operational manager's opinion cannot contaminate the senior management view of priorities.
- **Insurance follows spread.** One house: insure it. A thousand houses in distant locations: self-insure. A thousand houses crowded together: insure again.
- **Traffic lights beat numbers with senior management** — and where numbers exist they should be risk-adjusted KPIs, not raw ones.
- **The governance process and the risk management process are both feedback control loops** (Chapter 5, Figure 5-4). So is AS/NZS 4360.

### Benefits of good operational risk management (as the book sells it upward)
Keeping senior executives and board members **out of jail** · retaining an operating licence in a regulated industry · avoiding fines · cost reduction through reduced operational losses and improved process efficiency · reduced regulatory and economic capital allocation · improved shareholder value and confidence, *"especially valuable in times of crisis when shareholder trust is stressed to its maximum limits"* · risk-adjusted KPIs as better measures of true performance · competitive advantage through better decision support · more reliable forecasting · improved relationships with regulators.

## Anti-patterns
- Treating operational risk as "everything that isn't credit or market risk" — Basel II's definition explicitly does not work that way.
- Silo risk management: *"a corporate security team, a legal team, an insurance team, an ICT security team, a public relations team, a health and safety team, and so on, each working relatively independently of one another and unaware of the conflicts between each others' risk areas."*
- Computing a VAR in the catastrophic tail — infinity times zero.
- Estimating a very low probability from a loss database containing zero occurrences of that event.
- Relying on historical data alone: *"If you rely upon historical data only, you will never predict them."*
- Playing *"guessing games with numbers that are unreliable"* instead of staying within qualitative bounds where the data does not support quantification.
- Using checklists in a way that preconditions expectations rather than verifying completeness.
- Letting the risk item list grow unscreened until the system is *"bogged down with too many risk items."*
- Going straight from threat to business impact without the Business Attributes Profile, so estimates vary by whoever is in the room.
- Leaving the risk register with no named owner per risk, or with no access control — *"the risk register is an information repository that requires its own authorisation and access control."*
- Discarding closed risk entries instead of retaining them for historical analysis.
- Assuming insurance covers loss of confidential information (it usually does not — information is not tangible property) or consequential loss from employee dishonesty (excluded from fidelity bonds).
- Insuring a widely spread portfolio of risks that would be cheaper to self-insure.
- A blame culture in risk reporting — the framework explicitly calls for *"a no-blame enterprise risk management culture that encourages more honesty in risk event reporting and reduces the temptation to cover up mistakes."*

## Key Takeaways
- This chapter fills the Operational row / Motivation column of the SABSA Matrix: information security risk is one category inside a nineteen-category operational risk taxonomy (Table 15-1).
- Basel II: operational risk = losses from inadequate or failed internal processes, people and systems, or external events; four domains (People, Processes, Systems, External Events) plus the FSA's fifth (Outsourcing); reputation, strategic and systemic risk excluded — which is exactly where the largest impacts sit.
- VAR = Impact × Threat × Vulnerability. Frequency fits Poisson, severity fits Lognormal, the two combine by Monte Carlo; expected losses lie below μ+σ, and Basel II requires capital to the 99.9% confidence interval at roughly μ+4σ.
- Quantitative methods fail for want of loss data and because the past does not predict new threats; qualitative High/Medium/Low with traffic lights is the pragmatic default, and the SABSA Method (Chapter 9) is the reference implementation.
- AS/NZS 4360 — Establish Context, Identify, Analyse, Evaluate, Treat, wrapped in Communicate & Consult and Monitor & Review — is a feedback control loop and independently converges on the SABSA approach.
- Four control types map one-to-one onto the risk equation: deterrent→threat, preventive→vulnerability, corrective→impact, detective→triggers the rest.
- The eleven-step ITEC review is the chapter's reusable procedure, and its distinctive move is routing impact assessment through the Business Attributes Profile to normalise opinion.
- Table 15-6 turns impact × vulnerability into A (fail) / B (management decision) / C (pass).
- The risk register holds all risk types with a named owner per risk, hyperlinked action plans, retained closed items, and its own access control model.
- Residual risk is financed: accept (self-insurance, budgeted loss, policy excess, capital reserves, captives, finite insurance) or transfer (third-party insurance) — but information risk is poorly served by both, since information is not tangible property and there is no actuarial history.
- The risk management dashboard is the Business Attributes taxonomy in traffic lights, presented in three views — green field, post-baseline, post-special-treatment — with drill-down through hyperlinks, and XBRL as the emerging interchange standard for rolling risk data up from satellite applications.

## Connects To
- [Chapter 5 — A Systems Approach](ch05-a-systems-approach.md) — the feedback control loop (Figure 5-4) onto which the governance process, the generic risk management process and AS/NZS 4360 are all mapped; dependency tree modelling as a vulnerability assessment technique.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — the Business Attributes Profile, its performance targets and metrics, the Security Management Dashboard (Figure 6-1) and the taxonomy diagram (Figure 6-4) reused here as the risk dashboard.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — the prerequisite reading: assets/threats/impacts/vulnerabilities, the threat-modelling framework, the SABSA Method for qualitative risk assessment, and the green field risk view.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — the conceptual strategies for layering controls (Figures 10-1 and 10-2) that the cost/benefit analysis here is applied to.
- [Chapter 13 — Component Security Architecture](ch13-component-security-architecture.md) — XML, from which XBRL and the emerging risk-reporting sub-set derive.
- [Chapter 14 — Security Policy Management](ch14-security-policy-management.md) — the over-arching Business Operational Risk Management Policy that sits above information security policy, and the no-blame reporting culture.
- [Chapter 16 — Assurance Management](ch16-assurance-management.md) — audit and assurance as the verification half of the control loop.
- [Chapter 17 — Security Administration and Operations](ch17-security-administration-and-operations.md) — penetration testing and vulnerability scanning, used as inputs to Step 6 of the ITEC review.
