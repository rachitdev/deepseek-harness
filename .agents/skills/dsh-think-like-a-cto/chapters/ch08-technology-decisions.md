# Chapter 8 — Technology decisions

## Core Idea

"Although we like to think we are in a fast-paced industry, always changing, always evolving, the
reality is that production systems can last decades without any significant change. This is quite
a sobering thought — the system you are developing now could still be running when you are looking
to retire."

So every technology decision is a decision about the *far* future, and the governing test is
**leave a system the next person can build upon**: "The last thing you want is for the next person
to dismantle everything you did and start again."

The chapter is a sequence of decision frames — lock-in, build vs. buy, cloud vs. on-prem, disaster
recovery, data management, microservices vs. monolithic, open source licensing, and language and
framework selection — each with its own checklist.

## Frameworks Introduced

- **The four aging questions** for any technology choice.
- **Build vs. buy** — advantage/disadvantage tables plus a weighted evaluation checklist (Table 8.3).
- **Top 10 cloud warning signs.**
- **Three cloud mantras.**
- **DR's three foundational questions** — downtime tolerance, full or partial service, fail forward
  vs. bounce back — plus hot/cold standby and the switch logistics.
- **Five storage types and five data types**, each data type viewed through four lenses (ownership,
  retention, visibility, compliance).
- **Microservices vs. monolithic checklist** (Table 8.4).
- **Two open source license families** — copyleft and permissive.
- **Five questions for choosing a language or framework.**

## Key Concepts

- **Why systems last decades:** the technology stack does not lend itself to incremental updates
  easily; leadership doesn't wish (or know how) to touch the sacred cow; leadership has no
  experience of anything but the current technology; it becomes so critical that replacing or
  upgrading is too high a risk or cost; hubris — "it was built perfectly, so how can you improve on
  perfection?"
- **The oldest systems the author has seen:** 46 years old (IBM RPG via green screen terminals);
  commonly 30- to 40-year-old Cobol; many 30-year-old 4GL GUI systems in Visual Basic, Gupta,
  Delphi. "No matter their age, they are key to the success of the company."
- **Lock-in as an insurance policy.** "Should something happen, you have the necessary provision to
  cash in your policy and save yourself greater heartache." Java is the cited counter-example — a
  JVM (Oracle, IBM, OpenJDK, Amazon) abstracts hardware and OS, so the same code runs on Linux,
  Windows, Amazon, Google, or Microsoft.
- **Plan for license cost increases.** "Organizations often regret purchasing Oracle/SQL Server
  database licenses only to see prices increase over the years, with reduced flexibility (e.g., the
  inability to move into the cloud), all while realizing they aren't using anything specific to the
  vendor."
- **Where to build:** where your IPR or secret sauce lies — the part unique to you that sets you
  apart. "Anything outside of this is a distraction: solving problems that have already been solved."
- **Moat.** "A term investment groups use to determine how hard it is for competitors to copy what
  you are doing. If you are buying third-party tools, then nothing is stopping your competitor from
  doing the same thing." Corollary warning: if a bought solution works exactly as you want with
  minimal configuration and gets you to production quicker, presume a competitor can do the same —
  "an indication that you may not have as much IPR in your technology as you may like."
- **Cloud vs. on-prem definitions.** Cloud: CPU, memory, network, and storage consumed and charged
  per usage, sometimes down to the second, with no consideration of the underlying physical hardware.
  On-prem: that hardware owned and managed by the company. **Cloud fits** applications with peaks and
  troughs (hourly, daily, seasonal); **on-prem fits** intensive computing where the CPU runs 90%-plus
  most of the time — complex data or graphics modeling, or bitcoin mining.
- **Cloud native** — an app specifically designed to harness on-demand resources, scaling up and down
  automatically. The electricity analogy: plug into the wall, draw what you need, get billed monthly;
  "cloud native" is the standardized three-pin plug.
- **Private and hybrid cloud.** Provisioning your own hardware with a cloud OS unlocks per-usage
  management and extra life from existing investment — recommended only when you have the resources
  to manage the pool. Hybrid: the public cloud augments the shortfall transparently when private
  resources are exhausted.
- **DR reality check.** "Most organizations have a poor (read 'nonexistent') DR strategy, and the
  ones that think they have a good one have never truly tested it, with it serving as no more than a
  placebo, a safety net that isn't tethered to anything." And: "if using only a single zone, as soon
  as you are in a failed state... you no longer have a backup for the backup."
- **Latency governs DR.** Even 50–100 ms makes a huge difference to keeping databases synchronized.
  Watch for **peering arrangements between data centers** when choosing DR locations. Database design
  matters too: "if you are relying on database-generated, autoincremented, primary keys instead of
  GUID, then you have a struggle on your hands."
- **Data-lake-as-attic.** "The place that has all the things you can't bring yourself to throw out,
  because one day, you just might need them... At the very least, your organization needs to have
  some sort of data lake as part of your storage solution."
- **Split databases by the business area they serve**, with replication between them — "that way,
  when the CFO asks to run a huge report to figure out the end-of-quarter sales, they don't take the
  website down in doing so."
- **The microservice as deferred decision.** A File MS with two API calls (store, retrieve) can start
  by writing to a backed-up local directory, grow into a network SAN, and end up on Amazon S3 —
  without the rest of the enterprise noticing. "A microservice is like a code library but at a much
  higher granularity."
- **AWS as proof.** "AWS is just a series of microservices, all exposing their functionality through
  APIs" — grown out of the need to power Amazon's ecommerce sites, then sold externally.
- **Framework expertise is as important as language expertise.** Developers who specialize in a
  framework, while understanding the underlying language, may struggle to move to a different
  framework in the same language. Frameworks fall out of favor faster than languages.
- **Avoid single-company languages.** "If the primary company gets bought or goes bust, then the
  language is left to die on the vine" — Gupta, PowerBuilder, Delphi.
- **Language horizon: five years.**

## Mental Models

- **The poker hand with outs.** "A good poker player will never chase a single card, hoping to make a
  hand out of nothing. They want to know they have a number of hands at their disposal as the cards
  unfold on the table. Enterprise planning is the same."
- **Honey production / the genetically engineered bee.** Nature masters honey by not relying on any
  single bee — failure is built in, and one bee's death halts nothing. "This is how a good cloud-native
  application is designed — failure is part of its DNA... We have too many genetically engineered
  bees in our world."
- **The dimming light bulb.** On partial service: "you never want it to go out and leave you in
  darkness, but most likely, you can tolerate the light dimming for a time."
- **The server as a plant** needing sunlight and water — here power, cooling, and a network connection.
- **Godzilla.** The author's first mental image of "disaster recovery"; the real events are fire,
  flood, theft, and most likely of all, power, network, or hardware failure.
- **Enterprise software is not Microsoft Word.** "No off-the-shelf software will run the business. If
  there were such a thing, the business wouldn't need a CTO." The average restaurant can use online
  services (with DoorDash and Uber Eats integrations); McDonald's, running franchises worldwide,
  needs a custom solution — and a CTO to run it.
- **Whiteboard flawlessness.** "Just like with software development, where the whiteboard is the only
  platform where everything works flawlessly, their solution will always be the best, fastest, and
  quickest to configure before signing the contract."

## Anti-patterns

- **Choosing platforms or software that offer no alternative** — including back-office environments
  like Office 365, Google Workspace, and Salesforce, though for some applications this may be an
  acceptable trade-off with large vendors who don't make disruptive price changes.
- **Low/no code as a shortcut.** "The modern-day versions of Visual Basic... These tools lull you into
  thinking you have 80% of your work completed in a short space of time, but you can then spend way
  more time trying to close the last 20%." Worst case, you make compromises to justify the tool —
  "will your competitor compromise, too, or will they leap-frog you with a richer feature list?"
- **Treating the cloud like a data center.** "Many cloud installations can cost far more than the
  equivalent hardware setup would have been, simply because of a fundamental misconception of what
  the cloud really is."
- **Microservices that drift into mini monoliths** through accumulated responsibility — "it is in
  name only."
- **All-NoSQL architectures.** Teams swing too far and data inconsistency creeps in — the same field
  stored as `"2"` (string) in one document and `2` (integer) in another, because some process isn't
  adhering to the standard. "NoSQL does not enforce any data integrity or structure."
- **Copy-pasting from the web.** Just because it is on StackOverflow, a blog post, or an article does
  not mean it is free to incorporate. Rule of thumb: **anything more than 5 or 10 lines should be
  rewritten**, using the algorithm only for inspiration — "merely renaming variable names is not
  acceptable."
- **Self-hosting horrors.** The only server situated under a developer's desk and regularly used as a
  footstool, plugged into a shared power strip with no marking that it should not be unplugged. And
  the cleaner who unplugged a server to plug in her vacuum — "to this day, I never take power for granted."
- **Outsourced DR faith.** A client spending well over six figures a year with a three-letter company
  on DR software and hardware; questioning revealed it did not work as assumed. Rebuilt on AWS within
  a few weeks for **four figures** with a configuration that actually worked. "DR is hard and is not
  something that can be outsourced without getting involved."
- **DR that locks you out.** A portfolio company wrote all firewall rules with the primary zone in mind
  and locked themselves out of the failover zone. "The silver lining: the security worked really well!"
- **Index-everything.** A tech read that indexes speed up tables and created an index for every column
  on every table; "the database was drowning in its own metadata, storing more data about the data than
  the data itself."
- **Choosing the technology a proof of concept was written in** — "that doesn't generally scale too well
  when you need to productionalize it."
- **Claiming to use no open source.** "If I am doing due diligence on a development team, and they say
  they don't use any open source, I am instantly suspicious and will dig deeper than I normally would."

## Reference Tables

### Four questions for any technology choice

| Question |
|---|
| How will it age? |
| What happens if it becomes obsolete or no longer supported? |
| What dependencies does it need to function? How will those dependencies age? |
| Will there still be people wishing to work with the technology in the future? |

### Table 8.1 — Advantages and disadvantages of buying (as printed)

| Advantages | Disadvantages |
|---|---|
| Proven/tested platform | Don't own the IPR of the full solution stack |
| Reduced time to market | Vendor lock-in |
| Professional support network | Inflexible/limited feature set |
| If SaaS platform, then no scalability, security patching, or software updates to worry about | Can be harder to recruit for |
| | Bug fixes/feature releases constrained by vendor's schedule |
| | Competitors have access to the same tools |

Buy-side costs to factor: setup/license costs including annual or usage fees; professional services
to aid setup; if self-hosted, hardware costs and additional supporting licenses plus IT costs to
maintain hardware and software (backups, security updates); time to configure and customize; and
**whether the data can be easily and periodically exported.**

### Table 8.2 — Advantages and disadvantages of building (as printed)

| Advantages | Disadvantages |
|---|---|
| Complete control over product feature set | Product management, including design, architecture, implementation, and delivery |
| Control over the environment | Longer time to build |
| Recruitment is more mainstream | Greater effort to plan and resource |
| Own the complete IPR to the solution | Not as tested; specific |
| Quicker to react to bugs and feature requests | Support has to be provided in-house |

### Table 8.3 — Build vs. buy evaluation (as printed, positions shown as a Build↔Buy lean)

| Area | Leans |
|---|---|
| Need to react quickly to client's changes and feature requests | Strongly **Build** |
| Don't want to manage any infrastructure | **Build** side, near center |
| Would be advantageous to use the existing development team | Mid, slightly Build |
| The IPR of the solution is not important; not core to the business | Toward **Buy** |
| Data needs to be directly accessible by other systems | Mid, Build-leaning |
| The solution will be managed by nonengineering folks in the business | Toward **Buy** |
| Time to get solution to market | Toward **Buy** |

"If the overall checklist is tending toward a 'buy' solution, then this is where you focus your
investigation and solution. Again, be data driven, not gut driven."

### Who argues which side

| Camp | Background |
|---|---|
| Buy supporters | Little or no experience running a development team; believe what you lose in capability and flexibility is more than made up for in platform stability |
| Build supporters | Experience running development teams; know how to get things built for a specific job; get annoyed at the inflexibility of bigger solutions |

"Successful CTOs know how to use the best from both worlds, ruling nothing out, looking at each
requirement in its own right." The author's own method: **start by assuming you will build
everything**, break each problem space into manageable pieces, and let buy candidates and commodity
components present themselves as the requirements clarify.

Lock-in test for gray-area platforms (e.g. Salesforce): "if you can't lift your code and drop it on
another platform and just run it, you are locked in."

### The three cloud-resistance arguments and their answers

| Argument | Answer |
|---|---|
| "The cloud is not secure" | Advances in monitoring, network isolation, encryption management, container runtimes, and fine-grained access models have swung the pendulum. "You would be hard pressed to make on-prem as secure without a lot of cost and work" — but cloud does not equal security; the work still has to be done |
| "My data will be held hostage" | Data sits in your cloud vendor's stores, accessible through their tools and APIs, "no more inaccessible than it would be sitting on a local SAN." Large downloads cost network charges — "far from a hostage situation." The fear originates in an irrational fear of missing a monthly bill; "you have to go many (many) weeks before the larger cloud vendors will start arbitrarily deleting your data without warning" |
| "It's too expensive" | Yes — if you treat your cloud vendor like a data center. "A properly designed cloud-native application will have significant savings, year over year, especially as cloud costs continue to fall" |

### Top 10 cloud warning signs

| # | Sign | What it reveals |
|---|---|---|
| 1 | Your cloud bill is the same every month | Not using the elastic nature of the cloud — especially telling for a seasonal business |
| 2 | You require downtime when releasing new updates | Servers used in a static mode; modern practice replaces servers each release (containers, Docker), switches traffic when satisfied, and deletes the old ones |
| 3 | You're unable to scale up (or down) to cope with load | "You can't produce a baby in one month by getting nine women pregnant" — in the cloud you can; if you can't introduce another mother, the app is not cloud ready |
| 4 | You're running out of disk space | Static servers requiring manual maintenance instead of pay-as-you-go storage |
| 5 | You lose data | Single points of failure with no elasticity or redundancy |
| 6 | You assume servers are immune to failure | Too tightly coupled with no redundancy — hardware failures are rare but still occur |
| 7 | You're running your own XYZ, such as a load balancer | Load balancing, logging, caching, configuration are provider services now; running them yourself may show a lack of knowledge of available facilities |
| 8 | Your data center security practices in a cloud world | Manually configured firewalls on each server, or no real network separation, means missing a whole layer of provider security |
| 9 | Your DR still revolves around hot swaps | Redundant duplicate servers waiting for the day you hope never comes. "With a well-architected cloud, DR comes as part of the equation free of charge" |
| 10 | You have multiple services on a single server | The classic LAMP economics don't apply in the cloud; each server should be as light as possible, easy to replace, maintain, and scale |

### Three cloud mantras

| Mantra | Meaning |
|---|---|
| Security is not a given | Architect and design it into every part; assume no security just because you deployed the service. "Misconfiguration will get you every time, so test thoroughly" |
| Failure will happen | The cloud is not error-free. "Plan, build, and practice for failure." A well-architected cloud makes errors seamless without significant extra cost |
| It's easier to get into trouble | The cloud makes it easy to create *and destroy* resources. "The notion of a 'production' environment is just that — a constructional notion. The cloud provider assumes you know what you are doing and will happily oblige your request to remove the production database" |

### On-prem hosting essentials

| Concern | Detail |
|---|---|
| Hosting options | Office building (dedicated room, cupboard, or — not recommended but not unheard of — under a developer's desk) vs. remote data center in racks |
| The three needs | Power (clean, not overloading the circuit, uninterrupted), cooling, network |
| Data center pricing | Two primary metrics: physical rack space (rack units, or U's; **one U is 1.75 inches**) and bandwidth. Power is usually built into the price. "Pizza box" / rack-ready servers cost a little more than boxy containers |
| Self-hosting build-out | Routers, network hubs, optional firewall devices — which themselves need cooling and backup power. A lockable, well-ventilated room with dedicated power, logical layout, labeled and clamped cabling |
| Also maintain | Not just OS updates — the software consoles powering SAN, router, and server blade chassis hardware |

### Hardware inventory — the key metrics

| Metric | Detail |
|---|---|
| Purchase/installation date | When was it purchased, and when was it given power? |
| Purchase source | Where was it purchased, including full contact details |
| Configuration | Enough detail that you don't have to dismantle the unit — including free slots for future upgrades |
| Time to failure | Manufacturer guidance on expected life; "anything that has moving parts, such as fans and hard disks, is expected to fail in only a few years" |
| Access details | Does it have a console? Which network is installed, and where is the username/password stored? |

"It is so much easier to document these metrics at the time of installation, when you have all the
data at your fingertips, than it is years later."

### DR configurations

| Configuration | Cost | Behavior |
|---|---|---|
| Hot standby | Most expensive | Very little downtime; servers always up, continually synchronized, ready to take traffic. Consumes power and bandwidth and "is also running out the clock on your backup hardware lifetime" |
| Cold standby | Cheapest | Bare minimum kept running; machines are booted and synchronized when a failover event is initiated — the slowest, but "more than acceptable" for some situations |

| Post-event strategy | When |
|---|---|
| Fail forward | Like-for-like configuration: let the backup zone become primary. "Why incur another switch back when it's not required?" Focus then shifts to rebuilding the failed zone as the new backup |
| Bounce back | No like-for-like setup — common when in partial-service mode. You need not bounce back immediately: plan a second outage at a convenient time, usually over the weekend or late at night |

### DR switch logistics

| Step |
|---|
| Make sure all contacts, networks, and passwords are stored in a secure place that is immediately accessible |
| Determine which services need to be started up (if in cold standby) and populated with the most up-to-date data |
| Identify all network addresses needing updates — public and private DNS resolution; DNS changes take time to propagate depending on the zone records' time-to-live, so build this into the plan |
| Identify all roles, configurations, and tests needed to validate the recovery zone, and by whom |
| If bouncing back, write the reverse plan |

Testing cadence: **check weekly** that all data is still flowing where it needs to go; **practice a
full switchover roughly quarterly or every six months** so it is not seen as a big event. "The DR has
to evolve and should always represent the current state, because disasters are rarely telegraphed in
advance."

### Five storage types

| Type | What it is |
|---|---|
| Files/folders | A self-contained collection of data, named, on physical media. Formats like JSON, XML, CSV, TXT. Easily copied, encrypted, moved, shared; no limit to the type of data |
| Relational stores (SQL databases) | Common attributes (columns) relating to one another (rows) in tables; tables link to tables, a collection is a database, queried with SQL |
| NoSQL stores | Document stores that keep only the data you have, not placeholders for data you don't. A collection groups documents, and documents may differ radically. Examples: MongoDB, CouchDB, DynamoDB, Elasticsearch |
| Data lake | Large volumes of files of differing structure and size, for backup or future analysis; commonly a staging area for incoming data with no permanent home yet. Tools let you traverse the files (map/reduce). Examples: Apache Hadoop, Amazon S3 |
| Data warehouse | A database designed for analytics and business intelligence — large datasets arranged for fast reports and year-over-year analysis; data usually aggregated and anonymized. Examples: Amazon Redshift, SQL Server. "Only of use when a significant amount of data has been amassed" |

### Five data types

| Type | Definition |
|---|---|
| System of record | Data core to your company, usually in a database — the "truth" for a given entity |
| Sensitive/secure | Social Security numbers, driver's license numbers, bank information, credit card details |
| Transactional | Generated in pursuit or delivery of the core product; often called operational data |
| Business intelligence | Powers reporting analytics; usually a copy of transactional data, aggregated/anonymized |
| Log | Produced by the systems and software powering the enterprise; used for forensic bug/fault tracking, auditing, and performance analysis |

### Four lenses to apply to every data bucket

| Lens | Questions |
|---|---|
| Ownership | Who actually owns it? Ownership means you are free to sell it, delete it, or make it public. Many buckets turn out to be your clients'/users' data that you merely steward; data can also be rented for a limited period (mailing lists, analysis datasets) |
| Retention | How long will you hold it, in operational stores *and* backups? What is the aging policy to cheaper long-term storage? What business triggers delete all the data including backups — e.g. a client leaving you? |
| Visibility | Who sees this, internally and externally, and with what restrictions (encryption, access lists)? What changes when data ages from one classification to another — e.g. anonymizing user data moving from operational to analytics |
| Compliance | Does it fall under special compliance or regulation (PII, HIPAA, PCI) — with additional audit and security processes that may differ by geographic region? Personal, health, and financial data need special handling |

### Table 8.4 — Microservices vs. monolithic checklist (as printed)

| Characteristic | |
|---|---|
| Single code base, with no internal libraries | Yes / No |
| Single database that drives all systems | Yes / No |
| An outage causes everything to stop delivering | Yes / No |
| Single-server architecture | Yes / No |
| Upgrading software requires a planned outage | Yes / No |

**Three or more yes answers → you are most likely managing a monolithic architecture.**

### Open source license families

| Family | Examples | Effect |
|---|---|---|
| Copyleft | GPL | Any software utilizing a GPL must itself be released under a GPL, including all modifications. "If your secret-sauce algorithm is using GPL software, then you would be legally obligated to make your source code available to anyone who asked, including your competitors" |
| Copyleft, lesser | LGPL | You don't have to release your code as open source if you **dynamically** link rather than statically — much more business friendly than GPL |
| Permissive | BSD, MIT, Apache | Permit commercial software to use or extend open source and declare the work proprietary or closed source. Some require a public notice declaring usage (visible in the "About" section of e.g. Android phones) |
| Public domain | — | The most free: no ownership, no copyright, absolutely no restrictions. Not seen that often, "though code samples in books are often released under this type" |

### Open source usage guidelines for your team

| Guideline |
|---|
| Create a list of acceptable licenses your team can use — most likely Apache/MIT/BSD-style |
| Libraries should be used as a whole — code should never be lifted and incorporated as if developed independently. "Stealing is not acceptable" |
| Provide a centralized register of all external third-party libraries (e.g. a shared spreadsheet) recording: library name, URL website, license type, and the component it is used in |

Reasons to publish your own: give something back after years of using others' work; make your
software more robust by having others help debug it; establish a standard in a niche and accelerate
development. Obligation attached: "An open source project must be maintained and updated. No one
wants to contribute to a library that has not been updated in years." And clear it with your CEO and
legal team first — no company IPR or trade secrets, and make sure it is yours to release.

### Five questions for choosing a language or framework

| Question | What to look for |
|---|---|
| Who is behind it? | Especially if a single company — how much of their revenue rests on it, and how would it survive a landscape change? "Languages that are built around an independent standards committee are usually better long-term candidates" |
| How long has it been around? | Maturity. "Typically anything less than two years old should be avoided" — let others work out the kinks. "You are not here to be a champion for the language but to provide a solution and stability for your clients" |
| How active is the community? | Search results, active user groups, an active and vibrant open source community, how many books have been published |
| Is it being taught at university? | Universities must think long term about graduate relevance. "Typically, languages that are tied to a single company are not taught at university" |
| How many job listings are there? | Could you hire a sizable team tomorrow? Check who else is hiring *and* how many people are looking. "Being too early in the lifecycle may mean you are paying too much of a premium for a small pool of experts" |

## Worked Example

### DR designed from the three foundational questions

Assume two zones — the active one and the backup one ready to take over.

**1. What is the downtime tolerance?** The CEO can easily demand zero downtime; it is possible but
hugely expensive and complicated, especially where parts of the enterprise cannot run as multiple
instances. The prime example is the humble database: most organizations have not designed their
tables for real-time redundancy, and autoincremented primary keys instead of GUIDs make it a
struggle. Two concerns dictate complexity and cost across geographically separate zones:

- keeping data synchronized across both zones (bounded by literal network time — 50–100 ms latency
  matters, and modern SQL Server/Oracle synchronization features carry much higher license costs but
  are still limited by physical network speed)
- switching traffic (users) over to a new active zone

Once zero downtime is off the table, the question becomes what is tolerable to clients — minutes,
hours, or days — "a decision that only you and your CEO can come up with, after you have presented
the economics of the consequences of the acceptable downtime."

**2. Full or partial service?** Which services could be temporarily unavailable or read-only? Two
worked scenarios the book gives:

- *If you were in charge of Twitter, squeezed on budget:* let people **view previously published
  tweets** but temporarily restrict writing and publishing new ones.
- *An ecommerce site:* let customers **view and add items to a shopping cart** but not complete the
  purchase through checkout.

Also consider what you lose invisibly for a period: logging, reporting, and scheduled administrative
tasks. And the reprocessing hazard on full failover: "did it fail in the middle of sending a batch of
emails so we now have to fathom which emails went out successfully, or is it acceptable that some
users get duplicate emails?"

Caution: build in the possibility that the partial solution must become the full one. A power outage
will be fixed by the power company; fire, flood, or theft may make the partial zone your primary.

**3. Fail forward or bounce back?** See the table above. "If you find yourself wondering about this
situation, then be warned. This is a very telling sign, usually meaning that a DR event has never
happened (or been practiced)."

**Triggering the switch.** "Like the rainy day fund we all have, the hardest part is identifying
whether it's raining enough to be able to dip in and spend from it." Easy calls (think the NASA Apollo
launch go/no-go): complete hardware failure, fire, flood, theft. Hard calls: a power supply cycling
just enough that the UPS struggles to keep key servers going; a network outage where the upstream
provider says it will be back "very soon" — *define soon*. The fix is a **written switching strategy**
naming the criteria — which services are affected and for how long — that trigger the event.

**Cloud as your DR tier.** Even for a non-cloud-native enterprise, using the cloud as an on-demand,
temporary data center is "more than acceptable and can work out to be quite cost effective" — as hot
or cold standby you stream data to it and pay only for storage, and it is easier to build a like-for-like
configuration and test strategies on a small budget.

### The Flash-locked SAN

A client's SAN administration software went years without upgrades. When a small change was needed,
everyone was locked out — **the SAN administration console was built with Macromedia's defunct Flash.**
The recovery: hunt around for an old laptop that had been off the network long enough to have been
denied autoupdates. Once in, the SAN upgrade moved the admin console from Flash to HTML5.

The pattern this illustrates: it is not just your own in-house stack you must worry about, but every
third-party system you rely on. Compare the industry-wide version — Adobe/Macromedia Flash, Microsoft
Silverlight, and Sun Java Applets all removed from browsers within a few years of HTML 5 and improved
in-page JavaScript; and the mid-1990s Visual Basic apps on long-dead third-party libraries stuck on
Windows 95/XP, for which Microsoft eventually shipped a complete Windows XP virtual machine inside
Windows 10 as the only path forward.

## Key Takeaways

- Decisions on technology can have long-lasting consequences — pick solutions that do not lock you in,
  and prefer small incremental upgrades to huge wholesale rewrites.
- Build where your IPR lives; buy the commodity; run both evaluations with equal thoroughness and let
  a checklist, not instinct, decide.
- The cloud rewards cloud-native design and punishes data-center thinking — run the 10 warning signs
  against your own environment.
- On-prem still has a place (sustained high CPU), but the space is shrinking; if you keep hardware,
  keep an inventory.
- DR off the cloud is harder than most businesses realize; decide tolerance, scope, and direction, write
  the switch plan, and practice it.
- Categorize your data, then set ownership, aging, visibility, and compliance rules per bucket.
- Microservices break the enterprise into independently releasable units and defer technology decisions;
  monolithic still fits proofs of concept and startups.
- Know your open source licenses; register every third-party library; contribute back deliberately.
- Language and framework choices shape hiring and future growth — aim five years out.

## Connects To

- [Chapter 3 — Visionary planning](ch03-visionary-planning.md) — pillars, commoditization, and sunk
  cost, the strategic frame these tactical decisions sit inside.
- [Chapter 9 — Development](ch09-development.md) — the standards, version control, QA, CI/CD, and
  release practices that execute these choices.
- [Chapter 10 — Contract management](ch10-contract-management.md) — SOWs, SLAs, licensing, and support
  contracts behind the "buy" path.
- [Chapter 12 — Security](ch12-security.md) — securing what you build and buy, and the compliance
  handling of sensitive data.
- [Chapter 13 — Housekeeping](ch13-housekeeping.md) — hardware management, monitoring, backups, and
  keeping third-party software updated.
- [Chapter 11 — Documentation](ch11-documentation.md) — architectural, network, and data-schema
  documentation, plus license and audit tracking.
