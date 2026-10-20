---
name: dsh-think-like-a-cto
description: "Think Like a CTO by Alan Williamson (Manning, 2023) — a practitioner's field guide to the Chief Technology Officer role, covering vision, team, technology, and transactions. Use when working on CTO or VP Engineering responsibilities: the first 100 days, managing up to a CEO, CFO or board, building a technology vision and budget, hiring and job descriptions, skill and discipline matrices, annual reviews and terminations, build vs. buy, cloud vs. on-prem, disaster recovery, data management, microservices, open source licensing, project planning, CI/CD, technical debt, release strategy, SOWs and SLAs, documentation, security and breach response, monitoring, backups, due diligence, and CTO succession."
---

# Think Like a CTO

**Author**: Alan Williamson · **Publisher**: Manning Publications, 2023 · **ISBN** 9781617298851
**Pages**: 320 · **Chapters**: 15 · **Foreword**: Ankit Mathur
**Generated**: 2026-08-23

## How to Use This Skill

- **No arguments** — read this page for the core frameworks, then open the chapter index and pick the
  responsibility you are working on.
- **With a topic** ("build vs buy", "skill matrix", "due diligence", "SLA") — check the Topic Index
  below, then read that chapter's `## Reference Tables` and `## Worked Example` first; those hold the
  book's own templates and numbers.
- **With a chapter** — read `chapters/chNN-slug.md` end to end; each chapter carries the book's actual
  tables, checklists, and case studies, plus its printed chapter checklist.
- **Browse** — `cheatsheet.md` for decision tables and thresholds, `patterns.md` for the reusable moves
  in When/How/Trade-offs form, `glossary.md` for the book's vocabulary.

## Core Frameworks & Mental Models

**The operating assumption.** Williamson's CTO owns **both technology vision and execution**. The CEO
and board own the *what*; the CTO owns the *how*. Scope line: the CTO owns the **front office** — the
servers and software that make up the client-facing product — not the **back office** (desktops, mail,
phones, printers), which sits with IT under the CFO. In the preface's chess image: CEO is king, CFO is
queen, **CTO is the rook** — the piece that supports the others and clears paths into the distance.

**Four CTO archetypes**, each with a yes/no litmus test: prestartup in name only; funded startup
(technology expert with money); established company hiring its first CTO; established company with a
CTO already. Know which one you are in, because it determines what "good" looks like.

**The first 100 days.** 0–7 the company from the client's perspective; 7–30 listen and observe ("going
to lose for an hour" — watch the tells); 30–70 lift the lid on code, data, and infrastructure and note
easy wins; 70–100 begin execution on what keeps the lights on. Not on making changes.

**The grand vision.** A technology vision statement supporting the company's, stated over **two to five
years**. If it can be done in a few months it's a project, not a vision. Split it into *what/why* for the
business and *how* for your group, break it into bite-size deliverables ("how do you eat an elephant?"),
name it, and identify **pillars** — commodity foundations you can watch commoditize and swap without
resetting the clock. Test it against your own team: "Do they believe in your vision?"

**Communicate for reuse, not for accuracy.** The CEO is an **office**, not a person — everything you
say becomes the sound bite they relay upward. Use metaphors ("they put the wrong number on their
mailbox" beats DNS TTL). **Lead with data**; **think slow, act fast**; **build for input/change** (a
yes/no proposal has already halved your odds); and on bad news, own it, no blame, no names — if it
affects clients or revenue, the CEO hears it immediately.

**Money is a CTO discipline.** Your department is a cost center. Know your **burn rate** in three
buckets — Development / Production / Support — cost staff at **120–130% of salary**, and derive a daily
rate so future estimates are arithmetic, not guesswork. Build three budget models from **fast/good/cheap**.
Review monthly against invoices; review with the CFO quarterly. The secret to the CFO relationship is
**no surprises**.

**People frameworks that compound.** The **discipline matrix** (0–10 per skill per person, brackets on
single-holder skills, Need and Delta columns) turns "we need another Java developer" into a data-led
hire and exposes key person risk. The **skill matrix** (competencies × seniority, one or two sentences
per cell) makes reviews specific and shows people what the next level requires — and doubles as an
interview instrument. Hire for two to three core strengths plus secondaries, never a narrow skill set.
Never grant a title in lieu of a raise or as a reward for longevity.

**Autonomy has three buckets** — no reporting required / keep me informed / decision required. Publish
which is which per team, and the escalation question stops being a judgment call.

**Technology decisions are 20-year decisions.** Production systems last decades. Ask of any choice: how
will it age, what if it's unsupported, how will its dependencies age, will people still want to work
with it? Avoid lock-in — give yourself **outs like a poker hand**. Build where your IPR is; buy the
commodity; measure whether the "buy" path shrinks your **moat**. Design for failure like a beehive, not
like one genetically engineered bee.

**Security is a philosophy, not a purchase.** Identify (shrink the attack surface) → Protect (layers,
the Swiss cheese model) → Monitor (aggregate every log to one place). A penetration test is void the
moment you release again. Report incidents in **IIIR** — Issue, Identify, Impact, Remedy. Report outages
to the board in **PIP** — Problem, Impact, Prevention.

**Nothing is real until it's restored.** "A backup strategy that has never been restored is not a backup,
but merely a storage utilization strategy." Same for DR, documentation, and the deployment process:
rehearse them or they are placebos.

**Due diligence is a conversation.** Establish the "why" first, run it as two role plays (be a new
customer, then a new engineer), track coverage on a bingo card, and report **consequences, not facts** —
"there is no hot water, three weeks, $2,000," not "the breaker is faulty."

**Run You, Inc.** Ask your CEO for your own review, find a mentor who has no stake in your career, keep
your résumé current every six months as a journal against imposter syndrome, track the **leading edge,
not the bleeding edge**, and prepare a successor quietly over years.

## Chapter Index

| # | Chapter | What it covers |
|---|---|---|
| 1 | [The Chief Technology Officer (CTO)](chapters/ch01-the-chief-technology-officer.md) | What a CTO is, four archetypes, do we need a CTO, evolution from engineer, the first 100 days, top 10 qualities |
| 2 | [Managing up](chapters/ch02-managing-up.md) | Partnering with the CEO, counting on the CFO, peers, taking over from another CTO, the board, communications, internal politics, change management |
| 3 | [Visionary planning](chapters/ch03-visionary-planning.md) | The grand vision, engaging clients, long-term planning, seismic shifts and pillars, elevator pitch, budgets and ROI, near-term expectations |
| 4 | [Building a team](chapters/ch04-building-a-team.md) | Five resource types, knowing when to hire, the discipline matrix, sourcing candidates, writing the job description |
| 5 | [Interviewing, choosing, and onboarding](chapters/ch05-interviewing-choosing-onboarding.md) | Interview mindset and structure, phone/in-person/video, scoring candidates, saying no, onboarding |
| 6 | [Team management](chapters/ch06-team-management.md) | Team charters, four structures, titles and the ladder, reporting and accountability, one-on-ones, education, communication |
| 7 | [Annual reviews](chapters/ch07-annual-reviews.md) | The skill matrix, handling failure, termination in three forms, logistics, exit interviews, remote working |
| 8 | [Technology decisions](chapters/ch08-technology-decisions.md) | Lock-in, build vs. buy, cloud vs. on-prem, disaster recovery, data management, microservices vs. monolithic, open source, languages and frameworks |
| 9 | [Development](chapters/ch09-development.md) | Project planning and the PM role, tickets, waterfall vs. agile, standards, version control, QA, CI/CD, technical debt, release, client requests |
| 10 | [Contract management](chapters/ch10-contract-management.md) | Service agreements, statements of work, SLAs, evaluating vendors, software licensing, support contracts |
| 11 | [Documentation](chapters/ch11-documentation.md) | Why document, audiences and formats, validation, twelve documentation types, code comments, diagrams, whitepapers |
| 12 | [Security](chapters/ch12-security.md) | Patching, penetration testing, social engineering, data leakage, password rotation, identify/protect/monitor, secure development, breach response, the CISO |
| 13 | [Housekeeping](chapters/ch13-housekeeping.md) | Managing hardware, support contracts, providing support, monitoring inside and out, backup and restoration, budget oversight |
| 14 | [Company growth](chapters/ch14-company-growth.md) | Investment groups, the impromptu pitch, investment cadence, tuck-in integration, performing and answering due diligence, handing over and taking the reins |
| 15 | [You, Inc.](chapters/ch15-you-inc.md) | Reviewing yourself, mentors, keeping pace, monitoring change, succession planning, career success, stepping up |

## Topic Index

| Topic | Chapters |
|---|---|
| Role definition, archetypes, first 100 days | 1 |
| CEO, CFO, board, and peer relationships | 2, 15 |
| Communication, metaphors, bad news | 2, 3, 6, 12, 14 |
| Internal politics and change management | 2 |
| Vision, roadmap, elevator pitch | 3, 14 |
| Budgets, ROI, cost tracking | 2, 3, 13 |
| Hiring, sourcing, job descriptions | 4 |
| Interviewing, scorecards, onboarding | 5 |
| Team structure, charters, titles | 6 |
| One-on-ones, accountability, metrics | 6, 7 |
| Training and education | 6, 15 |
| Reviews, skill matrix, promotion | 7 |
| Failure, blame, root cause | 7, 12 |
| Termination and downsizing | 7 |
| Remote working | 7 |
| Build vs. buy, vendor lock-in | 8, 10 |
| Cloud vs. on-prem, cloud-native | 8, 13 |
| Disaster recovery and continuity | 8, 13 |
| Data storage, classification, compliance | 8, 11, 12, 14 |
| Microservices vs. monolithic | 8, 14 |
| Open source and licensing | 8, 10, 11 |
| Languages and frameworks | 8 |
| Project planning, agile, waterfall, tickets | 9 |
| Coding standards, version control, code review | 9, 11, 12 |
| QA, testing, code coverage | 9 |
| CI/CD, build pipeline | 9, 12 |
| Technical debt | 9, 13 |
| Release strategy, blue-green | 9, 12 |
| Client feature requests | 9, 3 |
| SOWs, SLAs, NDAs, vendors | 10, 13 |
| Support contracts | 10, 13 |
| Documentation types and diagrams | 11 |
| Patching, CVEs, penetration testing | 12 |
| Social engineering and data leakage | 12 |
| Passwords, credentials, secure environment | 12 |
| Breach response, kill switches, CISO | 12 |
| Monitoring, logging, alerting | 12, 13 |
| Hardware inventory, MTTF | 8, 13 |
| Backups and restoration | 11, 13 |
| Private equity, acquisitions, tuck-ins | 14 |
| Due diligence, both directions | 14 |
| CTO handover and succession | 2, 14, 15 |
| Self-review, mentors, imposter syndrome, career | 15 |

## Supporting Files

- [glossary.md](glossary.md) — the book's vocabulary, alphabetical, with chapter references.
- [patterns.md](patterns.md) — the reusable moves as **When / How / Trade-offs**, grouped by the book's
  four natural parts (the role and its relationships; people; technology and delivery; operations,
  contracts, and transactions).
- [cheatsheet.md](cheatsheet.md) — decision tables, thresholds and default values, tells and smells, and
  the book's process templates in scannable form.

## Scope & Limits

- **Edition and vintage.** First edition, Manning 2023, written over roughly two years spanning the
  COVID-19 pandemic. Technology references date from that window: Windows 10 and the arrival of Windows
  11; Log4j (late 2021); the F5 BIG-IP CVE-2022-1388 (Spring 2022); SolarWinds; serverless described as
  an in-progress seismic shift; the UK remote-working study running 2011–2020. There is no material on
  generative AI or LLMs — "machine learning" and "artificial intelligence" appear only as bandwagons to
  be tested against business need, and as entries in the shifts-in-technology table.
- **Company size.** Aimed at startups through SMBs (the author defines these as typically not more than
  $100 million revenue), with a strong private-equity portfolio-company perspective drawn from the
  author's work at MacLaurin Group and New Harbor Capital. Large-enterprise and public-company
  governance is largely out of scope.
- **Deliberately not a technology cookbook.** The author states explicitly he will not recommend Java
  over C#, Amazon over Azure, or cloud over data center. There is no code and no reference
  implementation; specific products are named only as examples.
- **Areas the book defers elsewhere.** Compliance specifics (HIPAA, PCI, ISO, GDPR) are named but
  delegated to third-party experts; legal drafting is delegated to counsel; general leadership theory,
  team psychology, and secure-coding technique are each pointed to other books. Compensation and
  redundancy packages are explicitly "beyond the scope of this book."
- **Geography.** Employment, notice periods, titles, and legal points are US-centric with UK asides
  (Managing Director vs. CEO; the legal meaning of "Director" in the UK). Local law will differ.
- **This skill.** Built from the complete book text; chapter files carry the book's own tables and
  figures as text. Diagrams and photographs in the print edition are described, not reproduced.
