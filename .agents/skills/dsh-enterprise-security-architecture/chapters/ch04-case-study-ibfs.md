# Chapter 4 — Case Study: Intergalactic Banking and Financial Services Inc

## Core Idea
The book consolidates most of its case material into one **fictional but realistic** global financial institution, **Intergalactic Banking and Financial Services Inc (IBFS)**, which is then used as the running example in almost every later chapter. The chapter's real teaching content is not the company profile — it is the **set of senior-management interviews**, which demonstrate how the contextual security architecture is actually gathered: by talking to business people, in their language, about their goals and fears, and then treating everything they say as a security requirement.

## Frameworks Introduced
**Requirements gathering by structured executive interview.** The architecture team's business analysts interview senior managers across functions — CEO, COO, eBusiness, CFO, Marketing & Distribution, CIO, Compliance — and capture the "main messages." Every interview yields raw material for the contextual row of the SABSA matrix: assets, risks, processes, organisation, geography, and time dependencies. This is the input to the Business Attributes profiling done in Chapter 6 and the risk model in Chapter 9.

## Key Concepts — the IBFS profile

**Business sectors (all of them):** retail banking (current accounts, direct debits, standing orders, debit cards, credit cards, cheque payments, Internet payments); corporate banking (current accounts, foreign exchange, treasury, payments); general insurance (household, motor, travel, healthcare); life assurance; pensions; personal investment products (unit trusts, annuities, regional special investment products); savings, loans and mortgages; asset management; custody and other agency services; securities trading; corporate finance (mergers, acquisitions, divestments, flotations); invoice financing / factoring.

**Scale and shape:** a global group of companies established in **84 countries**, dealing with multi-cultural, multi-lingual and multi-time-zone issues. Channels: branch offices, independent agents, newspaper and television advertising plus a call centre, and increasingly the Internet (web and e-mail).

**The trigger for the architecture programme:** early Internet offerings were *very tactical*, with small market penetration, designed and operated by small project teams inside business units. IBFS now needs a strategic approach — infrastructure in both technology and process — that will enable a wide range of new business applications.

## Reference Tables

### The seven IBFS interviewees
| Name | Role | Based | Profile |
|---|---|---|---|
| **David Smith** | Group Chief Executive Officer | New York | 52, Harvard, banking since graduation, postings in London, Frankfurt, Hong Kong; 10 years at IBFS, 4 in post; oversaw growth by major acquisition |
| **Juan Carlos** | Chief Operating Officer | London | 45, 7 years at IBFS, previously senior management in global retail; London chosen because it straddles the Far East / Americas time gap |
| **Rosemary Brown** | SVP, eBusiness | — | 48, in insurance since leaving school at 16; joined when her London-based-turned-global insurance group was acquired 3 years ago; now owns digital business channels across the group |
| **Helmut Meyer** | Group Chief Financial Officer | Frankfurt | 54, credit controller at 18 in sales invoice finance, financial operations management in hi-tech, Bank Treasury at 35; "rules with an iron hand" |
| **Brian Jones** | SVP, Marketing and Distribution | New York | 43, ex-bond trader turned derivatives/futures group leader; no formal marketing training but a powerful strategic thinker |
| **Ranjit Patel** | Chief Information Officer | — | 39, US telecoms then a large global voice-and-data networking company (of which IBFS was a customer) |
| **Ho Siew Luan (Sarah)** | Director of Compliance | Singapore | 35, first-class law degree in the UK, MBA in the US, appointments in Washington D.C., Hong Kong, Sydney; co-ordinates local compliance officers in all 84 countries |

### What each interview contributes to the security architecture
| Interviewee | Business drivers stated | Security architecture implication |
|---|---|---|
| David Smith (CEO) | Shareholder returns maximised over the investment horizon; long-term investment in people and technology; lasting customer relationships through value for money and excellent service; leadership position; restructuring and business process re-engineering; heavy Internet investment for first-mover advantage; strong legal and regulatory compliance and due diligence | **"The banking industry more than almost any other is based on trust."** Customers give you all their money. "You can withstand a few isolated incidents, but if there is a constant tide of events where trust has been breached, then you're finished." Protection of trust must be a **primary business requirement in all information systems**. Brand and reputation are the top-level assets. |
| Juan Carlos (COO) | Truly global operation: multi-cultural, multi-lingual, all time zones, all religious calendars — genuinely 365×24; virtual teams whose members rarely meet and whose hours barely overlap; encourages managers to spend overseas trips building relationships rather than on today's operational issues | Availability and continuity across all time zones; secure remote collaboration; the recognition that good human relationships are what carry teams through periods when *only* technical systems connect them. |
| Rosemary Brown (SVP eBusiness) | "In the good old days we used to control this business… In the future it is the customers who will be in control." The Internet has empowered people; a customer is a few mouse clicks from another provider. CRM and customer service determine retention. Customers expect **smoothness and consistency across all interfaces** and expect you to remember what they already told you — "once we know their name and address, we should never, ever ask for it again unless it is to change these details" | Enterprise-wide consistency of customer information management. Her greatest challenge: many business units spawning many application projects; if any one goes off doing its own thing on customer information, the enterprise-wide CRM initiative she owns is badly damaged. This is the **argument for architecture governance**. |
| Helmut Meyer (CFO) | Squeezed margins; profitability and cash flow under close scrutiny; substantial office property investment they want to realise; constant mergers, acquisitions, divestments and joint ventures, some short-lived | Must be able to **integrate and disintegrate ICT systems and network infrastructure** on demand — "flexibility for an ever-changing business strategy is one of the most important drivers." And the sceptic's demand: "I'm rather sceptical about security. It has cost us a lot in the past without any demonstrable benefits… I want to see a clear demonstration of a return on our investment." This is the requirement that Chapter 6 exists to answer. |
| Brian Jones (SVP Marketing) | Joint-venture partnerships, especially with similar firms in regions where IBFS lacked presence, for market access and cross-selling; acquisition products (low-interest credit cards, high-interest savings, low-cost motor insurance) used to hook customers for cross-sell | **Joint ventures need a special security approach**: both parties give the other access to business information systems while maintaining independence and their own control — "we share some things, but not everything." Also: usability is a **competitive issue** — endless layers of user IDs and passwords have drawn complaints from the agent and IFA network, who will simply use easier providers. And a delivery complaint: promotional offers such as 0% balance transfer are not followed through because systems are not changed properly or on time. |
| Ranjit Patel (CIO) | Legacy applications with a **stovepipe architecture** — own interfaces, own data repository, hard to integrate or share data; new applications architecture built around a **data warehouse** as a single central customer data repository for CRM; major global and Fortune 100 clients demand instant transfer of huge sums to chase interest rates; outsourcing all non-core operational services | Legacy batch-processing systems cannot give a real-time integrated picture of a client's total available funds, creating a **double risk**: transfer funds without sure knowledge they are available, or upset major clients. Discarding legacy is unacceptable and re-engineering everything is prohibitive — "the most major and pervasive issue that our architecture team faces." **Single sign-on is a mandatory strategy, not a nice-to-have** — maybe not in one step, but the architecture must allow progressive movement toward it. And the outsourcing guiding principle: **"any platform and any network can potentially be outsourced"**; the architecture must enable that with minimum disruption and minimum impact on ownership, policy-making and policy enforcement for each physical domain, and must handle liability and service level management. |
| Ho Siew Luan (Compliance) | Too many spectacular failures by major institutions; multiple regulators per country; 84 countries with different legal and regulatory regimes; international anti-money-laundering regulation; comprehensive EU data protection legislation; evaluating the **New Basel Capital Accord (Basel II)** response in operational risk management | **Insider dealing separation**: corporate finance (advising on mergers, acquisitions, takeovers, flotations) information must be kept from securities traders who could use it illegally — a domain-separation requirement. Board committed to strong governance and due diligence, requiring a **culture of compliance** plus technical and procedural control mechanisms giving assurance of compliance, and **reliable documentary evidence to prove compliance beyond reasonable doubt before a regulator or court**. |

## Mental Models
- **Everything a business executive complains about is a security requirement.** Margin pressure, joint-venture governance, legacy integration, agent attrition due to password fatigue, insider dealing walls, "prove it to a regulator" — all of these land somewhere in the SABSA matrix.
- **Interview the sceptic too.** Helmut Meyer's hostility is not an obstacle to route around; it defines the ROI deliverable the programme must produce.
- **A fictional composite is a legitimate teaching device** — the authors state IBFS and its people are entirely fictitious, constructed from wide industry experience so that practitioners recognise it as realistic.

## Anti-patterns
- Tactical, business-unit-funded Internet projects with small market penetration and no enterprise strategy — IBFS's starting condition.
- Stovepipe legacy applications with private interfaces and private data repositories.
- Letting individual application projects each define their own customer information management.

## Worked Example
The chapter *is* the worked example: an executive interview round producing raw contextual requirements. The most compact demonstration of how one sentence becomes an architecture requirement is Ranjit Patel's outsourcing principle:

> "The guiding principle for our security architecture must be that **any platform and any network can potentially be outsourced**. The architecture must enable this to happen with minimum disruption and minimum impact on the ownership, policy-making and policy enforcement for each of these physical domains."

Translated into SABSA terms: this is a *Where* requirement. At the contextual layer it is Business Geography plus outsourcing relationships (*Who*). At the conceptual layer it forces the **security domain model** to make domain ownership and policy authority explicit and separable from physical hosting. At the logical layer it becomes security domain definitions and associations. At the physical layer it constrains platform and network infrastructure design so that a domain can change operator without changing its policy. Chapter 10's domain and trust modelling is the direct answer to this one sentence.

## Key Takeaways
- IBFS: fictional global financial group, 84 countries, full product range, channels from branches to the web; running example for the whole book.
- The contextual architecture is gathered from senior-management interviews, one per major function.
- Trust is the CEO's primary asset; ROI proof is the CFO's precondition; usability is the marketing SVP's competitive issue; single sign-on and "anything can be outsourced" are the CIO's architectural constraints; regulatory evidence and insider-dealing separation are the compliance director's.
- Business-driven security architecture means these statements, not a technical threat list, are the source requirements.

## Connects To
- [Chapter 3 — Security Architecture Model](ch03-security-architecture-model.md) — the matrix these requirements populate.
- [Chapter 5 — A Systems Approach](ch05-a-systems-approach.md) — the IBFS Securities Trading Division case study.
- [Chapter 6 — Measuring Return on Investment](ch06-measuring-roi-in-security-architecture.md) — the IBFS Business Attributes profile that Helmut Meyer's scepticism demands.
- [Chapter 9 — Contextual Security Architecture](ch09-contextual-security-architecture.md) — the full contextual row for IBFS.
- [Chapter 10 — Conceptual Security Architecture](ch10-conceptual-security-architecture.md) — domain and trust models answering the outsourcing and joint-venture requirements.
