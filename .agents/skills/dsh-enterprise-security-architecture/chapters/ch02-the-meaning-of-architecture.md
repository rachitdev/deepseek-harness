# Chapter 2 — The Meaning of Architecture

## Core Idea
Architecture is **a set of rules and conventions by which we create things that serve the purposes for which we intend them**, and its primary job is to **manage complexity** — to break complexity down into apparent simplicity through *layering* and *modularisation*. The chapter's central distinction is between **architecture** and **plumbing**: both are valuable, but in ICT people confuse them constantly. "True architecture never happens by accident."

## Frameworks Introduced

### The three determinants of any architecture
Whether you are building a town or a business information system, only three factors determine what architecture you can create:
1. **Our goals**
2. **The environment**
3. **Our technical capabilities**

### A High-Level Reference Model for Business Systems Architecture (Figure 2-1)
Explicitly a creation of the authors, not an international standard. Four stacked layers, one front-plane and one backplane:

| Element | Role |
|---|---|
| **Business Architecture** (top layer) | How the business itself is structured — organisational model, processes, functions. The *primary* architecture; every other sub-architecture exists to support it. Often called the "business model". |
| **Information Architecture** | The framework within which business information is created, organised, processed, stored, retrieved and communicated. Describes information types, their structured relationships and organisation, information behaviour, information management processes, and physical locations/repositories. Information *is* the business, represented in a particular form. |
| **Applications Architecture** | How applications are designed, how they inter-operate, how they are supported in the infrastructure. Mirrors the automated parts of business processes. Its main objective is to **enable business processes** by being flexible, economic and responsive to change. |
| **Infrastructure Architecture** | Computer platforms (hardware + OS), computer networks (cables, lines, switches, routers), and the **middleware** layer that bridges physically different infrastructures and presents a consistent virtual interface. "What most people would recognise as technical architecture." |
| **Risk Management Architecture** (front-plane, cuts across all four) | Risk management is *pervasive* — it happens inside all four layers. Close to, but not the same as, security architecture. |
| **Management & Governance Architecture** (backplane, wraps everything) | Decision-making processes and levels of authority assigned to decision-making entities (individuals or committees). Essentially a model of how power is wielded and what span of control attaches to each entity. |

Characteristics of a modern applications architecture: **component based** (re-usable generic modules), **service oriented** (components offering services to one another), **built on a strategic middleware layer** for services integration, and **offering distributed processing**.

### Infrastructure Architecture Reference Model (Figure 2-2)
Applications sit on **Services Integration** (traditional middleware + data management + common services delivered transparently to applications). Beneath it: **Information Transfer** (the communications network) and **Data Processing** (platforms — hardware and operating systems, where raw bits and bytes are manipulated).

Three **pervasive backplanes** cut across all layers:
- **Security services** (including all services used to control the infrastructure, such as time service)
- **Directory services**
- **Service management**

And the whole model sits on a further backplane, **Operational Services** — the *people* and the operating processes and procedures they carry out. Concerned with people, not technology, but an integral part of the infrastructure.

### Definition of information systems architecture (the authors')
> "A consistent set of principles, policies and standards that sets the direction and vision for the development and operation of the organisation's business information systems so as to ensure alignment with and support for the business needs."

### The Eternal Triangle of Conflicting Objectives (Figure 2-3)
Simplify the wider requirement set to three and they pull against each other permanently:
**Cost control ↔ Security ↔ Usability.** Higher security or higher usability costs more; increasing security usually damages usability and vice versa. The successful architecture *balances the tensions*, it does not eliminate them.

### Geoff Rob's Ten Rules for the Solution Architect
1. **Listen and Learn** — understand the environment and business requirements fully before selling a solution; this builds trust.
2. **Lead Diplomatically** — the client pays for a motivated person to take charge and give clear direction; still give others time and space to express themselves.
3. **Your Area of Expertise** — know one technology area in depth and lead in it; collaborate with leaders who supplement you elsewhere.
4. **Repeatability** — capitalise on work already done for other clients; adapt it for faster delivery and higher success rate.
5. **Market Awareness** — have a global view of alternatives and be able to compare them with yours.
6. **Business Sense** — understand the costs and business impacts; keep business benefits and client priorities paramount.
7. **Design Acceptance** — seek acceptance early and openly rather than developing in isolation and fighting for acceptance later; discuss design principles and constraining factors, and defend your rationale.
8. **Don't Go to Extremes** — match planning and design to the client's situation; marketing hype and interesting experiments may not suit.
9. **Best Fit** — if a solution is too complex or costly, find the part that solves the majority of problems within budget.
10. **Leverage Client's Investment** — use infrastructure already in place; question technology installed at heavy cost for short-term use with doubtful benefit.

## Key Concepts
- **Business assurance** is the wider topic that contains information security. It embraces three major areas: **information security; business continuity; physical and environmental security** — and broader still, all aspects of **operational risk management**. The enterprise security architecture and the security management process should embrace all of them.
- **SABSA** = **S**herwood **A**pplied **B**usiness **S**ecurity **A**rchitecture, in development by the authors since 1995, a registered trademark of SABSA Limited. Layered, top layer = business requirements definition, then conceptual, logical, physical, and component ("the shopping list"), plus operational architecture covering management, administration and operations.
- **The model is generic; the output is unique.** SABSA is a starting point for any organisation; going through its analysis and decision-making produces an output specific and highly customised to that enterprise.
- **Small isolated projects do not need architecture** — a single chief designer can hold the whole design. Architecture becomes necessary when size, complexity or the need to integrate with other projects exceeds one designer's grasp, or when projects are built independently over an extended period.

## Mental Models
- **Sydney Opera House** — a building of that calibre could never be produced by many small teams each with its own ideas and a narrow view of the goal. It requires *a single central vision*, created by an architect who takes the widest view of all requirements from all interested parties, and who **remains in control throughout**, supervising so the design's integrity is not compromised later.
- **Architecture vs plumbing** — both valuable, not the same thing.
- **The chain is not good enough; use the car.** Security-as-a-chain (strength = weakest link, one missing link makes the rest valueless) is a "reasonably good analogy," but a checklist that names every component of a car does not give you a car. Complex systems have a *holistic design quality* that checklists cannot describe.
- **Think in business terms at all times**, even down at the nuts and bolts: *Why are we doing this? What are we trying to achieve in business terms here?*

## Anti-patterns
- **Piecemeal/tactical acquisition.** A requirement is identified, a product is sought and bought without regard to broader implications. Result: isolated point solutions that cannot integrate or inter-operate, increased complexity and support cost, exploding administration workload, no analysis of long-term/operational cost (the bulk of total cost of ownership), and no strategy identifiably supporting business goals.
- **Security relegated to add-on fixes** after all other design decisions have been frozen.
- **Technically-led architecture** — vendor interests and technical innovation driving strategy rather than business need.
- **Compiling business requirements, documenting them, shelving them,** then designing from technical thinking alone.
- **The checklist approach** — verifying that links exist without testing that they fit together into a secure chain.
- **Skipping layers to get to the solutions section** — "there simply is no substitute for doing architecture work the proper way."

## Reference Tables

### Generic business requirements for an information security solution (beyond CIA)
Primary requirements are protecting **availability, integrity, authenticity and confidentiality** of business information and providing **accountability and auditability**, established by detailed business-process analysis using direct interviews with operational business managers. The *generic* requirements that must be balanced against these:

| Requirement | What it demands |
|---|---|
| Usability | Appropriate to the technical competence of intended users and ergonomically acceptable to them |
| Inter-Operability | Meets long-term inter-operability needs between communicating systems and applications |
| Integration | Integrates with the wide range of applications and platforms it may be needed for in the long term |
| Supportability | Can be supported in its environment — number of end users, service delivery points, geography, distribution |
| Low Cost Development | Modular, hence integrable into a development programme at minimal cost |
| Fast Time to Market | Integrable with minimal delay |
| Scalability of Platforms | Fits the platform range — mainframes to mid-range servers to PCs, workstations, laptops, palmtops, and increasingly digital TVs, mobile telephones, any consumer electronics with processing and communications |
| Scalability of Cost | Entry-level cost appropriate to the range of applications intended |
| Scalability of Security Level | Supports the range of cryptographic and other techniques needed for the required range of security strengths |
| Re-Usability | Re-usable across similar situations for best return on acquisition and development |
| Operations Costs | Minimises cost impact on systems operations |
| Administration Costs | Provides an efficient means of security administration |
| Risk-Based Cost/Benefit Effectiveness | Risk reduction appropriate to acquisition, development, installation, administration and operation cost |

### Business-specific enablement requirements that shape security strategy
Exploiting the global reach of the Internet; global e-mail and e-messaging; outsourcing operation of networks and computer systems; providing remote access to third parties; developing on-line business services; delivering digital entertainment products (video, music); improving customer service through information integration and consistent user-interface presentation; obtaining software upgrades and system support through remote vendor access; tele-working, mobile computing, road warriors and the virtual office.

### The holistic questions checklists miss
Do you understand the requirements? Do you have a design philosophy? Do you have all of the components? Do these components work together? Do they form an integrated system? Are you assured that it is properly assembled? Does the system run smoothly? Is the system properly tuned? Do you operate the system correctly? Do you maintain the system?

## Worked Example
**Case Study: User Authentication** — the book's canonical demonstration of piecemeal cost.

Each business application requires a separate user ID, and each enforces *different syntax rules*, so a user cannot replicate one ID across systems. Each ID has a password, again with heterogeneous syntax rules and different change regimes. The end state: every user holds a different user ID and a different password for every system.

The authors deliberately set aside the security argument (which they call contentious and often debated) and cost the *ownership* consequences:
- Administering creation, maintenance and deletion of multiple user IDs and passwords;
- Help desk support for a flood of user login problems;
- Lost user productivity while users try to solve their own authentication problems.

Conclusion: it is exactly this class of problem that drove the adoption of **strategic, architecture-led user authentication services across multiple business applications — i.e. single sign-on**. The lesson is generalisable: the business case for architecture is often made on total cost of ownership and operability grounds, not on risk-reduction grounds alone.

## Key Takeaways
- Architecture = rules and conventions to manage complexity, via layering and modularisation.
- Goals, environment and technical capability are the only three determinants.
- Business architecture is primary; information, applications and infrastructure architectures exist to serve it; risk management cuts across all four; management and governance wraps them.
- Information systems architecture: "a consistent set of principles, policies and standards that sets the direction and vision… so as to ensure alignment with and support for the business needs."
- Cost, security and usability are in permanent tension; the architect balances, never eliminates, that tension.
- Business assurance = information security + business continuity + physical and environmental security, all inside operational risk management.
- Checklists cannot substitute for a holistic systems engineering approach — the car, not the chain.
- The successful architect is a communicator and a salesperson who needs senior management buy-in as a critical success factor.

## Connects To
- [Chapter 1 — The Meaning of Security](ch01-the-meaning-of-security.md) — what "secure" means before you architect for it.
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — the six SABSA layers and the 36-cell matrix.
- [Chapter 5 — A Systems Approach](ch05-a-systems-approach.md) — the systems engineering discipline behind layering and modularisation.
- [Chapter 8 — Managing the Security Architecture Programme](ch08-managing-the-security-architecture-programme.md) — selling the benefits and getting sponsorship.
