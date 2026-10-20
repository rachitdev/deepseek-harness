# Chapter 16 — Architects in Enterprise Architecture Practice

## Core Idea
Architects are the key actors, facilitators and organizers of an EA practice and the chief owners and producers of EA artifacts. Every architecture position, regardless of seniority, needs the same five profession-specific qualities. Beyond that, architecture positions differ along exactly two axes — **organizational scope** and **domain expertise** — producing **five common archetypes**.

## Frameworks Introduced

### Five profession-specific requirements
Broad expertise · communicability · orientation to teamwork · innovativeness · the ability to provide a "big picture" view. Translated into qualities: a good understanding of business and IT, effective communication and collaboration, an innovative mindset and systems thinking. "Ideal architects are proficient team players and communicators, proactive innovators and systems thinkers knowledgeable in both business and IT."

### Knowledge of business and IT
**Business side** — how organizations work; the specifics of their industry (banking, hospitality, retail) and business model (franchisor, platform provider, service aggregator); typical business terminology; business strategy, competitive advantages, business capabilities and processes; business decision-supporting tools (SWOT analysis, value chain, BCG matrix) and classical MBA topics; investment calculation and business case assessment techniques; a feel for organizational politics and culture.

**IT side** — three layers of knowledge:
1. *Hands-on experience*: developing systems using programming languages; creating and administrating databases; participating in various project types and roles; preparing contracts and tender documentation; monitoring technical trends, assessing and selecting technologies.
2. *IT management processes*: project management (delivering on time and within budget); release management (rollout through development, testing, production); change management (standardized repeatable change procedures); configuration management (tracking configuration and versions of deployed components); service management (controlling IT service quality, SLAs).
3. *External IT environment*: what technologies are available or emerging and what business potential they provide; what classes of systems exist in the market and what business problems they solve; what vendors are present and what they offer.

"It is arguably very difficult for a business manager to become an architect" — the vast majority of architects emerge from IT departments with technical or engineering backgrounds.

### Effective communication
Skills required: finding appropriate words for a verbal conversation with different stakeholder groups and specific individuals · finding appropriate representation formats easily understandable to the intended audience · finding arguments to demonstrate the value of specific artifacts · finding arguments for the benefits of disciplined planning in general.

Questions a constructive dialog requires understanding: What do the stakeholders generally want to achieve? What are their essential concerns and interests? What specific problems do they see? What limitations do they have?

**"The inability of an architect to achieve a productive dialog with a particular stakeholder should be viewed as the architect's fault. Stakeholders should not be blamed for communication problems."**

### Collaborative attitude
Required traits: readiness to find the middle ground between conflicting opinions and needs · willingness to trade local advantages for global benefits · preference for mutually agreed decisions over the most "intelligent" ones · readiness to follow and commit to collective decisions regardless of personal opinion. "Excessive individualism of architects can be detrimental"; "heated semi-religious disputes are also typically counterproductive."

### Innovative mindset
Architects are "champions of IT-led innovation and proponents of adopting new technologies for business purposes", responsible with senior IT executives for keeping the organization relevant to IT trends, and for identifying potentially disruptive technologies in time to adopt them, wait for further evidence, or reject them as unsuitable.

### Systems thinking
Ideal architects see connections between generalized patterns and their specific instantiations · high-level and low-level abstraction levels · global and local needs, problems and directions · existing tactical bottlenecks and possible future strategies · current and future organizational needs and specific technologies · recent industry trends and organizational opportunities. Beyond qualitative relationships they must think in **evaluative quantitative approximations** — estimating the relative significance and magnitude of relationships, distinguishing essential from inessential, critical from negligible, major from minor, primary from secondary. They must be able to see the whole "forest" and its individual "trees", zooming in and out at will.

## Key Concepts — the five archetypes

### Solution architects
The "narrowest" architects — plan separate IT initiatives with limited scopes. Specialize in concrete technologies aligned with technical EA domains (titled application solution architects, infrastructure solution architects, security solution architects); almost never focus on the business domain, though sometimes assigned to business functions or lines of business needing contextual knowledge. **The most widely represented — roughly 60–80% of all architects**, fluctuating with the number of active initiatives. Carry out **only Initiative Delivery**, providing end-to-end support from idea-level need to deployed solution. Develop **Outlines and Designs**. Small simple initiatives take one part-time solution architect (who may serve several projects); large complex ones take several with different skills.

**Program architects** — a subclass: large-scale solution architects for broad-scope transformation programs spanning organizational units, business areas and EA domains. Also titled senior or lead solution architects. They develop initiative roadmaps, Outlines and Designs, and manage or coordinate all other solution architects on their initiative. Found mostly in large companies.

### Domain architects
Organization-wide planning of separate EA domains — usually one, sometimes a few adjacent ones (business and applications, data and integration, infrastructure and security). Titled enterprise application architects, enterprise integration architects, enterprise security architects, or simply business architects, data architects, infrastructure architects; also enterprise cloud architects, storage architects, network architects. More often found in **centralized** companies. Their duties split sharply:

- **Business-supporting domain architects** (integration, infrastructure, security) carry out **Technology Optimization** and participate in **Initiative Delivery**. They set organization-wide standards, select technologies, plan improvements and formulate rationalization suggestions; work revolves around **Standards and Landscapes**; they receive Considerations and Visions from other architects. In Initiative Delivery they consult solution architects on available technologies, assets and best practices; supply relevant rationalization suggestions; review and approve all Outlines and Designs.
- **Business-enabling domain architects** (business, applications, data) carry out **Strategic Planning**, contribute to **Technology Optimization** and participate in **Initiative Delivery**. They communicate with executives, propose long-term domain strategies, negotiate them and document them as **Considerations and Visions**; work with Standards and Landscapes; review and approve Outlines and Designs for alignment.

**Typical artifact ownership**: business architects own Business Capability Models and Value Chains · application architects drive Target States and maintain Enterprise System Portfolios · data architects deal with Conceptual Data Models and Logical Data Models · integration architects establish integration Patterns · infrastructure architects use Technology Reference Models intensively · security architects contribute to security Policies and develop security Guidelines. Non-domain-specific artifacts: Principles (all business-enabling), IT Principles (all business-supporting), Roadmaps and IT Roadmaps (collaborative), Landscape Diagrams and Inventories (collective duty), Outlines and Designs (contributed to in relevant sections).

### Business area architects
End-to-end planning of all facets of IT for separate business areas — individual business units (lines of business, functions, divisions), cross-cutting aspects (customer, finance, HR) or capability groups (payments, production, supply chain management). Effectively **"local" enterprise architects**. Titled enterprise architects for insurance / wholesale / HR and finance, or retail architects, payments architects, supply chain architects; also ERP architects, CRM architects, BI architects. More often found in **large and decentralized** companies; usually missing in small centralized ones. Carry out **Strategic Planning and Technology Optimization for their areas** and participate in Initiative Delivery.

### Enterprise architects
The "broadest" — organization-wide planning of all EA domains; generalists, though with a natural bias towards fields they specialized in earlier (payments, supply chains, integration, security). Titled enterprise architects, less often chief architects or principal architects. Found in **small organizations** (where they fulfil both domain and business area architect duties) and in **large complex companies** (where they act as **chief integrators** of the local and domain-specific planning of all other architects, developing aggregate Considerations and Visions and a consolidated technology portfolio). Work with all organization-wide artifacts: Considerations, Standards, Visions and Landscapes.

### Architecture managers
In charge of managing other architects and organizing the productive work of the architecture function; often the official heads of architecture functions reporting to CIOs, CTOs or vice-presidents of IT. **The least represented** — even large companies employ only one or a few; in small and medium organizations the role is fulfilled part-time by the most senior architect or the CIO.

Responsibilities: defining specific architecture roles and their responsibilities · defining the overall structure and size of the architecture function · improving the quality of EA artifacts and EA-related processes · allocating resources and assigning architects to initiatives · interviewing, hiring and promoting architects. They organize all three EA-related processes and may participate as observers or analysts. "They can be regarded as meta-architects, i.e. architects of architecture functions and EA-related processes."

## Mental Models
- **The hierarchy from specialists to generalists (Figure 16.1).** Solution architects = narrow scope, narrow expertise (specialists, less senior). Domain architects = broad scope, narrow expertise. Business area architects = narrow scope, broad expertise. Enterprise architects = broad scope, broad expertise (generalists, more senior). Position in the hierarchy correlates with seniority.
- **The matrix (Figure 16.2).** Rows = EA domains, columns = business areas. Enterprise architects cover the whole matrix; domain architects cover rows; business area architects cover columns; solution architects cover cells; program architects cover wider areas; architecture managers sit outside the matrix. "Architecture functions can generally be viewed as matrix structures with overlapping but complementary responsibilities" — business area architects facilitate **local flexibility, autonomy and responsiveness**, domain architects facilitate **global consolidation and optimization**.

## Anti-patterns
- Doing planning work alone, locked in a room, unwilling to communicate — "usually only waste their time creating useless paperwork".
- Blaming stakeholders for communication failures.
- Believing formal education or certification makes an architect. "Becoming an architect is hardly a question of formal education and diplomas." University EA degrees are "symbolic and indicative". EA framework certifications (TOGAF, Zachman, FEAF, DoDAF) and framework-centred trainings "cannot be helpful to aspiring architects (but can be harmful if taken seriously) simply because the prescriptions of popular EA frameworks are misguiding and do not even resemble what real architects do."
- Trying to become an architect in an organization that does not practise enterprise architecture — impossible, because the job cannot exist separately from the rest of the organization.

## Reference Tables

### Process mapping of architecture positions (Figure 16.3)
| Process | Key actors | Contributors |
|---|---|---|
| **Strategic Planning** | Enterprise architects, business area architects, business-enabling domain architects | — |
| **Initiative Delivery** | Solution architects (including program architects) | Supervised by enterprise architects, business area architects and domain architects |
| **Technology Optimization** | Enterprise architects, business area architects, business-supporting domain architects | Business-enabling domain architects |

### Two adjacent roles that are not architects
| Role | Also called | What they do | Analogy / status |
|---|---|---|---|
| **Engagement managers** | Business engagement managers, engagement partners | Visit executives, elicit strategic considerations, collect demand for new systems, analyse total demand, identify common and idiosyncratic needs, group, prioritize and present to architects; "connect" architects and the right stakeholders | "Strategic business analysts" — like business analysts in software engineering but upscaled to organization-wide level. They use Considerations and especially Visions, but aim to uncover needs, not offer solutions. "Seemingly more characteristic for immature EA practices" |
| **Technical designers** | Designers, solution designers, technical architects, system architects, project architects | Included in project teams; co-create Designs and then produce more concrete implementation plans without architects; the most technically proficient team members, often subject-matter experts in relevant technologies or vendor products | "Microarchitects" — the "next" people in the delivery chain after solution architects. Belong to IT delivery functions, not architecture functions, often associated with centres of excellence (SAP, Oracle, CRM, big data). Act strictly within the boundaries set by Designs; not concerned with organization-wide context or global landscape optimization. Sit below solution architects in the hierarchy |

### Career paths
Specialists normally advance into architecture positions only after **at least 8–12 years** of active practical IT experience.

| From | Via | To |
|---|---|---|
| Software developers | Senior developers, team leads, software designers | Application solution architects |
| Database administrators, data warehousing and BI experts | — | Data solution architects |
| Infrastructure engineers | — | Infrastructure solution architects |
| Security specialists | — | Security solution architects |
| Solution architects | Deeper domain expertise or better business-area understanding | Domain architects or business area architects |
| Domain / business area architects | Substantial experience across domains and areas | Enterprise architects |

Paths often pass through positions unrelated to architecture (e.g. project managers).

## Worked Example
**Staffing a complex initiative.** "The planning and delivery of an IT solution that implies developing new business applications, adopting new integration middleware and installing new hardware infrastructure is likely to require a small architecture team consisting of a specialized application solution architect, integration solution architect and infrastructure solution architect knowledgeable in the corresponding aspects of IT."

**How to actually become an architect.** "Arguably the only way to develop into a true architect, cultivate the necessary competencies and advance in the architecture career is observing the behavior of seasoned architects and learning from more senior colleagues over the years of practical experience. Put it simply, to become an architect and grow, start working with accomplished architects and learn from them." Despite their uselessness for developing real skills, EA certifications "might still be beneficial for other symbolic purposes" — improving CVs, distinguishing from competitors, signalling intent, promoting careers.

## Key Takeaways
- Five qualities: business and IT knowledge, communication, collaboration, innovation, systems thinking.
- Five archetypes positioned by scope × domain expertise; solution architects are 60–80% of the population.
- Domain architects drive global consolidation; business area architects drive local flexibility — a deliberate matrix tension.
- Architects are grown at work over 8–12+ years, not certified into existence.

## Connects To
- [Chapter 2 — The Concept of Enterprise Architecture](ch02-the-concept-of-enterprise-architecture.md) — general responsibilities of architects
- [Chapter 6 — Processes of Enterprise Architecture Practice](ch06-processes-of-ea-practice.md) — the three processes mapped here to archetypes
- [Chapter 17 — Architecture Functions in Organizations](ch17-architecture-functions-in-organizations.md) — how many of each archetype and in what structure
- [Chapter 19 — The Lifecycle of Enterprise Architecture Practice](ch19-the-lifecycle-of-ea-practice.md) — why architects cannot exist without an EA practice
