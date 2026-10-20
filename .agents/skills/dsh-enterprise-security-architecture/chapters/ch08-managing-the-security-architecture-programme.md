# Chapter 8 — Managing the Security Architecture Programme

## Core Idea
Developing the architecture is the technical half; **the political and cultural half decides whether it survives**. This chapter is about selling the benefits, obtaining sponsorship and budget, building the right team, gathering information from senior executives, obtaining consensus from technical management, enforcing compliance through governance, maintaining the architecture, and — the lesson learned the hard way — keeping senior management confident over the long elapsed time of the programme.

## Frameworks Introduced

### The five rules for influencing opinion or behaviour
1. Address your remarks, document or presentation **only to the group whose opinion or behaviour you seek to influence**. Do not make it a catch-all for everyone.
2. Speak **only in the language, concepts and terminology used by that group**. Avoid terminology they are unfamiliar with.
3. Address **only those issues that are of interest to them**. Avoid issues that are not.
4. Focus on a set of **key messages** you want them to assimilate, and craft the communication around them.
5. **Prioritise your material, most important first.** If their attention span is short or they are distracted, at least they received your most important message before you lost them.

*Why these feel counterintuitive:* scientists and technologists are taught to develop a logical argument, present the steps in order, and deliver the conclusion at the end. Senior business people want the bottom line first, so they can judge rapidly whether to invest more time. **You may only ever get one chance.**

Applications beyond the business case: writing security policies aimed at a specific user group; the executive summary of a proposal (begin with the most important *benefit*); the executive summary of a consulting report (start with the most important *recommendation*).

### Definition of a team
A team is two or more employees who are organisationally empowered to: establish objectives; make decisions about how to achieve those objectives; undertake tasks to meet the objectives; be accountable (individually and collectively) for the results.

### Belbin's eight team roles
Successful teams contain all eight; some roles can be filled only once.
| Role | Characteristics |
|---|---|
| **Plant** | Creative, innovative thinker producing ingenious new ideas and novel strategies. Bright, radical, provocative, but not all ideas are practical. Usually introverted, works alone, thinks intensively. The ideas engine, though not really team-oriented. **Only one is needed — two is counterproductive.** Needs a good co-ordinator to draw out the best ideas, a monitor evaluator to reject the infeasible, and a resource investigator to make the good ones work. |
| **Co-ordinator** | Not necessarily intellectually outstanding but carries others' respect. Broker of harmony and consensus; redraws the team together after conflict. Can be overall leader, leading in a quiet, understated way. Dominant without being over-assertive. Goal-oriented, with personal energy and enthusiasm that motivates the whole team. |
| **Resource Investigator** | Not an original ideas person but very good at picking up others' ideas and making them work. Good at looking for resources beyond the immediate team, probing for information wherever it can be found. Strong interpersonal skills — sociable, friendly, fairly extroverted. Thinks on their feet, adaptable, finds ways to make things work when failure looms. An explorer, improviser and negotiator. |
| **Monitor Evaluator** | Intelligent, discerning, objective. Resolves debates and conflicts between innovative members by weighing facts dispassionately and deciding which competing idea is best. Not an achiever but an excellent judge. Intellectually stronger than the co-ordinator, so able to evaluate others' ideas. |
| **Shaper** | Highly motivated, high achiever with a lot of nervous energy. May be team leader, but in a very different style to a co-ordinator: the co-ordinator is highly moral, the shaper so strongly goal-oriented they will reach the goal by whatever means it takes, legitimate or not. Aggressively extroverted, single-minded, impatient, often critical of those who appear obstructive. Personifies the "JDI" (just do it) culture. **More than one shaper is usually counterproductive** — interpersonal problems between them. |
| **Team Worker** | Hard-working, conscientious, good communicator, trusting, sensitive, caring; puts group objectives and success above personal ambition. Perceptive and diplomatic; smooths conflicts between other personality types. Can be team leader and often makes a good senior manager, especially over junior line managers who are shapers. |
| **Completer-Finisher** | Pays attention to detail, ties up loose ends. Hard-working, conscientious, consistent, self-disciplined. Needed in the final stages when the bright-ideas people have got bored and want the next bright idea. Tenacious, even to a fault when refusing to give up on a project that should be abandoned. |
| **Implementer** | Similar in personality to a completer-finisher. Attentive to detail, well-organised, prefers order and routine. Capable of directing other team workers, implementing what others devised and designed. Emotionally well-controlled. Less anxious about detail than the completer-finisher, more focused on strategic success, with a high capacity for hard work that often means they are left carrying the load others find too heavy. |

**Belbin's most startling finding:** in later experiments the researchers let a group self-select its strongest "A-team," then built the "B-team" from the leftovers using role-type analysis. **The B-team always won.** The reason: people pick people who are like them and to whom they are socially attracted, which does not produce a balanced team.

### Tuckman's four phases of team development
**Forming** — getting acquainted, understanding objectives, establishing ground rules, each seeing his or her place in the group.
**Storming** — interpersonal conflicts, competing for attention and influence, reconciling divergent interests.
**Norming** — becoming cohesive, developing team identity and spirit, sharing, cooperating, collaborating; team goals override individual considerations.
**Performing** — strong team loyalties, high creativity and productivity on the tasks in hand.

Characteristics of an effective team: joint accountability and tolerance of individual mistakes (a "no blame" culture); deep involvement in and loyalty to the team effort, members feeding off each other's enthusiasm; consensus about key decisions and joint commitment to them; clear group consensus about goals and objectives; conflicts brought into the open and dealt with; alternative ways of thinking and new ideas welcomed and evaluated open-mindedly; leadership associated with respect and confidence from the rest of the team.

In a knowledge-creating environment, "give instructions and have them carried out" does not work. The differentiating words are **buy-in** (members become stakeholders in the results) and **empowerment** (delegation of authority to input new ideas, make decisions and execute them, involving trust, encouragement and a tolerance of mistakes — mistakes owned by the team as a joint learning experience).

### Architecture governance
**The Architecture Board** — a steering committee that sets and enforces the overall architectural direction. It is more likely to succeed if its remit is **all of ICT architecture, not just security architecture**, "since it is difficult to see how the board may gain sufficient power and teeth otherwise."

Two main functions:
1. Approve architectural standards, road maps and guidelines, ensuring these were developed with the strategic interests of the business uppermost.
2. Review individual project proposals and designs for compliance with those standards, and **approve budget for those projects only if the compliance criteria are met**. ("If you don't comply, you don't get budget approval!")

Potential members: at least one senior business manager; the head of ICT services; the head of ICT operations; the head of ICT systems development; the chief ICT architect (if the position exists); the most senior member of the information security expert team; the head of internal audit; **a senior representative of the finance team who has authority to control budget sign-off**.

*Fallback:* if no overarching ICT Architecture Board is possible, regulate through an **Information Security Management Steering Committee** of the type suggested in ISO 17799.

**The Design Authority** — usually a single person, such as the chief ICT architect or project director, taking ultimate responsibility for the integrity of the overall design and of the design process that produced it. Since a project's design must comply with the architectural standards, the Design Authority is **accountable directly to the Architecture Board** for that compliance, attending Board meetings to present project items, whether or not a permanent member.

**Architecture maintenance.** Three drivers of change: business requirements change over time (flexibility buys you time but eventually rework is needed); new technologies emerge, sometimes changing business direction (the Internet), especially at the component layer, and sometimes major enough to affect the *conceptual* architecture by making previously infeasible concepts realisable; and practical operating experience exposes flaws in decisions taken on a theoretical basis. Changes must be documented **with the business case** and brought to the Architecture Board for approval; significant budget expenditure also falls within its remit.

### Fast Track™ Workshops
A method for kick-starting the programme when it is too big to explain in a three-minute elevator pitch. Trademark of SABSA Limited.

**Shape:** five contiguous days, Monday morning to Friday afternoon. Training material is presented, but the most important part is that **after each presentation the participants do a workshop exercise applying the material to their own organisation** — beginning actual work on the architecture, a little in each area, so they get exposed to every aspect and a taste of what is involved. Typically 10–16 people, often organised into small syndicate groups (say three groups of four) each tackling a different but related aspect. After about an hour of brainstorming each syndicate reports back to the full group using flipcharts; the presentations promote debate so everyone sees all the work. Different groups attend different sessions — for example, the strategic business vision session on Day 1 involves senior managers, for a couple of hours, half a day or a whole day depending on how much time they will commit.

**The full package is 5 + 5 + 5 days:**
- *Prior to the on-site programme (5 consulting days)* — "it is essential not to start cold on the first day." Gain an initial understanding of the organisation, its goals and objectives through research and analysis of documents supplied in advance; gain an initial understanding of the current security and technology environment, or the position of any architecture programme already commenced; gain an initial understanding of the roles and objectives of all proposed participants; draft, prioritise and agree the specific Fast Track objectives and deliverables; draft, structure and agree the five-day programme and its detailed contents; compile and customise all presentations; design and produce appropriate detailed workshops.
- *During the on-site programme* — a real-time consulting element, because each workshop reveals new issues, tasks and priorities. Facilitators invest resource after the participants' working day: customisation of planned presentations to meet new objectives; development of new or replacement presentations; customisation of participant workshops; development of new or replacement workshops; end-of-day status meetings to review progress and plan mechanisms for altered priorities; review of workshop output and documentation of key architecture components.
- *After the on-site programme (5 consulting days)* — a Fast Track report with independent advice on next steps, potentially including: documented summary of the business case for security architecture development; documented summary of a plan to collect and verify the full set of security business requirements; documented summary of the outline presentations and draft reports developed on site, with advice on progressing to definitive detailed architecture plans; summary of existing physical, component and operational architecture with a high-level gap analysis against stated strategic architectural requirements; advice on integrating the security architecture with existing or in-progress business or technology architecture work; high-level implementation and migration strategy; summary of key performance indicators (KPI) for security; documented draft project plan of key tasks, milestones and future deliverables with key dependencies; documented draft project plan summary of what is needed to complete the security architecture, when, and what resources are required.

The Friday afternoon final session produces the **draft project plan** with resources allocated and estimates of durations and target dates.

## Reference Tables

### The benefit arguments, and who they are aimed at
| Benefit to sell | The argument |
|---|---|
| Ensuring you solve **business problems** | Identify the specific business problems in *your* organisation the architecture addresses, and explain in business language how solutions bring specific business benefits |
| **Maximum return on investment** | The organisation already spends on security and management knows it; the architectural approach maximises ROI and optimises the cost/benefit ratio |
| **Economies of scale and standardisation** | Lower acquisition cost (fewer different components, leverage bulk purchasing); lower support cost (fewer product types via corporate standards); lower training cost (fewer tools and products to train on) |
| **Improved corporate governance** | Control by the board fulfilling legal responsibilities to safeguard shareholders' funds — "at the most extreme end of the scale, corporate governance is about keeping the directors out of jail!" Remind them of their **personal** legal responsibilities |
| **Better risk management** | Consistent risk assessment across all parts of the organisation; consistent risk mitigation; levels of mitigation and control commensurate with risk as determined by those responsible for managing the business |
| **Improved preparedness for formal audit** | Nobody likes negative audit comments; internal audit can be a strong ally in building the business case. External audit safeguards shareholders — "careers have been known to falter and even terminate" over a bad report |
| **Influencing the financial and business analyst community** | Analysts (Meta Group, Gartner Group, Forrester Research, or investment-bank divisions) influence the share-buying community and therefore share price. Read their reports on corporate and ICT security, quote them to the leadership team, and know what they say about your own organisation |
| **More flexible response to new business opportunities** | Fast time to market brings competitive advantage; a security architecture road map supplies modular solutions so most of the problem solving is done up front. Common security services implemented through common security mechanisms are the key success factors. Also: development speed trades off against assurance — "mitigating one risk almost always increases at least one other risk" — but the architecture lets you swing further toward speed while keeping reasonable assurance |
| **More flexible response to business reorganisation** | Mergers, acquisitions, divestments, joint ventures, outsourcing. Monolithic security management makes these very difficult; a **domain-based** architecture lets you add, remove, rescale and rescope domains easily |
| **Reduced operating costs** | The help desk function (cost grows *exponentially* with technology diversity, "to the extent that with no architectural control, user support becomes impossible at some level of diversity and complexity"); user administration (workload = per-registration effort × number of registrations — a single user registration database also reduces the risk of forgotten dormant accounts); system administration, unless designed for high automation and centralisation |
| **Improved productivity through better user support** | Use the trouble-ticket system's MTTR statistics and problem volumes to compute lost productive time, with a weighting factor for users who can do some work without system access. Many organisations move to a **common operating environment (COE)** / standardised desktop, restricting desktop hardware and software to a limited number of registered, known instances so the help desk already knows the configuration when a call arrives |
| **Support for outsourcing strategy** | Distinguish **outsourcing policymaking and control (not recommended)** from outsourcing purely operational services that implement policy already made (perfectly feasible — e.g. managed intrusion detection and reporting) |
| **Reduced total cost of ownership** | Cost of user administration; system administration; user support and help desk; lost productivity through unresolved user security problems; lost productivity through time lost in complex security user interfaces with multiple logins |
| **Leveraging trusted business relationships** | Trust is an attribute of relationships, not created by technology; technology protects existing trust across an electronic network via data confidentiality, data integrity protection, authentication, non-repudiation |
| **Consistent user experience** | Login complexity and variety frustrate internal users, customers, suppliers and technical support alike, and **tempt people to subvert or circumvent the mechanisms**. A standardised consistent security user interface reduces frustration and cost |

### Additional persuasion points for budget holders and key influencers
- The increasing tendency in the regulatory and legal regime for directors, **including non-executive directors**, to be held personally liable and accountable for corporate losses where risk management practices are called into question.
- Relate security and risk control requirements to the overall investment in business ICT systems; associate the security budget with **protecting that investment**.
- Use Chapter 6's material to create ROI / return-of-value models showing the payback.
- **Draw parallels with other ongoing, non-revenue-producing functions such as marketing and accounting** so they understand this is not a one-off project but a lifetime commitment.
- Link the budget to achieving the business mission, realising the vision, and supporting critical success factors.
- Point out any history of positive contributions demonstrating the security programme's value.
- Point out any marketing edge or competitive advantage.
- Prepare a **business impact analysis** of the damage expected from inappropriate or insufficient information security.
- Prepare an **analysis of previous losses**.
- Draw attention to best practice statements in external standards and analyst reports and link them to due diligence and corporate governance.
- Target **key influencers** who do not hold the budget but can influence budget holders, and who can become champions raising the issues with individual decision makers and ensuring the right items reach meeting agendas.

### The single sign-on debate, as the authors frame it
Single sign-on tackles multiple passwords, differing syntax rules for passwords and identifiers between systems, and differing enforced password lifetimes. The counter-argument is that a single password gives access to everything. *"It is a debate that cannot be objectively resolved — people simply feel differently about these risk issues."* The authors' position: the benefits of single sign-on, from both a cost perspective and from the perspective of **helping a user to look after his or her password properly**, are easily demonstrated; your decision depends on whether you judge those benefits outweighed by the compromise risk.

Multi-factor authentication solves the technical vulnerability but is "even more controversial": added cost of acquiring and deploying equipment at user terminals and of the tokens themselves; operational cost of distributing tokens, reissuing after lifecycle expiry, and help desk support for the added complexity; and **user acceptance** — people do not want extra things to carry, leave the token at home ("Sorry, it's in the pocket of my other suit"), and resist gadgets that complicate login.

### Information collection
**About the business** — two approaches, of which the first is "by far the most useful and the most productive":
1. **Structured interviews with business managers** — the most senior you can reach. Expect the deflection "Why can't you see my technical manager?"; you must convince them it is *them* you need, that you want to explore what *they* perceive as the most significant business risks and most pressing needs to protect corporate information, and that **you will not ask technical questions — and then do not ask any**. The easiest route to arranging them is a champion who is a peer of, or senior to, the target.
2. **Reference to existing materials** — the annual report and accounts (what senior management wrote for shareholders) plus audit reports internal and external; press interviews and reports; analyst reports; consultant reports; mission statements, vision statements and supporting documentation; strategic vision documents; previous risk assessments; industry reports on legal and regulatory drivers.

**Opportunities of a senior interview:** build a relationship with an executive who becomes a friend and ally; sell the benefits at senior level, developing buy-in, support and endorsement and perhaps recruiting another champion; gain real insight into strategic direction and how it drives security requirements; hear requirements from key decision makers, getting a true picture especially about future opportunities and strategies; understand the risk issues that really concern senior management and the relative ranking of risks; **validate input gathered at less senior levels** to check it is true and not wishful thinking; communicate that you approach the programme from a business perspective, which makes them more supportive and willing to discuss.

**Risks of a senior interview:** time will be restricted, perhaps half an hour; straying outside the interviewee's domain of interest damages your reputation and makes another meeting hard to get; it tends to be **one-shot — if you blow it, there are rarely second chances**; you may raise expectations you cannot deliver (underline the benefits of a high-quality *process* without promising unpredictable outcomes); visibility and exposure to senior scrutiny is good when things go well and irritating when they do not; you may use terms and concepts they do not understand.

**Sample senior-manager interview questions:**
- I want to understand your role as fully as I can. What are the four or five bullet points that describe your management responsibilities in the organisation?
- Who are your key advisors, whose opinion you are most likely to seek when considering a decision?
- What are the mission and the vision for your particular part of the organisation?
- What are the main risks facing this organisation as a whole and your part of it in particular, and how do you think those risks can best be mitigated and managed?
- Is there anything that keeps you awake at night — any inhibitors or constraints to achieving your mission, or things you are really anxious about that should be addressed, perhaps through this programme?
- What do you expect the development of the organisation and its business to be over the next one to five years, and how far ahead does your strategic planning window stretch?
- What will be the main factors influencing this industry and this organisation over that window?
- To what extent do you think reorganisations, mergers, acquisitions and joint ventures will characterise the future landscape of this business?
- What new technologies do you see coming over the horizon that will have major impacts on this business?
- In particular, the Internet and its associated technologies continue to evolve — how do you see this affecting this business?
- How do you think technology will change the way we work and the way we do business?
- It is often said that information is power. Do you think this is true, and if so, how powerful is information in this business?
- If a hostile investigative journalist were digging around for dirt on this organisation, how would you feel? Is there anything that would worry you? Anything you need to be especially careful about? *(The authors note this last one may draw a blank because you may not be sufficiently trusted to know the true answer.)*

**Interview technique:** ask only about areas of direct interest to this person's job; keep things high level; leave the interviewee scope to reply at whatever level of detail they choose; use the replies to frame the next question, picking up areas of interest and backing off disinterest. Maximise the interviewee's talking time by minimising the length of your own questions and comments. Use **open questions**. Run interviews **two-handed** — one experienced interviewer to ask questions, parry challenges, keep momentum and be quick on their feet as direction twists, and one less-experienced interviewer as scribe who says very little but watches, listens and learns. Record results quickly, before interpretation and meaning are lost, and **in a consistent format** so comparisons can be made and patterns determined.

**Validation** — two ways: give the interviewee a copy of the interview notes and/or the initial draft business requirements and ask them to verify; or hold a **brief consensus-building workshop** on the initial draft, particularly useful when senior managers have differing requirements and priorities, "as they often do." *"It is not unusual for a team of executives to have almost unanimous views that turn out to be incorrect."*

**Collecting technical information** — relatively straightforward if you use authoritative sources: interviews and brainstorming workshops with technical managers, and reference to current technical documentation.

### Getting consensus on the conceptual architecture
Sign-off from the **technical management community** is required before detailed design; you cannot proceed without their support and buy-in. Build relationships early. In individual meetings, find out: Do they have any concerns? Are they pre-disposed for or against particular approaches? What will you need to do to get their agreement? What level of influence do they expect to have? What type of involvement in the process do they expect? Who are their key allies and opponents? What specific "buttons" do they have that you will need to "push"?

Then run **open forum workshops** exposing your ideas to debate from all perspectives — systems development, procurement, operations management. But **pre-sell first**: "It is dangerous to go to any open forum of the type just described without a clear idea of what all the players in the room think beforehand." Hold one-to-one meetings, work offline to overcome objections, present new versions, until the workshop discussion is a mere formality. *"You may not like it, but politics and diplomacy are essential to your success."*

## Worked Example — The Operational Cost of User Login at IBFS

Triggered by Brian Jones's interview complaint (Chapter 4): "There is nothing that discourages a user more than the endless layers of user IDs and passwords that many of our systems require. We have had lots of complaints."

IBFS had no formal measurements. A short consulting exercise used the system inventory, logs from a variety of platforms and applications, and historical help desk call statistics to cost three elements of the authentication service: **user set-up; day-to-day login activity; problem management and support.**

**Base information**
| Parameter | Value |
|---|---|
| Total IBFS system users | 120,000 |
| Annual staff turnover | 15% |
| Average number of different user IDs and passwords per user | 5 |
| Working hours in a business day | 8 |
| Working days per employee/contractor per year | 220 |

**User set-up.** New users each year = 15% × 120,000 = **18,000**. At five accounts each, the support teams create **90,000 new accounts per year**. Administration is relatively efficient: view request, check authorisation, create the account record, allocate group memberships, advise the new user of identity and initial password takes on average **120 seconds**.
→ 90,000 × 120 s = **3,000 resource hours per year**.

**Login activity.** With five identities/passwords, inactivity timeouts, and switching between applications, the average total logins per user per day = **20**. Average time per login (remember the correct identity, remember the correct password, enter both, get an acceptance response) = **15 seconds** — "a relatively short time for a large complex environment."
→ 120,000 users × 20 logins × 15 s = **10,000 resource-hours per day** = **2,200,000 resource hours per year**.

**Problem management.** On average **1% of logins fail**. Each must be repeated; average time for a repeated login, including puzzling over the reason for the failure, = **60 seconds**.
→ **88,000 resource hours per year**.

**Total direct resource effort** — excluding capital expenditure on authentication systems, initial deployment time, and staff costs to support existing systems:
| Element | Resource hours/year |
|---|---|
| User setup | 3,000 |
| Login activity | 2,200,000 |
| Repeat logins | 88,000 |
| **TOTAL** | **2,291,000** |

At 8 hours × 220 days, this is **more than 1% of the total resource available to the company — just spent logging in.**

**Lost productivity — the second half of the story.** Three consecutive failed attempts lock the account, so after two failures users are advised to contact the help desk. 10% of those who fail the first attempt also fail the second → **2,400 failed login reports to the help desk every day**. Because continuity of operations is fundamental at IBFS (especially in trading and dealing rooms), Security Administration and the help desk comply strictly with a **four-hour SLA** with the business.

The authors' point: even four hours, which many organisations would call rapid, is devastating for total cost of ownership, because access is a prerequisite to doing the job. A user unable to log in can only be doing one of three things — using someone else's ID and password (**which renders security totally ineffective**), nothing at all (**totally ineffective employee**), or, perhaps worse, doing nothing productive *while actively and openly complaining that security is a business constraint*.

| Metric | Calculation | Lost productivity |
|---|---|---|
| SETUP | 18,000 user support requests/year × 4-hour service time | max **72,000** resource hours/year |
| PROBLEM MANAGEMENT | 2,400 failed second-attempt logins/day × 4 hours = 9,600 hours/day | up to **2,112,000** resource hours/year |
| **Total potential productivity impact** (even if the help desk meets its targets) | | **2,184,000 resource hours/year — again more than 1% of total available resource** |

**Grand total: more than 2% of the available human resource was being used (wasted?) on logins and associated support and problem resolution.**

**The quick win.** Centralised, single-identity authentication understandably became a major IBFS strategy. But a **short-term tactical project was initiated within the framework of the strategic solution defined by the conceptual security architecture**, to identify legacy authentication elements that could be quickly integrated and rationalised, if only partially. The result was *not* full secure single sign-on — it was a reduction in the average number of identities per user **from five to four**. "At first glance this is not a fantastic result, but the impact was a **20% reduction in the total cost of ownership** — more than enough to create an almost instant return on investment for the project and gain visible credibility and support for the security architecture project."

## Two more case studies

**A US-based eLearning provider — the consensus workshop that surfaced a hidden risk.** Consultants developing the contextual security architecture interviewed all eight members of the executive board. Asked about the potential impact of a specific risk scenario, **seven believed it a negligible risk requiring no action; the eighth considered it catastrophic** — but had never had any apparent reason to discuss his specialist knowledge with colleagues, because he assumed they understood the risk as he did. A brief facilitated consensus workshop gave him a platform, and it produced **enterprise-wide consensus on high-priority actions** for the security architecture programme.

**Interviewing the hostile CFO of a global materials manufacturer** (a first-person account). The consultants arrived several minutes late for a 10:00 interview with a drop-dead limit of 11:00. Large tidy office, enormous polished empty desk, an efficient PA — intimidating in itself. Pre-selling by another manager had told him why they were there. He was pleasant but blunt: *"I can't see any need for information security in this firm. There is nothing secret about our business. The most competitive thing is the price list, and we all find out each other's prices as soon as they come out anyway. I don't know why we just don't all fax each other a price list and save ourselves a lot of trouble."*

They turned it around. They may have used the hostile investigative journalist question; they certainly discussed publishing stock-market-sensitive information on company performance and the demands of the financial regulators. **The trigger that caught his imagination was integrity and reliability of financial records and how systems downtime can increase "debtor days."** When his watch alarm went off at 10:55 he did not make them leave until 11:00. He had agreed information security was really important, had given them a considerable amount of useful information about real business requirements, and had become a strong supporter.

**Senior Management Losing Its Religion** — the failure case, told in the authors' own religious metaphor. A large corporate client's information security manager became a convert and engaged the authors for preparatory work consulting "The Business," producing an initial contextual-security-architecture report of business reasons the organisation needed security. That fed a presentation to the CEO and his immediate reports; the sponsor had pre-sold internally and prepared the ground. The presentation impressed; the concept was bought; budget was created and the programme planned.

*"And so off we went to get on with The Work. It took a long time and we were making steady progress, but the costs were racking up as time went by. We made a fatal mistake. We did not go back and preach our Religion. We did not tell of the Miracles that we had performed. We did not carry the Good News back to the CEO and executives. They could see the costs in the management accounts, but they could not see any benefit. They lost their Religion, they stopped being Believers and they pulled the plug on The Work."*

**The rule derived from it — quick wins:** build into the programme a series of deliverables that deliver visible, tangible business benefit, **at intervals of no more than two months, preferably less**. When each arrives, run an internal public relations programme so that all senior management sponsors, supporters **and critics alike** receive the good news. This maintains the momentum built at the beginning and the confidence that approving the programme was correct.

## Mental Models
- **Bottom line first.** Prioritise every communication for a business audience by importance, not by logical derivation order.
- **The B-team wins.** Never let a group self-select the team.
- **Budget approval is the enforcement mechanism.** Governance without control of money has no teeth.
- **Preach the Religion continuously.** Costs are visible in the management accounts; benefits are only visible if you carry them back.
- **Politics and diplomacy are not a distraction from the work — they are the work** in the consensus phase.

## Anti-patterns
- Writing a catch-all communication for everyone.
- Presenting a logically-ordered argument with the conclusion at the end to a senior audience.
- Asking technical questions in a senior business interview after promising not to.
- Self-selected teams, two plants, or two shapers.
- A Shaper leader doing their own detailed project planning instead of delegating it to someone whose profile fits.
- Walking into a consensus workshop without knowing what everyone in the room thinks.
- An Architecture Board whose remit is security only — it will lack power and teeth.
- Outsourcing security **policymaking and control** rather than only the operational implementation of policy already made.
- Running a long programme with no visible deliverable for months.

## Key Takeaways
- Five rules of influence: right audience, their language, their issues, key messages, most important first.
- Sell on business problems, ROI, economies of scale, governance and personal director liability, risk management, audit preparedness, analyst opinion, time to market, reorganisation flexibility, operating cost, productivity, outsourcing support, TCO, trust, and consistent user experience.
- Belbin's eight roles; Tuckman's forming/storming/norming/performing; buy-in and empowerment in a no-blame culture.
- Fast Track™ = 5 consulting days + 5 workshop days + 5 write-up days, ending in a draft programme plan.
- Interview the most senior managers you can reach, two-handed, with open questions; validate by review or consensus workshop.
- Get technical management sign-off on the conceptual architecture before design, by pre-selling one-to-one then confirming in workshop.
- Enforce compliance through an Architecture Board with budget authority plus a named Design Authority per project; maintain the architecture through the same board.
- Deliver quick wins at least every two months, and publicise them.

## Connects To
- [Chapter 4 — Case Study: IBFS](ch04-case-study-ibfs.md) — Brian Jones's password complaint, which the login cost model quantifies.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — the ROI models used to support budget requests.
- [Chapter 7 — Using This Book as a Practical Guide](ch07-using-this-book-as-a-practical-guide.md) — the lifecycle milestones this chapter's governance protects, and the "improvements programme / early wins" deliverable.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — where the interview output lands.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — domain-based architecture enabling reorganisation, and outsourcing strategy.
- [Chapter 14 — Security Policy Management](ch14-security-policy-management.md) — writing policy for a specific audience, and security culture development.
