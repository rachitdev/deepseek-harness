# Patterns — Think Like a CTO (Alan Williamson, Manning 2023)

The book's reusable moves, in **When / How / Trade-offs** form, grouped by its four natural parts.

---

## Part 1 — The role and its relationships (Ch 1–3)

### The first 100 days

**When** — You have just taken a CTO role, whether founding, inherited, or promoted, and the urge to fix
what you can already see is strong.
**How** — 0–7 days: learn the company from the client's perspective — what it does, its terminology,
pricing model, top clients. 7–30 days: listen and observe every department, let engineering run on
autopilot, learn people's backgrounds, contributions, frustrations, goals. 30–70 days: get system access
and look yourself — source code quality, data stores, infrastructure — noting easy wins, and meet
established clients over lunch. 70–100 days: begin executing on missing process and on what keeps the
lights on, and meet the CEO repeatedly about their vision.
**Trade-offs** — You look slow for three months and forgo early credit for obvious fixes, in exchange for
never making a rash decision on incomplete context and for a reputation built on thoughtfulness. Engineers
find the enforced patience genuinely difficult.

### Data-led case for the role or the hire

**When** — Management asks whether they need a CTO, or your CFO asks whether you really need another head.
**How** — Answer with a checklist, not a feeling. For the role: the 14-question test (custom software,
third-party platform customization, missing usage data, compliance exposure, password sharing, self-run
client-facing servers, no sandbox, junior team without leadership, no patching, no documentation, no DR,
key person risk, support absorbing outdated-software problems, aging systems). For the hire: the project
plan, the attrition analysis, or the exhaustion tells.
**Trade-offs** — Assembling evidence takes time you would rather spend fixing the problem, and the answer
may be no. But "the most successful [cases] are the ones where it is hard to argue, because they are data
led, not emotionally led," and gut instinct still gets to confirm what the data says.

### Speak to the office, not the person

**When** — Any communication upward that will be relayed to a board, PE partners, or another department.
**How** — Treat the CEO as an office. Give them the sound bite they can reuse in their own language. Reach
for a metaphor over the mechanism (the vendor "put the wrong number on their mailbox," not DNS TTLs).
Match the level of detail the CEO has told you they want.
**Trade-offs** — Metaphors lose precision and can be repeated back at you inaccurately. The compensating
gain is that the message survives three relays intact, and non-technical people feel able to contribute.

### Lead with data, think slow, act fast

**When** — Any proposal or request under social pressure, especially in a large meeting.
**How** — Steep the proposal in data — productivity gained, money saved on support or licenses. Take
requests under advisement rather than saying yes in the room; take the space to think through permutations.
Then act with no second-guessing.
**Trade-offs** — You will disappoint people who wanted an immediate yes, and you risk seeming
non-committal. The gain: you stop over-committing to things you cannot deliver, and your decisions hold.

### Build for input, never yes/no

**When** — Presenting anything the CEO or board must approve.
**How** — Offer several relevant options and a phased, incremental approach with wins along the way, rather
than an all-or-nothing ask.
**Trade-offs** — Phasing costs more total effort and can extend the timeline; irrelevant options are worse
than none. But "if you present a proposal to a CEO that has only two outcomes, yes or no, then you have
already limited your successful outcome by 50%."

### Own the bad news first

**When** — Anything has gone wrong that affects clients or revenue.
**How** — Tell the CEO immediately. Own it, present without emotion or blame, never name the team member
responsible. Use PIP — Problem, Impact, Prevention — for the board and the company.
**Trade-offs** — You absorb responsibility for other people's mistakes and take the discomfort personally.
The alternative is worse: "Your worst nightmare is when your CEO tells you about a problem you already
knew about."

### Interview the outgoing CTO

**When** — You are taking over from someone and have any access to them at all.
**How** — Ask the five questions: what kept you up at night; what pressure did the business place on you;
who is the most important client; which client placed the most overhead on your department; who in your
team did you rely on most. Celebrate their work publicly and never disparage it in any circle.
**Trade-offs** — It requires swallowing genuine frustration at what you inherit, and the answers are
self-reported. It buys historical context nobody else has and keeps the phone line open later.

### Boil the frog, with an excitement committee

**When** — Introducing change that will reach beyond your department.
**How** — Secure the CEO and department leads first. Assemble representatives from across the business as
advocates from day one. Release small and often. Name the criteria for "burning the ships" — the moment
the old system is switched off — and get all business leaders to agree to it in advance.
**Trade-offs** — Slower than a flag-day cutover and requires running two systems for a period, which is
exactly what you eventually want to end. Without it, "they will revolt and jump straight out of the pan."

### The three-bucket burn rate

**When** — You have inherited a department or joined an organization, or cannot answer what your department
costs this month.
**How** — Split costs into Development, Production, and Support, in a simple spreadsheet. Cost staff at
120–130% of salary. Derive a daily rate ($100K salary → $120K loaded → $10K/month → $500/day at 20 working
days). Review with the CFO quarterly.
**Trade-offs** — The exercise reveals numbers you may not enjoy sharing, and the daily-rate simplification
hides real variation between people. The payoff is that future estimates become arithmetic, and the CFO
learns your figures can be trusted.

### Build the grand vision, then pitch it in eight steps

**When** — You have finished discovery and the business needs to know where technology is going.
**How** — Write it for two to five years. Split what/why (business) from how (your group). Write up to five
benefit items and five internal feature items, sleep on it, cut to six, sleep again, rank, write a technical
and a nontechnical sentence for each, then find the grabber opener. Practice aloud with technical and
nontechnical listeners.
**Trade-offs** — The exercise takes days of elapsed time and feels indulgent next to delivery work. But an
unpitched vision cannot recruit support, and "a CTO without a vision is a captain without a destination."

### Pillars and splinter teams

**When** — A multi-year vision must survive technology changing underneath it.
**How** — Name the pillars — the commodity foundations you would rather outsource (an API mechanism, a data
lake). Watch each one commoditize. When one shifts, spin up a splinter team for a few days to fetch intel
before deciding to pivot.
**Trade-offs** — Designing for replaceability costs more up front than committing hard to one choice, and
this discipline applies only to foundational pieces — changing frameworks or languages is a different, more
disruptive class of decision.

### Three budget models from fast/good/cheap

**When** — Any large initiative going for approval.
**How** — Build the best case with the CFO line by line. Then model faster (contractors, more heads — and be
ready for "if money were no object, how fast?"). Then model cheaper, naming exactly which features are cut.
Keep all three current so you can answer instantly when the funding tap opens.
**Trade-offs** — Three live models mean three things to maintain monthly. The alternative is being caught
without an answer at the moment the business is most willing to fund you.

### Control the narrative

**When** — A long project is running and the organization is filling silence with its own conclusions.
**How** — Communicate slippage and its reasons rather than glossing. Use the house-construction analogy for
invisible early progress (foundation → plumbing → cabling → drywall). Reset expectations in every periodic
update — what you are delivering *and what you are not*. Celebrate sprints internally, save the loud
celebration for delivered business value.
**Trade-offs** — Repeatedly correcting misconceptions creates small disappointments and can feel like
under-selling. It beats "having to defend the potential failure of a project to live up to its hype because
you were too polite to correct misconceptions."

---

## Part 2 — People (Ch 4–7)

### Always be hiring

**When** — Always. "You should never see recruiting as a one-time event."
**How** — Keep at least two sourcing channels warm — referrals with a $500–$1,000 bonus, an agency you have
mystery-shopped, headhunting from vendors and open source projects and book authors, graduate fairs and
internships, self-serve online. If a great resource appears with no open position, ask the CFO to squeeze
the budget.
**Trade-offs** — Continuous pipeline work costs time with no immediate deliverable, and graduate channels
run six to twelve months ahead of need. It is what lets successful CTOs "bring people on nearly immediately."

### Match the resource type to the need

**When** — You need capacity and have defaulted to "hire an FTE."
**How** — Run the five options against time frame, budget, and whether the skill persists past the project:
FTE for long-term, client-facing, vision-aligned work; contractors for defined tasks with a start and stop;
vendor outsourcing for commodities you would rather not staff; onshore consultancies for standalone defined
projects; offshore where architecture is settled and code quality can be checked automatically.
**Trade-offs** — Every non-FTE option trades loyalty, learning, and in-house expertise for speed or cost.
Offshore in particular costs far more management time than expected.

### Discipline matrix before the requisition

**When** — Data says you are understaffed and you are about to name a role.
**How** — Score everyone 0–10 per discipline, bracket any skill held by one person, total and average per
row, add a subjective Need column and compute Delta. Build the first draft from résumés and past reviews,
then let the principals rewrite your numbers — "it's so much easier to edit than to create."
**Trade-offs** — Scores are subjective and the exercise can be uncomfortable. It reliably surprises teams
about their own strength and turns key person risk into something visible before it bites.

### Write the job description yourself

**When** — HR has offered you the standard template.
**How** — Title first, standard industry title, metadata-rich (Senior AWS Architect, not Cloud Architect;
React Mobile Developer, not Mobile Developer). First sentence describes what the candidate will *do*, never
the company. Sell the stretch. Keep minimum requirements to genuinely non-negotiable items. Cut implied
skills and years of experience. Publish the pay scale.
**Trade-offs** — Publishing pay invites internal awkwardness where existing people are underpaid — which the
book treats as a problem to fix rather than hide. Writing it yourself costs an afternoon; not writing it
costs every downstream step.

### Treat every candidate as a client

**When** — Every interview, from phone screen to exit walk.
**How** — Greet them yourself, chat on the way in, show them the space they would work in, offer refreshments,
seat them without a desk between you, hold a conversation rather than an interrogation, ask about a past
project like the work you need, leave time for their questions, and have someone uninvolved escort them out
and ask how it went.
**Trade-offs** — It costs meaningfully more time per candidate than a checklist interview, and it makes
rejecting people harder emotionally. It also produces the information a canned question list never will.

### Scorecard before the first interview

**When** — More than one person will interview, or more than one candidate will be seen.
**How** — 10–20 criteria split soft and hard, drawn from the job description's required skills. Write the
score range for each (the AWS example: 0 none; 1–3 a handful of services; 4–6 own account and IAM users;
7–10 VPC subnets, security groups, NAT gateways, role-based IAM). Brief every interviewer. Fill it in
immediately after.
**Trade-offs** — Building scales per role is real work and the highest score is explicitly *not* an
auto-hire. The value shows up in the disagreements: large score gaps between interviewers are the most
productive conversations in the process.

### Say no properly

**When** — Every unsuccessful candidate, always.
**How** — Definitely-not: short, honest, high level, hard skills only, and do not let it become a
negotiation. Definitely-maybe: tell them a stronger candidate won, ask to keep their details, connect on
LinkedIn.
**Trade-offs** — Honest feedback exposes you to challenge, so keep it brief and factual. Ghosting is
cheaper today and costs you a prevalidated pool and your reputation with people who may end up at your
client or competitor.

### Charter every team

**When** — You form a team, or inherit one that has none.
**How** — Write purpose, customer, resources, skill requirements, and key metrics — bullet points are enough.
Hand the leader the charter and the job of producing a plan to fulfil it, then leave them to execute.
**Trade-offs** — Writing charters for inherited teams requires first understanding why things are as they
are, which delays change. Without one, "you don't have a team — you have a collection of people drawing a
salary."

### Define autonomy in three buckets

**When** — Your teams are independent and you still need to not be blindsided.
**How** — Publish what needs no reporting (interviewing, internal process, PTO, project execution), what
keeps you informed (client issues, production problems, security concerns, inter-team changes, individual
problems, timelines), and what needs your decision (final interviews, HR and termination, tool purchase and
licensing, technology change).
**Trade-offs** — Rules of engagement can feel like bureaucracy to experienced leads and need tuning per
team. They eliminate the guesswork about when to escalate, which is where most surprises originate.

### The no-update update

**When** — Any high-visibility incident or long-running issue.
**How** — Update on a fixed clock — top and bottom of the hour, or every 30 or 60 minutes by severity —
even when the only content is that work continues. Extend the same courtesy to people waiting on you.
**Trade-offs** — It consumes attention during exactly the period when attention is scarcest. It also stops
stakeholders from concluding they have been forgotten and generating their own interruptions.

### The skill matrix review

**When** — Annual review time, and again when you interview for that role.
**How** — Competencies down, seniority across, one or two sentences per cell. Give a week's notice plus a
copy of the matrix and ask for a self-assessment. Assess separately using last year's matrix. In the
meeting, overlay the two and discuss only the differences. Never skip levels on promotion. Announce
promotions with the reasons.
**Trade-offs** — Building 30-plus cell descriptions per role is a significant one-time investment, and
overconfident self-assessments must be levelled back down, sometimes losing people who disagree. It pays
back at every review and interview thereafter.

### Handle failure without blame

**When** — Human error has caused an outage or loss.
**How** — No shouting, no threats or insults, no instant terminations. Find the root — the five whys —
sorting it into mistake/incompetence or lack of foresight/planning. Ask whether *your* unauthorized purchase
order caused it. Fix the system with guard rails (the obnoxiously coloured production terminal). Document
the incident, the reason, and the rectification. Feed it into your interview process.
**Trade-offs** — Absorbing responsibility publicly ("the buck stops here") is costly and repeat offenders
eventually still need moving. But the alternative kills the reporting culture: "will the next person feel
compelled to speak up should they make an error?"

### Terminate on data, never in isolation

**When** — Downsizing, skills no longer required, or poor performance.
**How** — Exhaust alternatives first — retraining, another department. Get CEO/CFO and HR agreement; if you
cannot convince them, rethink. For performance, isolate the failing areas, put the person under review with
specifics and dates and genuine support, and consider a structured demotion. Deliver first thing in the
morning, never the last working day of the week, never alone, saying as little as needed while on message.
Then remove access.
**Trade-offs** — The process is slow and emotionally taxing, and it costs the option of an instant reaction.
It is the only version that survives scrutiny and leaves the person able to act on their next opportunity.

---

## Part 3 — Technology and delivery (Ch 8–9, 11)

### Give yourself outs

**When** — Any decision that could become permanent — vendor, language, framework, platform.
**How** — Prefer choices with alternatives and portability (the JVM across Oracle, IBM, OpenJDK, Amazon and
across Linux, Windows, and three clouds). Model what happens when license costs rise. For gray-area
platforms, apply the test: can you lift the code and run it elsewhere?
**Trade-offs** — "It is hard and sometimes more expensive to take a path that gives you more future
options." Treat it as an insurance premium: "infinitely better to manage and fund small, incremental
upgrades or updates than huge, wholesale rewrites."

### Build vs. buy, by checklist

**When** — Management believes buying delivers overnight and your engineers have not-invented-here syndrome.
**How** — Start by assuming you will build everything, then break the problem into pieces and let buy
candidates present themselves. Write must-have and nice-to-have requirements as a punch list, shortlist to
two, and make the vendors build proofs of concept. Score the decision on a Build↔Buy axis across reaction
speed, infrastructure appetite, existing team fit, IPR importance, data accessibility, who will operate it,
and time to market.
**Trade-offs** — Research and evaluation "can take months and be a huge time sink." Buying shrinks your moat;
building costs product management, time, and in-house support forever.

### The 10 cloud warning signs

**When** — Someone says "we're in the cloud" and the bill or the release process suggests otherwise.
**How** — Check for: a flat monthly bill; downtime on release; inability to scale; running out of disk;
losing data; assuming servers never fail; running your own load balancer; data-center security habits;
hot-swap DR; multiple services per server.
**Trade-offs** — Fixing these means rearchitecting, not reconfiguring, and cloud-native design demands skills
the team may not have. The status quo cost is that "many cloud installations can cost far more than the
equivalent hardware setup."

### Design DR from three questions

**When** — You have a non-cloud-native enterprise whose absence would seriously hurt clients.
**How** — Decide downtime tolerance (having priced the consequences with the CEO), full or partial service
(read-only tweets, browsable-but-not-checkout carts), and fail forward vs. bounce back. Choose hot or cold
standby. Write the switch trigger criteria and the switch logistics — credentials location, cold-start
order, every DNS record and its TTL, validation roles and tests, and the reverse plan. Check weekly, rehearse
fully every quarter or six months.
**Trade-offs** — Real DR is expensive, complex, and constantly aging; a partial solution may have to become
the full one after fire, flood, or theft. Untested DR is "a safety net that isn't tethered to anything."

### Classify data through four lenses

**When** — You do not know who owns what data or how long you keep it.
**How** — Bucket data as system of record, sensitive/secure, transactional, business intelligence, and log.
For each bucket define ownership (much of it you only steward), retention including backups and aging
triggers, visibility including what changes when data moves between buckets, and compliance by jurisdiction.
**Trade-offs** — Classification is unglamorous and slows down "just store it all." It prevents both ethical
and legal blindsiding — and prevents the discovery that data you thought was an asset was never yours to sell.

### Microservices as deferred decisions

**When** — Parts of the enterprise must evolve at different speeds, or you want to postpone a technology bet.
**How** — Define a strong, consistent API around each narrow responsibility, each with its own data store,
runtime, and release cycle. Let the implementation move underneath — local directory to SAN to S3 — without
consumers noticing. Arrange teams around units.
**Trade-offs** — Without tight control microservices "evolve into a series of mini monolithic apps." Monolithic
remains correct for proofs of concept and startups, where the point is that it works at all.

### Open source with guard rails

**When** — Your team is using, publishing, or copying code.
**How** — Publish the list of acceptable licenses (Apache/MIT/BSD-style). Require libraries be used whole, never
lifted. Register every third-party library with name, URL, license type, and consuming component. Rewrite
anything over 5–10 lines copied from the web. Get CEO and legal sign-off before publishing, and commit to
maintaining what you publish.
**Trade-offs** — Registration is ongoing overhead nobody enjoys. It is also a standard due diligence request —
and the author has "not come upon a team that has this information at their fingertips."

### Productionalize delivery

**When** — Releases depend on one person or machine, or new hires take weeks to become productive.
**How** — A project manager past five engineers. Tickets sized under four hours. Coding standards enforced by
IDE plugins and SonarQube, plus real code reviews. Version control with a named branching pattern. Separate QA.
A CI/CD pipeline — commit, code analysis, compile/build, unit tests, security tests, deploy — starting as simply
as Commit > Build > Deploy. Blue-green releases. 10–30% of output on technical debt.
**Trade-offs** — Every element is upfront cost against features, and process can suffocate a small team. The
counter-example is the SaaS company whose sale price was reduced to fund the repairs.

### Filter client requests

**When** — A large client asks for something only they need.
**How** — Treat every request as if it were for all clients. Ask whether others would value it, whether it can be
generalized, and whether the underlying need can be met another way — data import/export, an API, or a plugin
pattern.
**Trade-offs** — Saying no to a big client is uncomfortable and the business will push. The alternative is
`if (Client == XYZ)` in the code base forever, or worse, a branched repository doubling your workload.

### Document as you go, in the audience's language

**When** — Continuously; documentation "left to languish quickly becomes a massive undertaking."
**How** — Pick one web-based tool meeting the seven criteria. Create templates. Ask the person who runs a system
to write a going-on-vacation email and file it as the owner's manual. Record demos and meetings. Layer code
comments as what (Javadoc, Swagger), how, and why. Draw the architecture at state-highway level, then per
component. Validate everything by having someone unfamiliar run it.
**Trade-offs** — Documentation competes with delivery every single day and decays without maintenance. The
alternative is document debt: nobody can rebuild the enterprise from source control.

---

## Part 4 — Operations, contracts, and transactions (Ch 10, 12–14)

### Contract with counsel, digitize your copy

**When** — Any service agreement, SOW, SLA, or license crosses your desk.
**How** — Write the SOW's eleven sections, especially when IPR assignment occurs. Have an NDA pre-validated and
ready. Read someone else's SLA arithmetic (AWS EC2's 99.99% is about six minutes a month for a 10% credit). Build
your own SLA from seven inputs, never a number plucked from thin air. Let legal write the legalese; keep a digital
copy while the CFO keeps the signed paper.
**Trade-offs** — Legal review adds time and cost to every engagement. Skipping it is how a private equity firm
walked away from a deal because the target's outsourced development had never assigned IPR.

### Buy support for the gaps

**When** — Deciding what to cover and what to self-support.
**How** — List every component critical to customer service and continuity and ask who you call when it breaks —
and what happens if that person is unavailable. Run the four risk questions per component. Always cover hardware.
Always cover unmodernized legacy. Re-examine at renewal.
**Trade-offs** — Full coverage is expensive and often unnecessary where you have real in-house depth. Under-covering
means the business cannot serve clients while you wait in a free-tier support queue.

### Patch on a schedule you actually keep

**When** — Always; and specifically after every release.
**How** — Maintain the software and version inventory. Subscribe to vendor notices, open source release notes, and
CVE/OpenCVE feeds — and periodically confirm the subscriptions still work. Update at least monthly, small and
frequent, tested through blue-green or a staging environment. Where you must patch production live, do it Saturday.
Include anything with an IP address.
**Trade-offs** — Monthly patching means monthly short-term disruption and testing effort. Falling behind means a
critical patch you dismiss as inapplicable because you never knew you were running the component.

### Identify, protect, monitor

**When** — Continuously; the attack surface changes with every release and new service.
**How** — Identify and shrink the attack surface with cheap moves (isolate production, individual credentials).
Protect in layers — the Swiss cheese model — consulting the business on where friction is acceptable. Monitor by
aggregating every log (OS, routers, firewalls, applications, VPN, network, authentications) into one place, and log
successful and failed logins.
**Trade-offs** — Each protection layer costs money and user friction, and no configuration is ever final. Relying on
a single line of defense means one dropped basket breaks every egg — as F5's own CVE-2022-1388 demonstrates.

### Prepare kill switches before you need them

**When** — Before an attack, not during one.
**How** — Decide in advance what you can seal off: disabling network access to a machine or subnet, powering off or
suspending. Then during an incident: don't panic, isolate, keep the executive team updated on the no-update cadence,
determine the damage, verify the attackers cannot resume, and hire a specialized security firm without embarrassment.
**Trade-offs** — Kill switches buy time rather than solving anything, and pulling one may take business offline. The
payment provider without one leaked sensitive data across its whole platform for nearly a day.

### Report incidents in IIIR

**When** — After any breach or security incident.
**How** — Issue (what happened and how), Identify (how it came to your attention), Impact (the result), Remedy (the
steps taken). Write facts, not plans. Keep it internal, as the source for any public statement.
**Trade-offs** — Writing plainly exposes exactly how the hole existed. That is the point: it is the source of truth
the executive team decides from, and it prevents a public statement built on guesswork.

### Monitor from outside and inside

**When** — Before a client tells you something is down.
**How** — Outside: emulate a client from a different cloud or region, escalating from page availability to
authentication to a low-overhead database query, every 5–30 minutes, alerting through an alias on infrastructure the
outage cannot take down. Inside: CPU, memory, disk, plus every application log piped to a central service; establish
normal so you can spot abnormal; sync all clocks to UTC.
**Trade-offs** — Building and tuning dashboards is trial and error, and alert noise is its own failure mode. Being
told by a client "inspires no confidence in you or your company."

### Restore, don't just back up

**When** — Annually at minimum, more often if you can.
**How** — Decide what and why you back up, set frequency by how often data changes, keep archives encrypted on a
separate network or account, back up third-party services too (GitHub, Bitbucket, Salesforce, Drive, Office 365,
Dropbox), grandfather retention (daily for a week, weekly for a month, monthly after), separate archive access from
archive readability via encryption keys, and log every backup so failures alert. Have new hires perform the
restoration in a clean environment.
**Trade-offs** — Fire drills consume a working day and can be nerve-wracking. Skipping them leaves you with "merely a
storage utilization strategy."

### Time projects against the transaction horizon

**When** — Considering any project of a year or more where a transaction may be coming.
**How** — Ask the CEO, and the operating partner if you have one, about the likely window. If the project cannot
finish, restructure milestones so there is delivered value with a clear path to the finish — plumbing and electrics
done, kitchen semi-usable. Build heavy contingency for the diligence work that will pull you away.
**Trade-offs** — You may defer a project the platform genuinely needs, or start one you cannot finish. Touring buyers
through a construction site invites them to discount for work they will have to complete.

### Due diligence as two role plays

**When** — Evaluating a company's technology, on either side of the table.
**How** — Establish the "why" first. Role play one: be a new customer and let them demo and onboard you — learn their
vocabulary, learn which features clients actually use, and probe naturally ("where does that file actually go?").
Break for lunch and thank them genuinely. Role play two: be a new engineer being onboarded. Track coverage on a bingo
card. Celebrate rather than judge.
**Trade-offs** — It takes far longer than a questionnaire and demands you suspend criticism you can see is warranted.
It produces information a spreadsheet never will, and it builds a relationship with people who may become your team.

### Report consequences, not findings

**When** — Presenting diligence results to executives, bankers, lawyers, investors, and accountants.
**How** — Four sections: overview of target; methodology (when, who was present, what was provided, what was not);
summary findings ordered by severity of consequence, with a separate list of things worth celebrating; supporting
materials. Translate every technical fact into its consequence, effort, and cost. Score red/amber/green. Never flag a
Coca-Cola-vs-Pepsi preference. Pre-review with the people you interviewed.
**Trade-offs** — Translation strips detail technologists value and the pre-review costs a round trip. It is what lets
non-technical readers act, and it removes any chance the interviewees are ambushed.

### Hand over, then take over, deliberately

**When** — A CTO transition in either direction.
**How** — Handing over: communicate the constraints behind each decision, walk each team member from their last
review, outline your vision anyway, and open your calendar to the last 30 days focusing on meetings outside your
department. Taking over: do nothing for 30 days, sit in and observe, run one-on-ones opening with "if you were
king/queen for the day, what would you change?", offer no solutions, and use the honeymoon period to ask peers basic
questions.
**Trade-offs** — Both halves feel like inaction while people expect leadership. "You can be held responsible only for
the moment, not the future in your absence" — and change without context is how good decisions get undone.

### Run You, Inc.

**When** — Continuously, because nobody else is doing it.
**How** — Ask your CEO to review you on communication, level of detail, business intelligence, and initiative/vision.
Create a channel for your team's feedback on you. Find a mentor with no stake in your career — and more than one, by
area. Update your résumé every six months as a journal. Follow three industry sources and act only when a theme
recurs across them. Prepare a successor quietly over years.
**Trade-offs** — Asking for feedback invites answers you may not want, and preparing a successor requires the
confidence to make yourself replaceable. "There is only one person responsible for your success, and that is you."
