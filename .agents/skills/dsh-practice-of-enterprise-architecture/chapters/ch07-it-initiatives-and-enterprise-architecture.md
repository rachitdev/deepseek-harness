# Chapter 7 — IT Initiatives and Enterprise Architecture

## Core Idea
An **IT initiative is the key unit of work** in an EA practice — all IT-related plans materialize only through executing concrete initiatives. Initiatives are the "middle link" connecting abstract plans with practical implementation. Five archetypes of initiative differ by **origin and motivation**, and all of them must pass through the **IT investment portfolio management and budgeting** filter before becoming real.

## Frameworks Introduced

### Attributes of an IT initiative
Specific organizational need or problem addressed · goals, objectives and success criteria · specified scope · essential requirements · business case (initial investments and anticipated returns) · executive sponsors providing funding · separate allocated budget · dedicated manager coordinating activities. Small initiatives equal single projects; larger ones are multi-project transformation programs with a common underlying idea, unifying purpose and (often staged) business case.

### Business initiatives vs. IT initiatives (Figure 7.1)
| Components | Example | Relevance to EA |
|---|---|---|
| Business component only | Improving processes based on existing systems, training personnel, enhancing motivation schemes | Irrelevant to an EA practice; not discussed in the book |
| Business **and** IT components | Automating business activities, consolidating information for analytics, introducing digital sales channels | **The primary focus of an EA practice and the book** |
| IT component only | Replacing network equipment, augmenting hardware capacity, upgrading outdated IT infrastructure | Relevant to an EA practice; covered as *architectural initiatives* |

"IT initiatives in this book must be understood not as change initiatives including only an IT component, but as all change initiatives with an IT component."

### The two questions an EA practice answers about initiatives
1. **What** IT initiatives should be implemented in the future — shaping the optimal portfolio for maximum business value.
2. **How** exactly should they be implemented — finding the most optimal technical delivery.

Organization-wide discussion points (operating model, business capabilities) answer *what*; initiative-specific discussion points (business processes, business requirements) answer *how*. Specific business needs sit in the middle and correspond directly to individual initiatives.

### Multifaceted meanings of an IT initiative
Architectural — modifications of the landscape structure to be planned and optimized. Technical — solutions or systems to be implemented and deployed. End-user — prospective enhancements in business processes. Managerial — change projects or programs to be coordinated. Financial — monetary investments to be evaluated, justified and funded. Business development — concrete business needs to be addressed.

## Key Concepts — the five types of IT initiative
- **Fundamental initiatives** — strategy-agnostic; address profound "permanent" business needs important regardless of any strategy; identified top-down by global business executives + architects, often from the requirements of the adopted operating model. Grow permanent capabilities and build a reusable digitized platform.
- **Strategic initiatives** — stem directly from the current business strategy as part of its execution; identified top-down by global executives + architects, often from required business capabilities. The "workhorses" of strategy execution.
- **Local initiatives** — address critical tactical needs of individual business units; proposed **bottom-up** by local business executives (often via formal business proposals / project proposals / initiative proposals / investment proposals on standard forms), then judged by global executives and architects for organizational fitness.
- **Urgent initiatives** — address unanticipated, unpredictable, critical needs (overlooked problems, new operational demands, regulatory changes, competitor moves); originate directly from the external business environment; fundamentally unplanned. The best executives and architects can do is align them to the general strategic direction and ensure they do not undermine consistency.
- **Architectural initiatives** — no business component, only an IT component; improve landscape quality (consolidate duplicated systems, decommission legacy assets, replace aging hardware, upgrade obsolescent software, install new infrastructure, address architecture debt). Proposed top-down by architects from Technology Optimization, approved by IT executives, often funded directly by the CIO. Follow the regular two-step Initiative Delivery process with IT stakeholders acting instead of the missing business stakeholders.

These are **pure archetypes** — real initiatives often combine several types (fundamental and strategic, or strategic and local), and all of them can and ideally should introduce technical improvements as well.

### IT investment portfolio management and budgeting
- **IT investment portfolio management** — an executive-level decision-making process: analyse candidate initiatives for the forthcoming budgeting period, pick the most critical within budget (by value, costs, risks, strategic alignment, and the relative power and authority of their sponsors), and document funding decisions in the **program of work** (also: IT investment plan, investment slide) listing selected initiatives with brief descriptions, estimated costs, commencement dates and supplementary information. Signed off by business and IT executives including the CFO.
- **Budgeting** — the accompanying, largely administrative process of allocating money.
- The portfolio management process **acts as a filter, or sieve**, placed conceptually "around" Initiative Delivery, separating it from all sources of new initiatives (Strategic Planning, Technology Optimization, external environment). It produces no EA artifacts and is not itself an architectural process — but it leverages early **Outlines** for value/time/cost estimates, **Considerations and Visions** for organizational fitness and strategic alignment (especially for urgent initiatives), and **Standards and Landscapes** for technical suitability.
- Budgeting periods vary: yearly fiscal cycles, or shorter (quarterly, bimonthly) or even continual allocations; length is influenced by the desired degree of agility. Amendments mid-period are admissible — residual funds from under-spending initiatives can admit new ones.
- Real-world nuance: major business units often have separate budgets and local portfolios; **CIOs typically control small budgets** enabling simplified funding of initiatives business managers cannot appreciate (e.g. architectural initiatives); individual business leaders may control discretionary budgets; minor initiatives are often funded through less formal channels bypassing the central process.

## Mental Models
- **Portfolio goal = maximize aggregate value** by increasing the ratio of fundamental and strategic initiatives and minimizing urgent and architectural ones — while accepting that real portfolios contain all five types in different proportions, because environments are uncertain and landscapes imperfect.
- **Architectural initiatives are a last resort.** Ideally organizations "should avoid executing purely architectural initiatives altogether" and instead fold technical improvements into fundamental, strategic and local initiatives — but even mature practices periodically need some.

## Anti-patterns
- Letting urgent initiatives dominate the portfolio instead of "sifting" identified needs through disciplined Strategic Planning.
- Funding architectural initiatives separately when they could ride along with business initiatives — unnecessary overhead expense.

## Reference Tables

### Table 7.1 — Main properties of the five types of IT initiatives
| Initiatives | Origin | Motivation | Initiation | Nature | Concerns |
|---|---|---|---|---|---|
| **Fundamental** | Strategic Planning process | Grow permanent capabilities and build a reusable digitized platform | Initiated directly by global business executives | Top-down and planned | Ensure positive business case, strategic and tactical value |
| **Strategic** | Strategic Planning process | Execute the current business strategy | Initiated directly by global business executives | Top-down and planned | Ensure lasting business value beyond the current strategy |
| **Local** | Strategic Planning process | Address the essential tactical needs of business units | Proposed by local executives and approved by global executives | Bottom-up and planned | Ensure alignment with the business strategy and fitness to the organization |
| **Urgent** | External business environment | Address critical, unexpected and urgent business needs | Identified by any business leaders and immediately executed | Bottom-up and unplanned | Align with the general strategic direction as much as possible, ensure overall consistency |
| **Architectural** | Technology Optimization process | Optimize the structure of the IT landscape | Proposed by architects and approved by IT executives | Top-down and planned | Try to incorporate required architectural improvements into regular business initiatives |

### Flow of initiatives into Initiative Delivery (Figure 7.3)
| Source | Initiative types |
|---|---|
| Strategic Planning | Fundamental, strategic, local |
| External business environment | Urgent |
| Technology Optimization | Architectural |

## Worked Example
**A multi-project IT initiative.** "The IT initiative to consolidate various legacy CRM systems used in different geographical divisions into a shared corporate CRM platform may require executing several individual IT projects that together form a coherent change program: upgrading network infrastructure to provide the necessary bandwidth capacity between the divisions, migrating all customer information from local data stores to a global database and then replacing old CRM applications in each division with the new system."

**A local initiative going through the filter.** "If the head of the direct marketing department needs an IT solution to enable email marketing campaigns and submits the corresponding business proposal, then global business executives and architects should estimate to what extent this local initiative contributes to the current business strategy and to the organization in general (e.g. how it improves strategic business capabilities and aligns with the adopted operating model) and then decide whether this initiative should be delivered." Approved local initiatives are prioritized and scheduled alongside fundamental and strategic ones in Strategic Planning.

**A fundamental initiative.** "If the target operating model implies standardized business processes for managing suppliers in all business units, then a new IT initiative may be proposed directly by global business executives to automate some of these processes (e.g. registration of new suppliers) in a standard way across all business units, even though the approved business strategy for the next three years may not require significant improvements in supplier management processes."

## Key Takeaways
- The initiative is the unit of work; the EA practice exists to decide which initiatives happen and how.
- Five archetypes by origin and motivation; portfolio quality is measured by their mix.
- No initiative becomes real until it passes portfolio management and gets funded.

## Connects To
- [Chapter 6 — Processes of Enterprise Architecture Practice](ch06-processes-of-ea-practice.md) — the three processes that produce and consume initiatives
- [Chapter 13 — Outlines](ch13-outlines.md) — the artifacts used to evaluate and fund initiatives
- [Chapter 12 — Landscapes](ch12-landscapes.md) — IT Roadmaps as the home of architectural initiatives
- [Chapter 18 — Instruments for Enterprise Architecture](ch18-instruments-for-enterprise-architecture.md) — measuring investment portfolio quality
