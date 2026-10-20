# Chapter 5 — A Systems Approach

## Core Idea
Rational security architecture is achieved by taking a **systems approach**, grounded in established systems engineering principles. Boardman's definition, quoted by the authors: systems engineering is *"a rational approach to decision making related to the solution of complex problems in engineering planning design and operation."* The chapter's job is not to teach systems engineering but to promote **systems thinking** in security architects — because complexity that is not decomposed is destabilising, and because a sub-system that perfectly meets its own brief can still fail the business.

## Frameworks Introduced

### The three complexity-simplifying techniques
1. **Top-down decomposition** — the whole system is a set of smaller sub-systems, each itself a system that can be decomposed again. Masks complexity, gives a simplified view at each level, and makes each level a manageable piece of work. *"If you want to eat an elephant, then the only way is by taking it in small pieces, one at a time."*
2. **Black box modelling** (Figure 5-1) — each sub-system is a box whose inner workings are hidden; you see only inputs, outputs, and a defined transformation function with a defined interface to peer boxes at the same level of decomposition. Later you open the box and find smaller black boxes of finer granularity.
3. **Logical flow analysis** (Figure 5-2) — at each level you get black boxes linked in a flow pattern, the output of one becoming the input of the next.

### The three behaviours a systems approach forces
- **Documenting the ideas.** "Write down what you really need" — the simplest task to describe and the most challenging to execute. *If you can't write it down, you haven't thought it through.* "Writing down" means a mixture of descriptive text, bulleted lists and diagrammatic representations.
- **Peer review.** Structured review and feedback: the synthesis of one person or team is subjected to analysis and constructive criticism by another of similar professional standing. Sharpens the whole team's thinking and promotes collaborative teamwork.
- **Communicating ideas.** Models let you convey the approach to others planning and executing projects, and preserve critical descriptive information for future projects and for the review of existing ones.

### Boardman's five basic considerations for thinking about a system
| Consideration | Meaning in security architecture |
|---|---|
| **Total system objectives** | Expressed as the *business requirements* the system must satisfy. Prime importance: decide up front **how performance will be measured**. "If you cannot measure whether or not you are hitting the objectives, you are not managing the solution adequately." Common fault: designers start from technical requirements that do not reflect business needs and do not lend themselves to easy compliance measurement. |
| **The system's environment** (Figure 5-3) | Outside the system boundary but influencing its operation, and **outside your control**. The designer must recognise its influences and design the system to respond to them. |
| **The system's resources** | Inside the boundary; the means by which the system operates — the machines that process information, their operators and users, the power that drives them, the consumables they use. |
| **The parts of the system** (sub-systems), their activities, goals and measures of performance | Decompose so behaviour can be modelled and measured against whole-system objectives. Frequently overlooked: **maintaining overall system objectives when delegating sub-system design**, producing a sub-system that meets its entire design brief and still fails business expectations. |
| **The management of the system** | Setting system objectives; collecting metrics against them; analysing and evaluating the metrics; making and implementing design and operational decisions from the analyses; if necessary reviewing and resetting the objectives. |

Note the deliberately wide reading of **"performance"**: not just throughput, latency, response time and percentage up-time, but *fulfilling all of the system objectives* — including maintaining confidentiality, protecting integrity and holding users accountable. This is precisely why the Business Attributes technique of Chapter 6 exists: it normalises any real business model into a conceptual model of standard abstract attributes and supplies suggested metrics for each.

### The Feedback Control Loop (Figure 5-4)
The simplest and by far the most common control concept in the book, built from **three sub-systems**:
- **Control sub-system** — exerts control (heating system: the component that turns heat on/off or varies output rate).
- **Monitoring and measurement sub-system** — measures the state of the system, which the control sub-system affects (heating: the thermometer).
- **Decision sub-system** — decides based on the measurements (heating: is the temperature above or below the set point?).

The cycle: control changes parameters → system state changes → monitoring measures and reports the new state → decision compares to the **set point** → either instructs "do nothing" or calls for more change. *Measurements are fed back to the decision module, which feeds instructions back to the control module.* This loop recurs throughout the book as the model for managing security in business systems.

### Threats vs vulnerabilities, located precisely
- **Threats are environmental** — properties of the system environment, coming from outside.
- **Vulnerabilities are internal** — properties of the system itself, weaknesses in how it is built or operated.
- Design so that vulnerabilities to recognised threats are minimised within cost constraints. **"If you ignore the threats, it is impossible to understand what might be the vulnerabilities."**
- The authors footnote the semantic subtlety: authorised interactions and the people who make them are *part of the system*; unauthorised interactions and the people who make them are *part of the environment and are threats*. "Unauthorised" includes authorised people interacting in unauthorised ways, and authorised people making human errors.

## Advanced Modelling Techniques (in increasing difficulty)

### 1. Business Process Engineering
A business process is a system that takes inputs and transforms them into outputs *that have value to the business*. Examples: manufacturing a food product from raw materials; assembling a motor car from components and sub-assemblies; processing transaction data against a master database of customer bank accounts.

Common enterprise processes: acquire customer order; fulfil customer order; collect revenue from customers; manufacture products; acquire and store raw materials; pay suppliers. There is also at least one **meta-process** — a process used for managing the business processes.

**Functional model vs process model.** The functional model organises the enterprise by *what people do* (sales, production, finance departments). The flaw: no one department has oversight of how the business actually operates. "Fulfil Customer Order" runs through sales (has the order), production (makes it), quality assurance (checks it), the warehouse (stores it) and logistics (ships it). *Which department has overall control? None of them! And there lies the problem.* Organising on a process basis gives each process a **process owner** who co-ordinates the functions end to end. This has a significant effect on business continuity management.

Benefits of business process engineering:
- **Cost accounting per stage** — total process cost; identify high-cost sub-processes; look for lower-cost implementations via automation, integration, re-sequencing or outsourcing.
- **Value-chain analysis** — understand where the enterprise adds real value (core business) and which sub-processes could be outsourced because they add no real value.
- **Process analysis** — test whether re-sequencing economises execution time or operational cost; assess risk of failure at each sub-process level and seek more robust implementations of vulnerable ones; identify significant control points; eliminate hidden unnecessary complexities; ensure all decision points are clear, that decision parameters are known at decision time, and that decision criteria are well defined.

### 2. Dependency Tree Modelling
Higher-level systems depend on lower-level sub-systems, and *how* they depend on them is determined by the combination:
- **Series = AND relationship** (Figure 5-5): both A **and** B must be operational. **AND increases risk of system failure** — one component fails, the whole system fails regardless of the other.
- **Parallel = OR relationship** (Figure 5-6): either A **or** B suffices. **OR decreases risk of system failure** — one fails, the other keeps the system going.

Developed into a sophisticated dependency modelling toolset by Professor John Gordon of Concept Laboratories Limited; a main application is computing risk in complex systems.

### 3. Finite State Machine (FSM) Modelling
A system is conceptualised as a finite number of **stable states**, mutually exclusive, one at a time, each described by the values of **state variables**. Change from one state to another is a **state transition**, triggered by an **event** (external interrupt, internal timer expiry). Represented in a **state transition diagram** (Figure 5-7 uses traffic lights, with state variable `lightColour` ∈ {red, amber, green}).

- **Predicates** — Boolean variables (true/false) whose logical expressions, combined with AND, OR, NOT, NOR and XOR, select which of several possible outgoing events fires.
- **Specific actions** — local responses to an incoming event rather than outgoing events, e.g. "start timer", "increment counter".
- **Atomicity** — states are indivisible; the system is never between states. When an event causes a transition, all functions associated with it must be processed in full before another event is accepted; further events queue.
- **Identify all possible states first** — that is the essential first step of building the model.

**What to model as an FSM:** interactive communications protocols between two or more sub-systems. "If such a system is designed without producing a finite state machine model, it is likely that at some stage the system will fail because it enters a state that the designer has not foreseen and for which there are no planned actions." In security architecture: cryptographic protocols; authentication protocols (such as password protocols); handshaking protocols; two-phase commit protocols in distributed databases; protocols for distribution, replication or duplication of security-related data (e.g. for backup); synchronisation protocols between application sub-systems.

**Where high assurance is needed:** safety-critical systems (malfunction costs lives or injuries); high-security systems (malfunction has massive business impact); basic infrastructure systems such as data communications networks that support many applications and are fundamental to business continuity.

**Exhaustive model checking.** Model as an FSM, then run the model through an automated model checker that exercises every possible state and event. Caveat: if the model itself is incorrect — a state or event missing — the checker will not normally detect that; proving correctness of the model needs formal specification languages, beyond the book's scope.

## Worked Example

### Case Study A — IBFS Securities Trading Division: the encryption project that was the wrong answer
**The presenting problem.** The Securities Trading Division ran a financial trading application used by market analysts to gather and consolidate securities-market information. A routine internal audit found that system administration — originally done at the main processor on a locally connected terminal — had been upgraded so administrators could log on remotely from any user terminal. Because the system now operated internationally (the original design was single-office), **remote administrators' user IDs and passwords were being sent in the clear across untrusted networks**, as were all other users' credentials. The application data itself was not considered sensitive (believed public-domain material being post-processed and consolidated internally). Company security policy required all user passwords to be encrypted across untrusted links.

**The client's planned remedy:** install hardware encryption devices at all network entry points, so every path the credentials could travel would be encrypted. A consulting team was engaged to help plan this global encryption scheme.

**First observations.** The data paths to be protected extended worldwide. Encrypting the entire transmission bandwidth would be **prohibitively expensive and operationally impractical**, and the dynamic network — routing changed over time — meant the encryption devices would need virtually constant maintenance.

**The original brief** had been to build an application that: provided the market analyst, on the desktop, with consolidated financial information from three sources; allowed access into the company's research database of previous historical financial information; operated within the UK on existing communications infrastructure but was capable of international expansion; and was developed in accordance with the company's standard security operating practices.

**Analysis of shortcomings.**
1. No structured or formal specification. Built with a Rapid Application Development approach in a correspondingly short time; documentation limited to administration and user manuals.
2. The designers had scoped the system as the *trading application alone*. Against that limited remit they succeeded — but delivered a **point solution** with no flexibility for future business needs.
3. **The original conception of the system boundary was short-sighted.** Expanding the boundary to the whole company and asking what the business really needed produced a very different picture.

**The real business requirements** (recovered by re-interviewing management):
- Make money by timely trading of securities and maximise the chance of success; expand from limited markets to **global** markets.
- Main trading-area office space was costly; make better use of facilities already owned elsewhere in the country.
- Use the most up-to-date and correct information as the basis for trading, to maximise returns.
- Reduce the overhead cost of running computer networks, which was starting to dominate operating expense.
- Increase transaction volume through additional trained staff, as soon as possible.

**Total system objectives derived from those requirements:**
- Real-time financial information to market analysts with minimum delay — **in no event more than 5 seconds** from provision by the information provider.
- Information received must be **correct and complete**.
- Execute trades on user request **within 5 seconds** of the analyst committing on screen.
- Support **100 simultaneous users** without performance degradation, and up to **150 users with no more than a 30% reduction in response times**.
- Be enhanceable to support additional users **without changing the basic system architecture**.
- Be available **99.99%** of the time during the normal working week (defined as opening of business in Asia on Monday to close of business in the USA on Friday).
- Be fully accessible to any authorised user in any company office worldwide during that week.
- Operate consistently with compliance monitoring needs, including **full accountability for all trades through a tamper-protected audit trail** of all trader actions.
- Use the most cost-effective computing and communications infrastructure available **without compromise of security** as defined in the security policy.

The book's sting: *the business managers considered only the last two objectives to be security-relevant — in reality all of them are, but only the last reflects why the team was asked in (the password problem). It turns out that problem is easy to solve as part of a proper overall system design.*

**Functional problems found in the existing design:**
1. No insight into timeliness — the operational system was delivering financial information to the desktop **up to two minutes after it was required**.
2. No planned expansion outside the UK; the application had been funded from one business manager's departmental research fund, giving her a disincentive to spend beyond her own department's needs.
3. A **hard-coded maximum of 64 users**.
4. Correctness and completeness drove the trading process, yet there were **no integrity checks** on the business-critical information being delivered.

**The system's environment:** the communications infrastructure came from a third-party supplier with **no Service Level Agreement**; network response time to support the required transaction turnaround had never been discussed.

**The system's resources:** the trading application shared hardware with many applications under a simple round-robin task scheduler. The main processor had a UPS and fault-tolerant hard disks; the traders' desktop machines had **neither** — yet the application cached information to and from the main processor in local storage on those machines.

**Sub-systems, goals and measures:** documentation showed no decomposition below "main processor" and "traders' processors." Substantial work had gone into the user interface, but **no performance model and no availability model** existed for either component.

**Management of the system:** a systems administrator whose documented primary tasks were user account administration (add, modify, delete) and regular tape backups. **No performance-measuring tools** to show how the system behaved and **no simple auditing tools** to show how users had acted in processing trades.

**Critical shortcomings.** The business requirements overwhelmingly demanded high performance, high reliability and high integrity, and the company had never produced any performance or availability model. Most notable: the outsourcing decision for external communications was taken by the Information Systems Department to cut cost and focus on core business — in line with strategy — but **without a formal SLA, the performance of that component was entirely outside the company's control**, and there had been **no review of the business risks that decision introduced** to applications using the infrastructure.

**Recommendation:** a complete redesign based on a proper systems approach, taking account of the organisation's wider requirements. IBFS was not committed to re-engineering the whole business (the authors note this is frequently the case) but sponsored the re-engineering of the trading system using a systems approach reflecting the high-level needs of the business as a whole. The closing honesty: this rigour would probably have increased the project budget, "but you usually get no more than that for which you are prepared to pay."

### Case Study B — The Case of the Missing Taxis
A television broadcaster's studios sat some kilometres from central government offices and the parliament building. Current-affairs programming required politicians to visit at short notice in response to breaking news; most travelled by taxi. A secretary in the basement, working for the facilities manager, had one job: arranging the taxis that brought politicians to and from the building.

The overall process of running current-affairs programmes crossed many functional departments, **had not been recognised as a business process**, had never had its criticality assessed, and had **no process owner**. In a cost-conscious review the facilities manager saw a head count line for a woman whose only job was to order taxis — surely excessive luxury, people can order their own taxis — and she was let go.

**Result: the entire process of getting politicians to the studio on time collapsed, and the core business purpose of the current affairs section was destroyed overnight.**

The lesson, in the authors' words: **every single function in an end-to-end critical business process must be afforded the same level of criticality as the overall process it supports.** One function fails in the middle and the entire process is broken. And: any attempt to manage business continuity in a culture without business process engineering has a questionable chance of success.

### Case Study C — FSM model of a password authentication protocol
Two sub-systems: the **Prover** (must prove its identity) and the **Verifier** (must verify it), with an interactive authentication protocol between them and a database of Prover IDs and passwords behind the Verifier (Figure 5-8). A prior registration protocol introduces a new Prover to the Verifier and must itself be modelled as an FSM; here it is assumed done.

Prover states:
| State | Meaning |
|---|---|
| IDLE | Doing nothing |
| WFLP | Waiting for an incoming "login prompt" event after an outgoing "wake-up" event |
| WFID | Waiting for an incoming "user enters ID" event after a "login prompt" event |
| WFPP | Waiting for an incoming "password prompt" event after a "user enters ID" event |
| WFPW | Waiting for a "user enters password" event after a "password prompt" event |
| WFLC | Waiting for a "login completed" event after a "user enters password" event |
| WFLO | Logged in and working after "login completed", waiting for "logout", "activity timeout" or "session timeout" |

The authors are explicit that this is deliberately incomplete: no error conditions (incorrect ID or password) and no individual timeouts on intermediate states are analysed. *"To build a truly robust system, every possible state and every possible event must be identified and included in the model."*

The model is documented as a state transition diagram (Figure 5-9) **plus seven tables**:
1. **Incoming events table** — event name, the interface it is received through, and its meaning.
2. **Outgoing events table** — event name, interface sent through, meaning.
3. **States table** — the states and their meanings.
4. **Predicates table** — predicate names, meanings and values.
5. **Specific actions table** — the local specific actions.
6. **State variables table** — state variables, data types, ranges of values.
7. **Event/state table** — maps all possible events to all possible states, showing the result of each event on each state.

The Verifier is left unanalysed but is noted to be considerably harder: it must handle password expiry (which invokes yet another protocol to model), account expiry, account blocking after more than three failed password attempts, and every possible error condition.

## Mental Models
- **Eat the elephant one piece at a time** — top-down decomposition.
- **The black box** — accept a defined function with a defined interface now; open it later.
- **The feedback loop** — control → state change → measure → decide → control. Every security management process in the book is an instance of it.
- **Threats live outside the boundary; vulnerabilities live inside it.**
- **AND is fragile, OR is resilient** — the shape of the dependency tree *is* the risk profile.
- **The system boundary is a design decision, and drawing it too tightly is the commonest error.** The IBFS trading system met its brief perfectly and was still the wrong system.

## Anti-patterns
- Starting from a set of technical requirements that do not reflect business needs and do not lend themselves to compliance measurement.
- Drawing the system boundary at the application, not the enterprise — producing a successful point solution with no future flexibility.
- Ignoring the environment because it is outside your span of control.
- Delegating sub-system design without carrying the overall system objectives into the sub-system brief.
- Departmental funding of enterprise-relevant systems, which creates a disincentive to build beyond the department's own needs.
- Outsourcing infrastructure with no SLA and no review of the business risk introduced.
- Building an interactive protocol without an FSM model — it will eventually enter an unforeseen state with no planned action.
- Cutting a "trivial" function out of an unmapped critical process.
- Applying a policy-compliant technical fix (encrypt everything, everywhere) to a symptom without re-examining the requirements.

## Key Takeaways
- Systems engineering = "a rational approach to decision making related to the solution of complex problems in engineering planning design and operation" (Boardman).
- Simplify with top-down decomposition, black-box modelling and logical flow analysis; behave by documenting, peer-reviewing and communicating.
- Five considerations: objectives, environment, resources, parts (with their activities/goals/measures), and management.
- Define how you will measure performance *while* you define objectives; "performance" means fulfilling all objectives, including security ones.
- The feedback control loop — control, monitoring/measurement, decision — is the recurring control pattern.
- Threats are environmental, vulnerabilities are internal; you cannot identify the latter without the former.
- Advanced tools: business process engineering (process owners, cost accounting, value-chain and process analysis), dependency trees (AND raises risk, OR lowers it), and finite state machines for every interactive protocol.

## Connects To
- [Chapter 2 — The Meaning of Architecture](ch02-the-meaning-of-architecture.md) — layering and modularisation as complexity management.
- [Chapter 4 — Case Study: IBFS](ch04-case-study-ibfs.md) — the Securities Trading Division belongs to this company.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — Business Attributes as normalised, measurable system objectives; the feedback loop reappears as the security management dashboard.
- [Chapter 12 — Physical Security Architecture](ch12-physical-security-architecture.md) — control structure execution and sequencing.
- [Chapter 17 — Security Administration and Operations](ch17-security-administration-and-operations.md) — business continuity management, which depends on process engineering.
