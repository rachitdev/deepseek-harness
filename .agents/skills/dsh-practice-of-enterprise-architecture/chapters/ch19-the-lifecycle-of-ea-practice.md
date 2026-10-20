# Chapter 19 — The Lifecycle of Enterprise Architecture Practice

## Core Idea
An EA practice **cannot be "switched on" by hiring an architecture team**. It redistributes power and authority, modifies most IT decision-making procedures and must be organically integrated with regular management processes — so it is established gradually, artifact type by artifact type, over years. Two routes exist: the **historical path** (Initiative Delivery → Technology Optimization → Strategic Planning, the way the industry actually evolved) and the newer **deliberate path** (start from the organization's biggest business problem and introduce the artifacts that solve it). Maturity is hard to measure, uneven, and — because it cannot be copied or bought — a genuine source of **sustainable competitive advantage**.

## Frameworks Introduced

### The chicken-and-egg problem
Establishing an EA practice "requires initiative from both sides simultaneously, i.e. **executive mandate at the top** to sponsor the introduction of EA-related processes and the presence of **skilled architects on the ground** capable of organizing and carrying out these processes." Because the practice *is* the organizational system of disciplined decision-making, "it is impossible to become an architect in isolation from the organizational system... in the same way in which it is impossible to become a politician in a country without any political institutes and elections."

Kotusev's analogy: adopting an EA practice is like a person studying a foreign language — "beneficial and perfectly doable, but require commitment, hard work and should not be approached superficially."

### The step-wise establishment method (Figure 19.1)
Introduce and institutionalize **one type of EA artifact, or a small manageable number, at a time**, under a designated EA leader with an executive mandate (the CIO, architecture manager or an experienced architect). "The attempts to introduce too many different EA artifacts at once can overwhelm the involved stakeholders, exceed their change capacity and undermine the entire effort."

**Seven aspects that must be addressed to master one artifact type:**
1. **Senior executive support and mandate** secured — the CIO for IT-focused artifacts, chief business executives for business-focused artifacts.
2. **Commitment of relevant stakeholders** to use the artifacts for disciplined decision-making obtained.
3. **Architects appointed as owners** of the type, responsible for its usability (creating and polishing standardized templates).
4. **Consistent processes organized** around the artifacts — periodical development, update, review and re-approval with all key stakeholders.
5. **Architecture governance bodies established** to officially approve and endorse them, with regular meetings scheduled.
6. **Enforcement mechanisms introduced** (decisions artifacts only) — formal governance procedures, peer-reviews of "downstream" artifacts, or direct supervision of project teams.
7. **Valuable benefits felt, understood and widely acknowledged.**

In large companies it is often more practical to introduce and master an artifact **in one business unit or area first**, then replicate and uplift to organization-wide level.

### The historical path (Figure 19.2)
The mainstream route taken by early-adopter organizations (large banks, insurers) that experimented with information systems plans and information systems architecture for decades, learning from harsh experience with BSP, Information Engineering, EAP and then TOGAF. It runs **from the simplest to the most complex practices**:

| Stage | Process | Artifacts | Commonly called | Key practices |
|---|---|---|---|---|
| **1 — appeared first** | Initiative Delivery | **Outlines and Designs** | "Solution architecture" | Business cases for IT initiatives; a consistent step-wise solution delivery methodology with control gates informed by EA artifacts |
| **2** | Technology Optimization | **Standards and Landscapes** | "Enterprise (enterprise-wide) IT architecture" | Centralized organization-wide selection of technologies and approaches; formal architectural review, approval and supervision of all initiatives; a separate funding mechanism for architectural initiatives |
| **3 — appeared last** | Strategic Planning | **Considerations and Visions** | "'True' enterprise architecture" | Constructive strategic dialog between senior business and IT stakeholders; collective prioritization of IT investments; direct architecture governance at chief-executive level |

Stage 3 is "currently still not fully mastered even in many companies with rather mature EA practices." Because it is a natural progression, this path "arguably can be the easiest and least risky one for organizations to follow."

### The deliberate path (Figure 19.3)
The newer route, enabled by the existence of proven best practices and the availability of experienced architects. Where the historical path is "conservative, reactive and descriptive", the deliberate path is "progressive, proactive and prescriptive" — and "more effective and value-adding... although this approach can be considered riskier and requires experienced architects skilled with different types of EA artifacts."

Its basis is **EA artifacts as instruments for problem solving**. Business and IT leaders collectively answer one question — **"What is our biggest business problem in relation to IT?"** — and the answer selects the entry point:

| Symptoms voiced by executives | Problem class | Start with | Process to institutionalize |
|---|---|---|---|
| The IT budget goes mostly on keeping existing systems running · new systems are expensive to introduce · existing systems are exceedingly hard to modify · IT cannot accommodate growing business demands · numerous legacy applications (often inherited from acquisitions) pose substantial business risks | Exorbitant IT expenditures and risks, low agility, excessive dependence on legacy | **Standards and Landscapes** | Technology Optimization |
| Business does not understand what IT is doing or how IT dollars are spent · IT investments have little or no strategic impact · the landscape cannot support envisioned strategic plans · IT's contribution to business goals is marginal | Insufficient effectiveness and transparency of IT investments; strategic misalignment; poor data availability | **Considerations and Visions** | Strategic Planning |
| Too many initiatives fail to realize anticipated benefits · most projects dramatically exceed original time and cost estimates · an extensive track record of unsuccessful projects · IT is often unable to deliver on its commitments | Inefficient, unpredictable and late delivery; misunderstanding of real business impact and value | **Outlines and Designs** | Initiative Delivery |

The specific issues each subtype addresses "highly correlate with the typical decisions supported by these artifacts" (Figure 15.5) — e.g. Business Capability Models and Value Chains solve the alignment of IT investments to strategic capabilities and activities; Technology Reference Models address the problems ensuing from uncontrolled technology proliferation.

**The path is situation-specific and adaptive**: unlike the linear historical path it "offers no predefined best sequence of EA artifacts."

## Key Concepts

### Eight measures for organizational acceptance
1. **Avoid IT-babbling** — adapt language and terminology to the audience. If executives use the BCG growth-share matrix or the SCOR model, appeal to those same models and terms. "Ideally, architects should act essentially like chameleons able to easily change their color to fit the surrounding environment."
2. **Use simple, intuitive representation formats** — except for artifacts intended exclusively for architects. "Incomprehensible EA artifacts can only scare away the people and lead to general disappointment with enterprise architecture."
3. **Introduce each artifact type with a sound underlying motivation** — always know why it exists, who will use it, how, when and what decisions it facilitates; then keep optimizing it (add sought-after information, remove irrelevant or redundant sections). This avoids "producing architecture for the sake of architecture."
4. **Master artifacts gradually** — no more than one or a few types simultaneously, keeping actors inside their comfort zones.
5. **Leverage and adapt existing documents** rather than sweeping them aside. Non-architectural proposal papers become Initiative Proposals or full Solution Overviews by adding architectural sections; value chains already used for strategic communication among executives become architectural **Value Chains** — "which may be a much better approach to establishing the strategic dialog between business and IT than introducing unfamiliar Business Capability Models for the same purpose."
6. **Do not start by installing specialized EA tools**, unless the architects are already experienced with them. Most companies already have MS Office, SharePoint and a CMDB.
7. **Organize appropriate training** on the goals of the practice and everyone's specific role in it.
8. **Start from the most critical business problems** to achieve early wins, prove business value and win executive sponsorship.

### Two problems with assessing maturity
- **It is determined by ends, not means.** Maturity comes from "the genuine quality of engagement and partnership achieved between business and IT stakeholders, rather than by any formal factors or criteria." Artifacts, processes and governance bodies "can be easily imitated without enhancing the quality of communication", so maturity "cannot be reliably measured with simplistic checkbox lists, structured questionnaires or surveys."
- **It is uneven across two orthogonal dimensions.** (a) *By process* — some artifact types are mastered far better than others, so each of the three processes has its own maturity. Historically **Initiative Delivery tends to be the most mature and Strategic Planning the least**; "it makes more sense to discuss the maturity of each of the three key EA-related processes instead of... the EA practice as a whole." (b) *By business unit* — especially in large decentralized organizations, where techniques are piloted in some units before propagating. "In these cases, it makes more sense to discuss the maturity of an EA practice in specific business units."

### Timeframes
"In small organizations it may take **around two years** to establish a reasonably mature EA practice from scratch, while in large companies this process may require **up to several years** of persistent work of multiple people."

### Maturity as sustainable competitive advantage (Figure 19.4)
Packaged systems, hardware infrastructure and outsourced IT services are commodities and confer no advantage. Custom strategic systems, innovations, and even new products and services are "readily imitated by all major market players". So advantage now comes from the ability to consistently outperform competitors — **"in the 21st century the competition between organizations is a competition between their management models and approaches, rather than between their products and services as it was before."** Advantage from IT comes from managing IT better, i.e. a more developed **IT capability**, of which EA capability is a critical element alongside IT delivery and support. Because EA maturity "cannot be easily acquired, copied, mimicked or 'stolen' from other companies", it qualifies as a full-fledged factor of sustainable competitive advantage.

### Three productive consulting engagements (Figure 19.5)
| Engagement | Motivation | Consultants act as | Processes | Outcome |
|---|---|---|---|---|
| **Initiative-based** | Fill knowledge gaps in highly specific, exotic technologies; or add capacity for transient peaks of commencing initiatives | **Solution architects**, under the supervision of internal enterprise, business area and domain architects | Initiative Delivery only | Deployed and functioning IT systems, or solutions at the later development stages with all architecturally significant questions and risks addressed. "The most basic, short-term and common form of relationship" |
| **Strategic** | Stay relevant with waves of technological progress and latest industry trends; experience with massive IT-driven transformations or with the potential of disruptive technologies | **Enterprise, business area or domain architects**, working closely with internal architects | Strategic Planning and Technology Optimization | Some artifacts and initiatives, but "the most critical outcome expected... is the transfer of knowledge from EA consultants to internal architecture teams" |
| **Developmental** | Establish an EA practice from scratch, or improve the existing practice, where in-house EA expertise is inadequate | **Architecture managers** | Organizing and fine-tuning all three | Qualitative improvements in the practice plus knowledge transfer to internal EA leaders. "The most advanced form of partnership" — it also propagates practical experience across the industry |

## Mental Models
- **Maturity is doing ordinary things better, not doing special things.** "The maturity of an EA practice is manifested mostly not in doing some special activities, following more sophisticated processes or creating more advanced EA artifacts, but rather in doing ordinary things better, more systematically and in a more predictable manner." A mature practice is "a well-oiled, fine-tuned and self-optimizing clockwork mechanism, where all operations are simple, but decently honed, properly scheduled and perfectly coordinated with each other."
- **"Just enough" of the right artifacts.** Mature practices master **ten to fifteen** types, all six general types adequately represented — "maturity means using 'just enough' of the right EA artifacts, rather than more EA artifacts."
- **Sovereignty.** Internal architecture teams "are necessary to maintain the 'sovereignty' of an organization from the perspective of information systems planning." Companies "can outsource only information systems delivery, but not their strategic planning."

## Anti-patterns

### Establishment anti-patterns
- Believing hiring architects starts a practice.
- Introducing many artifact types at once.
- Starting from the wrong entry point. Kotusev's worked failure: "if business executives are eager to improve the strategic alignment between their IT investments and long-term business goals, but an EA practice in their organization has been started from introducing Standards and Landscapes, then the incipient EA effort may be viewed only as an additional burden, useless paperwork and futile bureaucracy... In this case, the executive support and sponsorship of an EA practice are likely to be discontinued."
- Replacing familiar existing decision documents with unfamiliar new EA artifacts.
- Starting with a sophisticated EA tool.

### Three counterproductive consulting relationships
1. **Project-based documentation-oriented engagements** — the old BSP study pattern: "consultants analyze the organization for several months, produce a comprehensive set of required documents, collect their paychecks and leave the organization." It "can only waste money, discredit the very word 'architecture' and even undermine all further attempts to establish an EA practice." Avoid engagements where architectural documents are the primary output.
2. **Complete outsourcing of the EA practice.** "An architecture function is a strategic organizational function that should not be outsourced." Delegating it means "companies essentially start to implement the business strategies of these consultancies, rather than their own strategies", and vendor-affiliated consultancies lead dependent clients into vendor lock-in. "As a rule, most members of an architecture team should be permanent employees of an organization, not external EA consultants."
3. **Engaging consultants to develop facts EA artifacts** (current-state Landscape Diagrams, Inventories). These are conceptually simple, imply no planning decisions, and though tedious "do not require much specific knowledge, deep expertise or particularly high qualification." Since consultants cost more than internal architects this is "economically inefficient" — justified only when in-house expertise is missing even for that, e.g. no experience with the specialized tool.

### Historical abuses of EA consulting
Short-term relationships motivated consultancies only to "sell" more documents, caring neither how they were used nor about their quality · long relationships were used as a vehicle to sell the consultancy's own products, hardware or follow-on development and support contracts · the same architectural plans were resold to multiple clients with minor adjustments while positioned as client-specific · the buzzword "enterprise architecture" was used to market whatever services were on offer, "sometimes bearing little or no connection between what was promoted and what was actually offered". Financially these planning engagements "represented sheer losses for client organizations, but riskless profits for consulting companies."

## Reference Tables

### Five outcome questions for senior business executives
Because "evaluation of its maturity should arguably concentrate on ends rather than means", ask the executive-level business audience:
1. Do you understand what IT is doing?
2. Do you understand how IT contributes to the achievement of your business goals?
3. Do you understand where and on what particular initiatives your IT budget is spent?
4. Do you understand how IT is transforming your business?
5. Do you understand what business value IT brings to your organization?

"If the answers to these questions are negative, then an EA practice cannot be considered successful regardless of how many architects have been hired, how many EA artifacts have been created and how many meetings have been conducted."

### Six signs of a mature EA practice
| # | Sign |
|---|---|
| 1 | **All six general types of EA artifacts are mastered** — specific artifacts may be organization-specific, but every type is adequately represented; often **ten to fifteen** types in total |
| 2 | **All EA-related processes are predictable and repeatable** — clearly defined, associated with standardized artifact types as regular inputs or outputs, consistently followed with all relevant decision-makers. "Many actors of a mature EA practice can accurately and unambiguously describe the respective processes and artifacts" |
| 3 | **Significant attention to optimizing artifacts and processes** — quality formally reviewed within the architecture function periodically, often yearly; inefficiencies identified and corrective actions planned; templates realigned to stakeholder information needs |
| 4 | **Direct and open dialog between business executives and architects** — no engagement managers needed as intermediaries; both sides use a common language and regard each other as partners rather than counterparts |
| 5 | **A lower ratio of urgent and local initiatives** than less mature peers in the same industry — mature practices convert many urgent initiatives into foreseen local ones and launch more top-down strategic and fundamental initiatives. (Urgent and local initiatives are never eliminated entirely, especially in dynamic industries like retail) |
| 6 | **A higher percentage of strategic IT investments** than less mature peers in the same sector — business value is better assessed at the outset; architecture debts are explicitly managed and investment-quality indicators are used to monitor the strategic ratio |

### The historical path as a maturity model
Because most organizations evolved along it, the historical path doubles as a rough three-level maturity model: Initiative Delivery institutionalized → Technology Optimization institutionalized → Strategic Planning institutionalized. "Even though this model is certainly not perfect and may be inapplicable to some organizations (e.g. organizations that started their EA practices from scratch following the deliberate path), it may still help many companies determine their approximate position on the EA maturity curve."

### The EA consulting market
Historically "information systems planning services had been offered seemingly by almost every self-respecting IT consultancy" — BSP by IBM (late 1960s), Method/1 by Arthur Andersen, Information Engineering by James Martin and Clive Finkelstein, EAP by consultants. Today EA consulting still "forms a separate multibillion-dollar global market", offered by Accenture, Capgemini, Cognizant, Deloitte, DXC Technology, EY, HCL Technologies, HP, IBM, Infosys, KPMG, Microsoft, Oracle, PwC and TCS, plus countless local, niche and boutique consultancies. But the role has changed: consultants now **complement** internal architects rather than plan on their behalf. "Internal architects constitute the permanent skeleton of an architecture team, understand the needs and specifics of their organization and protect its interests from external manipulations, while consulting architects offer a broader outlook, understand latest industry trends, best practices and technologies and enrich the organization with their expertise."

Note also: "even though many successful EA practices today are still operating under the 'signboard' of TOGAF and other popular EA frameworks, the actual prescriptions of these frameworks are never treated seriously by architects and do not affect their real activities in any sense."

## Worked Example
**Mastering Business Capability Models, end to end.** Obtain the commitment of senior business executives to adopt BCMs for focusing and prioritizing IT investments; assign or hire architects to create and maintain them; organize and consistently carry out a collective priorities-identification process involving all business leaders to heatmap the models; establish a **C-level strategy committee** to formally endorse the BCMs and the corresponding investment priorities; and finally modify the existing IT investment selection and approval procedures to take those priorities into account. "After the improved effectiveness and transparency of the new IT investment prioritization approach are acknowledged and business executives start to believe that 'it works', the use of Business Capability Models can be considered fully mastered."

**Mastering Technology Reference Models, end to end.** Secure the commitment of the CIO and other senior IT stakeholders to adopt TRMs for standardizing and consolidating the technology portfolio; nominate architects as owners; organize a periodical technology assessment and review process to identify emerging and retiring technologies from collective intelligence and update the TRMs; establish an organization-wide **technology committee chaired by the CIO** to endorse them and the resulting technology recommendations; introduce informal peer-review, formal approval and exception management procedures to ensure compliance of all initiatives. "When better cost-effectiveness, reduced risks and other organizational benefits of the new disciplined technology selection approach are appreciated and become self-evident, the use of Technology Reference Models can be considered fully mastered."

**A deliberate-path start on Technology Optimization.** Create **Inventories and Technology Reference Models** to track available assets and utilized technologies; sketch out **Landscape Diagrams** to understand dependencies between existing applications and databases; establish general **IT Principles** and more specific **Guidelines** to standardize preferred approaches; develop **IT Roadmaps** for platform simplification and consolidation, planning the retirement of duplicated assets and obsolescent technologies; and if improved data integration is required, create **Logical Data Models** to standardize core data entity structures across the landscape.

**A deliberate-path start on Strategic Planning.** Develop overarching **Principles** to articulate the most fundamental requirements for IT; create **Business Capability Models or Value Chains** for disciplined decision-making on long-term investment priorities; develop more detailed investment **Roadmaps**; add explicit **Target States** if significant landscape change is required; and maintain **Policies** where better security or regulatory compliance is needed.

**A deliberate-path start on Initiative Delivery.** For every initiative develop first **Options Assessments** (to determine business leaders' preference among implementation options), then more detailed **Solution Overviews** (informed decision-making on benefits and costs), then technical **Preliminary Solution Designs** (refine and confirm the earlier estimates), and finally low-level **Solution Designs** (describing precisely how business requirements should be addressed).

## Key Takeaways
- An EA practice takes ~2 years in small organizations and several years in large ones; it needs both an executive mandate and skilled architects, simultaneously.
- Master one artifact type at a time through all seven aspects — mandate, commitment, ownership, process, governance body, enforcement, acknowledged benefit.
- Historical path = Outlines/Designs → Standards/Landscapes → Considerations/Visions (safest). Deliberate path = start from the answer to "What is our biggest business problem in relation to IT?" (faster, riskier).
- Assess maturity by outcomes (the five executive questions) and the six signs — never by artifact counts or meeting frequency.
- EA maturity is a non-imitable resource and therefore a genuine sustainable competitive advantage.
- Use consultants for initiative-based, strategic or developmental engagements; never for document-production projects, facts artifacts, or wholesale outsourcing of the practice.

## Connects To
- [Chapter 3 — The Role of Enterprise Architecture Practice](ch03-the-role-of-ea-practice.md) — the framework lineage (BSP → TOGAF) whose failures shaped the historical path
- [Chapter 15 — The CSVLOD Model Revisited](ch15-the-csvlod-model-revisited.md) — artifacts as decision points, the basis of the deliberate path
- [Chapter 16 — Architects in Enterprise Architecture Practice](ch16-architects-in-ea-practice.md) — the archetypes consultants stand in for, and engagement managers as a sign of immaturity
- [Chapter 17 — Architecture Functions in Organizations](ch17-architecture-functions-in-organizations.md) — the governance bodies each new artifact type needs
- [Chapter 18 — Instruments for Enterprise Architecture](ch18-instruments-for-enterprise-architecture.md) — measurements, templates and architecture debt as markers of an advanced practice
