---
name: dsh-google-cloud-ai-reports
description: "Two Google Cloud vendor research reports — \"The ROI of AI in customer experience\" (Google Cloud + National Research Group, survey of 3,466 senior leaders, fieldwork April 18 – June 3, 2025) and the \"Google Cloud AI Agent Trends 2026\" report — with their statistics, survey bases, question wording, frameworks, and named customer case studies. Use when citing enterprise gen AI or AI agent adoption rates, ROI benchmarks, CX and contact center metrics, agentic AI early adopter comparisons, AI budget and IT spend shares, C-suite sponsorship correlations, agent maturity levels, A2A / MCP / AP2 protocols, agentic SOC design, AI upskilling programs, or customer stories from Toolstation, Commerzbank, Mercari, Mr. Cooper, TELUS, Suzano, Elanco, Danfoss, Torq, and Home Depot."
---

# Google Cloud AI Reports — ROI of AI in CX (2025) and AI Agent Trends 2026

**Publisher:** Google Cloud (report 1 with National Research Group)
**Pages:** ~34 (report 1) + 49 (report 2) = ~83
**Reports:** 2
**Generated:** 2026-08-23

## How to Use This Skill

- **No arguments** — read this file for the load-bearing statistics, frameworks, and the caveats that must travel with them, then open a report file for depth.
- **With a topic** — go straight to [cheatsheet.md](cheatsheet.md) for the number and its survey base; then to the matching report file's Reference Tables for context.
- **With a report** — open [reports/roi-of-ai-in-customer-experience.md](reports/roi-of-ai-in-customer-experience.md) for survey findings and ROI evidence, or [reports/ai-agent-trends-2026.md](reports/ai-agent-trends-2026.md) for the 2026 trend framing, protocols, and operating-model changes.
- **Browse** — [glossary.md](glossary.md) for terminology, [patterns.md](patterns.md) for reusable moves as When / How / Trade-offs.

## Core Frameworks & Mental Models

**The agentic AI early adopter cohort (R1's central analytical device).** A cohort defined by two conjoined criteria: at least 50% of future AI budget allocated to AI agents, *and* agents already deeply embedded across operations (n=460 of 3,466). Every headline comparison in report 1 is this cohort against the total. Because the definition is budget-and-deployment based rather than attitudinal, it functions as a commitment marker — but the relationship to outcomes is correlational, and the 50% line is a survey convention, not a validated threshold.

| Measure | Early adopters | All organizations |
|---|---|---|
| ROI on >=1 gen AI use case | 88% | 74% |
| More than 10 agents in production | 82% | 39% |
| Gen AI in production over a year | 78% | 52% |
| AI share of total annual IT spend | 39% | 26% |
| ROI on CX and field service | 55% | 37% |
| Report improved CX | 76% | 63% |

**Levels of AI agent maturity (R1).** Level 1 simple tasks (chatbots, information retrieval, image generation) → Level 2 AI agent applications (customer service agents, creative agents) → Level 3 multi-agent workflows (agentic workflows, agent orchestration). Use this to place a deployment before proposing a next step.

**The survey's own definition of an AI agent (R1).** "Specialized LLMs that have specific roles, context, and objectives to independently plan, reason, and perform tasks with access to data function call APIs and can interact with other AI agents if needed." Every adoption percentage is measured against this definition — quoting the numbers against a looser "chatbot" definition misrepresents them.

**Five proven areas where AI delivers ROI in CX (R1).** Faster financial returns → happier customers → more productive human agents → improved CX beyond the contact center → data-driven intelligence.

**Human plus machine / super agents (R1).** Agents absorb repetitive work; humans keep empathy and critical-thinking interactions; less experienced staff ramp faster on real-time information, sentiment analysis, and recommendations. The stated goal is converting the contact center from a cost center into a high-value problem-solving hub — which only happens if freed capacity is redeployed rather than removed.

**Instruction-based → intent-based computing (R2).** The 2026 shift: employees stop specifying operations and start stating outcomes, with LLMs and agents determining how to deliver. This is the report's single most load-bearing sentence.

**The employee as human supervisor of agents (R2).** Four responsibilities that constitute the new job description: **delegate** (identify agent-suitable tasks), **set goals** (define the desired outcome), **outline strategy** (apply judgment to nuanced decisions), **verify quality** (final checkpoint for quality, accuracy, tone).

**The digital assembly line and its protocols (R2).** An agentic system is "a human-guided, multi-step workflow that orchestrates multiple agents to run a business process end to end." **A2A** connects agents across vendors and organizations; **MCP** connects agents to live data and tools (Cloud SQL, Spanner, BigQuery, third-party systems), solving frozen training knowledge and inability to act; **AP2** governs agent-initiated payment under human pre-approval, with authority, hallucination, and fraud accountability explicitly unresolved.

**Grounding (R2).** Anchoring model responses to verifiable internal enterprise data. R2's decision rule: "The difference isn't just the AI, it's the data." Consistent with R1's finding that data quality is the second-ranked investment priority (41%).

**The semi-autonomous SOC cycle (R2).** AI agents handle alert detection, triage and investigation, threat hunt and research, malware analysis, response, and detection engineering. Recommendation and escalation stay human-managed. The business case is 82% of security professionals concerned about missing real threats due to alert volume (Forrester Consulting for Google, July 2025).

**Five pillars of AI learning (R2).** 01 establish measurable goals → 02 secure sponsorship (executive sponsor + groundswell lead + AI accelerator) → 03 sustain momentum (digital hub, gamified idea exchange, leaderboard, weekly executive comms, quarterly awards) → 04 integrate into workflows (internal hackathons, Field Days) → 05 prepare for risk (data-use training, AI social engineering).

**Decision rules a practitioner would actually use:**
- Quote the base with the number. Nearly every R1 statistic has a different denominator, and CX-improvement figures are filtered to respondents who already reported improvement.
- Fund agents as a named budget line, not a slice of general IT. Budget commitment is the strongest observed differentiator.
- Secure comprehensive C-level sponsorship (78% report ROI vs. 72% without) but do not expect it alone to move the needle — the gap is 6 points.
- Start with high-volume repeatable tasks where a small percentage gain compounds (Mr. Cooper: 3.53% handle-time improvement across 500,000 monthly calls = 28,000 hours a year).
- Instrument engagement first (83% of improvers cite it) and satisfaction or NPS second (75%).
- Grant governed tool access before expecting agent value; an agent without CRM, logistics, or billing access cannot produce the outcomes described.
- Design the smart handoff, escalation triggers, and summary format before launch.
- Define what ROI means before measuring it — "measure AI impact" ranks last among investment priorities at 28%.

## Report Index

| # | Report | File | Substance |
|---|---|---|---|
| 1 | The ROI of AI in Customer Experience (Google Cloud + National Research Group, 2025) | [reports/roi-of-ai-in-customer-experience.md](reports/roi-of-ai-in-customer-experience.md) | Full survey methodology and base sizes, early adopter comparisons, five ROI areas, industry breakdowns, spend and sponsorship data, the ROI AI agent checklist, four quantified customer cases |
| 2 | Google Cloud AI Agent Trends 2026 | [reports/ai-agent-trends-2026.md](reports/ai-agent-trends-2026.md) | Five 2026 trends, intent-based computing, agent supervision duties, A2A / MCP / AP2, agentic SOC cycle, five learning pillars, three worked scenarios, nine customer cases |

## Topic Index

| Topic | Report |
|---|---|
| Survey methodology, sample composition, base sizes | 1 |
| Agentic AI early adopter definition and comparisons | 1 (re-cited in 2) |
| Agent adoption rates and use-case mix | 1, 2 |
| AI agent maturity levels | 1 |
| CX and contact center ROI, engagement, NPS | 1 |
| Industry-by-industry agent adoption | 1 |
| AI budget, IT spend share, funding sources | 1 |
| C-suite sponsorship and ROI correlation | 1 |
| LLM provider selection criteria | 1 |
| Business objectives, 2024 vs. 2025 | 1 |
| Human plus machine / super agents | 1 |
| Intent-based computing, agent supervision | 2 |
| A2A, MCP, AP2 protocols | 2 |
| Grounding in enterprise data | 2 |
| Agentic concierge and proactive service | 2 |
| Agentic SOC, alert fatigue, SAIF 2.0, CodeMender | 2 |
| Upskilling, learning pillars, sponsorship roles | 2 |
| Agentic ecommerce and payments | 2 |
| Customer case studies with numbers | 1, 2 |

## Supporting Files

- [glossary.md](glossary.md) — terminology from both reports, tagged R1 or R2, including the exact survey definition of an AI agent.
- [patterns.md](patterns.md) — reusable moves as When / How / Trade-offs, grouped into business case, deployment, workflow and protocol, and adoption and enablement.
- [cheatsheet.md](cheatsheet.md) — all base sizes, headline numbers, year-over-year deltas, selection matrices, customer outcome tables, process templates, tells and smells, and citation hygiene.

## Scope & Limits

**These are vendor marketing research documents, not independent research.** Both are published by Google Cloud, and every product they evaluate favourably is a Google Cloud product (Customer Engagement Suite, Vertex AI, Vertex AI Search for Commerce, Gemini Enterprise, Gemini models, Google Security Operations, BigQuery, Cloud SQL, Spanner, Google Skills). Report 1's survey was commissioned by Google Cloud and fielded with National Research Group. The Forrester Total Economic Impact study it cites (207% three-year ROI, payback under six months) was commissioned by Google. Report 2's Ipsos and Forrester citations are likewise Google-commissioned. Treat every figure as sponsored research: useful for the direction and rough magnitude of enterprise sentiment, not as neutral benchmarking.

**What the numbers can and cannot support.** Report 1's statistics are self-reported executive perceptions, not audited outcomes. Many are conditioned on a prior positive response — the CX-improvement magnitude figure (51% at 6–10%) is asked only of respondents who already said CX improved, so it answers "how much" and never "how often." Year-over-year comparisons span 1,539 (2024) and 3,466 (2025) respondents. Early adopter comparisons are correlational; the cohort is defined by budget allocation, and organizations already succeeding with agents plausibly fund them more.

**Report 2 fielded no new primary survey.** It states its method as "a blend of qualitative and quantitative data, including internal Google Cloud and Google DeepMind interviews with AI leaders, customer case studies, and insights from The ROI of AI 2025 report... with analysis using NotebookLM and Google AI Studio." Its 52%, 49%, 46%, 45%, 43%, and 88% figures are re-citations of report 1. Most of its qualitative authority comes from Google Cloud's own executives rather than independent analysts.

**Dating.** Report 1's fieldwork ran April 18 – June 3, 2025; its forward-looking questions ask about the next 2–3 years. Report 2 is a 2026 forecast published in late 2025, and its protocol material (A2A, MCP, AP2) describes standards that were early at time of writing. Product names, protocol maturity, and the "agent orchestrator role does not exist yet" claim will all date quickly.

**Coverage of these files.** Both reports were retrieved in full and the extractions are clean and complete. Where a figure's assignment had to be read from chart layout rather than body text — the without-sponsorship bars in report 1's C-suite chart, and the 42% top-ranked investment area — the report file says so explicitly. All percentages, base sizes, question wording, and customer numbers come directly from the source text; none are inferred or estimated. Statistics that are projections rather than realized results (Mercari's 500% ROI) are labelled as such.

**A third report in the same Drive folder could not be retrieved.** `25952_grdg_enterprise_technical_guide_ebook_final.pdf` (11.5 MB) exceeds the connector's 10 MB download limit and was skipped. Nothing in this skill draws on it, and its content is unknown.

**Not covered.** Neither report contains implementation code, architecture diagrams, pricing, product configuration guidance, or independent competitive comparison. Report 1's regional cells outside the US, Brazil, and Mexico are small (as low as n=3 for Kuwait, reported only within a MEA aggregate) and no country-level cross-tabs are published. Report 1 gives no breakdown of results by company size or revenue band.
