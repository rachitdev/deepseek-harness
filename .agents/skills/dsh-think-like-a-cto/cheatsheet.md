# Cheat Sheet — Think Like a CTO (Alan Williamson, Manning 2023)

## Thresholds and default values

| Value | Number | Source |
|---|---|---|
| Vision horizon | 2–5 years | Ch 3 |
| Long-term thinking cycle | 5-year blocks | Ch 1 |
| Language/framework decision horizon | 5 years | Ch 8 |
| Minimum maturity before adopting a language | ~2 years old | Ch 8 |
| Staff cost loading over salary | 120%–130% | Ch 2 |
| Worked daily rate example | $100K → $120K loaded → $10K/mo → $500/day (20 working days) | Ch 2 |
| Annual salary increase assumption | 2%–5% depending on performance/inflation | Ch 3 |
| Referral bonus | $500–$1,000 | Ch 4 |
| Recruitment agency fee | 10%–25% of first year's salary | Ch 4 |
| Tender bids to collect | 3–5; discard outliers >20% apart | Ch 4 |
| Max direct reports | 5–7 | Ch 6 |
| Technical writer worth hiring at | 20+ engineers | Ch 11 |
| Dedicated project manager needed at | >~5 engineers | Ch 9 |
| Ticket size | ≤ 4 hours | Ch 9 |
| Sprint length | 1–2 weeks | Ch 9 |
| Realistic productive hours per working day | ~6 | Ch 9 |
| Code coverage that is "doing very well" | 50%–70% | Ch 9 |
| Technical debt paydown | 10%–30% of team output | Ch 9 |
| Debt-paydown rest period | every 2–3 sprints | Ch 9 |
| One-on-one cadence and length | weekly or biweekly, 15–30 min | Ch 6 |
| Engineering meeting | weekly/biweekly, ≤30 min | Ch 6 |
| Team lead meeting | weekly, ≤60 min | Ch 6 |
| Default ad hoc meeting length | 15 or 30 min, never 60 | Ch 6 |
| Team reporting cadence | weekly or every 2 weeks | Ch 6 |
| Team structure review | once a year | Ch 6 |
| Review cadence | annual (anything more is overhead) | Ch 7 |
| Re-evaluation for a near-miss promotion | ~6 months | Ch 7 |
| Peer lunch cadence | monthly with each peer | Ch 2 |
| CEO "how can I be successful" check-in | at least quarterly | Ch 2 |
| CFO budget review | at least quarterly; expenditure reviewed monthly | Ch 2, Ch 3 |
| Cloud expenditure review | at least monthly | Ch 13 |
| Software license register review | at least every 6 months | Ch 10 |
| Patch cadence | at least monthly, small and frequent | Ch 12 |
| Live-ammo patch window | out of hours, ideally Saturday | Ch 12 |
| DR data-flow check | weekly | Ch 8 |
| Full DR switchover rehearsal | quarterly or every 6 months | Ch 8 |
| Backup restoration fire drill | at least annually | Ch 13 |
| Backup grandfathering | daily → 7 days, weekly → 4 weeks, monthly thereafter | Ch 13 |
| External monitoring interval | every 5–30 minutes | Ch 13 |
| No-update update cadence | every 30 or 60 min by severity; author's variant: top and bottom of the hour | Ch 6 |
| Interview scorecard criteria | 10–20 items, rated 1–10 | Ch 5 |
| Phone screen length | ≤30 min; terminate politely at 10 min if clearly wrong | Ch 5 |
| Required-skills application threshold | candidates apply at ~75% match | Ch 4 |
| Contractor no-poach clause | up to 2 years post-termination | Ch 10 |
| Service agreement termination notice | 2–8 weeks; ~1 month typical for contractors | Ch 10 |
| Data center rack unit | 1U = 1.75 inches | Ch 8 |
| AWS EC2 SLA example | 99.99% monthly ≈ 6 min downtime; 10% bill credit | Ch 10 |
| 99.999% annual uptime | ≤ 8.7 hours downtime | Ch 13 |
| DNS latency that matters for DR sync | 50–100 ms | Ch 8 |
| PE hold period | 3–5 years | Ch 1, Ch 14 |
| Tuck-in size | ~1–10 employees | Ch 14 |
| New CTO expectations | nothing for 30 days; vision by ~90 days | Ch 14 |
| Résumé update cadence | every 6 months | Ch 15 |
| Industry news sources to follow | at least 3 | Ch 15 |
| Analyst subscription cost | a few thousand a year (Gartner, Forrester) | Ch 15 |
| CVE registrations | ~5,000 per quarter | Ch 12 |
| Remote worker promotion penalty | 50% less likely | Ch 7 |
| Remote worker bonus penalty | 38% less likely | Ch 7 |
| Sickness absence, remote vs office | 0.9% vs 2.2% | Ch 7 |

---

## Decision tables

### Which CTO am I? (Ch 1)

| Signal | Archetype |
|---|---|
| No one to challenge decisions; you are the only developer; first real job; no process; only you can maintain it | Prestartup, in name only |
| <3 years old; nobody in the department >18 months; 2nd or 3rd platform; spending exceeds revenue; features change weekly | Funded startup |
| Paying clients, mature product; platform >10 years old; a handful know how but not why; company underrates its own tech reliance; no policies, security, or compliance focus | Established, first CTO |
| CTO role ≥3 years; defined hierarchy; process adhered to; roadmaps and project plans exist; technology stable and understood | Established with CTO |

### Which resource type? (Ch 4)

| Need | Choose |
|---|---|
| Long term, client facing, shapes the company, manages others | FTE |
| Specific task with a start and stop, skill won't persist | Contractor |
| Unsure the role is permanent | Contract-to-hire |
| A commodity area you'd rather not staff (desktop IT, DB management, cloud management, security) | Vendor outsourcing |
| Standalone project with clear outcomes, needing short-term breadth of expertise | Onshore consultancy |
| Cost-driven, architecture already settled, quality checkable by tooling; also testing | Offshore |

### Build or buy? (Ch 8)

| Indicator | Leans |
|---|---|
| Need to react quickly to client changes and feature requests | Build |
| Don't want to manage any infrastructure | Build (near center) |
| Would be advantageous to use the existing development team | Build (slight) |
| IPR of the solution is not important, not core to the business | Buy |
| Data needs to be directly accessible by other systems | Build (slight) |
| Will be managed by nonengineering folks in the business | Buy |
| Time to get solution to market | Buy |

Lock-in test: **if you can't lift your code and drop it on another platform and just run it, you are
locked in.** Moat test: if buying gets you to production fast and easy, a competitor can do the same.

### Cloud or on-prem? (Ch 8)

| Workload | Choose |
|---|---|
| Peaks and troughs — hourly, daily, or seasonal | Cloud |
| CPU at 90%+ most of the time (complex data or graphics modeling, bitcoin mining) | On-prem |
| Large existing hardware investment you want to extend | Private cloud (if you can staff it) |
| Private capacity exhausted at peaks | Hybrid |

### Monolithic or microservices? (Ch 8)

Three or more "yes" → you are running a monolith: single code base with no internal libraries; single
database driving all systems; an outage stops everything; single-server architecture; upgrading requires
a planned outage.

Monolithic is *correct* for proofs of concept and startups, where the point is that it works at all.

### Which open source license? (Ch 8)

| License | Effect on you |
|---|---|
| GPL (copyleft) | Your code using it must also be GPL — competitors can demand your source |
| LGPL | Safe if you link dynamically, not statically |
| BSD / MIT / Apache (permissive) | Commercial and closed-source use permitted; some require a public usage notice |
| Public domain | No ownership, copyright, or restrictions |

Team rule: publish the acceptable list (Apache/MIT/BSD), use libraries whole, register every one.

### Waterfall or agile? (Ch 9)

| Situation | Choose |
|---|---|
| Requirements stable, cost and time must be predicted, work going out to external tender | Waterfall |
| Requirements will change, stakeholders and end users should be involved throughout, value needed early | Agile |

### DR configuration (Ch 8)

| Downtime tolerance | Configuration |
|---|---|
| Minutes | Hot standby, continuous sync, like-for-like → fail forward |
| Hours to a day | Cold standby, boot and sync on failover |
| Reduced service acceptable | Partial service — read-only mode on the non-critical paths |
| Cheap DR for a non-cloud-native app | Cloud as an on-demand temporary data center |

### Support contract or not? (Ch 10, Ch 13)

Ask per component: If it fails, how does this impact end user delivery? How much knowledge or training
exists in-house? How often have you had issues? Is it a single point of failure?

Always buy: **hardware**, and **unmodernized legacy systems** (bridge until you replace them).
Buy for the gaps only where you have real in-house depth and no single-person reliance.

---

## Tells and smells (symptom → diagnosis → fix)

### Team

| Symptom | Diagnosis | Fix |
|---|---|---|
| Sprint deadlines missed more than hit, retro blames "resources pulled away" | Capacity, not estimation | Data case for a hire (Ch 4) |
| PTO not being used | People feel they can't leave, not that they love the job | Investigate load; treat as an exhaustion tell |
| Panic when one person takes PTO | Key person risk | Bracket them in the discipline matrix; document; cross-train |
| Prolonged late nights and weekends | You are asking people to work two jobs | Blip or pattern? If pattern, hire |
| Every team member gives a different one-sentence purpose | No charter | Write the charter (Ch 6) |
| A manager checks every step and calls it quality | Micromanager, a bottleneck to scalability | Coach, then remove (Ch 6) |
| Long-tenured people paid above market but technically weak | Longevity rewarded instead of achievement | Give a runway to prove capability before writing them off (Ch 2) |
| Someone rates themselves two levels above where they are | Overconfidence, citing one incident | "One swallow does not make a summer" — level back to consistency |
| Remote staff promoted and bonused less | Out of sight, out of mind | Put explicit checks in place (Ch 7) |

### Delivery

| Symptom | Diagnosis | Fix |
|---|---|---|
| "Only Fred can update that service" | Build depends on one person or machine | CI/CD, or at minimum a virtualized build environment |
| "Well, it works on my machine" | No pipeline-based build | CI/CD gates |
| A disabled test that was never re-enabled | Technical debt introduced under deadline | Debt budget and rest sprints |
| A ticket that takes days or weeks | Ticket sized wrong | Split; the halfway point is the moment to ask for help |
| Sacred code blocks nobody will touch | Missing standards and missing documentation of *why* | Standards, reviews, ticket links in comments |
| Downtime required for every release | Static servers, not cloud-native | Blue-green / container replacement |
| `if (Client == XYZ)` in the code base | Unfiltered client request | Generalize, or offer export/API/plugin |
| A branch per client | Doubled workload on all fronts | Stop; consolidate |
| Cloud bill flat every month at a seasonal business | Not using elasticity | Rearchitect for scale-down |
| Bill rising | Forgotten resources, or genuine growth | Monthly review with provider tags; identify which, and celebrate growth with the CFO |

### Security and operations

| Symptom | Diagnosis | Fix |
|---|---|---|
| Penetration certificate older than the last release | Placebo security | Continuous pen testing in the deployment pipeline |
| Passwords salted and hashed but appearing in logs | Data leakage via captured POST payloads | Mask sensitive fields before committing to logs |
| Debug/stack-trace error page visible to the public | A development switch that survived promotion | Turn it off; audit error handling |
| Password never changed because changing it needs a restart | It is a configuration attribute, not a security control | Real-time credential retrieval from a secrets manager; track areas needing rotation |
| Shared database credentials | No accountability, cannot exclude an individual | Individual accounts |
| Production and development on the same network | Enlarged attack surface | Isolate production |
| Status page always green | Manual updates as a political game | Automate it; hiding does not work long-term |
| Client reports the outage before you notice | No outside-in monitoring | Emulated client checks from another cloud/region every 5–30 min |
| Breach found in an area with no logs | Predictable and easily remedied | Extend logging; log successful and failed logins |
| Backups on the same machine and network | Not a backup | Encrypted, separate network or account, with restoration drills |
| Sourcing spare parts on eBay | Hardware past supportability | Get the replacement on the roadmap |

### Business and process

| Symptom | Diagnosis | Fix |
|---|---|---|
| Your CEO tells you about a problem you already knew | Trust destroyed | Escalate anything touching clients or revenue immediately |
| Two departments naming the same entity differently (project ID / campaign ID) | Vocabulary drift | Ask the basic question; learn each department's language |
| Plans keep failing approval | You are pitching the wrong person | Find the rainmaker whose counsel the CEO seeks |
| The business is losing faith mid-project | Valley of anguish | Deliver a bite-size win; house-construction progress framing |
| A project accumulating features it never promised | Feature creep | Reset expectations every periodic update — what you will and will not deliver |
| A vision your team can't articulate | Wrong vision, or not broken down enough | "Do they believe in your vision?" |

---

## Process templates

### The first 100 days (Ch 1)
`0–7 client's-eye view` → `7–30 listen and observe` → `30–70 lift the lid, note easy wins, meet clients` →
`70–100 fix process gaps, secure the lights-on components, get project wins`

### Elevator pitch build (Ch 3)
`5 benefit items` → `5 internal feature items` → `sleep on it` → `cut to 6` → `wait an hour` → `rank by
business importance` → `technical + nontechnical sentence for each` → `find the grabber opener` →
`practice aloud with both audiences`

### Vision statement grid (Ch 3)
`Statement | What | How | Success` — one row per initiative, success stated as an observable event
("when the last server is turned off").

### Job description (Ch 4)
`Title` (industry-standard, metadata-rich) → `Description` (first sentence = what they will do) →
`Minimum requirements` (non-negotiable only) → `Required skills` (interview-worthy only, no years) →
`About/legal` (company, products, benefits, WFH policy)

### Interview funnel (Ch 5)
`Résumé` → `phone screen ≤30 min` → `fetch, tour, seat without a barrier` → `conversation, not
interrogation` → `exercise (whiteboard or pseudo code)` → `their questions` → `exit walk with an
uninvolved escort` → `scorecard filled in immediately` → `group comparison on the gaps`

### Team charter (Ch 6)
`Purpose` → `Customer` → `Resources` → `Skill requirements` → `Key metrics`

### Rules of engagement (Ch 6)
`No reporting required` | `Keeping you informed` | `Decision required`

### Annual review (Ch 7)
`1 week's notice + matrix copy` → `their self-assessment` → `your assessment vs. last year's` →
`overlay and discuss only the differences` → `outcome: no improvement / overconfidence / underperforming
/ outperforming` → `promote without skipping levels` → `announce with reasons`

### Termination logistics (Ch 7)
`Notice period?` → `first thing in the morning, never the last working day of the week` →
`agreed points for the person and for the group` → `never alone — HR, else CFO or CEO` → `remove access`

### CI/CD pipeline (Ch 9)
`Commit` → `Code analysis (SonarQube)` → `Compile/build` → `Unit tests` → `Security tests` → `Deploy`
Start at `Commit > Build > Deploy` if this is new.

### Outage release plan (Ch 9)
`What goes offline` → `customer-facing maintenance page?` → `time frame (shutdown + update + startup +
pre-user testing + re-enable users)` → `precise steps per area` → `owner and handoff per step` →
`business-agreed date at lowest usage` → `progress-report and completion distribution lists` →
`postmortem`

### Blue-green release (Ch 9)
`Stand up new environment` → `test while old serves traffic` → `switch traffic` → `tear down old` —
or `tear down new and retry`. Schema rule: **always add, never drop.**

### Statement of work (Ch 10)
`Scope (in and out)` → `Deliverables` → `IPR assignment point` → `Access required` → `Personnel and
subcontracting rules` → `Out-of-scope authorization` → `Warranty` → `Time frame` →
`Extension/renewal/termination` → `Solution limitations` → `Payment terms`

### Your own SLA (Ch 10)
Inputs: `worst third-party SLA you depend on` + `release/upgrade downtime` + `24h or core hours?` +
`time to resolution` + `support staffing including vacations` + `track record` + `weakest single point
of failure`

### DR switch (Ch 8)
`Trigger criteria written in advance` → `credentials in an immediately accessible secure place` →
`start and populate cold services` → `update every public and private DNS record (mind the TTL)` →
`assign validation roles and tests` → `reverse plan if bouncing back`

### IIIR security incident report (Ch 12)
`Issue` (what happened and how) → `Identify` (how it surfaced) → `Impact` (the result) →
`Remedy` (what was done). Facts only; internal only.

### PIP board/company report (Ch 2)
`Problem` (in business terms) → `Impact` (on clients/employees/partners) → `Prevention` (what has been
learned and instigated)

### Due diligence report (Ch 14)
`1 Overview of target` → `2 Methodology (when, who was present, what was provided, what was not)` →
`3 Summary findings ordered by severity of consequence (+ a separate celebration list)` →
`4 Supporting materials`. Red/amber/green scoring. Every finding stated as consequence + effort + cost.

### Due diligence session (Ch 14)
`Establish the why` → `role play 1: be a new customer` → `break, praise genuinely` →
`role play 2: be a new engineer being onboarded` → `mark the bingo card` → `pre-review the report with
the interviewees`

### Answering due diligence — have these ready (Ch 14)
`Architecture/network diagrams` · `all technologies with versions` · `licenses, renewal dates, costs` ·
`org chart` · `new employee onboarding document` · `recent support issues` · `future development
priorities`

### CTO handover (Ch 14)
Leaving: `constraints behind each decision` → `team member walk-through from last reviews` →
`your vision and why` → `last 30 days of your calendar, focusing outside your department`
Arriving: `do nothing for 30 days` → `observe meetings` → `one-on-ones opening with "if you were
king/queen for the day..."` → `offer no solutions` → `use the honeymoon to ask peers basic questions`

### You, Inc. (Ch 15)
`CEO review on communication / level of detail / business intelligence / initiative-vision` →
`team feedback channel` → `mentor(s) with no stake` → `résumé updated every 6 months` →
`3 news sources, act when themes recur` → `successor prepared quietly` → `career plan with defined
success`
