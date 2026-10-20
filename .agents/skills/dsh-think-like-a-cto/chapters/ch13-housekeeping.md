# Chapter 13 — Housekeeping

## Core Idea

"We spend all this effort in the design, build, and deployment, but sometimes we forget what it takes
to keep the lights on, head off any potential problems, and create an environment where, should
something go off the rails, recovery is available."

The framing question the chapter opens with: **"Many of us believe we are fully backing up our
platforms, but when was the last time you actually tried to restore a system? How sure are you that
the backup has covered everything you need?"**

Technical debt, the chapter notes, is not only a code phenomenon: "technical debt can also occur if
you are not keeping an eye on the enterprise or platform as a whole. Keeping the platform serviced so
no surprises pop up is an often-overlooked role."

## Frameworks Introduced

- **Hardware inventory and MTTF-driven replacement.**
- **Support-contract risk analysis** — four questions per component.
- **SLA components** you offer your own clients.
- **Two monitoring viewpoints** — outside looking in, inside looking in.
- **Backup strategy** — purpose, frequency, retention with grandfathering, and archive security.
- **Budget oversight** — monthly cloud expenditure review.

## Key Concepts

- **Hardware is a living entity.** "Anything that has a moving part will eventually stop moving,
  causing untold consequences. The goal is to ensure you get ahead of failure, plan for it, expect it,
  and ultimately not be concerned when it happens." Cooling fans count as moving parts.
- **MTTF** — mean time to failure, the expected lifetime of a component. "Components can last longer,
  but they can also die sooner. It is important to source components early and keep replacements in
  stock, if possible."
- **The eBay endgame.** "Hardware becomes troublesome when it reaches an age where sourcing components
  becomes hard or nearly impossible... Plan to get well ahead of that stage because if you're not
  careful, it can sneak up on you."
- **Support contracts as insurance policies.** "When things go wrong, they are there to help you get
  back up and running. That said, as with insurance policies, not all support contracts are the same,
  and they may not provide the sort of help or support you really need."
- **Support as priority weighting.** With an internet line fault "the provider will resolve it at some
  point, but chances are, they will prioritize clients with a support contract over clients with
  none." Cloud providers offer basic support free "but can take days to answer or restore systems
  versus hours for those with a support contract." Think of a support contract as "that number you
  call when all other avenues have been exhausted."
- **Legacy systems always warrant support.** "For older systems that have not yet been modernized or
  updated, you should definitely have a support contract to bridge that gap until such time as you
  plan to address the underlying component."
- **Open source has commercial versions.** Mongo, Postgres, MySQL, Elasticsearch — usually the same
  open source software with a support contract. Some offer a commercial version that is a few
  revisions behind (sometimes with extra enterprise-only features) "so you don't have to tolerate
  release 'teething' issues that sometimes come with rapid-release software," and that keeps you
  informed of important security updates without relying on someone checking online resources.
- **Never offer support you cannot staff.** "Never be in a position of offering 24/7 support with no
  real plan to deliver it. There will be a need to deliver on it just as Thanksgiving or Christmas
  dinner is laid out. Relying on your team's years of goodwill, or hoping it will never happen, is not
  a support strategy. **Provide the level of support you would demand from a vendor.**"
- **Monitor from outside your own network.** "If you are hosted on AWS, consider a solution that uses
  Azure or Google, or, at the very least, a completely separate AWS account in a different geographic
  region."
- **Design alerts to localize the fault.** Monitoring the login page is a good first step but doesn't
  confirm the authentication API. Log in with a test account; then hit another API that runs a
  low-overhead query against the main database. "By adding a few extra steps, look at how much more is
  being validated: network, API, and database." Frequency: **every 5 to 30 minutes**, depending on
  what makes sense.
- **Alert routing.** "If you manage email on the same network as that being monitored, you run the risk
  of the email not arriving because the same problem stopping client access could be preventing email
  from getting through." Outsource email to Google or Microsoft, or use a free email address (Gmail,
  Yahoo) that forwards to people at addresses not on your primary domain. Either way, create an
  **"alert" alias** — "it's much easier to administer an email list than to try to maintain individual
  emails."
- **Establish "normal" to spot abnormal.** "If a sudden drop in memory occurs, that could signal a
  process that has died, or one that is flapping (going up and down). You are looking for unexplained
  changes — spikes in usage that are not attributable to increased traffic or client uptake."
- **Central log aggregation reveals cross-system causation.** "Increased SQL errors from the web
  application points to a database server having issues, maybe due to a long-running statement that has
  locked-up tables. Such unintended consequences are very hard to spot."
- **You can never log too much.** Ship logs from operating systems, routers, firewalls, applications,
  the VPN, and networks — including when users, clients, and processes authenticate.
- **Centralized time.** Keep all systems in sync on a common time zone, usually **UTC (Universal Time
  Coordinated)**, using the OS time-synchronization utility. "It makes forensic analysis of log tracing
  much easier to read, because you can spot cause-and-effect patterns by following the timeline."
- **Back up third-party services too.** Source code on GitHub or Bitbucket should be regularly backed
  up to a secure, managed location. Salesforce is "another area often overlooked, particularly if you
  are using this more like a database in the cloud." Google Drive, Office 365, Dropbox — "anywhere you
  have data, ask yourself, what is the consequence if that is no longer available or you get locked out
  of the account?"
- **Backups belong elsewhere.** "More often than not, particularly for smaller organizations, it is
  common to have the backups sitting on the same machine, and not even on a separate network. Always
  work on the assumption of not if but when failure will happen."
- **Alert on backup failure.** Once running, backups "should be on autopilot, with reports being logged
  to your central logging environment, so you create alerts for when a backup fails or hasn't even run."
- **Archive access is separate from archive contents.** "The person charged with running and validating
  the backups doesn't need to be able to see the data in the archive. By controlling access to the
  encryption keys, you can maintain the integrity and visibility of the backup archives... Don't let
  your backups leak data."
- **Cloud bills rise for two reasons** — forgotten resources, and genuine growth. "You need to be able
  to identify the reason for the increased cloud bill, so when the CFO asks, you can celebrate the
  reason with them."
- **A budget is only ever an estimate.** "If you need to make adjustments to your original budget, then
  so be it — it was only ever an estimate, and as time goes on, you are firming it up with what is
  really going on."

## Mental Models

- **The unserviced car, again.** "Yes, you can skip changing the oil and save some coin, but at some
  point the engine will blow up... While there is no oil to change in the tech hardware world, you do
  have hard disks (including SSD drives) and fans that should be replaced."
- **The untethered safety net.** Ryan Burch, director of IT at New Harbor Capital: *"It may seem
  obvious, but a backup strategy that has never been restored is not a backup, but merely a storage
  utilization strategy — something to make you feel safe, like a safety net that is not tethered to
  anything."*
- **Sacred servers.** "It is all too easy for servers to be spun up, then their real need is forgotten
  about, and over time, they become sacred. People fear untold damage if they terminate them, so it's
  just easier to keep them running (spending money)."
- **You get what you pay for** — the spread between top-end hardware contracts (remote monitoring with
  an engineer dispatched automatically before you notice a failure) and low-end ones (diagnose by phone
  or email, ship you a part to fit yourself).

## Anti-patterns

- **Learning about an outage from your client.** "There is nothing more unprofessional, and frankly,
  embarrassing, than having a client report to you when something is down or unavailable... Never be in
  a position where the only monitoring you have is your clients calling in. This inspires no confidence
  in you or your company."
- **The manually updated status page.** A public company vendor with a terrible uptime record whose
  status page was always green across the board — because it was **updated only after a meeting agreed
  a public notification was warranted.** "Such political games are counterproductive, especially with
  the popularity of public validation sites such as Downdetector and IsItDownRightNow. All that was
  achieved was confirmation of how untrustworthy this organization was. **Hiding things in the technical
  world does not work, especially in the long run.**"
- **Backups on the same machine, unencrypted, on the same network.**
- **Backing up too often for no gain.** "If it changes only once a week, then a daily backup isn't going
  to yield much difference between archives. Backing up too often may generate too much overhead, both
  in performance and in disk usage."
- **Renewing support out of habit.** "When it comes time for renewal, ask whether that money is being
  well spent and whether you really need the software."
- **Setting the SLA in isolation** — it must be built with sales, product, the CEO, and the CFO.

## Reference Tables

### Hardware inventory fields

| Field |
|---|
| Manufacturer |
| Type/model |
| Purchase date / install date / next service date |
| Support details |
| Maintenance log |
| Location/login details |

"A simple spreadsheet will suffice, but specialty inventory software exists to aid you with this."

### Common types of support cover

| Type |
|---|
| Fast, minor upgrades or security updates |
| Guaranteed response time to a query (this does not mean resolution, just an acknowledgment of your question) |
| Scheduled backups (if online) |
| Disaster recovery/failover |
| Parts/components replacement on failure |

### Risk analysis — do you need support for this component?

| Question |
|---|
| If the component fails, how does this impact end user delivery? |
| How much knowledge or training exists in-house to resolve an issue? |
| How often have you experienced issues? |
| Is it a single point of failure? |

### What goes into the SLA you offer clients

| Element |
|---|
| Uptime for a period of time — **99.999% over a year means you will not have downtime for more than 8.7 hours** |
| Time to acknowledge an inquiry, detailing business and out-of-hours response times |
| Remote monitoring you may have |
| Software patches, including bug fixes and security updates |
| Maximum time spent on an individual issue or overall |
| Billing for any "not-at-fault" issues |

Built in partnership with your sales team, product manager, CEO, and CFO: "It is a result of what the
client wants, what the sales team can sell, and, ultimately, what you can deliver." If out-of-hours
support is agreed, staff it — remote teams in different time zones, a rotation of employees with a
pager, or around-the-clock staff — and **put it in the budget to pay for it.**

### Two monitoring viewpoints

| Viewpoint | What it does |
|---|---|
| Outside looking in | Emulates a real client interaction from off your network — availability of a service through to latency and response times. Design it so an alert gives your team clues where the problem lies. Consider a **public status page** with uptime indicators per public component (plenty of open source initiatives make this trivial) — hosted outside the organization's main network |
| Inside looking in | Greater access and granularity: CPU, memory, disk usage — "not glamorous, [they] do catch a lot of things before they become bigger issues." Plus application logs, wherever they land: individual files on disk, the Windows Event service, Amazon CloudWatch. Pipe everything to a central service such as Elasticsearch to compare, contrast, monitor, and alert |

### Backup strategy questions

| Question | Guidance |
|---|---|
| What are you backing up? | Data only; configuration and data; or application, configuration, and data — decided by the goal of the backup |
| Why? | Continuity of service in event of a failure; periodic record for compliance; testing/development purposes |
| How often? | Driven by **how frequently your data changes**. "If you need a high resolution of recovery, you should be looking at building that into the storage from the ground up, for example, multiregion data replication in near-real time" |
| How long to retain? | Storage space is no longer the limit with pay-as-you-go cloud; the question is what is useful in a recovery scenario. The old rule was to keep the last three backups against backup corruption; with big storage plus sophisticated checksums and encryption, "as long as you have at least one backup archive, that will suffice for most incidents" — but partial recovery needs push retention to months |
| Where? | Encrypted, on a separate network or cloud account |
| How do you know it works? | Practice restoration; **have fire drills at least once a year**, more frequently if possible |

**The grandfathering ladder** (as printed): "after, say, seven days, you only keep a weekly archive;
then after four weeks, keep only monthly archives. This grandfathering technique keeps storage costs
down."

### Budget oversight rhythm

| Practice |
|---|
| Cloud: daily alerts when thresholds are surpassed; **a cloud expenditure review at least once a month** to determine whether running resources are still necessary; use the provider's tags liberally to track services at a project or department level |
| Off-cloud: expenditure comes in large one-off amounts (servers, memory, disk) — review current usage for efficiency; if storage is growing without end, offload long-term storage to the cloud, "pay for what you need, with no hardware renewals to worry about" |
| Licenses and support: easier to predict but they do increase year over year. Commercial database license costs "tend to run high, so as you need to grow, step back and determine whether alternatives (cheaper or free) are available" |

Note on the industry shift: "This is one of the reasons many software companies have moved away from
support contracts to a more subscription-based model, forcing you to pay as long as you continue to use
the product."

## Worked Example

### Building the outside-in monitor, layer by layer

The chapter builds a remote monitor by successive refinement, each step validating more of the stack:

1. **Monitor the availability of the web page used for logging in.** "Although this is a good first
   step, it doesn't confirm the API for authenticating clients is available."
2. **Log in with a test account.** "That would confirm authentication; however, is that touching the
   primary services?"
3. **Hit another API that would run a query (low overhead) against the main database.**

"By adding a few extra steps, look at how much more is being validated: network, API, and database. Do
this every 5 to 30 minutes (depending on what makes sense), and you will know when things start to go
south very quickly."

Then the alert-delivery design: the monitor must run off your network (a different cloud, or at minimum
a separate AWS account in a different region), and the alert email must not depend on the infrastructure
being monitored — hence outsourced mail, or a free forwarding address, routed through an **alert alias**
rather than a hand-maintained list of individuals.

### The partial-recovery scenario that sets retention

"Say you discover a given set of data was accidentally deleted, but it took a few days or weeks to
discover — a not-uncommon situation when handling multitenant client data. Being able to reach back and
do a partial extraction for a given point in time may be better than nothing. Therefore, keeping data
for months can be justified."

But not at full resolution: apply the grandfathering ladder — daily for seven days, weekly for four
weeks, monthly thereafter.

## Key Takeaways

- Keep an inventory of hardware under your control to better schedule preventive maintenance.
- Determining component lifetimes guides purchasing requirements ahead of time.
- Support contracts are like insurance policies — make sure you are covered for what you need, and only
  where you are thin.
- Monitoring from a client's perspective gives you insight into issues before clients have to tell you.
- Internal monitoring lets your team get ahead of potential problems before they become real problems.
- Backup frequency depends on how frequently your data changes and how far back you need to go.
- Restoration must be practiced periodically to validate the backup coverage.
- Watch the budget weekly or monthly to head off surprises.

## Connects To

- [Chapter 8 — Technology decisions](ch08-technology-decisions.md) — on-prem hardware, disaster recovery,
  and the cloud economics that budget oversight tracks.
- [Chapter 10 — Contract management](ch10-contract-management.md) — SLAs, support contracts, and vendor
  evaluation in full.
- [Chapter 12 — Security](ch12-security.md) — log aggregation, archive security, and monitoring as
  security practice.
- [Chapter 11 — Documentation](ch11-documentation.md) — the owner's manual and backup/restoration
  documentation these procedures depend on.
- [Chapter 3 — Visionary planning](ch03-visionary-planning.md) — where the budget being overseen was built.
- [Chapter 9 — Development](ch09-development.md) — technical debt in code, the sibling of platform-level
  debt here.
