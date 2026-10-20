# Chapter 12 — Security

## Core Idea

"Security is one of those areas that, if done properly, looks effortless and easy. If done poorly,
however, like doing the minimum to tick a box, it gets in the way. Some organizations, on the face
of it, have great security, but scratch the surface, and you'll discover that they have left the key
under the mat, digitally speaking."

The chapter's central warning is complacency: **"Security is a seductive force — the longer you go
without any sort of incident, the more you can convince yourself into believing you run a secure
environment, not unlike thinking your house is fireproof because it hasn't suffered a fire."**

And the operating definition: **"Security is not a feature, something you add or a thing you buy.
Instead, security is a philosophy and a set of principles you live and work by."** It is a continual
process — evaluating the threat surface, monitoring systems for attacks or breaches, and trying to
keep one step ahead of the attackers.

## Frameworks Introduced

- **Two-pronged patch strategy** — identify patches and files; schedule the update.
- **Identify / Protect / Monitor** — the three basic actions of a secure environment.
- **The attack surface** and its reduction.
- **The Swiss cheese model** of layered defense.
- **Kill switches** — digital bulkheads.
- **IIIR** — Issue, Identify, Impact, Remedy — the incident report structure.
- **CISO readiness test** — three signals, and the reporting-line rule.

## Key Concepts

- **The "we have nothing worth stealing" excuse.** "Although your data or systems may not be a
  destination, they could be a gateway to a greater prize. If they can crack open your systems and,
  say, get credentials to a third-party service, then what damage could a determined hacker do, using
  your brand for their nefarious purposes?"
- **Where security issues originate:** poorly developed/tested code, or "a well-intentioned feature
  that had an unforeseen side effect."
- **CVE** — https://www.cve.org/ — an online database tracking known security flaws since 1999, with
  approximately **5,000 flaws registered each quarter**. Each CVE number details the flaw, the versions
  affected, and links to resolutions. "The 'Wikipedia' of security breaches." Sister site for filtering
  and subscribing to notifications: https://www.opencve.io/
- **Subscription hygiene.** "Periodically confirm you are subscribed to the right outlet. Mailing lists
  and notification URLs can change."
- **Patch cadence: at least monthly, small and frequent** — "which will allow impact in the short term
  but keep you protected for the long term." Ideally with blue-green deployment so patches are easy to
  test before committing. Where there is no environment to test in and you must "play with live ammo,"
  update **out of hours, ideally Saturday, so Sunday is a spare day to resolve issues.**
- **If it's got an IP address, it is a potential threat.** Printers, scanners, TVs, routers, network
  switches — "any device that has a network connection needs to be treated like a production system,
  with their security and password management, however benign, needing your attention."
- **Isolate production networks.** "Sharing the network with the back office and your production
  network is opening up your attack surface to unknown devices coming on and off the network."
- **The penetration-test placebo.** "As soon as your penetration test is completed, it is already out
  of date... the next time you perform a software release, all historical penetrative tests are rendered
  null and void. Why? Because you don't know whether this release has exposed a new flaw." Pen testing
  "should be a continually ongoing, integral part of your continuous deployment." A good pen test probes
  **within layers** — not just from the public, but what an employee with elevated rights could get into.
- **Social engineering targets.** "Although everyone in your group is a potential target, the most likely
  targets are those who interface with external people — support, naturally, but also account managers
  and anyone helping sales land accounts." Train them regularly. "Do not underestimate the human desire
  to please and the lengths people will go to exploit that."
- **Passwords as living entities.** "A password that is never changed and hardcoded is no longer serving
  any security role — it's merely an additional configuration attribute." Software should ask for a
  password in real time from a password or security manager — the cloud providers all offer such
  services, forming the basis of a strong cloud-native app where "neither the local or remote credentials
  are known until a connection is initiated."
- **All or nothing on rotation.** "If you are doing a rotation, do it across the whole board. If you
  suspect one or two credentials have been compromised, assume all are at risk. Resist the urge to only
  do selected items."
- **The security/accessibility trade-off.** "The most secure database, for example, is one not connected
  to the network and with no one having any access to it. As appealing as that is from a security lens,
  it is completely impractical in a business." Find the balance per system, then consult the business so
  everyone buys into the processes and policies.
- **Never rely on a single line of defense.** "Putting all your eggs in one basket with the hope that
  the basket is never dropped, smashing all your eggs!"
- **The monitoring problem.** Each protection layer has its own console or log files, and "it's
  completely impractical to have someone log in to each system to check. Human fatigue will eventually
  kick in." You need a single aggregator — Elasticsearch or Splunk as a start, more advanced tools using
  AI to analyze ingested logs — so that things which seem unrelated on the surface reveal themselves as
  "part of a coordinated attack."
- **Log the logins.** Successful *and* failed attempts — this permits observing unusual activity such as
  "an employee account being used late at night or access to a recently closed account being attempted."
  It also forms part of your auditing process.
- **Assume they took everything.** After a breach, "the safest thing you can do is to assume the attackers
  took everything that was accessible to them."
- **Digital forensics.** "Unlike when physical assets are removed — it is obvious because they are no
  longer there — digital assets can be transparently copied, leaving no trace. Hopefully, your logging is
  sufficient that there will be some trace; otherwise, how would you know of the theft in the first place?"
- **The missing-logging discovery.** "What will inevitably happen is that you will discover that the breach
  is in an area where there is a lack or absence of logging. This is to be expected, and is easily remedied."

## Mental Models

- **The house that hasn't burned down** — the false confidence of an incident-free record.
- **The beacons of Gondor.** On patch watching: "it takes only one person to see the fire so action can be
  taken."
- **The cracked pipe.** "Just like a cracked pipe or rusted faucet leaks water, your enterprise can also
  leak data, in small packets" — except "you can't listen for the drip or splash."
- **Swiss cheese model.** Many layers, each protecting different areas, so no single hole goes all the way
  through. Example in action: a firewall blocking all but a handful of ports, which lets users only into a
  network subnet serving as a bastion network for further access deeper in.
- **Ship bulkheads.** Kill switches are "the digital versions of bulkheads in a ship that are designed not
  only for rigidity but also to seal and isolate a hull breach so a hole doesn't sink the whole ship." (With
  the aside that badly designed bulkheads cause more problems than they solve, as with RMS Titanic.)
- **Whack-a-mole.** "There is nothing worse than playing 'whack-a-mole,' thinking you have resolved it, but
  as soon as you shut down, up again they pop."
- **Quis custodiet ipsos custodes** — "Who will guard the guards?" (Andy Wu, CEO of CodeArcs), the argument
  for the CISO not reporting to the CTO.

## Anti-patterns

- **Security as a one-time exercise** to be completed before moving on to the next thing.
- **Trusting a certificate.** "Too often, companies feel at ease with their penetration certificate only to
  discover it is more than six months old."
- **Salting and hashing passwords while logging them in clear text.** See the Worked Example.
- **Debug error pages in production.** JSP, CFML, and PHP sites once dumped complete stack traces and server
  memory on a failed page — "this information was never designed to go into production, but somewhere along
  the line, that switch was never turned off as it moved outside of the development/testing environment."
- **Over-returning data to the browser.** "What is being rendered on the screen may be only a subset of the
  data that is actually returned, and a quick inspection of the browser's developer consoles yields much
  more (and possibly more valuable) data."
- **Credentials in version control** — which then propagate to code-quality tools that make their own copies
  for performance and analysis reasons.
- **Shared database credentials** — no accountability and no ability to exclude an individual.
- **Production and development servers in the same network space**, with too many people holding access.
- **A looser development environment used to build production artifacts.** "What measures are being taken to
  ensure that a rogue library or malicious piece of code has not been added in without anyone noticing?"
- **Unlocked library versions** in Maven or npm — "you run the risk of deploying code and may not know what
  is really in it."
- **Outsourcing security and forgetting it.** Even F5, a company purely focused on security, shipped
  **CVE-2022-1388** (Spring 2022), which permitted unrestricted root privileges to be executed from a simple
  HTTP POST request — rated **9.8–10** and "one of the most severe security alerts in modern history."
- **No kill switch and no logging** — the payment provider in the Worked Example took nearly a day to stop
  leaking.

## Reference Tables

### Data leakage sources

| Source | The leak |
|---|---|
| Logging | Sensitive data arriving in API payloads ends up in the log repository even though it is encrypted at rest. "Be deliberate about what you log, and put in the safeguards to mask sensitive data prior to committing it to your log environment. Don't use security as an excuse not to be verbose with log statements — just be mindful of the data inside the messages." Logs "generally are not as secure as their database counterparts" |
| Application errors | Error pages full of debugging data shown to the public — database connections and other environmental data. "Crashing a website can yield huge clues for would-be hackers" |
| Data exports | BI tools, reporting, user-run queries, or an API returning more than intended |
| Version control | Committed credentials spreading into unintended places; GitHub and similar tools perform rudimentary checks and send alerts |

### Password rotation tracker (as printed)

| Area | Software | Format | Restart | Target |
|---|---|---|---|---|
| Web | Tomcat | web.xml | Yes | Database |

Store this alongside your inventory of systems: identify every area requiring a username/password
combination and, for each, detail the change policy procedure — what needs to be changed and restarted.

### When to rotate credentials

| Trigger |
|---|
| An employee leaves the organization |
| A suspected security breach occurs |
| Software is updated |
| It's time for periodical updates (every month, for example) |

### Identify / Protect / Monitor

| Action | What it means |
|---|---|
| **Identify** | Know the areas you must protect — the **attack surface**, "all the possible avenues where a potential attack or leak could occur." Your job is to reduce it as much as possible and, where you can't, make it as secure as possible. Not a one-time exercise: "Every time a new release is deployed, or a new service introduced, your attack surface changes" |
| **Protect** | Layer defenses knowing each layer costs money and adds user friction. Consult the business on the balance |
| **Monitor** | Continually watch for attacks and breaches — twofold: confidence the current defenses are intact and working ("nothing is worse than realizing after a reboot that the firewall software didn't properly come back up again"), and spotting weak areas to tighten |

### The escalating protection ladder (as printed)

| Layer |
|---|
| No username or password |
| Shared username or password |
| Individual username or password |
| VPN access to get at database |
| MFA (multifactor authentication) and VPN and individual username and password |

"As each layer is introduced, more cost is associated with managing the process, in time and additional
features and licenses. Yet, each layer is adding more friction or hoops for the user to contend with."

### Attack-surface reductions (examples from the wild)

| Problem | Reduction |
|---|---|
| Production servers running in the same network space as development servers, with too many people holding access | Put production into an isolated network |
| Shared credentials to a database — no accountability, no ability to exclude an individual | Give only those who need access their own username and password |

"These are examples, seen in the wild, that don't require huge, sweeping changes to make a significant impact."

### Forms an attack may take

| Form |
|---|
| Denial of service attack |
| Data being exported |
| Data being deleted or encrypted |
| Someone actively moving around your systems without notice |

### Two kill switches

| Kill switch |
|---|
| Disable network access to the machine or subnet |
| Power-off (or suspend, if virtual) |

"The idea of a kill switch is not to solve or eradicate the problem but to buy time to resolve the issue at
its core." And: "it is safer to go dark for a period of time, especially if you suspect sensitive user data
is being compromised."

### After a breach — four questions

| Question |
|---|
| Who needs to be notified? |
| What steps can you take to neutralize the value of data? |
| What safeguards can you put in place to prevent further thefts? |
| Are there any legal ramifications from the theft? |

"My best advice is to not try to do this on your own. Involve your executive team, consult legal counsel, and
get the services of a specialized company." Depending on the type of data — typically financial or health care —
you may have to file publicly about the breach; seek legal advice first.

### Are you ready for a CISO?

| Signal |
|---|
| Your company needs to adhere to various security-related certifications, including compliance |
| The type of data being handled requires specific procedures and audit trails |
| The security-auditing environment becomes large enough to be a full-time job |

**Reporting line rule:** "the CISO should not be under your leadership as CTO. The CISO can't be in a position
where it is easy to overrule or dampen their impact. They need to work alongside your group while, at the same
time, holding your group to account." Framed not as a loss of power but "an acknowledgment of the growth and
maturity of the platform powering the business."

## Worked Example

### A complete IIIR incident report (as printed)

The four-step framework for structuring an incident report so it is digestible by people who are not
technically savvy:

- **Issue** — Define what happened, including how it occurred
- **Identify** — Illustrate how the issue was brought to your attention
- **Impact** — Review the result of the issue
- **Remedy** — Detail the steps to resolve and address the incident

The book's filled-in example:

- **Issue** — The web form used to update users' address details allowed anyone with an account to see any
  other address, by simply incrementing the ID to the API call via the browser's developer console.
- **Identify** — An increase in usage of API calls alerted the Ops team to look further, and they found
  suspicious address API calls from the same IP address.
- **Impact** — A single IP address, thought to be one user, viewed approximately 1,000 addresses before the
  Ops team shut down their access. The user could not be identified because IP logs are not stored with
  recent user logins.
- **Remedy** — The API endpoint has been updated and deployed to return only the address of the logged-in
  user. All other APIs are undergoing a similar audit and update. IP logs are being stored as part of the
  user's session tracing. A list of all accounts whose addresses were viewed has been created.

Scope note: "These IIIRs can be lengthy, depending on the incident. They are intended for internal purposes
only, giving enough detail that would serve as the source to draw on, should a public announcement be deemed
appropriate. **This should be a statement of fact, not a plan to determine what to do next** — that is the
responsibility of you all as an executive team."

### SQL injection, walked through

Consider a website with a lookup feature letting the user type in a name. The code touching the database:

```
stmt = "SELECT id, email FROM table X WHERE name='" + inputFromUser + "'";
```

"Innocent enough, yes? Except, on closer inspection, the developer has created a huge security hole here that
may permit the whole database to be compromised or even deleted."

Instead of typing `noah` into the frontend search box, the attacker types `'2'; TRUNC X;'` and sends that
request to the API. That creates **two separate SQL statements run one after another** (assuming the driver
permits it, "which most do"):

1. The first looks for a name equal to `2` — harmless.
2. The second, `TRUNC X`, **empties all the rows from the X table.**

"After some trial and error, an attacker can wreak havoc on the database."

The point the book draws out: "the hacker doesn't even need to gain any special network access or elevated
database access — they are merely using all the paths that have been laid down by the company, including a new
UI to accept the compromised SQL statement." The mitigation: keep the coding team trained, hold coding
standards and extensive code reviews, "and to basically trust nothing the user gives them." Code-quality tools
like SonarQube catch some standard issues "but will never replace the experienced human eye."

### The client who hashed passwords and logged them in plaintext

A client proudly demonstrated that they never stored the end user's password — it was **salted and hashed.**
Brilliant.

"However, their logging infrastructure had all the passwords in clear text, easily searched because they
captured all incoming POST API requests. Oops."

### Leaking data like a fire hose — and the missing kill switch

A client integrated with a third-party payment provider; part of the integration was the client's API being
notified when a payment cleared. **Their monitoring caught bizarre behavior on the receiving API** — more
volume than normal, and a lot of unknown payee errors.

On closer inspection, their API was receiving **sensitive details on payments being made across the provider's
entire platform.**

The sequence that followed:

1. They reported it instantly to the provider.
2. The provider **at first didn't believe them** — they lacked the proper logging.
3. When the provider did acknowledge it, it still took **nearly a day** before they turned it off — they
   lacked a kill switch.

The root cause, as later reported: **a poor query missing a WHERE statement**, so the client received
everyone's payments, not just their own. "They clearly lacked code review and testing, too — a whole host of
issues at play with this incident — and this from a public company (even the 'big boys' can get it
spectacularly wrong)."

### Social engineering: the cupboard behind the door

Two people turned up at the office claiming to be the landlords, asking to see the cupboard behind the door
because work was being done on the office next door. In this small office, **the cupboard was the drop point
for Verizon and contained a number of networking devices.**

"Our engineer, normally very vigilant online, not only didn't ask them to confirm their identity but held open
the cupboard door while they looked and poked around." It genuinely was the landlords, and all was well — "but
the lesson was made nonetheless — anyone can be caught off guard."

**The mystery shopper drill the book proposes:** if you have a visitor unknown to your team but whom you trust
coming for a meeting, ask them to try to get to your office without signing in. "Their mark is to try to get
seated in your office before you are there, in true Hollywood/TV theatrical style. It's a simple and harmless
thing but can highlight weaknesses in an environment that you can do something about."

### Log4j and SolarWinds — the two ways the supply chain fails

**Log4j (late 2021)** — the security firm Tenable cited it as "the single biggest, most critical vulnerability
of the last decade." It allowed remote code to be run on vulnerable machines by simply having the end user put
special code in logging messages; any system that logged users' searches, input, or chat messages was open.
"The feature was designed that way in the first place, but it was never thought someone would configure it the
way they had — unintended consequences from altruistic motives."

**SolarWinds** — the software was used by large organizations to manage IT infrastructure. "The hackers found
the perfect Trojan horse: they didn't need to hack each target, just the one common piece of software they all
shared. Then, as part of the normal software update process, malicious code would be distributed, deep inside
networks, with no alarm bells going off."

The prescribed defenses: put your build process in a separate environment; lock down specific version numbers
of third-party libraries and upgrade only when you decide and have validated them; and watch for **sudden
changes in file size not attributable to major code changes** — "a deployment being twice the size it usually
is should set off an alarm bell."

## Key Takeaways

- Patching is more than updating software; it requires testing and scheduling.
- Penetration tests, if not done after each release, are no more than a placebo.
- Employees are the weakest link in the chain, due to socially engineered attacks.
- Passwords that can't be changed easily are no longer performing any security function.
- Securing an environment requires constant vigilance and work.
- Developing code requires education and review to make sure no breaches are introduced.
- The build environment can be a source of security holes.
- Creating digital bulkheads, or kill switches, enables containment of a breach.
- Managing a security breach requires level-headedness and good internal communication — keep the executive
  team in the loop with the periodic no-update-update.
- You will grow to a point where a dedicated resource is needed to manage security, sitting outside your group.

## Connects To

- [Chapter 9 — Development](ch09-development.md) — the CI/CD pipeline whose security gate and build environment
  this chapter hardens, plus blue-green as the safe way to test patches.
- [Chapter 11 — Documentation](ch11-documentation.md) — the software and license inventory that makes patch
  identification possible, and compliance documentation.
- [Chapter 8 — Technology decisions](ch08-technology-decisions.md) — data classification, cloud security, and
  the "security is not a given" mantra.
- [Chapter 13 — Housekeeping](ch13-housekeeping.md) — monitoring, backups, and hardware upkeep as ongoing
  security hygiene.
- [Chapter 2 — Managing up](ch02-managing-up.md) — the PIP format and board presentation for a major breach.
- [Chapter 7 — Annual reviews](ch07-annual-reviews.md) — handling the human failure behind an incident without
  blame.
