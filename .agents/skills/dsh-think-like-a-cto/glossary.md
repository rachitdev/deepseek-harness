# Glossary — Think Like a CTO (Alan Williamson, Manning 2023)

**24-hour rule** — Jeff Hanner's practice of giving people 24 hours from a perceived infraction to address it head on; after that it is never mentioned again (Ch 2)

**Agile** — Guidelines and principles focused on continuous delivery, breaking milestones (epics) into tasks arranged in one- or two-week sprints, each ideally delivering interactable functionality; contrary to belief, still requires planning (Ch 9)

**Architect** — Title level, 10+ years: deep knowledge across several disciplines within their sphere, responsible for thinking broader (Ch 6)

**Architectural diagram** — The enterprise "map": major components with lines joining connected pieces, no logistics detail; the state-and-interstate level view (Ch 11)

**Associate** — Title level, 2+ years: has industry experience but is still learning their trade (Ch 6)

**Attack surface** — All the possible avenues where a potential attack or leak could occur; the CTO's job is to reduce it and, where reduction is impossible, secure it (Ch 12)

**Automated test** — A test requiring no direct human interaction, run as part of the build-and-release pipeline: unit tests, API endpoint testing, end user testing (Ch 9)

**Bench strength** — Depth of candidates or skills held in reserve; the payoff for treating recruiting as continuous rather than event-driven (Ch 4)

**Bingo card** — A high-level list of topics to cover during due diligence, marked off as conversation naturally yields each, in place of canned questions (Ch 14)

**Blast radius** — The scope of damage when a failure cannot be prevented; minimizing it is the fallback goal after prevention (Ch 7)

**Bleeding edge** — Technology too early to adopt — "called bleeding because it is sharp and can hurt if it goes wrong" (Ch 15)

**Blue-green release** — Standing up a complete new environment with new software, testing it while the current one serves traffic, switching traffic over, then tearing down the old one; failure means tearing down the new one instead (Ch 9)

**Boiling the frog** — Introducing change slowly enough that people adapt rather than reject it; the agile release-small-release-often mentality made into a change strategy (Ch 2)

**Bounce back** — DR strategy of returning to the primary zone once the issue is resolved; forced when the backup zone is not a like-for-like configuration (Ch 8)

**Brown bag lunch** — Informal internal training where a topic is discussed over lunch; "always a goodwill gesture to buy lunch for everyone who attends" (Ch 6)

**Burn rate** — What your department costs, if nothing changes, monthly and yearly; the litmus test of financial grip (Ch 2)

**Burn the ships** — James Milbery's term for the identified moment the old system is turned off and can no longer be used, forcing adoption (Ch 2)

**Business fatigue** — The point at which the business forgets a long project was always going to take this long and starts questioning whether to continue (Ch 3)

**Charter** — A team's statement of purpose, customer, resources, skill requirements, and key metrics; justifies the team's existence and defines its contribution (Ch 6)

**CI/CD** — Continuous integration / continuous deployment: once code is reviewed, approved, and merged, it is automatically compiled, tested, and deployed, with any failure flagged immediately (Ch 9)

**CISO** — Chief Information Security Officer; should sit outside the CTO's group so they can hold it to account — "who watches the watchers?" (Ch 12)

**Cloud** — An environment where CPU, memory, network, and storage are consumed and charged per usage, sometimes down to the second, with no consideration of underlying physical hardware (Ch 8)

**Cloud-deflation paradox** — Cloud unit costs fall, but savings are lost because cheapness encourages more usage than needed (Ch 3)

**Cloud native** — An application specifically designed to harness on-demand resources, scaling up and down automatically (Ch 8)

**Code coverage** — Measurement of how much underlying code actually runs as part of the tests; 100% is nearly impossible, 50–70% is doing very well (Ch 9)

**Cold standby** — DR configuration keeping the bare minimum running, booting and synchronizing machines only when failover is initiated; cheapest and slowest (Ch 8)

**Commoditization** — When a process or technology reaches a state of acceptability in role, innovation, and competition — web servers, mail servers, and increasingly databases (Ch 3)

**Contract-to-hire** — A "try before you buy" contractor arrangement convertible to FTE for a small fee (Ch 4)

**Copyleft** — Open source license family (GPL) requiring any software using it to be released under the same license, including modifications; generally not business friendly (Ch 8)

**CVE** — Common Vulnerabilities and Exposures, cve.org: the online database tracking known security flaws since 1999, roughly 5,000 registered each quarter (Ch 12)

**Data lake** — A store for large volumes of files of differing structure and size, for backup or future analysis; commonly a staging area for data with no permanent home yet — "your data attic" (Ch 8)

**Data warehouse** — A database designed for analytics and business intelligence, arranged for fast reporting and year-over-year analysis; data usually aggregated and anonymized (Ch 8)

**Death by a thousand cuts** — Performance failure across several fronts at once, no one of which is fatal; hardest to spot in new hires who interviewed well (Ch 7)

**Decision log** — A single collated record of why big decisions were made (Ch 11)

**Decision paralysis** — Where fear of making the wrong decision prevents any decision from being made; "any decision is better than making no decision" (Ch 1, Ch 3)

**Digital forensics** — Determining what was taken after a breach, given that digital assets can be copied transparently leaving no trace (Ch 12)

**Discipline matrix** — A grid scoring each team member 0–10 against each discipline, with brackets marking single-holder skills, plus Total, Average, Need and Delta columns; a human capital audit (Ch 4)

**Document debt** — The state where anyone technically competent could not rebuild every component of the enterprise from source control (Ch 11)

**Due diligence** — A conversation, not an interrogation, to determine whether the technology can do what the buyer needs and, if not, what it will cost — "the home inspection report" (Ch 14)

**Elevator pitch** — A roughly 30-second description of your vision anyone can understand, built from an eight-step exercise and practiced aloud (Ch 3)

**Epic** — Agile milestone, turned into a series of tasks ready for execution (Ch 9)

**Excitement committee** — Kelley Powell's term for representatives from across the business involved in a large initiative from the start, acting as diplomats and advocates (Ch 2)

**Fail forward** — DR strategy of letting the backup zone become the new primary, with the failed zone rebuilt as the new backup (Ch 8)

**Feature creep** — Small features added because each seems too small to matter, until delivered expectations bear no relation to reality (Ch 3)

**Fellow** — Title level, 15+ years: a special title like a lifetime achievement or university tenure, recognizing long-term continued efforts (Ch 6, Ch 7)

**Fiefdom** — Tribal behavior, term from the early 1800s: one or more people serving their own goals first in a semi-loose, unofficial alliance (Ch 2)

**Five whys** — Iterative narrowing approach devised by the founder of Toyota, starting questioning at a high level and narrowing at each answer (Ch 7)

**Front office** — The core platform, enterprise, or product line servicing clients — the CTO's domain (Ch 1)

**FTE** — Full-time employee: on your payroll, bought into the company vision, persisting beyond a single project (Ch 4)

**Gitflow** — Branching pattern with master (deployable) and develop branches; features branch from develop, merge back, and a release merges develop into master; nothing is committed directly to either main branch (Ch 9)

**GitHub flow** — Simpler branching pattern with only master; features branch off and merge back once reviewed and tested (Ch 9)

**Grand vision** — The CTO's own vision statement, focused on the group's output while supporting the company vision, covering two to five years and stating what the business will be able to do on arrival (Ch 3)

**Hot standby** — DR configuration with servers always up and continually synchronized, ready to take traffic; most expensive, consumes power, bandwidth, and backup hardware lifetime (Ch 8)

**Hybrid cloud** — Public cloud augmenting a private cloud transparently when private resources are exhausted (Ch 8)

**IIIR** — Issue, Identify, Impact, Remedy: the four-step internal security incident report structure (Ch 12)

**Imposter syndrome** — The feeling you are somewhere you do not belong and will be discovered; most people feel it at some point (Ch 15)

**Information hoarder** — Someone who believes sharing valuable information diminishes their role and puts them at risk of termination (Ch 2)

**IPR** — Intellectual property rights; the "secret sauce" that defines where you build rather than buy, and a critical clause in any service agreement (Ch 8, Ch 10, Ch 14)

**Key person risk** — Reliance on one individual who alone knows how to do something essential; private equity in particular hates it (Ch 4)

**Kill switch** — A prepared response that isolates a breach — disabling network access to a machine or subnet, or powering off/suspending — buying time rather than solving the problem; the digital ship's bulkhead (Ch 12)

**Lead/Manager** — Title level, 5+ years: ready to take responsibility for the output of others (Ch 6)

**LGPL** — Lesser GPL: you need not release your code as open source if you dynamically rather than statically link; more business friendly than GPL (Ch 8)

**Lifecycle centric** — Team structure arranged like an assembly line — development, testing, release, DevOps/infrastructure, support (Ch 6)

**Low/no code** — Point-and-click or drag-and-drop development environments; they deliver 80% quickly and make the last 20% disproportionately hard (Ch 8)

**Manual testing** — Testing requiring a human, used where automated tests are too difficult or brittle (Ch 9)

**Moat** — Investment term for how hard it is for competitors to copy what you are doing; buying third-party tools shrinks it (Ch 8)

**Monolithic** — Enterprise built and deployed as a single entity, generally a single code base compiled as if no one else uses it, with little code reuse and tight coupling (Ch 8)

**MTTF** — Mean time to failure: a component's expected lifetime, guiding when to source and stock replacements (Ch 13)

**NDA** — Nondisclosure agreement: a mini contract letting two parties exchange information confidentially without fear of loss or misuse of IPR, limited in scope and time (Ch 10)

**No-update update** — Kelley Powell's practice of giving updates on a fixed cadence (every 30 or 60 minutes by severity) even when there is nothing to report but that work continues (Ch 6)

**NoSQL store** — Document store keeping only the data you have, not placeholders for data you don't; enforces no data integrity or structure (Ch 8)

**Offshore resourcing** — Resourcing from outside your own country — India, Eastern Europe, Latin America are the classic examples; cheap, but with time zone, vetting, dispute, and cultural risks (Ch 4)

**On-prem** — On-premises: underlying hardware owned and managed by the company, whether in the office or a dedicated data center (Ch 8)

**Onshore resourcing** — Engaging a local or in-country consultancy that does the bulk of the work at their own offices, usually through an account or project manager (Ch 4)

**Operating partner** — In private equity parlance, the person who rolls up their sleeves and helps the portfolio company; embrace them, they do not want your job (Ch 14)

**Outage release** — A release requiring the system to come offline, needing a planned time frame, per-step ownership, and business-agreed dates (Ch 9)

**Owner's manual** — The documentation explaining how to keep the enterprise up and running: components, monitoring, and how to bring things back online (Ch 11)

**Passive aggression** — Aggression expressed indirectly: dragging feet, voicing approval to the group then undermining the initiative outside (Ch 2)

**Penetration testing** — External attempts to penetrate your system, looking for exploitable holes; void the moment you release again, so it belongs in continuous deployment (Ch 12)

**Permissive license** — Open source license family (BSD, MIT, Apache) permitting commercial or closed-source use and extension (Ch 8)

**Pillars** — The foundational, commodity components a vision relies on, chosen so any one can be replaced while the others keep holding the vision up (Ch 3)

**PIP** — Problem, Impact, Prevention: the format for reporting an outage or breach to the board and the company (Ch 2)

**Pizza box** — Rack-ready server in a long, flat configuration for tight vertical packing; slightly more expensive than boxy containers (Ch 8)

**Private cloud** — Your own hardware provisioned with a cloud OS so it can be managed per usage; extends the life of existing hardware investment (Ch 8)

**Process diagram** — How data flows around the system and what happens to it as it passes through services — the traffic overlay on the architectural map (Ch 11)

**Product centric** — Team structure organized by product line, each with its own development, testing, and support (Ch 6)

**Productionalized** — Systematized: a repeatable, reliable development process that makes new hires productive in hours or days rather than weeks (Ch 9)

**Public domain** — The freest license: no ownership, no copyright, no restrictions; rare, though book code samples often use it (Ch 8)

**Relational store** — SQL database: common attributes as columns, related records as rows, tables linking to tables, queried with SQL (Ch 8)

**Release notes** — A document of everything that changed in a release, sometimes generated automatically from tagged tickets or sprints (Ch 9)

**Repository** — Version control unit; "a living, breathing time capsule, a collection of parallel universes" — granularity is an art, neither too many nor one (Ch 9)

**Rules of engagement** — The three-bucket definition of team autonomy: no reporting required, keeping you informed, decision required (Ch 6)

**Scorecard** — Interview instrument of 10–20 criteria, soft and hard, each rated with a defined scale and a Notes column, filled in immediately after the interview (Ch 5)

**Seismic shift** — An industry-wide change large enough to reset assumptions mid-vision, requiring pillars designed for replacement (Ch 3)

**Senior** — Title level, 5+ years: knows their trade very well, to great depth (Ch 6)

**Skill matrix** — A grid of competencies against seniority levels, each cell holding one or two sentences describing what success looks like; the basis of a data-led annual review and an interview aid (Ch 7)

**SLA** — Service-level agreement: the minimum or maximum limits of a service, including uptime, acknowledgment times, and the recourse when limits are missed (Ch 10, Ch 13)

**Social engineering** — Convincing someone inside a company to enable access while they believe they are helping; support, account managers, and sales-facing staff are the likeliest targets (Ch 12)

**SOW** — Statement of work: the detailed list of what you expect from an engagement, detailed enough for companies to tender against (Ch 10)

**Splinter team** — A small team spun up for a few days to dive deep into a narrow area and fetch intel for a decision (Ch 3)

**Sprint** — A short burst of activity, usually one or two weeks, typically designed to deliver a piece of functionality, followed by a review (Ch 9)

**SQL injection** — Attack turning one database statement into several by crafting an input value; requires no special network or database access (Ch 12)

**Starter's guilt** — The CTO's version of buyer's remorse: the feeling that the whole project hinges on one decision you may have gotten wrong (Ch 3)

**Stay-on bonus** — Payment ensuring coverage until a project completes, used when people whose skills will no longer be needed might otherwise leave early (Ch 7)

**Stretch goal** — Lou Adler's idea of making a role sound a little harder than what the candidate does now, but not out of reach (Ch 4)

**Structured demotion** — Adjusting expectations and level so someone can succeed where they actually operate, possibly with a compensation adjustment (Ch 7)

**Sunk cost** — The feeling that heavy prior investment obliges you to finish; the poker player pulls out when the hand can no longer win (Ch 3)

**Swiss cheese model** — Layered defense so no single hole passes all the way through; never rely on one mechanism (Ch 12)

**System of record** — Data considered core to the company, treated as the "truth" for a given entity (Ch 8)

**Technical debt** — Debt incurred every time a shortcut is taken — hardcoding a value, minimal checks, an unapplied patch; pay down 10–30% of team output (Ch 9, Ch 13)

**Ticket** — A uniquely identified unit of work assigned to a person, with a lifecycle from open to closed; sized at no more than about four hours (Ch 9)

**Town hall** — A meeting reserved for a specific topic as need arises: a presentation followed by open Q&A (Ch 6)

**Tuck-in** — Acquisition of a small company (perhaps 1–10 employees with a niche) to augment a larger one — "purchasing a fully formed, self-running department"; success depends on speed of integration (Ch 14)

**Valley of anguish** — James Milbery's term for the point in a long project where fear and disruption make leaders question whether to continue (Ch 2, Ch 3)

**Version control** — A library and time vault for digital assets, enabling parallel development without contorting development environments (Ch 9)

**VP of Engineering** — Sometimes a naming scheme, sometimes a distinct role reporting to the CTO and owning the logistics of running the engineering team; the book assumes they are on the road to CTO (Ch 1)

**Waterfall** — Project model planning as much as possible up front before work begins; predictable in cost and time, poor at coping with change (Ch 9)

**Whitepaper** — A published document for an external audience detailing a philosophy or process, without trade secrets; often used in the sales cycle (Ch 11)

**WFH** — Work from home; requires stated core hours, deliberate check-ins, and active guarding against the out-of-sight promotion and bonus penalty (Ch 7)
