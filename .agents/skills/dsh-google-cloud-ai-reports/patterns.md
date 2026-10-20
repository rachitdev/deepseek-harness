# Patterns — Google Cloud AI Reports

Reusable moves drawn from the two reports, each as **When / How / Trade-offs**. Source key: **R1** = The ROI of AI in Customer Experience (2025 survey); **R2** = AI Agent Trends 2026.

---

## Part 1 — Building the business case (R1)

### Fund agents as a dedicated budget line
**When** you are trying to move from pilots to consistent ROI and cannot explain why some peers report returns and you do not.
**How** Allocate a named share of future AI budget to agents specifically. R1's early adopter cohort is defined by at least 50% of future AI budget going to agents, and that cohort reports 88% ROI on at least one gen AI use case versus 74% overall. Early adopters also run AI at 39% of total annual IT spend versus a 26% mean.
**Trade-offs** The 50% threshold is a definitional line drawn by the survey, not a validated optimum, and the correlation is not causal — organizations already succeeding with agents plausibly fund them more. Treat it as a marker of commitment level, not a target to hit mechanically.

### Secure comprehensive C-level sponsorship before scaling
**When** an AI program is technically working but stalling on budget, roadblocks, or cross-functional alignment.
**How** Aim for what R1 rates a 5: "Comprehensive C-level sponsorship and clear corporate vision for gen AI objectives." 78% of organizations at that level report ROI now on at least one gen AI use case, versus 72% without. Strong alignment (rating 4 or 5) rose from 69% to 73% year over year. Give the sponsor three jobs: advocate for initiatives, clear roadblocks, align to results.
**Trade-offs** The measured gap is 6 points, not transformational on its own; sponsorship is necessary but clearly insufficient. It is also the easiest item to claim and the hardest to verify.

### Start with repeatable-task automation, not the hardest problem
**When** choosing among competing AI proposals.
**How** R1: "Not all AI projects are created equal. Focus your energy on building AI agents that can automate repeatable tasks to deliver clear ROI." Prefer high-volume, low-variance work where a small percentage improvement compounds — the Mr. Cooper shape (3.53% AHT improvement across 500,000 monthly calls = 28,000 hours a year).
**Trade-offs** Repeatable-task automation produces efficiency savings that are easy to measure but capped; it will not produce the revenue-side stories (Toolstation's 5.5% search revenue lift) that sustain executive interest over multiple budget cycles. Run one of each.

### Define what ROI means before measuring it
**When** stakeholders disagree about whether a deployment succeeded.
**How** Trimble's framing in R1: "Leaders need to first decide what ROI means. It goes beyond financial returns. We have to ask if it's making people more efficient and building towards business objectives — really clearly define what we're trying to achieve." Note that "Measure AI impact" ranks last (28%) among R1's investment priorities — most organizations are not doing this.
**Trade-offs** Broadening ROI beyond cash makes the program defensible but also unfalsifiable if the non-financial measures are not instrumented as rigorously as the financial ones.

### Use engagement as the leading indicator, satisfaction as the lagging one
**When** you need early evidence that a CX agent is working.
**How** Of executives reporting improved CX (n=2,177), 83% cite increased user engagement — engagement score, traffic, CTR, time on site — while 75% cite improved user satisfaction or NPS. Instrument the engagement metrics first; they move earlier and are cheaper to capture.
**Trade-offs** Engagement can rise because an interface is confusing as easily as because it is good. Pair with a resolution or completion metric.

---

## Part 2 — Deployment patterns (R1 and R2)

### The human-plus-machine super agent
**When** deploying agents into a contact center or service operation.
**How** R1's named strategy: agents handle repetitive, mundane tasks; human representatives keep complex interactions requiring empathy and critical thinking. Give less experienced staff real-time information, sentiment analysis, and personalized recommendations so they ramp faster. Position the outcome as converting a cost center into a "high-value, problem-solving hub."
**Trade-offs** The efficiency case and the quality case pull in different directions — if freed capacity is simply removed as headcount, the "high-value hub" outcome does not materialize. Decide up front which one you are buying.

### Grant governed tool access before expecting agent value
**When** an agent demos well but produces nothing useful in production.
**How** R1: "For an AI agent to do the work, it needs access to your internal enterprise systems, like your CRM or Drive. Grant it secure, governed access." R2's proactive-service scenario is only possible because the agent holds write access to the logistics system and the billing system, plus a pre-set credit authority.
**Trade-offs** Every grant expands the blast radius. R2's own security trend notes AI infrastructure is "dramatically expanding an enterprise's attack surface area." Scope access per agent, per system, per action.

### Ground agents in enterprise context
**When** deciding whether the differentiator is the model or the data.
**How** R2: "The difference isn't just the AI, it's the data." Anchor responses to internal ground truth — CRM purchase history, logistics tracking, knowledge bases, past work. The chatbot asks for a 12-digit order number; the grounded concierge already knows the customer, the item, and the delivery status.
**Trade-offs** Grounding quality is bounded by data quality, which is why "enhance quality of data and knowledge management" is R1's second-ranked investment priority at 41%.

### Design the smart handoff as a first-class feature
**When** an agent will handle emotionally charged or high-stakes interactions.
**How** R2: the agent executes a handoff with a full summary for complex or emotionally charged issues so human staff can step in mid-case. Build the summary format and the trigger conditions before launch, not after the first escalation failure.
**Trade-offs** Aggressive handoff thresholds erode the automation rate you are measuring; permissive ones produce the failure the pattern exists to prevent.

### Orchestrate a role-differentiated agent team, not one general agent
**When** a single job function has several distinct recurring tasks.
**How** R2's marketing manager runs five agents with distinct contracts: data (pattern finding), analyst (daily one-page report), content (drafts in brand voice), creative (images and video from strategy and copy), reporting (Friday campaign summary). Note the mix: two produce scheduled artifacts, two produce drafts for review, one does open-ended discovery.
**Trade-offs** Multiple specialized agents multiply the surface you must supervise and keep grounded. The four supervision duties below become a real time cost.

### Apply the four agent-supervision duties
**When** defining what an employee's job becomes once agents do the tasks.
**How** R2's four responsibilities: **delegate** (identify which tasks suit an agent), **set goals** (define the desired outcome), **outline strategy** (apply human judgment for nuanced decisions), **verify quality** (final checkpoint for quality, accuracy, tone).
**Trade-offs** This is a genuinely different skill set, and R2 concedes the market for it does not yet exist — you will be training, not hiring.

---

## Part 3 — Workflow and protocol patterns (R2)

### Build the digital assembly line on open protocols
**When** a process crosses systems, teams, or organizational boundaries.
**How** A2A for agent-to-agent orchestration across different developers, frameworks, and owners; MCP for agent-to-data-and-tools connection (Cloud SQL, Spanner, BigQuery, third-party tools); AP2 for agent-initiated payment under human pre-approval. R2's telecom example chains network anomaly remediation, field service ticketing, and contact center notification into one sequence.
**Trade-offs** Open protocols are early. AP2 in particular leaves the three hardest questions open by R2's own admission: proving user-granted authority, merchant confidence that a request is not a hallucination, and fraud accountability.

### Bound agent authority with explicit conditions
**When** an agent will take an irreversible action such as a purchase.
**How** R2's pattern is a conditional standing instruction: "Purchase this jacket when it becomes available in black. Don't purchase it if the price is more than $100." Authority is scoped by object, trigger condition, and ceiling.
**Trade-offs** Conditions cover the anticipated case. Anything outside them either fails silently or requires a fallback path back to the human, which is the part the scenario does not specify.

### Keep recommendation and escalation human-managed
**When** designing an agentic security or risk workflow.
**How** R2's SOC cycle assigns alert detection, triage and investigation, threat hunt and research, malware analysis, response, and detection engineering to AI agents, while recommendation and escalation stay human-managed. Agents share common enterprise context and telemetry and coordinate via A2A and MCP.
**Trade-offs** The human stages are the throughput bottleneck the automation was meant to remove; if analyst capacity does not shift toward those stages, the queue simply moves.

### Promote analysts from watchers to hunters
**When** agents absorb tier-1 work.
**How** R2's three replacement activities: threat hunting driven by human intuition ("Agent, I have a hunch about this server. Hunt for any unusual outbound data transfers."), supervising agents (tuning rules of engagement, performance-reviewing automated responses), and defending (long-term posture, architecture, anticipating the next attack wave).
**Trade-offs** Requires people who can do both jobs; R2's own line is that security professionals "must be deeply bilingual in both AI and security."

---

## Part 4 — Adoption and enablement patterns (R2, pillar model)

### Run the five pillars of AI learning as a program
**When** tooling is deployed but usage is thin.
**How** (01) Establish measurable goals aligned to organizational need — e.g. every team member using an agent somewhere in their workflow. (02) Secure sponsorship with three named stakeholders. (03) Sustain momentum: a digital hub with a gamified idea exchange and leaderboard, peer-to-peer knowledge sharing, weekly executive emails, all-hands content, quarterly awards. (04) Integrate into daily workflows via internal hackathons (winning concepts adopted officially) and Field Days — structured, challenge-based practice events. (05) Prepare for risk: train on what data can and cannot go into AI tools and on AI-enabled social engineering.
**Trade-offs** This is a sustained communications and events program with real headcount cost, justified in R2 by correlational evidence (71% of organizations engaging with learning resources report a revenue increase) from a Google-commissioned Ipsos study of 902 decision-makers.

### Staff the three-role sponsorship team
**When** an AI initiative has funding but no traction, or traction but no funding.
**How** **Executive sponsor** — funding, high-level backing, consistent messaging. **Groundswell lead** — the "AI megaphone": grassroots campaigns, excitement, collecting employee ideas. **AI accelerator** — the technical expert turning prioritized ideas into functional solutions. R2's logic: "the support to be funded, the momentum to engage employees, and the technical capability to deliver."
**Trade-offs** Three named people is the minimum viable team, not a program office; at scale the groundswell lead role becomes a full-time internal marketing job.

### Test customer-facing agents on employees first
**When** rolling out an agent that will speak to customers.
**How** R2's financial services example: a bank rolls its customer support agent out internally first, so employees ask real questions, test it, understand the customer experience, and drive improvements before external launch.
**Trade-offs** Employees are not representative customers — they know the product, the vocabulary, and the workarounds, so they will under-detect confusion failures.

### Close the advocacy gap
**When** employee enthusiasm outruns organizational signal.
**How** R2's evidence: 84% of employees would like a greater organizational focus on AI while only 29% say AI is broadly advocated across their organization, and 61% at AI-implementing organizations already use AI daily. The gap is a communications problem more than an adoption problem — pillar 03 exists to close it.
**Trade-offs** Both figures come from a Google-published employee study; the sample and question wording are not disclosed in the trends report itself.
