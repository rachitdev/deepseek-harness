# Cheatsheet — Google Cloud AI Reports

Dense reference for the two reports. **R1** = The ROI of AI in Customer Experience (Google Cloud + National Research Group, fieldwork Apr 18 – Jun 3, 2025, n=3,466). **R2** = AI Agent Trends 2026 (no new primary survey; re-cites R1).

---

## 1. Base sizes — always quote these with the percentage (R1)

| Base label | n | What it gates |
|---|---|---|
| Total (global) 2025 | 3,466 | Adoption, spend, objectives, LLM criteria, meaningful-impact areas |
| Total (global) 2024 | 1,539 | Every "vs. 2024" comparison |
| Agentic AI early adopters | 460 | All early-adopter figures |
| Early adopters, IT spend question | 459 | 39% AI share of IT spend |
| Early adopters, agent-ROI question | 456 | ROI by agent type |
| Executives leveraging agentic AI | 1,814 | AI agent use-case mix (also R2's source) |
| Currently using gen AI for CX/field service | 2,190 | Internal vs. external split |
| Using or planning gen AI for CX/field service | 3,285 (2025) / 1,445 (2024) | 37% vs. 34% CX ROI |
| Early adopters, same filter | 449 | 55% CX ROI |
| Reporting improved CX | 2,177 (2025) / 921 (2024) | Engagement 83%, NPS 75% |
| Reporting improved CX, sized | 2,133 (2025) / 912 (2024) | 6–10% magnitude |
| AI share of IT spend, total | 3,315 | 26% mean |
| Comprehensive C-level sponsorship | 1,269 (2025) / 540 (2024) | 78% ROI figure |
| No comprehensive sponsorship | 2,197 (2025) / 999 (2024) | Comparison bar |
| Agent types deployed, total | 1,802 | ROI by agent type |

**Rule of thumb:** any R1 figure about CX improvement is filtered to respondents who already said CX improved. It answers "how much," never "how often."

---

## 2. Headline numbers at a glance

| Figure | Statement | Base |
|---|---|---|
| 88% | Early adopters seeing positive ROI on at least one gen AI use case | n=460 |
| 74% | Same, all organizations | n=3,466 |
| 82% / 39% | More than 10 agents in production: early adopters / all | 460 / 3,466 |
| 78% / 52% | Gen AI in production over a year: early adopters / all | 460 / 3,466 |
| 76% / 63% | Report improved CX: early adopters / all | 460 / 3,466 |
| 55% / 37% | ROI on CX and field service: early adopters / all | 449 / 3,285 |
| 39% / 26% | Mean AI share of total annual IT spend: early adopters / all | 459 / 3,315 |
| 63% | Using gen AI for CX and field service | 3,466 |
| 54% | Using gen AI for digital commerce and enhanced experiences | 3,466 |
| 52% | Gen AI users with AI agents in production | 3,466 |
| 52% | CX/field service gen AI used both internally and externally | 2,190 |
| 49% | Agents used for customer service and experience | 1,814 |
| 83% | Of those reporting improved CX, cite increased user engagement | 2,177 |
| 75% | Of those reporting CX impact, cite improved satisfaction or NPS | 2,177 |
| 51% | Size the CX improvement at 6–10% | 2,133 |
| 70% | Report improved productivity from gen AI | 3,466 |
| 77% | Gen AI spend increased as technology costs fall | 3,466 |
| 58% / 48% | Net new budget / reallocating non-AI budget | 3,466 |
| 78% | With comprehensive C-suite sponsorship, ROI now on >=1 use case | 1,269 |
| 73% | Strong alignment of gen AI adoption to business goals (rating 4 or 5) | 3,466 |

---

## 3. Year-over-year deltas (R1, 2024 n=1,539 → 2025 n=3,466)

| Measure | 2024 | 2025 | Direction |
|---|---|---|---|
| Improved CX from gen AI | 60% | 63% | up 3 |
| Improved productivity | 71% | 70% | down 1 |
| ROI on CX/field service | 34% | 37% | up 3 |
| CX improvement sized 6–10% | 53% | 51% | down 2 |
| Strong C-suite alignment | 69% | 73% | up 4 |
| Net new budget | 61% | 58% | down 3 |
| Reallocating non-AI budget | 44% | 48% | up 4 |
| Operational efficiency objective | 53% | 51% | down 2 |
| Improved CX objective | 52% | 50% | down 2 |
| Employee productivity objective | 52% | 49% | down 3 |
| Competitiveness objective | 50% | 41% | down 9 |
| Greater AI agent deployment objective | not asked | 43% | new |

**Read:** impact measures are roughly flat; the movement is in *how* AI is funded (more cannibalization, less net new) and *which* objective gets attention (agents enter at 43%, competitiveness drops 9 points).

---

## 4. Selection matrices

### Which industry benchmark to cite (R1, agents for customer service and experience)
| Industry | % | n |
|---|---|---|
| Financial services | 57% | 297 |
| Manufacturing | 56% | 292 |
| Public sector | 51% | 138 |
| Retail and CPG | 47% | 300 |
| Telecommunications | 45% | 274 |
| Media and entertainment | 44% | 252 |
| Healthcare and life sciences | 42% | 261 |

Public sector n=138 is the thinnest cell; treat its 51% with the widest interval.

### Which protocol for which problem (R2)
| Problem | Protocol | Notes |
|---|---|---|
| Two agents from different vendors must cooperate | A2A | Open standard; works across developers, frameworks, owners |
| An agent needs live data or must act on a system | MCP | Two-way; named sources Cloud SQL, Spanner, BigQuery, third-party security tools |
| An agent must complete a purchase | AP2 | Requires human pre-approval; authority, hallucination, and fraud accountability are open questions |
| An agent must answer from company truth | Grounding (not a protocol) | Anchor to internal enterprise data |

### Which agent maturity level you are at (R1)
| Level | You have | Next move |
|---|---|---|
| 1 — Simple tasks | Chatbots, information retrieval, image generation | Give an agent a role, objective, and tool access |
| 2 — AI agent applications | Customer service agents, creative agents | Connect agents to each other via A2A |
| 3 — Multi-agent workflows | Agentic workflows, agent orchestration | Instrument the end-to-end process, not each agent |

### Which LLM provider criterion dominates (R1, n=3,466)
| Factor | % |
|---|---|
| Data privacy and security | 37% |
| Integration with existing systems | 28% |
| Cost | 27% |

---

## 5. Investment priority ranking (R1, top-3 selection, n=3,466)

| Rank | Area | % |
|---|---|---|
| 1 | Align business and technology to support change management for user adoption | 42% |
| 2 | Enhance quality of data and knowledge management | 41% |
| 3 | Upskill staff, hire talent, develop outsource partnerships | 40% |
| 4 | Provide the right tooling and compute resources | 37% |
| 5 | Govern and manage risk of AI | 33% |
| 6 | Deploying AI agents | 31% |
| 7 | Reconsider organizational structure and operating models | 29% |
| 8 | Measure AI impact | 28% |

Note the ordering: change management and data quality outrank deploying agents by 10+ points, and measurement is last.

---

## 6. Named customer outcomes — the numbers

### R1
| Organization | Product | Result |
|---|---|---|
| Toolstation | Vertex AI Search for Commerce | 5.5% increase in search-based revenue; 10% lift in click-through rates |
| Commerzbank | Customer Engagement Suite ("Bene") | Over 2 million chats handled; 70% of all inquiries resolved |
| Mercari | Google AI contact center overhaul | Projected 500% ROI; at least 20% reduction in CSR workload (projection, not realized) |
| Mr. Cooper | Google AI in AgentIQ | 3.53% average agent handle time improvement across 500,000 monthly calls = 28,000 hours annually |
| Forrester TEI (vendor-commissioned) | Customer Engagement Suite with Google AI | 207% ROI over three years; payback under six months |

Named but unquantified in R1: Verizon, Best Buy, Bouygues Telecom, loveholidays, Indosat, Bayer Consumer Health, Dasa, Trimble, Deutsche Bank.

### R2
| Organization | Build | Result |
|---|---|---|
| TELUS | Enterprise AI adoption + Google Skills | 57,000+ team members using AI; 40 minutes saved per AI interaction; 96% reported increased confidence; 96% committed to applying the tools; program impact doubled Feb–Sept 2025 |
| Suzano (with Sauter) | Gemini Pro NL-to-SQL agent over SAP Materials data in BigQuery | 95% reduction in query time; 50,000 employees using the data |
| Elanco | Gemini models in Elanco.ai | 2,500+ unstructured policy/procedure documents per manufacturing site processed; avoids up to $1.3M productivity impact at large sites |
| Danfoss (with Go Autonomous) | Email order-processing agents | 80% of transactional decisions automated; response time 42 hours → near real-time; 5 systems → 1 interface |
| Torq | Socrates AI SOC analyst | 90% of tier-1 analyst tasks auto-remediated; 95% decrease in manual tasks; 10x faster response |
| Home Depot | Magic Apron | 24/7 how-to, product recommendations, review summaries (no metrics) |
| Salesforce / PayPal / Specular | A2A / AP2 / Gemini 2.5 Pro security agents | Protocol and platform adoption (no metrics) |

---

## 7. External evidence carried in R2

| Figure | Claim | Source |
|---|---|---|
| 82% | Security professionals concerned or very concerned they are missing real threats due to alert and data volume | Forrester Consulting for Google, *Threat Intelligence Benchmark*, July 2025 |
| 82% | Decision-makers agree technical learning resources help stay ahead in AI | Google/Ipsos Cloud Learning Services Market Pulse, n=902, fielded Sept–Nov 2024 |
| 71% | Organizations report revenue increase since engaging with learning resources | Same |
| 61% / 39% | Employees at AI-implementing organizations using AI daily / at least weekly | Google, *Beyond AI Optimism*, 2025 |
| 84% | Employees wanting greater organizational focus on AI | Same |
| 29% | Employees saying AI is broadly advocated across their organization | Same |
| 4 yrs / 2 yrs | Half-life of a professional skill overall / in tech | Forbes, 2024 |

---

## 8. Process templates

### The ROI AI agent checklist (R1, closing section)
1. Find your executive champions — cultivate C-suite sponsorship to advocate, clear roadblocks, align to results.
2. Demonstrate value to secure AI budget — build the case for AI as its own investment.
3. Start with the biggest wins — agents automating repeatable tasks with clear ROI.
4. Build trust from day one — (a) data governance and enterprise security framework first, (b) always keep a human in the loop.
5. Give agents the tools to be useful — secure, governed access to CRM, Drive, and other internal systems.
6. Create your AI rulebook now, not later — enterprise-wide guidelines for data, IP, compliance.
7. Invest in talent and internal AI education programs.

### The agentic SOC cycle (R2)
Alert → **[AI agents]** detection → triage and investigation → threat hunt and research → malware analysis → response → detection engineering → re-evaluate. **[Human-managed]** recommendation and escalation. Agents share enterprise context and telemetry; coordinate via A2A and MCP; MCP extends workflows to third-party tools.

### The five pillars of AI learning (R2)
01 Establish measurable goals → 02 Secure sponsorship (executive sponsor + groundswell lead + AI accelerator) → 03 Sustain momentum (digital hub, gamified idea exchange, leaderboard, weekly executive emails, quarterly awards) → 04 Integrate into daily workflows (internal hackathons, Field Days) → 05 Prepare for risk (data-use training, AI social engineering awareness).

### The four agent-supervision duties (R2)
Delegate → Set goals → Outline strategy → Verify quality.

---

## 9. Tells and smells

| Symptom | Likely diagnosis | Fix from the reports |
|---|---|---|
| Agents demo well, produce nothing in production | No governed access to enterprise systems | Grant scoped, governed access to CRM, logistics, billing (R1 checklist item 5) |
| Chatbot asks customers to re-identify themselves every time | Not grounded in enterprise context | Ground in CRM and operational data (R2 concierge pattern) |
| Analysts still drowning after SOC automation | Automation without reasoning; SOAR-style scripting | Move to reason-act-observe-adjust agents; keep only recommendation and escalation human (R2 trend 4) |
| Pilots everywhere, ROI nowhere | Effort spread evenly across unequal projects | Concentrate on repeatable-task automation with clear ROI (R1) |
| Executives can't say whether AI worked | ROI never defined; measurement ranked last at 28% | Define ROI including non-financial measures before deployment (Trimble, R1) |
| Employees enthusiastic, usage low | Advocacy gap — 84% want more focus, 29% say it is advocated | Run pillar 03: digital hub, leaderboard, weekly executive comms (R2) |
| AI budget keeps getting cut from elsewhere | Reallocation without a dedicated line (48%, up from 44%) | Build the business case for AI as its own investment (R1 checklist item 2) |
| Agent purchase goes wrong and nobody owns it | Payment authority not modeled | AP2-style framework: proof of authority, request verification, fraud accountability (R2) |
| Security team can't evaluate AI risk | Not bilingual in AI and security | Train for both; apply SAIF 2.0 for autonomous agent risk (R2) |
| Nobody can run the agent fleet | The "agent orchestrator" role does not exist in the market | Train internally; four supervision duties as the job description (R2) |

---

## 10. Citation hygiene

- Both documents are **published by Google Cloud about Google Cloud products**. R1's survey was commissioned by Google Cloud and fielded with National Research Group; the Forrester TEI it cites was commissioned by Google.
- R2 fielded **no new survey**; its quantitative claims are re-citations of R1 plus three other Google-commissioned or Google-published studies and one Forbes article.
- Year-over-year comparisons in R1 span very different sample sizes (1,539 → 3,466).
- Several R1 figures are conditioned on prior positive responses; state the filter when quoting.
- Mercari's 500% ROI is a **projection**. Toolstation, Commerzbank, Mr. Cooper, and the R2 customer figures are stated as realized.
