# Chapter 9 — Development

## Core Idea

Every developer has their own way of doing things, "their own mark they like to leave on
something." Building a team means stepping outside that world: "What was clever and cool one day
is a pain to maintain and adapt for someone else." The chapter's word for the transition is
**productionalized** — turning project planning, coding standards, version control, QA, CI/CD,
technical debt, and releases into a professional, repeatable, reliable process.

The success test: "one where you can bring on new resources and make them productive in hours or
days instead of weeks as they wrestle with the idiosyncrasies of an antiquated coding process."

## Frameworks Introduced

- **The project manager's remit** — seven responsibilities.
- **The ticket system discipline** — the four-hour ticket rule.
- **Waterfall vs. agile**, with the Miami-to-San-Francisco illustration.
- **Version control branching patterns** — Gitflow and GitHub flow.
- **Two testing modes** — manual and automated, plus code coverage targets.
- **The six-gate CI/CD pipeline** — commit, code analysis, compile/build, unit tests, security
  tests, deploy.
- **Two release strategies** — outage release and blue-green.
- **The client-request filter** — three questions before building anything client-specific.

## Key Concepts

- **Why technologists miss deadlines.** "We are eternal optimists, believing things are never as
  complicated as they actually are. Generally, we are right; however, what we don't allow for is
  all the external forces that come in to disrupt a perfectly good deadline."
- **Hire a project manager past five people.** "As soon as the engineering group is more than
  around five people, a dedicated project manager is a must." The unacceptable excuse is "we don't
  need one because the CTO knows how to project-manage." Scrum masters are treated as having the
  same goals.
- **What a PM is not.** "Contrary to popular belief, a project manager is not a task master or
  there to hold people accountable — that is the role of the leads."
- **The six-hour working day.** Project managers "know there really are only six hours in a typical
  working day (factoring in breaks, lunchtimes, restroom visits, daydreaming, and so on)" — and
  that someone returning from two weeks on a beach needs time to come back up to speed.
- **Ticket size: no more than four hours.** Bigger tickets get split. The bonus: at the halfway
  mark someone who isn't going to finish has a natural moment to stop and ask for help — maybe they
  are overcomplicating it, or the setter didn't appreciate the complexity. "You never want to be in
  a situation where a ticket takes multiple days or weeks to complete."
- **Tickets as a shared language.** "It gives me such joy to hear a team toss ticket numbers around."
  A ticket number lets anyone look up the background, comments, and associated code and make a more
  informed contribution. Well maintained, the system becomes a living knowledge base — which requires
  whoever closes a ticket to record the steps to resolution.
- **Write project outcomes in outside-engineering language.** Internal: *"Upgrade the customer intake
  process, removing the reliance on Windows XP, and rewriting it with Java running on a serverless
  environment."* External: *"Modernize the customer intake process, rewriting it for the cloud for
  greater flexibility and increased throughput."*
- **Agile is not the absence of planning.** "A common misconception about agile is that it equals a
  lack of planning, which couldn't be further from the truth" — milestones (epics) become tasks,
  tasks are arranged into one- or two-week sprints, each ideally delivering functionality others can
  interact with, followed by a review.
- **Sacred code blocks.** "Every company has its 'sacred code blocks' that, over the years, have
  taken on a mythology of their own — a perceived complexity — and as soon as anyone suggests
  rewriting them, there are long, deep intakes of breath."
- **Repository granularity is an art.** Ask: does it have a defined function or role? Is a different
  or narrow skill set required to maintain it? How does it relate to other components? How is it
  released? "Try not to have too many repositories, and conversely, don't have a single one that
  encapsulates everything" — the Goldilocks problem.
- **Engineers make poor testers of their own work.** Hence separate QA teams inside engineering, away
  from the source code, testing as an end user would.
- **Code coverage reality.** "It is nearly impossible to achieve 100% code coverage; if you are
  getting anywhere between 50%–70%, you are doing very well." And coverage should concentrate on the
  modules that matter most — the ones that run most often or provide a crucial service.
- **Technical debt budget: 10%–30%** of your team's output devoted to paying it down. "A good CTO
  knows the level of technical debt their group is carrying at any point in time." A good tip when
  working in sprints: a rest period every two or three sprints to pay debt down.
- **Database schema rule for releases: always add, never drop.** Dropping makes rollback impossible
  without resorting to backups. Drop tables, columns, and indexes in a much later release, once you
  are confident you never need to go back.
- **Release notes always.** A basic text file, a wiki, or an Atlassian Confluence page; some ticket
  systems tag tickets or sprints to a release and produce notes automatically. Useful because the
  business may contextualize the raw list for the end client.

## Mental Models

- **The drone in a gust of wind.** A project manager is like drone software analyzing all inputs and
  correcting the flight path in real time — projecting what happens if the current path is followed
  and feeding it back to you to take corrective action.
- **Miami to San Francisco.** Waterfall: the team can't start driving until the complete route is
  mapped. Agile: as soon as you know how to get out of Florida, the team starts driving to the
  Floridian border while the next phases are planned.
- **The unserviced car.** Skipping oil, filters, and tires saves money short-term while incurring
  "car debt," until the engine locks up or tires blow on the highway at much greater cost. Technical
  debt behaves the same way — and is best thought of "like using a credit card: it's okay to put the
  short-term cost on the card, but be sure to pay it off at the end of the month."
- **The 10,000-test car** (Glen Martin, former Product Manager for the Java Enterprise
  specification). See the Worked Example.
- **Out of the frying pan, into the fire** — the developer habit of rewriting whatever they don't
  understand.

## Anti-patterns

- **Un-productionalized teams.** A SaaS company with a great product whose CTO laid down no standards
  and enforced no formal release policy: "a Wild West of code and libraries, with no security to
  really speak of," with the release process depending on a single person (the CTO). The technical
  debt was severe enough that **the company price was devalued to allow for the repairs.** "The CTO
  was simply not equipped with the right knowledge to grow the team beyond their own development skills."
- **"Only Fred can update that service."** Relying on a single machine or person to compile, package,
  and deploy production code. One portfolio company's responsible person let the Windows machine
  update to the latest .NET libraries, breaking the build — "the company now could not update its core
  product." Short-term fix: a virtualized build environment.
- **No version control**, justified by "we don't need it," "the language doesn't lend itself to
  checking in and out," or "it would only slow us down." "All the excuses... are completely invalid.
  What it comes down to is fear of the unknown."
- **Developers who believe their code is untouchable** and needs no review. "Beware of these types,
  root them out quickly, and, if you can't get them to change, let them go."
- **Testing fatigue.** A section that always passes gets skipped on the assumption it will work —
  "a dangerous situation because it is hard to spot."
- **Disabling a failing test to hit a deadline.** You convince yourself the failure is new business
  logic, a safe false negative. "Days pass. Weeks pass. Before you know it, the tests are never
  reenabled or resolved."
- **"Well, it works on my machine"** — the excuse a CI/CD pipeline removes.
- **Going radio silent during a release** when things run long or go off plan.
- **`if (Client == XYZ)`** — "the surest and quickest way to create technical debt. It leads to a
  complicated code base, creating areas that are really hard to test, and saddles you with code that
  you may never get the opportunity to change because the client will be depending on its existence."
- **Branching a whole repository for one client.** "The most dangerous situation and sets you up for
  sure-fire failure. All that has happened is you have doubled your workload on all fronts."
- **Agile as bureaucracy.** "Some organizations get themselves so wrapped up in the rules that they
  suck all enjoyment and flexibility from the project."

## Reference Tables

### What a project manager gives you

| Contribution |
|---|
| Getting a handle on the output of each resource |
| Identifying milestones and deliverables |
| Predicting and continually improving accuracy of deadlines |
| Coordinating release cycles |
| Managing handoffs between milestones and communications with internal/external teams |
| Providing you with actionable items, based on data, to help improve your time management |
| Reporting the state of all items, making sure nothing gets lost or falls off the radar |

Where there isn't enough work for a full-time PM, it is common for them to step into the **business
analyst** role, defining the details of a project from the business perspective.

### Waterfall vs. agile

| | Waterfall | Agile |
|---|---|---|
| Premise | Plan as much as possible up front; everything starts at the top and flows down | React to a changing environment in incremental steps toward the outcome |
| Strengths | Easier to predict time and cost for a well-defined project; good position for putting the work out to external company bids | Change is much easier to incorporate; you see whether something will work before too much time is wasted; continuous delivery involving all stakeholders including the end client or user, so issues are found and fixed sooner |
| Weaknesses | Up-front definition time, especially for large projects — no work starts until definition ends; poor at coping with change (technology or requirements no longer relevant, new feature requests). "It is not uncommon for projects to be delivered that no longer fit the purpose they were designed to do, with outdated technology" | Can be very process oriented and intimidating at first; teams can over-apply the rules |
| Artifacts | Much more up-front documentation — wireframes, mockups — produced by subject matter experts, business analysts, architects, project managers | Epics → tasks → sprints (typically one or two weeks) with a review after each |

Reference the book gives for deeper comparison: https://www.guru99.com/waterfall-vs-agile.html

### Version control branching patterns

| Pattern | Structure |
|---|---|
| Gitflow | Two main branches: **master** (all code here is deployable) and **develop**. Development happens on a feature branch cut from develop, reviewed, then merged back into develop. A release is a merge of develop into master, pushed out. **No code is committed directly to either master or develop** |
| GitHub flow | A simpler version with only a main **master** branch. Development branches off into a feature branch which, once reviewed and tested, is merged into master ready for deployment |

Within a pattern you define: integration with your ticket system, the naming convention of feature
branches, the granularity or lifetime of a branch, and the release cycle.

### The CI/CD pipeline gates

| Gate | What happens |
|---|---|
| Commit | The trigger — usually a successful commit/merge of code to a given branch, typically develop after a pull request merge |
| Code analysis | The branch's source is run through quality tests using a tool such as SonarQube |
| Compile/build | Code is compiled (if applicable) and built into a deployable artifact |
| Unit tests | The built code is put through unit tests to ensure no bugs were introduced |
| Security tests | Any tests or checks aimed at maintaining a high confidence of security |
| Deploy | Deployed to a server environment — development, testing, or, depending on the branch, production |

Tooling: GitHub, Bitbucket, and CodeCommit all offer some form of pipeline support; Jenkins for
orchestration across multiple repositories and services. Starting advice: "do not be too clever or
overreach straight out of the gate — a simple Commit > Build > Deploy has served many an
organization just fine."

### Common issues that stop a build

| Issue |
|---|
| Compilation error |
| Missing library dependency |
| Code compliant/standard violation |
| Test failure |
| Security scan failure (leaving in passwords or hardcoded keys) |
| Database misalignment |

Release frequency once the pipeline is right: "Some teams do a couple of releases a week, some once
a day, or some even many times a day (one large Fortune 100 even performs thousands a day across
their infrastructure). If you get your pipeline right, there is no limit."

### Outage release planning checklist

| Step |
|---|
| Determine what needs to be taken offline: which servers, components |
| How will this look to end customers — will you need a maintenance page? |
| Calculate a time frame including: time to shut down; time to update software and hardware; time for starting back up; time for pre-user testing; time to enable users to go back in |
| For each area, detail the precise steps required to achieve success |
| Allocate a resource to each step and the hand-off process to the next |
| Talk with the business to come up with potential release dates, preferably at the lowest usage period |
| Determine the list of those needing progress reports and those to be notified when the release has been done |

Padding rule: "if the business can tolerate a weekend outage, yet you think you need only 2–4 hours,
take the weekend." Finish with a postmortem "to celebrate what went well and learn from what went
wrong or could be done better next time."

### Client request filter

| Question | Consequence |
|---|---|
| Is this something others would see value in? | If so, it should be on the road map anyway |
| Does it need extra definition or discussion to make it appealing to a wider audience? | Reshape it before building |
| If it doesn't make sense for everyone else, is there another way to achieve what they wish? | Data export/import so they can build something external, possibly against your API; or a plugin pattern for customizable systems, extending without touching the core |

"If you find yourself ever uttering the phrase 'It's just for this one client/feature,' it's time to
snap out of it! Your future self will thank you."

## Worked Example

### Blue-green release, end to end

The historical model was replacing code on a server — physically replacing files on a server
filesystem — which carries risk and makes rollback hard, "because no matter what, something changed
on the filesystem."

Virtualization changed that. The blue-green sequence:

1. **Create a completely new environment**, loaded with new software, fully configured and ready to
   go. With something like Docker this can be scripted to happen quickly.
2. **Test it while the current architecture continues to service live traffic.**
3. **Direct traffic over to the new "release" environment** once confirmed good.
4. **Tear down the old environment** when all traffic has moved and let the new one take the load.
5. **If things are not right, tear down the new one and try again** — no harm done to existing traffic.

Side benefit: it continuously tests your infrastructure-creation scripts. "The more complex your
architecture, the harder this technique is to master, but not impossible. It is ideal for web-style
applications."

The one hard part remains **database schema changes** — always add, never drop.

### The 10,000-test car

Glen Martin, who served as Product Manager for the Java Enterprise specification, asked the author
whether he would rather drive a car that had **10,000 tests run on it** or one that had **only one
test**.

"Naturally, as a fresh-faced engineer, I answered the car with 10,000 tests, of course. Yet, I fell
into the classic trap of assuming quantity equaled quality. He then noted that the car with 10,000
tests did not include one that turned on the engine, but the one test the other car underwent was
turning on the engine."

"A simple and obvious example, but after nearly 25 years, it stayed with me." The rule it teaches:
weight coverage by what the code actually does for the business, not by test count.

### The week of lost work at the United Nations

Early in the author's career, on an engagement with the United Nations in Rome, writing Java Servlets
to process satellite photos of agricultural land in Africa. The existing system was Perl CGI scripts
letting academics access the photos via the Mozilla browser; the team was invited to trial the new
Java Servlet standard under real load.

There were **no coding standards, no processes, and no version control** — not uncommon 25-plus years
ago. He had made his own backups as he went, on a separate network directory, thinking he was being
clever. Rushing to finish a section before a flight home, he ran a test; part of the process deleted
temporary files (disk space was a real premium). Something was wrong in the rush, and it **deleted all
the source files, both locally and from the backup network drive.** Nothing could be restored.

"I had deleted my entire week's work. I had no choice but to stay over the weekend and rewrite
everything. There were so many lessons I learned that week that, to this day, I have never had a
repeat occurrence of this issue."

### Testing the testers

Some CTOs, while rotating new recruits through the QA team, **purposely give them failing areas
instead of real tests** to see whether they pick up on them — anything from outright failing
functionality to areas that don't quite conform to the standard, be it a process or the look and
feel. "This practice may be sneaky at first blush, but it instills a level of detail and confidence
in both the process and the new recruit."

The broader practice: the test team is a good place to rotate new developers through before they are
let loose on code — it familiarizes them with the product and shows them the level of testing the
group does, "so they know they can't let just anything slip through."

### The automated-test chicken and egg

"Say you have a test to delete a person's record. Part of the test is to delete the record and then
confirm it was deleted. Easy? Yes, but now we need to create the person record in the first place to
be able to delete it. Vicious cycle — welcome to testing."

## Key Takeaways

- Timelines are hard because of external and unknown events; smaller components (agile) make
  prediction easier.
- A well-maintained ticket system gives the whole team a shared language and a real-time view of work.
- Past ~5 engineers, a strong project manager is "worth their weight in gold."
- Basic development standards — formatting, design patterns, function density, class hierarchy, naming
  conventions — make code readable and therefore maintainable; enforce them with IDE plugins and build-time
  tools like SonarQube, and hold regular code reviews where no one's code, including yours, is beyond
  reproach.
- Version control is non-negotiable; pick a branching strategy that enables parallel streams without
  complicated timing.
- Testing is hard, but "something is better than nothing"; prefer quality of coverage over quantity of tests.
- Recognize technical debt first, then budget 10%–30% of output to pay it down.
- Releasing need not be risky — the secret is planning, practice, and communication; blue-green removes
  most of the remaining risk.
- Welcome client feedback but filter it, so the code base doesn't become a fractured mess.

## Connects To

- [Chapter 8 — Technology decisions](ch08-technology-decisions.md) — the microservices, cloud, and
  open source choices this process delivers.
- [Chapter 6 — Team management](ch06-team-management.md) — the lifecycle-centric team structure that
  mirrors this pipeline, and the reporting metrics drawn from JIRA, SonarQube, and Jenkins.
- [Chapter 3 — Visionary planning](ch03-visionary-planning.md) — the vision the projects serve and the
  communication plan around slippage.
- [Chapter 11 — Documentation](ch11-documentation.md) — deployment process, source/code comments, and
  the release notes habit.
- [Chapter 12 — Security](ch12-security.md) — securing the build process and creating secure code.
- [Chapter 13 — Housekeeping](ch13-housekeeping.md) — the maintenance that keeps technical debt from
  compounding.
