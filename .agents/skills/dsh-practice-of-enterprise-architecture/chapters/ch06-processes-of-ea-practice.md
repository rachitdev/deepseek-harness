# Chapter 6 — Processes of Enterprise Architecture Practice

## Core Idea
An EA practice **cannot** be represented as a single sequential step-wise process with a definite number of steps. It consists of **three distinct, simultaneous, interrelated processes** — Strategic Planning, Initiative Delivery and Technology Optimization — each revolving around one pair of CSVLOD artifact types, each with different goals, participants and outcomes.

## Frameworks Introduced

### The three EA-related processes
| Process | Artifact pair | Meaning |
|---|---|---|
| **Strategic Planning** | Considerations + Visions | **strategy-to-portfolio** |
| **Initiative Delivery** | Outlines + Designs | **need-to-solution** |
| **Technology Optimization** | Standards + Landscapes | **structure-to-rationalization** |

Strategic Planning and Technology Optimization are continuous, largely unstructured and somewhat informal — they cannot be broken into repeatable linear steps. Initiative Delivery is a finite linear step-wise process with specific inputs and outputs.

### Strategic Planning
Translates fundamental factors of the external business environment (economic, technological, social, demographic, legal, political, ecological, customer, competitor) into Considerations and Visions. Usually a single organization-wide instance; decentralized companies (diversification or coordination operating models) may run several unit-level instances too. Integrated with regular strategic management (environmental analysis, competitive advantage identification, goal formulation) and the annual business planning cycle. Consists of meetings, presentations, workshops and even informal "elevator" discussions, plus periodical formal approvals and sign-offs, often yearly, aligned with fiscal years, budgeting cycles, board meetings or strategy updates.

Questions Considerations answer: What role should IT play? Which IT capabilities should be provided organization-wide? What IT-related policies should be complied with? What level of business continuity and security is required? Which IT-based business innovations should be adopted?

Questions Visions answer: What should IT deliver to the business in the long run? Where should future IT investments go? What types of IT investments should be made? When? In what sequence?

### Initiative Delivery
Translates specific business needs — or more rarely specific technical needs — into tangible IT solutions. Multiple parallel instances, one per active initiative. Integrated with regular project management (scoping, estimating, scheduling, resourcing, monitoring) and follows a project lifecycle with phases and control gates (e.g. scope, evaluate, plan, build, test, deploy). Regardless of organization-specific phases it always consists of **two inherent consecutive steps**:

1. **Initiation** — business executives + architects. Formulate goals and essential requirements; generate implementation options; evaluate pros and cons; select the most appropriate option; get executive approval (also known as a feasibility study). Key artifacts: **Outlines**. Discussion point: **business processes**. Outlines answer: What do the proposed solutions look like conceptually? How will they modify established business processes? What is their immediate and long-term business value? What is their overall business impact? What is the cost and when can they be delivered?
2. **Implementation** — architects + IT project teams + business representatives. Elaborate business requirements, develop detailed implementation plans, deliver. Key artifacts: **Designs**. Discussion point: **business requirements**. Designs answer: What business requirements should be addressed? What new software should be developed or installed? Which data types and entities should be used? What servers and hardware should be deployed? How exactly should new systems interact with existing ones?

**Complementary documents:** Outlines pair with **business cases** (purely business-specific financial justification; Outlines inform their time and cost estimates). Designs pair with **project management plans** (purely administrative; Designs inform delivery schedules, resource demands, quality assurance strategies). Both pairs are approved together before their respective stages proceed.

### Technology Optimization
Translates information on the current structure of the IT landscape into technical rationalization suggestions. Single instance usually; several in highly decentralized organizations. An **"internal" IT-specific housekeeping process**, not integrated with other regular organizational processes, carried out mostly by architects inside the IT department with some involvement of other IT experts and the CIO. Also aims to reduce **architecture debt**.

In Standards architects mark technologies, approaches and practices as desirable, current or strategic vs. undesirable, deprecated or retiring. In Landscapes they mark IT assets as "healthy", reusable or strategic vs. "unhealthy", non-reusable or to-be-decommissioned. Periodical formal approvals by the CIO or IT executives, often yearly.

Questions Standards answer: What technologies and vendor products are used? What approaches and best practices are adopted? Which are redundant or fulfil similar purposes? Which cause problems? Do they meet general business needs?
Questions Landscapes answer: What IT assets are maintained? Which are not actively used or duplicated? Which are no longer vendor-supported? Which may cause problems in future? Are they adequate for business needs?

## Key Concepts — the three synergies
- **Strategic Planning ↔ Initiative Delivery** → *strategic effectiveness of delivered IT initiatives*. Visions represent planned business needs; Considerations provide their initial requirements. Ideally all initiatives launch from Visions; in reality initiatives launch either from planned needs (Visions) or from urgent needs arriving directly from the external environment. Cancelled initiatives feed back and may force executives to adjust Considerations and Visions.
- **Initiative Delivery ↔ Technology Optimization** → *technical optimality of delivered IT solutions*. Rationalization suggestions are incorporated into Outlines (early) and Designs (later); Outlines and Designs are peer-reviewed by the architects responsible for Technology Optimization. Feedback: Landscapes are updated at project completion; Standards are updated after successful innovative initiatives.
- **Technology Optimization ↔ Strategic Planning** → *effective mutual interrelationship between strategic business directions and strategic IT capabilities*. Considerations and Visions guide the evolution of Standards and Landscapes; conversely, knowledge of actual IT capabilities and constraints stops architects approving unfeasible Considerations and Visions and "facilitates the formation of sober and realistic business strategies."

## Mental Models
- **Two goals per initiative.** Most IT initiatives pursue two orthogonal goals simultaneously: address a business need and improve the current landscape. Mature practices combine them so that regular business initiatives "maintain, cleanup or even improve the IT landscape along the way".
- **The "ivory tower" syndrome.** Insufficient stakeholder involvement in the corresponding process makes artifacts inexplicable, irrelevant and useless. Since most artifacts (except most Landscapes and some Standards) are decisions artifacts, artifacts created without their real stakeholders are ignored and shelved — "architecture is produced for the sake of architecture."

## Anti-patterns
- Architects making strategic business decisions alone → never treated seriously or acted upon; Strategic Planning effort wasted.
- Architects deciding the necessity and value of initiatives alone → never sponsored or funded; Outlines never proceed.
- Architects making project-level implementation decisions alone → never respected or committed to by project teams; even perfect Designs ignored.

## Reference Tables

### Table 6.1 — Strategic Planning, Initiative Delivery and Technology Optimization
| | Strategic Planning | Initiative Delivery | Technology Optimization |
|---|---|---|---|
| Instances | Single, or several in highly decentralized organizations | Multiple — one per active IT initiative | Single, or several in highly decentralized organizations |
| Goal | Articulate the desired future course of action for business and IT | Deliver optimal IT solutions for specific needs | Improve the overall quality of the corporate IT landscape |
| Meaning | Strategy-to-portfolio | Need-to-solution | Structure-to-rationalization |
| Question | How is the business environment changing and what should we do to react? | What is the best way to address the requested need and all associated requirements? | What is wrong with the current IT landscape and what should we do to improve it? |
| Focus | Long-term and mid-term future | Short-term and immediate future | Current situation with some future outlook |
| Nature | Continuous and largely unstructured | Sequential: initiation then implementation | Continuous and largely unstructured |
| Integration | With regular strategic management | With regular project management | Not integrated with any regular processes |
| Actors | Business leaders and architects | Initiation: business leaders + architects. Implementation: architects + project teams | Architects alone |
| EA artifacts | Considerations and Visions | Initiation: Outlines. Implementation: Designs | Standards and Landscapes |
| Inputs | Fundamental factors of the external business environment | Specific business, and sometimes technical, needs | Current structure of the organizational IT landscape |
| Activities | Informal discussions, meetings, presentations, workshops, periodical formal approvals and sign-offs | Initiation: discussion of implementation options. Implementation: actual technical implementation | Numerous informal discussions and periodical formal approvals |
| Discussion points | Operating model, business capabilities, specific business needs | Initiation: business processes. Implementation: business requirements | Little or no discussion between business and IT |
| Outputs | High-level strategic plans reflected in Considerations and Visions | New working IT solutions | Technical rationalization suggestions reflected in Standards and Landscapes |

### Typical technical rationalization suggestions passed to Initiative Delivery
Reuse specific IT assets, products or technologies · follow certain implementation approaches or practices · integrate with particular existing systems · substitute the functionality of specific legacy systems · migrate, transform or enrich data from legacy databases · decommission assets replaced by the new solution · create a reusable IT asset for future initiatives · pioneer new technologies or products · try a new proof-of-concept approach.

## Worked Example
**How Strategic Planning produces an initiative (Kotusev's own chain).** Visions suggest that a certain business capability should be uplifted in the future (business capability map). Considerations indicate that all business operations should be resilient and failsafe (business continuity principle). Together these form a **planned business need**: deliver a highly available and reliable IT solution enhancing the requested capability. A new IT initiative is launched to explore the need, discuss implementation options via Outlines and deliver the best possible solution. If the Outlines-based discussion concludes that all options are too expensive, too risky or too slow, the initiative is cancelled — which feeds back into Strategic Planning and may force leaders to revise Considerations and Visions and conclude that "there are no easy ways to implement the approved business strategy as initially planned."

## Key Takeaways
- Three processes, three artifact pairs, three meanings: strategy-to-portfolio, need-to-solution, structure-to-rationalization.
- Only Technology Optimization can run on architects alone; the other two require conscious stakeholder participation.
- The value of the whole is in the three bidirectional information flows, not in any single process.

## Connects To
- [Chapter 4 — Enterprise Architecture and City Planning](ch04-enterprise-architecture-and-city-planning.md) — the six artifact types these processes revolve around
- [Chapter 7 — IT Initiatives and Enterprise Architecture](ch07-it-initiatives-and-enterprise-architecture.md) — the five types of initiative and their flow through these processes
- [Chapter 16 — Architects in Enterprise Architecture Practice](ch16-architects-in-ea-practice.md) — which archetype of architect carries out which process
- [Chapter 18 — Instruments for Enterprise Architecture](ch18-instruments-for-enterprise-architecture.md) — architecture debt and agility dimensions per process
