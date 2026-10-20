# Chapter 11 — Documentation

## Core Idea

"As technologists, documentation is the one area where we fail terribly. It is rare to encounter a
company who has adequate documentation — they're all full of good intentions, but nothing is written
down."

The reframe the chapter insists on: **"Documentation is continuity. Documentation is scalability.
Documentation is freedom. It is not a sign of weakness or an administrative chore that must be
tolerated and put off until the very last minute."** No matter how modern the architecture or
readable the code, there is a layer of rules and logic applicable only to your organization — the
reasons behind the design decisions — "that reading code will never thoroughly explain.
Documentation is the operating manual for your enterprise."

Economics of timing: "if you keep updated as you go along, it somewhat takes care of itself. If left
to languish, however, it quickly becomes a massive undertaking."

## Frameworks Introduced

- **Four audiences** — end user, support, IT/DevOps, developers.
- **Seven criteria for evaluating a documentation platform.**
- **Documentation validation** — a second pair of eyes, and a run-through by someone unfamiliar.
- **Twelve documentation types.**
- **The what / how / why layering of code comments.**
- **The map metaphor** for architectural, process, and network diagrams.
- **Whitepapers** as external-facing documentation.
- **Seven best practices.**

## Key Concepts

- **What happens when the reasons are lost.** An eccentric business rule or obscure restriction drove
  a design down a path — the right decision at the time. Years later, one of two things happens:
  (1) "Someone new will look at it, think there is an easier way, change it, and subsequently break
  something in the process." (2) "People are scared to touch or ever think about redoing it. It takes
  on mythical properties." Both introduce far too much risk, and "both of them could have been avoided
  completely by having a little documentation sprinkled around."
- **"Comments are little love notes to your future self"** — quoted from a sage developer.
- **Speak to the audience.** Explanations of algorithms make little sense to end users or support but
  help developers maintain and adapt software; detailed screenshot walk-throughs don't help IT/DevOps,
  "who don't really care what the app looks like. They just want to know how to manage things, say,
  the backups."
- **Video as a first-class format.** "A 5 minute video can be the equivalent of several pages of
  traditional descriptive text." Locking documentation in PDF or DOC files "although great 20 years
  ago, is no longer the go-to method." A document that describes or educates on the platform "is a
  living, breathing entity."
- **Findability is the bar.** "No matter how good a piece is, if it can't be found when needed, then
  it is next to useless."
- **Good content doesn't have to look great** — "it just needs to be of help to the reader. The less
  friction to authoring and contributing, the better."
- **Validate documentation like code.** Have someone other than the author confirm usefulness and
  check for presumptions and omissions. "It doesn't have to be a formal document review process, but
  it should include a second pair of eyes." Test the build process document by having someone
  unfamiliar run it — someone "who won't subconsciously skip over the gaps."
- **Single-person areas deserve the most attention.** "Has everything they do been adequately
  captured so anyone can help backfill the role? Only by testing the documentation can you really
  answer that question with confidence."
- **Document debt.** "There should be enough notes and guidance that anyone technically competent
  should be able to rebuild every component of the enterprise from source control. If this is not
  achievable, then you are in document debt."
- **Job security has changed.** "No one on your team should feel that the only reason they have a
  role on the team is their in-house knowledge of a particular system. This was a relatively common
  attitude 10–20 years ago, but it is something I rarely bump into today. People's values and job
  security are not what they know but what they contribute to the team."
- **Code comments in reviews.** "Make sure adequate source documentation is incorporated into code
  reviews. Once developers get into the habit, they will never look back."
- **Link the weird `if` to its ticket.** Leaving web links to the ticket number gives the future
  developer depth on *why* — "instead of some future developers thinking they are doing everyone a
  favor by refactoring it away and potentially introducing more bugs than they are solving."
- **Compliance as commonsense protocols.** Whatever the standard, "treat it as a set of commonsense
  protocols to keep all parties operating honestly. Organizations with no documentation, process, and
  flows struggle the most... Those with good documentation and standards find it nowhere near as
  painful." And compliance is more than documentation — "you need to walk the talk," which may require
  code or architectural changes. "Treat it as a badge of honor."
- **Nobody has the software list.** "A list of software is a standard part of due diligence, and as of
  writing, I have not come upon a team that has this information at their fingertips."

## Mental Models

- **The box of chocolates.** "When you get a box of upmarket chocolates, what is the first thing you
  look for? The map, of course!" It gives you a fighting chance of avoiding the hazelnut hiding within;
  connoisseurs plot their taste journey with it. The enterprise needs the same map.
- **"Draw some boxes with lines."** What to say when you ask for the architecture map and get blank
  stares.
- **State highways vs. local roads.** The top-level architectural diagram is the state-level boundaries
  with interstate highways and only major cities marked; each component gets its own map exposing rural
  highways and towns. "Like the US states, cities and highways never change, but the local roads within
  each state and city are continually evolving."
- **Google traffic overlay.** If the architectural diagram is the highway map, the process diagram is
  the traffic flow overlaid, "showing where the cars are."
- **Documentation is like test cases** — "you can never have too much."
- **You are not writing the next Harry Potter book.** An owner's manual is numbered steps, each cogent
  enough that the reader knows when it's done and can move on.

## Anti-patterns

- **"Our team knows how things work."** Compared in the chapter to believing your code has no bugs and
  therefore needs no testing — "absurd when you see it written down like that — yet many sincerely
  believe it."
- **Documentation as PDFs and DOC files** that cannot be linked to, updated, or found.
- **"Make sure to restart component XYZ"** as an owner's-manual entry. How do you restart it, what
  commands, from where, what indicates success, what remediates failures? "Too much of this detail is
  often overlooked and forms part of someone's muscle memory."
- **Release ability bottlenecked on one person.** "It's not uncommon to come across a company whose
  ability to update is dependent on their senior DevOps person — when they're not out on PTO."
- **Over-detailed diagrams** — "this makes the map busy and hard to comprehend."
- **Unrecorded meetings and demos.** "Meetings can be a great source of information exchange, with lots
  of ideas and suggestions. Yet they can be an area where little or no notes are taken, so much
  valuable information is lost — what a waste."
- **Undocumented data schemas** — "given the importance of this role, it is one of most undocumented
  areas, but it doesn't have to be."

## Reference Tables

### Four documentation audiences

| Audience | Who they are |
|---|---|
| End user | Users of the system who interact with the finished product |
| Support | Frontline support who assists end users with their queries |
| IT/DevOps | Backend team responsible for keeping systems up and functional |
| Developers | The team who creates or fixes features |

Smaller teams early in their journey have a lot of overlap and may have no end-user documentation at
all; as things grow, each audience develops its own process flow requiring support from the tier below.

### Evaluating a documentation platform — seven criteria

| Criterion | What it means |
|---|---|
| Linking directly to specific content | Send someone to a specific section without a series of navigational steps |
| Organizing related content | Easy to bring related areas together — ideally automatically, through tags or semantic intelligence |
| Easily augmented | Content updated as new information comes to light, particularly for support |
| Versionable | Go back to a previous version quickly; lets the reader judge whether what they're reading matches the version they work with |
| Secure | Protect areas or sections for different groups |
| Feedback | Readers can leave comments or small notes to augment content |
| Attachments | Add third-party files (images, video, PDF) and keep them close to the document they pertain to |

Candidate tools named: wikis, Atlassian Confluence, Google Docs, Office 365. The rationale: "When Tim
Berners-Lee first came up with the web, his goal was to share information easily. So, it makes sense
that the web should be the focal point of a document store."

### The twelve documentation types

| Type | What it captures |
|---|---|
| Meeting notes | Designate a note-taker per meeting and build in time to write them up; keep them in the main knowledge base as the history of decisions. Some organizations maintain a separate **decision log** in a single place for why big decisions were made. Quick meeting on a ticket → bullet points in the ticket comments. Record online meetings. Copy gems out of Slack/Teams exchanges into the knowledge base (bots and plugins can tag and forward them) |
| Walk-throughs / demos | Record product demos and sales pitches — each iteration yields something new. Watching sales demos shows how features are pitched and what clients need from them; "there can be quite the gap between the two different constituents." Volume is fine — the archive becomes a time capsule of historical versions |
| Owner's manual | How to keep the enterprise up and running — the pieces of the platform, how they function, how to monitor them, and what to do to bring them online after a power outage or other offline event. Include early warning signs of impending problems. Screenshots of expected console output ease anxiety for someone who has never seen those log lines |
| Backing up and restoration | More than database export/import: the enterprise as a whole, hardware to operating system to software. What is required to breathe life back into a component; what parts of the OS must be available for it to run |
| Deployment process | Every step to prepare a build, what is happening, criteria for success, how to detect failure, the order of execution, and the point beyond which you cannot cancel. Separate processes for patch, hot fix, and full release should each be detailed — even with an orchestration tool like Jenkins, "it still warrants a guide on how it has been configured for your environment" |
| Source/code comments | The what / how / why layers (below) |
| Architectural diagrams | Major components with lines joining connected pieces; no logistics detail — enough to know the File Microservice is used by the module for the user's profile photos, not whether it's an API call, a queue, or a shared directory |
| Process diagrams | How data flows around the system and what happens to it as it passes through a service or library — where data is stored, cached, or exchanged, and especially how data enters your world and how it leaves again |
| Network diagrams | The architecture through the lens of real networks and servers; pertinent network addresses, routes, firewalls, gateways. With physical servers, show data center and rack locations (a separate document holds specs, purchase and expiration dates, configuration records) |
| Data schemas | Tools such as DB Schema introspect a relational database and draw an overview, dissecting foreign keys to produce table relationships; augment and annotate it with the business knowledge. Modern databases support comments directly on tables and columns. Non-relational stores need manual external documentation, added to the release cycle |
| Compliance documentation | Process steps, audit reports, summary reports — heightened in health care and financial industries, and in certain government and military projects (ISO). For certification, "work with a third-party expert"; large accountancy firms have dedicated departments |
| License and audit tracking | A knowledge-base record of all external software reliance, purchased and open source. "Every time a Java developer adds a new library to their Maven project, or a JavaScript developer imports a new npm module, make it part of the process to update the documentation, including source and licensing terms" |

### Code comments: what / how / why

| Layer | Audience | Guidance |
|---|---|---|
| **What** | Other developers *using* that code — library public functions and methods, API endpoints consumed remotely | Use the language's convention or meta language so IDE code completion picks it up. **Javadoc** for class- and method-level documentation (also generating web pages); **Swagger** for API endpoints in HTML, with a utility for remotely invoking the endpoint for testing and research. "Kept in the same source file as the code, which makes it easy to keep updated" |
| **How** | Only other developers *maintaining* the same code | "Do not advocate too much detail. Well-designed and -written code should be obvious to the reader" — assuming sensible function and variable names. **"Code should be written for readability, not performance."** Add a few lines where a block genuinely needs explanation, and link the ticket number |
| **Why** | Higher, business level | For anything the "how" can't convey: is there a business rule or a specific way a third party operates that forced a design decision that isn't at first obvious? |

### Whitepaper examples (as printed)

| Example |
|---|
| API security principals |
| Data usage recommendations |
| Best practices for API integrations |
| Data state transformations |

A whitepaper is "a published document designed to be read by an external audience... It doesn't give
away any trade secrets or detailed engineering methods. It is designed to bring an external party into
your line of thinking so it's easier for them to interact with your product." Often used in the sales
cycle to show professionalism and depth of knowledge.

### Seven best practices

| Practice |
|---|
| Choose your knowledge base tool carefully to deliver as many of the core documentation features as possible in one tool |
| Create templates so your team can quickly add to pages — "it's much easier to edit something than create something fresh" |
| Start small and build slowly. "This is a marathon, not a sprint" |
| Encourage everyone to augment content with emails and chat messages |
| Video and screencasts are quick ways to fill out content |
| Describe what source code does as part of the code-review process |
| Practice all the process details when it comes to managing systems |

## Worked Example

### The vacation email trick for producing an owner's manual

Creating an owner's manual "can be quite intimidating to create at first." The book's technique:

> Ask the principal person who manages a particular system to **pretend they are going on vacation and
> to write an email to the person taking care of it in their absence.**

The reasoning: "An email is less formal, and people have a tendency to be more verbose in an email,
than when faced with a blank wiki or Google Doc page." Take the email and add it to the knowledge base.

"This exercise works many times, resulting in a rich, deep library of how-to documents built up over
time."

### The missing ImageMagick dependency

Someone was rebuilding a new machine to perform thumbnail generation and kept hitting a wall running
the Java application. **The Java app needed the open source ImageMagick library installed on the
operating system to work.** The app's log output was poor, so the cause was far from obvious. There was
no documentation and no clues where to look — the dependency was only discovered by cracking open the
code.

The lesson the chapter draws, updated for today: with container computing (Docker), operating-system
dependencies are caught during the build process — "that doesn't negate the need for dependencies to be
clearly listed, though. Even if everything is just a container, how the containers all fit together and
run at a higher level needs to be detailed."

**The practice that proves it:** rehearse the restoration at least once a year, more often if possible —
trivial and risk-free in a cloud environment. "A good practice to get into is to have new hires execute
this restoration process in a clean environment. This exercise serves as a neat way to get people
familiar with the process, while learning how everything fits together and ensuring the documentation
is relevant."

### Hiring a technical writer

In one CTO engagement the author recognized the group needed significant documents for both internal
purposes and external communication and education. **He hired an experienced technical writer with a
background in software engineering.**

The role was explicitly *not* to document everything, but to:

- take the notes and create diagrams — "literally on the back of an envelope at times"
- flesh those notes out into structured documents
- organize them
- convert the diagrams into a common format
- identify which ones needed reworking
- keep an eye on the release notes to flag areas that potentially needed updating

"It truly was one of my best hires, and I advocate for a technical writer to be added to any
engineering group of **20 or more people**."

## Key Takeaways

- Document early while the knowledge is still fresh.
- Choosing the right tool makes it easier for everyone to contribute.
- Keep it simple — this encourages those less prose-oriented not to feel intimidated.
- Videos, voice notes, and diagrams are rich vehicles for capturing details.
- Validate anything that details the running and management of a system.
- "Even simple notes or bullet points are better than nothing."
- A good top-level architectural diagram "should be something that can be used to communicate how things
  are set up to others outside your group without bogging people down in details or complicated terms.
  Don't be surprised if it finds its way to a board deck because a good diagram is a great way to sell
  your vision."

## Connects To

- [Chapter 9 — Development](ch09-development.md) — code reviews, release notes, and the CI/CD pipeline
  the deployment documentation describes.
- [Chapter 6 — Team management](ch06-team-management.md) — the team knowledge base and defined processes
  a charter requires.
- [Chapter 12 — Security](ch12-security.md) — compliance, the security audit report, and credential
  tracking.
- [Chapter 13 — Housekeeping](ch13-housekeeping.md) — backup and restoration, monitoring, and hardware
  records in practice.
- [Chapter 8 — Technology decisions](ch08-technology-decisions.md) — the open source library register and
  the data classifications the schemas document.
- [Chapter 14 — Company growth](ch14-company-growth.md) — where the software list and documentation
  quality get tested by due diligence.
