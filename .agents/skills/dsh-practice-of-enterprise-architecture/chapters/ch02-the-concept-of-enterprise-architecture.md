# Chapter 2 — The Concept of Enterprise Architecture

## Core Idea
Enterprise architecture is **a collection of special documents (artifacts) describing various aspects of an organization from an integrated business and IT perspective, intended to bridge the communication gap between business and IT stakeholders, facilitate information systems planning and thereby improve business and IT alignment.** An EA *practice* (also called enterprise architecture management, EAM) is the organizational practice of using those artifacts. The single most consequential distinction in the whole book is introduced here: **decisions EA artifacts vs. facts EA artifacts** — they have opposite development processes, and confusing them can destroy an EA practice.

## Frameworks Introduced

### Six common EA domains (Figures 2.2 and 2.3)
A layered stack, lower layers underpinning higher:
1. **Business** — capabilities, processes, tasks, roles, locations, value streams, customer experience
2. **Applications** — applied programs, corporate systems, websites, mobile apps, custom software, vendor products
3. **Data** — data entities, structures, formats, databases, warehouses, master data sources, big data
4. **Integration** — interfaces, connections, protocols, integration buses, messaging middleware, ETL
5. **Infrastructure** — hardware, servers, data centres, operating systems, system software, cloud, networks, telephony
6. **Security** — firewalls, authentication, identity and access management, cryptographic protocols

Split into **business-enabling (functional) domains** — business, applications, data: define core business functionality, visible to business stakeholders, decisions agreed with business — and **business-supporting (non-functional) domains** — integration, infrastructure, security: unrelated to business functionality, invisible to business, usually not discussed with business. Only the business domain is technology-neutral; the other five are technical.

### An EA practice sits between strategizing and implementation (Figure 2.4)
Strategic management (business executives) → *abstract business considerations* → **EA practice (translation)** → *implementable designs of IT solutions* → project management (IT project teams) → *optimal IT solutions*. All three processes are continuous, simultaneous and involve constant feedback.

### Duality of EA artifacts (Figure 2.5)
Information relevant to two audiences at once, satisfying both, in a format appealing to both.
- **Explicit duality** — different parts of the artifact are for different audiences (e.g. a solution overview whose business sections cover benefits, capability impact and process change, and whose architectural overview section covers the new IT system).
- **Implicit duality** — the same part is interpreted differently by different actors.
Both are often combined. Not all useful EA artifacts are dual.

## Key Concepts
- **Informational contents of EA artifacts vary along**: representation format (textual / graphical / tabular / mixed); level of detail; organizational scope; EA domains covered; temporal state (current, short-term <1 yr, mid-term 2–3 yrs, long-term 3–5 yrs, transitions, or stateless); storage approach (MS Word/Visio/Excel, CMDB, EA repository); volume (one page to hundreds, A4 to wall-sized posters).
- **EA is not traditional architecture.** Organizations are organic, living, evolving entities; nobody designs them the way architects design buildings. The term "enterprise architecture" is "purely metaphorical, conceptually meaningless, highly confusing and largely misguiding" — a conditional umbrella term.
- **Architects are experts, not managers** — no direct subordinates, no administrative power, no final say. Their advice must be "sold" via argumentation and persuasion.
- **Architecture function** — a supporting corporate function, usually reporting to the CIO, a planning subunit of the IT department; hosts all architects plus architecture governance bodies.

## Mental Models
- **For decisions artifacts, the process is more important than the product; for facts artifacts, the product is more important than the process.**
- **Every EA artifact must be produced with a clear idea of its intended future use.** From the infinite universe of possible descriptions, successful practices select only a limited subset actually helpful for decision-making. Unused artifacts are wasted investment.
- **The shortcut that kills EA practices**: developing decisions artifacts the way facts artifacts are developed — interviewing stakeholders and then producing the document *for* them. This "substitutes real practical planning with wishful thinking, benevolent phantasies and idealistic imaginations" and turns the practice "from the driver of communication into the factory of useless documents."

## Anti-patterns
- Producing decisions EA artifacts alone on behalf of their real stakeholders → they are ignored and never acted upon.
- Even "harmless" unilateral format changes (e.g. changing colours) to decisions artifacts without prior discussion — causes confusion, should be avoided.
- Modeling an organization comprehensively "for unclear purposes".

## Reference Tables

### Table 2.1 — Decisions and facts EA artifacts
| | Decisions EA artifacts | Facts EA artifacts |
|---|---|---|
| State | Either the future state or stateless | Only the current state |
| Represent | Made planning decisions | Documented objective facts |
| Implications | Always have implications for the future | Have no implications for the future |
| Developed | Collaboratively by all stakeholders | Solely by specific actors |
| Format | Optimized for productive teamwork, ease of editing and distribution | Optimized for long-term storage, searchability and analysis |
| Nature | Subjective — based on interests and opinions of specific people | Objective — based on acknowledged facts, independent of people |
| Role | Primary — instruments for communication, decision-making and planning | Supporting — the information base for developing decisions artifacts |
| Purpose | Help make optimal planning decisions | Help store facts important for IT planning |
| Outcome | Stakeholders act according to the made decisions | Can be used by any actors as reference materials |

### Table 2.2 — Permanent and temporary EA artifacts
| | Permanent | Temporary |
|---|---|---|
| Scope | Broad, beyond specific IT initiatives | Narrow, limited to specific IT initiatives |
| Lifespan | Long-lived, often years | Short-lived, months or weeks |
| Usage | Created once, periodically updated, constantly used, occasionally discarded | Created for a purpose, used as intended, then immediately discarded |

### Figure 2.7 — Two development processes
| | Decisions EA artifacts | Facts EA artifacts |
|---|---|---|
| Developed | Collaboratively by all stakeholders | By individual architects alone |
| Architects | Act as drivers and facilitators | Act as sole developers |
| Process | Complex, creative and politicized | Straightforward and routine |
| Value | Realized during the development process | Realized after development is completed |
| Priority | Process over product | Product over process |
| Success factor | Involvement of all stakeholders | Accuracy of all descriptions |

## Worked Example
**Figure 2.6 — three real artifacts analysed side by side.**

| Property | Principles | Landscape Diagram | Solution Design |
|---|---|---|---|
| Example | Principle 1: Standardized Business Processes; Principle 2: Single Customer View; Principle 3: Business Continuity — each with Statement / Rationale / Implications | A one-page snapshot of the current IT landscape in a specific business function | Sections: 1 Brief Overview, 2 Goals and Objectives, 3 Detailed Requirements, 4 Solution Context, 5 Data Architecture, 6 Application Architecture |
| Description | General imperatives defining how the whole organization needs to work, updated yearly | Snapshot of the current IT landscape in a business function, maintained up to date | Detailed technical description of a specific IT project going to be implemented shortly |
| Format | Textual | Graphical | Mixed (textual and graphical) |
| Detail | Very low | Low | High |
| Scope | Entire organization | Business function | Separate IT project |
| Domains | Business | Applications, data, integration | Business, applications, data, infrastructure |
| State | Stateless (no specific time focus) | Current state | Short-term future state |
| Storage | MS Word | MS Visio | MS Word |
| Volume | A few regular pages | One large page | Tens of regular pages |
| Dual | Yes | No | Yes |
| Meaning | Decisions | Facts | Decisions |
| Lifecycle | Permanent | Permanent | Temporary |

**Development walkthroughs Kotusev gives.** For *principles* or a *solution design* (decisions): the architect schedules meetings with relevant stakeholders (business executives / project teams respectively), proposes initial versions from collected opinions, runs workshops to elaborate and complete, then distributes final versions for formal approval and sign-off; from that moment all parties are committed. For a *landscape diagram* (facts): the architect reads recent project documentation, interviews the IT support department, searches the central CMDB, produces a compact one-page drawing, then returns to the IT support team to double-check accuracy; from that moment anyone can use it as a reference baseline.

## Key Takeaways
- EA = documents; EA practice = using them; architects = chief developers and facilitators, not designers.
- Classify every artifact on three axes: contents, meaning (decisions/facts), lifecycle (permanent/temporary).
- Dual artifacts are the mechanism that makes business–IT partnership possible.
- Use the right development process for the right artifact type — this is "absolutely critical for the success of an EA practice".

## Connects To
- [Chapter 4 — Enterprise Architecture and City Planning](ch04-enterprise-architecture-and-city-planning.md) — the six general types built on these properties
- [Chapter 8 — The CSVLOD Model of Enterprise Architecture](ch08-the-csvlod-model.md) — the formal taxonomy
- [Chapter 16 — Architects in Enterprise Architecture Practice](ch16-architects-in-ea-practice.md) — architect skills and archetypes
- [Chapter 17 — Architecture Functions in Organizations](ch17-architecture-functions-in-organizations.md) — where architects live and how decisions get endorsed
