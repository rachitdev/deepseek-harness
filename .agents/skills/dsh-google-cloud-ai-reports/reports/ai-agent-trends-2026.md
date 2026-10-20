# Report 2 — Google Cloud AI Agent Trends 2026

*Google Cloud. Subtitle: "Five shifts that will redefine roles, workflows, and business value in 2026." Published as an interactive report, ~49 pages. Every page carries an "Ask the report" link to a NotebookLM version.*

## Core Idea

The decisive near-term shift is not AGI but **agentic AI**: systems that move beyond answering questions to "understanding a goal, making a plan, and taking actions across applications to achieve it with extensive human guidance and oversight." The report organizes 2026 around five shifts — agents for every employee, every workflow, your customers, security, and scale — and argues the binding constraint is human: culture, orchestration skill, and upskilling, not model capability.

Its framing sentence for the operating model change: employees move from **instruction-based computing** ("analyze this spreadsheet," "write this code") to **intent-based computing**, where the employee states a desired outcome and the LLM-plus-agent system determines how to deliver it.

## Survey Methodology

This is not a survey report. It is a trends report assembled from mixed sources, and it should be cited accordingly.

- **Method as stated:** "a blend of qualitative and quantitative data, including internal Google Cloud and Google DeepMind interviews with AI leaders, customer case studies, and insights from The ROI of AI 2025 report (based on a global survey of 3,466 enterprise decision makers), with analysis using NotebookLM and Google AI Studio."
- **No new primary survey was fielded for this report.** Its quantitative backbone is re-cited from the ROI of AI 2025 survey (see the companion report file).
- **Named external sources it borrows from:**
  - Google Cloud, *The ROI of AI, 2025* — n=3,466 total; n=1,814 for agent use-case questions; n=460 for early adopters.
  - Forrester Consulting on behalf of Google, *Threat Intelligence Benchmark: Stop Reacting; Start Anticipating*, July 2025 — the 82% alert-fatigue figure.
  - Google/Ipsos, *Cloud Learning Services Market Pulse*, fielded September–November 2024 — decision-makers n=902, across US, UK, France, Germany, India, Brazil, Mexico, Japan, Australia/New Zealand. The report itself notes: "Findings reflect the opinion of survey respondents only."
  - Google, *Beyond AI Optimism: Five ways to move your business from saving time to sparking innovation*, 2025 — the employee-side figures.
  - Forbes, "AI Puts The Squeeze On The Shrinking Half-life Of Skills," 2024 — the skill half-life claim.
- **Everything else is executive quotation or customer anecdote.** Most quotes are from Google Cloud's own leadership (Global Strategic Industries directors, product VPs), not from independent analysts.

## Frameworks Introduced

### 1. The five trends
| # | Trend | One-line thesis |
|---|---|---|
| 1 | Agents for every employee | Empowering individuals to achieve peak productivity |
| 2 | Agents for every workflow | Running your business with grounded agentic systems |
| 3 | Agents for your customers | Delighting customers with concierge-like experiences |
| 4 | Agents for security | Advancing security from alerts to action |
| 5 | Agents for scale | Upskilling talent will be the ultimate driver of business value |

### 2. The employee as human supervisor of agents
Every employee, "from an entry-level analyst to a senior vice president," becomes a supervisor of agents. Four named responsibilities:
- **Delegate** mundane or repetitive tasks — identify which tasks suit an agent and assign them.
- **Set goals** — clearly define the desired outcome for the agent.
- **Outline strategy** — use human judgment to guide agents and make the final, nuanced decisions AI cannot.
- **Verify quality** — act as the final checkpoint for quality, accuracy, and tone.

### 3. Grounding
"The process of anchoring an AI model's responses to a specific, verifiable set of facts — its 'ground truth.' For an enterprise, this ground truth is its own internal data." Grounding is the report's stated reason the agentic concierge works: "The difference isn't just the AI, it's the data."

### 4. The digital assembly line
An **agentic system** is defined as "a digital assembly line — a human-guided, multi-step workflow that orchestrates multiple agents to run a business process end to end." Its enabling protocols:

| Protocol | What it solves |
|---|---|
| **A2A (Agent2Agent)** | Open standard for integration and orchestration *between agents*, across different developers, frameworks, and owning organizations |
| **MCP (Model Context Protocol)** | Standardized two-way connection between LLM applications and data sources or tools (Cloud SQL, Spanner, BigQuery). Solves the two named LLM limitations: knowledge frozen at training time, and inability to act on the outside world |
| **AP2 (Agent Payments Protocol)** | Framework for an agent initiating a payment under human pre-approval; addresses proof of user-granted authority, merchant confidence that a request is not a hallucination, and fraud accountability |

### 5. The semi-autonomous security operations cycle
An agentic SOC orchestrates task-based agents around a received alert:
- **AI agent stages:** Alert detection → Triage and investigation → Threat hunt and research → Malware analysis → Response → Detection engineering, then re-cycling.
- **Human-managed stages:** Recommendation and Escalation.
- Agents share common enterprise context and security telemetry, communicate and adapt through A2A and MCP, and should be trained on continuously evolving real-world insights from security experts. MCP support extends custom agentic workflows across the whole security stack, including third-party tools.

### 6. The five pillars of AI learning
| Pillar | Content |
|---|---|
| 01 Establish goals | Pick what matters and what is measurable (example: 100% AI tool adoption, every team member using a Gemini Enterprise agent somewhere in their workflow). Align to organizational need; make it measurable |
| 02 Secure sponsorship | Three primary stakeholders — see below |
| 03 Sustain momentum and reward innovation | A "digital hub" with a gamified idea exchange and leaderboard; peer-to-peer knowledge sharing; weekly executive emails, all-hands content, quarterly awards |
| 04 Integrate AI into daily workflows | Internal hackathons where teams develop and pitch AI solutions (winners integrated into the official program); Field Days — structured, challenge-based practice events, in person or virtual |
| 05 Prepare for increasing risks with trusted frameworks | Security becomes everyone's responsibility: train on what data can and cannot be used in AI tools, and on AI-enabled social engineering |

### 7. The three-role sponsorship team (inside pillar 02)
- **Executive sponsor** — provides funding, high-level backing, and consistent messaging on AI's importance.
- **Groundswell lead** — the "AI megaphone": grassroots campaigns, excitement, collecting employee ideas.
- **AI accelerator** — the technical expert who turns prioritized ideas into functional solutions.

The stated logic: "the support to be funded, the momentum to engage employees, and the technical capability to deliver tangible AI tools."

## Key Concepts

**Agent adoption baseline.** 52% of executives in gen AI-using organizations have AI agents in production (ROI of AI 2025, n=3,466). Of those with agents in production (n=1,814): 49% customer service, 46% marketing or security operations, 45% tech support, 43% product innovation or productivity and research.

**Chatbot vs. agentic concierge.** The report's contrast, given as literal dialogue. Chatbot: "Please enter your 12-digit order number." Agentic concierge: "Hi, Elizaveta. I see you're calling about the blue sweater you bought last week. Our system shows it was just delivered. Are you calling to start a return or an exchange?" The difference is grounding in enterprise context the business chooses to share — CRM purchase history, logistics package tracking.

**Alert fatigue as the attacker's advantage.** 82% of security professionals are concerned or very concerned they may be missing real threats due to alert and data volume (Forrester Consulting for Google, July 2025). The report's framing: "they only need to be right once, while the defender has to be right every single time." SOAR delivers "only incremental benefits" because it cannot reason, act, observe, and adjust.

**Security tooling named.** Secure AI Framework (SAIF) 2.0, expanded to address risks from autonomous agents; DeepMind's CodeMender, an AI agent that improves code security automatically and has already found new zero-day vulnerabilities in well-tested software.

**The skills half-life claim.** A professional skill's half-life is now four years, and as short as two years in tech (Forbes, 2024). The report pairs this with the observation that the "agent orchestrator" or "Chief of Staff for AI" role "simply doesn't exist in the market yet."

**The role shift is not autonomy.** Repeated explicitly: "There is a common misconception that agents act without control. Humans will remain the orchestrators and final decision-makers." The security cycle keeps recommendation and escalation human-managed; the customer concierge performs a "smart handoff" with a full summary for complex or emotionally charged issues.

## Mental Models

- **Instruction-based → intent-based computing.** The single sentence that carries the most weight in the report. State the outcome; the system plans the path.
- **Digital assembly line.** A workflow is not a chain of tools but a chain of agents, each with a role, coordinated by protocols and supervised by a human.
- **Grounding as the moat.** Model quality is increasingly commodity; the differentiator is the enterprise context you connect to it.
- **Agents as force multipliers for analysts, not replacements.** The security analyst moves from alert-watching to threat hunting ("Agent, I have a hunch about this server. Hunt for any unusual outbound data transfers."), supervising agents (tuning rules of engagement, performance-reviewing automated responses), and defending (long-term posture and architecture).
- **The gap is human, not technical.** "The 2026 opportunity can seem technical, but it is fundamentally human."
- **Value accrues during the build, not only at the end.** "The true value is not just in the final product, but also in the innovation and optimization that happens during the process. The companies that experiment today are not just building tools; they are building the critical, in-house expertise to manage, govern, and scale this new capability."

## Anti-patterns

- **Buying tools without changing the operating model.** "Unlocking the value of these trends requires more than simply adopting new tools. It also demands that leaders question old assumptions and drive the cultural change necessary to thrive."
- **Treating agents as autonomous.** Explicitly called a "common misconception."
- **Relying on SOAR alone.** Automation without reasoning "may offer only incremental benefits."
- **Scripted call deflection.** The pre-programmed chatbot that makes callers repeat "operator!" is the named failure mode agentic CX replaces.
- **Assuming payment infrastructure works for agents.** "Today's payment systems assume a human is directly initiating the purchase." Building agentic commerce on that assumption leaves authority, hallucination, and fraud accountability unanswered.
- **Focusing on models, platforms, and prompts while ignoring people.** "It is tempting to focus on the technology... but this misses the most critical element: the people."
- **Treating AI adoption as a launch rather than a program.** TELUS: "AI adoption should be an ongoing story, instead of just one chapter in a book."

## Reference Tables

### Agent use cases in production
Base: executives whose organization is leveraging agentic AI, n=1,814 (ROI of AI 2025).

| Use case | % |
|---|---|
| Customer service | 49% |
| Marketing | 46% |
| Security operations / cybersecurity | 46% |
| Tech support | 45% |
| Product innovation | 43% |
| Productivity and research | 43% |

### Learning and skills evidence

| Figure | Statement | Source and base |
|---|---|---|
| 82% | of decision-makers agree technical learning resources help their organization stay ahead in AI | Google/Ipsos Cloud Learning Services Market Pulse, n=902, fielded Sept–Nov 2024 |
| 71% | of organizations surveyed realize an increase in revenue since engaging with learning resources | Same |
| 61% | of employees at organizations that have already implemented AI use AI daily (remaining 39% at least weekly) | Google, Beyond AI Optimism, 2025 |
| 84% | would like a greater organizational focus on AI | Same |
| 29% | say that AI is broadly advocated across their organizations | Same |
| 4 years / 2 years | half-life of a professional skill overall / in tech | Forbes, 2024 |

The 84%-want-more versus 29%-broadly-advocated gap is the report's implicit argument for pillar 03.

### Named customer outcomes

| Organization | What was built | Measured result |
|---|---|---|
| TELUS | Enterprise-wide AI adoption | Over 57,000 team members regularly use AI, saving 40 minutes per AI interaction; 96% of team members reported increased confidence using AI tools; 96% committed to applying them; Google Skills training program doubled in impact from February to September 2025 |
| Suzano (with Sauter) | Gemini Pro agent translating natural language to SQL over SAP Materials data in BigQuery | 95% reduction in time required for queries, across 50,000 employees using the data |
| Elanco | Gemini models in the Elanco.ai platform sorting, extracting, comparing, and restructuring over 2,500 unstructured policy and procedure documents per manufacturing site | Improved accuracy and consistency; avoids outdated or conflicting information that could cost up to $1.3 million in productivity impact at large sites |
| Danfoss (with Go Autonomous) | Email-based order processing automation on Google Cloud | 80% of transactional decisions automated; average customer response time cut from 42 hours to near real-time; five systems consolidated into a single interface |
| Torq | Socrates, an AI SOC analyst coordinating specialized agents on Google Cloud | 90% of tier-1 analyst tasks auto-remediated without human involvement; 95% decrease in manual tasks; 10x faster response times |
| Home Depot | Magic Apron, a 24/7 expert-guidance agent | How-to instructions, product recommendations, review summaries (no metrics given) |
| Salesforce | Cross-platform agents using A2A | Interoperability milestone (no metrics given) |
| PayPal | Agentic shopping and commerce via AP2 | Protocol adoption (no metrics given) |
| Specular | Gemini 2.5 Pro agents automating attack surface management and penetration testing | Prioritization and response speed (no metrics given) |

Danfoss operates in over 100 countries; Suzano is the world's largest pulp manufacturer.

## Worked Example

The report gives three fully specified scenarios. Reproduce them when the question is "what does this actually look like."

### A. The 10x marketing manager — a five-agent system
The old job was "a constant scramble of drafting posts, pulling data, and watching competitors." The new job is orchestrating five specialized agents so the manager can focus on brand storytelling and strategic campaign development:

1. **Data agent** — sifts through millions of structured and unstructured data points to find actionable patterns in market trends.
2. **Analyst agent** — monitors market trends, competitor announcements, and social sentiment 24/7; delivers a one-page report of key insights to the manager's inbox every morning.
3. **Content agent** — given a strategic theme for the week, drafts copy for several social posts and a blog article in the company's brand voice, then shares it for review.
4. **Creative agent** — given the marketing strategy, creative guidelines, and social copy, generates images and videos for the posts and shares them back.
5. **Reporting agent** — connects to analytics platforms, pulls and analyzes weekly campaign data, delivers a one-page summary of key insights every Friday.

Note the shape: two agents produce scheduled artifacts (daily inbox report, Friday summary), two produce drafts for human review, one does open-ended pattern finding. Every output routes back to the human for the verify-quality step.

### B. Agentic ecommerce with AP2 — the conditional purchase
A customer finds a winter jacket unavailable in the color they want. They instruct their agent: **"Purchase this jacket when it becomes available in black. Don't purchase it if the price is more than $100."** The agent monitors price and availability and, with human pre-approval, executes a secure purchase the moment that variant appears — "capturing a high-intent sale that would have otherwise been lost."

The three unresolved questions the report says AP2 exists to answer: how to prove user-given authority for the purchase; how a merchant can be sure an agent's request is accurate and not a hallucination; who is accountable in case of fraud.

### C. Proactive customer service — the failed delivery
A logistics agent flags a delivery as "failed" at 3 PM. Rather than waiting for an angry call, the concierge agent:

1. Checks the backend and confirms the delivery van broke down.
2. Accesses the logistics system to reschedule the delivery for the first available slot tomorrow morning.
3. Logs in to the billing system and applies a $10 service credit for the inconvenience.
4. Notifies the customer by text: "Hi, we're so sorry. Your package was delayed due to a vehicle issue. We've added a $10 credit to your account for the trouble. I'm happy to reschedule it for tomorrow between 9–11 AM. Please reply 'Yes' to confirm the new time or respond with your preferred time."

The design detail that makes this safe: the agent is built to execute a **smart handoff** with a full summary for complex or emotionally charged issues, so human staff can step in. Note also what the scenario assumes — write access to the logistics system and the billing system, and a pre-set credit authority. Those authorizations, not the language model, are what make the sequence possible.

## Key Takeaways

1. The quantitative spine of this report is borrowed from the ROI of AI 2025 survey; there is no new primary survey here.
2. Intent-based computing, not autonomy, is the actual claim. Humans stay as orchestrators and final decision-makers in every scenario the report presents.
3. Four named employee responsibilities — delegate, set goals, outline strategy, verify quality — are the concrete job description for "agent supervisor."
4. Three protocols carry the interoperability story: A2A between agents, MCP between agents and data or tools, AP2 for agent-initiated payments.
5. Grounding in proprietary enterprise data is positioned as the differentiator, not model choice.
6. Alert fatigue (82% concerned about missed threats) is the security business case; the agentic SOC keeps recommendation and escalation human-managed.
7. The strongest customer metrics are Torq (90% tier-1 auto-remediation, 95% fewer manual tasks, 10x faster response), Suzano (95% query-time reduction across 50,000 employees), and Danfoss (80% of transactional decisions automated, 42 hours to near real-time).
8. The skills gap is the report's named bottleneck, and the "agent orchestrator" role does not yet exist in the labor market.
9. The 84% wanting more organizational AI focus against 29% saying it is broadly advocated is the clearest evidence gap in the report.
10. Every product mentioned is a Google Cloud product — Gemini Enterprise, Vertex AI, Customer Engagement Suite, Google Security Operations, BigQuery, Cloud SQL, Spanner, Google Skills.

## Connects To

- [The ROI of AI in Customer Experience](roi-of-ai-in-customer-experience.md) — the source survey for this report's 52%, 49%, 46%, 45%, 43%, and 88% figures, with full base sizes and question wording.
- [../glossary.md](../glossary.md) — A2A, MCP, AP2, grounding, agentic SOC, SAIF, CodeMender, Gemini Enterprise.
- [../patterns.md](../patterns.md) — the five learning pillars, the three-role sponsorship team, and the agent-supervisor responsibilities as reusable moves.
- [../cheatsheet.md](../cheatsheet.md) — protocol selection, the SOC cycle, and all customer numbers in one place.
