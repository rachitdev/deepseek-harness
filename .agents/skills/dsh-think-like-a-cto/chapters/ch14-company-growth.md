# Chapter 14 — Company growth

## Core Idea

At some point the company may be acquired by another firm, acquired by a private equity or
investment group, purchase a smaller company, or merge with another. "Each of these occurrences
comes with its own challenges and strategies for success... but they all share a common threat of
being a huge distraction and time sink if not properly handled."

The CTO's job in a transaction: **"you are responsible for positioning all the work done to date and
all future plans in the best possible light without misrepresentation or lying."**

The chapter's other central reframe is due diligence itself. It is **a conversation, not an
interrogation** — "akin to the home inspection report, a document created by professionals who will
know what to look for, so you, the buyer, have no surprises after purchase."

## Frameworks Introduced

- **Investment cadence** — timing large projects against the 3–5 year transaction horizon.
- **The tuck-in acquisition** and its integration requirements.
- **Due diligence, performed** — seven areas to explore.
- **Due diligence, conducted** — the two role-play sessions and the bingo card.
- **The four-part due diligence report** — overview of target / methodology / summary findings /
  supporting materials — plus traffic-light severity scoring.
- **Due diligence, answered** — the seven documents you should already have.
- **Handing over the reins** and **taking the reins.**

## Key Concepts

- **Who investment groups actually are.** "Contrary to popular belief, investment groups are not just
  rich old men with money. The vast majority of these groups are pension funds, university endowments,
  and family trusts... your success will be helping fund people's retirements and help put students
  through higher education." That is precisely why the extra oversight exists: "when putting other
  people's money to work, you want an extra level of care and attention."
- **How PE makes money.** They take a lump sum, invest it somewhere relatively safe, and **within
  three to five years** typically return it larger than they started. "An investment group, therefore,
  makes its money or returns when a transaction happens — when they sell their share." Contrast with
  venture capital, "which is more about investing in an idea and building a company around that, than
  investing in an existing, successful, operational company."
- **The operating partner.** In PE parlance, those who roll up their sleeves and help. "Should you find
  yourself working with an operating partner, embrace it. Do not feel threatened — they don't want your
  job. They want to lift you up and prepare you for success." (With the caveat that if they judge you
  unqualified they can replace you — "by the time it gets to that stage, you've managed to ignore all
  the advice and guidance that has been given to you.")
- **Be prepared for the impromptu pitch.** "You never know when a group of 'suits' will be touring and
  you will be asked to give a review of where things are now and, more importantly, where your plans or
  vision will take the company in the future." These encounters "can make the difference between moving
  forward or killing the deal there and then. Your greatest contribution is to instill confidence (not
  arrogance) that the technology, while maybe not perfect, has a future and a plan."
- **The tuck-in.** A smaller company (perhaps 1–10 employees with a particular niche) purchased to
  augment a larger one — "not mergers in the classic sense, where both parties have nearly equal
  footing. Think of a tuck-in as more like purchasing a fully formed, self-running department."
  **Success is predicated on the speed of integration** and how quickly the acquired business starts
  adding value — which is itself a consideration for investment groups evaluating targets.
- **Absorbability as an architectural goal.** "You want to make sure your technology stack can quickly
  absorb or be absorbed by another. Modern micro architecture or API environments are great foundations
  to build upon. An API buys time for evaluating whether the backend should be rewritten or ported."
  Version control and build processes "are areas that should be targets for consolidation." And a tuck-in
  need not be rewritten — "the technology should be able to use a lot of the 'housekeeping' facilities,
  such as logging, security, platform, and monitoring."
- **Email and chat first.** "Nothing makes a new team feel more welcome and at home than using the same
  email and chat system."
- **The "why" is the missing half of most due diligence.** "Due diligence is not running down a list of
  prepared, standard questions in a spreadsheet. That then becomes an interrogation, a Q&A, with no
  feeling or opportunity to celebrate what is being discovered." Every purchase solves a problem — the
  fit-for-purpose question. "The goal of the overall exercise is simple: can the technology do what we
  need, and if it can't, how much money and effort will it take to do so?"
- **Why the CTO, not an accountant's associate, should do it.** "Most things you will come across will
  either be Coca-Cola or Pepsi (in other words, not wrong — just different) or small enough they are
  easily resolved... You can spot these issues quickly, allowing the time to focus on the areas that
  could become a problem."
- **Everything you find becomes yours.** "Any problems or issues unearthed will become yours to address
  if the transaction goes through successfully. Hopefully, though, there will be things that will assist
  your team... There is always something to glean."
- **The vacation question.** To find the crucial team members: *"Which person is missed the most when
  they go on vacation?"* "Most teams will answer this quickly, without giving it a moment's thought."
- **Judging code fairly.** "It is neither practical nor fair to pore over every single line of code...
  We all have code in our past we would not wish to be judged against, so pick some code that has
  recently been committed" — find it from the version commit history or the ticket system and follow its
  journey from reporting through fixing to release.
- **Data you collected is not automatically yours.** "I have often come across companies who think just
  because they have an email address stored in their database it's theirs to do with as they please.
  This is not the case. How was the data collected? If the user didn't consent to their data being used
  (or sold), then its perceived value is worthless. I have seen the blood drain from many a face when
  they realize they don't have as much value in the data as they thought."
- **Highest bidder doesn't always win.** "Most of the time, the decision comes down to who is the best
  cultural fit. Due diligence is a data point involved in that decision."
- **You are not being judged.** "If you feel you are, that is the fault of the other person." And if
  someone reads twenty minutes of canned questions, "politely say if it would be quicker if they just
  emailed you the list of questions to be answered."
- **Access limits when answering.** Source code: fine, via a zip in the data room or a read-only version
  control account. Databases: "restricted to a screen share at best, with no direct control permitted.
  This is your clients' data... You can run counts, show sample records, but take care to protect your
  data." Infrastructure: a specific read-only cloud account. **"A reputable professional conducting due
  diligence will never ask for any sort of access that could compromise or change a system."**
- **Don't stop running the department.** "If you are in the process of being sold, you could be doing
  this a number of times, with different suitors over a short period of time. Do not forget to run your
  department." Same on the performing side — "it takes more time than you may allocate up front."

## Mental Models

- **Selling a house / the kitchen refit.** "The last thing you want is to be in the middle of replacing
  the kitchen as you tour prospective buyers through a construction site. They will look at it and make
  the logical conclusion they may end up finishing the kitchen themselves." Applied to any project of a
  year or more against a likely transaction date — and if you start anyway, adjust the milestones "so
  the kitchen could be semi-usable": get the plumbing and electrical done at least.
- **The home inspection report** — what due diligence actually is.
- **The two Bobs** (*Office Space*, 1999) — "What would you say you do here?" — the caricature of due
  diligence that the chapter argues against; and *Chicago PD* questioning suspects handcuffed to the
  table, the tone to avoid.
- **The TV buyer.** "If you want a TV to simply use for your Xbox, then you do not care about any smart
  apps or the number of channels it can receive. You are interested only in resolution and refresh rate.
  A different buyer may be looking to watch local news and Netflix, and their due diligence is going to
  look a lot different."
- **Highlander.** "There can only be one" — being acquired when there isn't room for two CTOs.
- **Maverick, again.** Taking the reins: sit and watch the poker players for the first hour before
  playing.

## Anti-patterns

- **Being caught with no answer.** The 5 p.m. call to join a dinner because a couple of PE partners
  stopped by — "no presentations, no deck, just good, old-fashioned discussions over a dinner table
  (with way too much cutlery for a simple engineer to grasp)."
- **Starting a multi-year project without checking the transaction horizon** with the CEO and, if you
  have one, the operating partner.
- **Tuck-ins left as independent departments.** "Successful tuck-ins are ones that do not remain as
  independent departments but are integrated fully, pulling in the same direction... The last thing you
  want is a completely different tech stack, with different skills, making it hard for you to move
  resources around."
- **The condescending or arrogant diligence "expert."** "Sadly, there is little you can do in this
  situation but smile through it."
- **Reporting only the technical fact and not its consequence.** See the Worked Example.
- **Flagging Coca-Cola vs. Pepsi as a concern.** "If the target chose X over Y, but Y is your preferred
  choice, it makes no difference and should not be highlighted as an area of concern."
- **Lying or misrepresenting when answering.** "This will only end in tears." Politely refer a question
  to the CEO instead.
- **Personal credit cards owning core accounts.** "It is not uncommon (particularly in smaller companies)
  to have common services such as AWS, GitHub, or GoDaddy accounts being 'owned' by the original developer
  simply by virtue of their credit card being debited each month, which is then reimbursed with an expense
  form" — the legality of ownership "needs to be identified and resolved before any transaction is completed."
- **Leaving under a cloud.** "Never leave under a cloud, even if one was put there."
- **Changing for the sake of change** on arrival.

## Reference Tables

### Due diligence — the seven areas to explore

| Area | Questions the book poses |
|---|---|
| Technology stack | How different is it from yours? Does your team have enough knowledge to consume it? If outdated, what would it take to keep supporting it — including recruiting engineers and maintaining hardware or OS? What could be more efficient with modern techniques? What would a rewrite cost in effort and disruption? |
| Intellectual property rights | How much of the technology is unique to the business? Is the secret sauce the algorithms and code, or the application of the tool? Any patents — and who owns them, an individual, a company, or a third party? "As strange as it seems, who actually owns the source code? It's not always obvious, especially if the company has outsourced its principal development" |
| Data | How is data stored and used? How much is kept and available for analysis? Who are the main producers and consumers? Who owns it and what restrictions apply? What jurisdictions apply (is GDPR relevant?), and is any of it under compliance such as PII, PCI, or HIPAA? |
| Team | Structure, average tenure, skill set. In smaller organizations people often learned everything on the job — "who did those people learn from?" Which individuals are crucial and most knowledgeable (the vacation question)? Any contractors or outsourced firms, their responsibilities and contract lengths, and any contractual considerations around ownership or termination |
| Support | What issues do users typically report? How much time goes to support versus new feature development? How much effort goes into onboarding new clients, and does the technical group do it? What major outages or issues occurred in the last 12 months, what was the client impact, and what has changed since? "What is the one thing that keeps the current leadership up at night, or, if they had a magic wand, what would they like to be addressed instantly?" |
| Security | The general attitude toward security. When were core passwords last rotated? How much visibility does a developer have? Is there a separate production environment away from development? How are stored passwords held — plain text versus salted ("you will be surprised at how many still store passwords in clear text!")? What SQL injection or penetration tests have been run on production? "Security is something a company either builds in from the start or layers on at a later date — the latter is not what you want to see" |
| Source code | Follow one recently committed change end to end. Look at how configuration, usernames, and passwords are handled; how APIs are constructed and whether there is consistency; how errors and exceptions are handled and whether there is "too much assumption that things don't fail." Overall structure — clumped and functional, or laid out in objects with strong code reuse? Which open source libraries are used, and are their licenses compatible and legally honored? Is logging consistent enough to filter exceptions and errors from informational entries? Then the surrounding management: code reviews, version control, test, build, and release |
| Production system | "At this moment, this very moment, how do you know all is well?" — asked with a dramatic pause. What daily checks are made, what logs are looked at, what the early signs are. Fault-tolerance plans, single points of failure, backups and recovery, and any documentation or read-me files someone could follow to restore the environment. Confirm **who is responsible for paying and renewing services** |

On code maturity, the book's own guardrail: "It is common for small teams to be very informal —
rudimentary, if you will. The important thing is not to judge — these are merely data points in the
journey. No matter what they are doing, good or bad in your view, they are doing something right
overall, or you wouldn't even be thinking of acquiring them."

### The four-part due diligence report

| # | Section |
|---|---|
| 1 | Overview of target |
| 2 | Methodology: when was it conducted, who was present, what materials were provided, what was not available |
| 3 | Summary findings |
| 4 | Supporting materials |

Summary findings are "a list, ordered by severity of the consequences of what was found. In a separate
list, you can detail all the good things that were found that are cause for celebration." Technical
reasons and sources go in the appendixes or supporting documentation.

**Traffic-light scoring:** the author's own scale developed across many diligences for acquirers and PE
firms, "that makes it easy for the reader to focus on the 'red' areas, keep an eye on 'amber,' and be
glad to see the 'green' areas as they relate to the goal of the purchase."

**Pre-review the report with the people you interviewed**, if time allows. Two goals: accuracy ("maybe
you misheard something, or got it mixed up... This is not opinion, but facts"), and building a bond and
trust with people who could soon be on your team. "End every session noting that they will get to review
the report for accuracy and there will be nothing in it that will be a surprise and that you haven't
already talked about. No one needs that level of stress or mistrust."

### Answering due diligence — what you should be able to produce

| Document |
|---|
| High-level diagrams of architecture/network |
| List of all technologies (including versions) |
| Licenses, renewal dates, and costs |
| Organizational chart with everyone listed |
| New employee onboarding document |
| Recent support issues |
| Future development priorities |

"The more astute reader will recognize this list: they are the very things this book has been highlighting
that should be continually maintained as part of being CTO."

### Handing over vs. taking the reins

| Handing over | Taking over |
|---|---|
| Communicate the **constraints** behind each decision — "these reasons are valuable business knowledge that even the smartest CTO will not know" | **Do nothing.** "No one is really expecting anything from you in the first 30 days, and you have 90 days before any sort of vision starts to materialize" |
| Go through each team member's strengths and weaknesses — never a personality evaluation. Start from the last review process, highlighting improvements or backward steps since | Sit in on as many meetings as possible; listen and observe the interactions |
| Outline your vision even though you won't see it through — "your vision is steeped in years of in-depth business knowledge and experience of the client issues that keep occurring" | Hold as many one-on-ones as possible; opener: *"If you were king/queen for the day, what would you change/fix?"* — then **offer no solutions or assurances** |
| Open your calendar and walk the **last 30 days**, focusing on meetings with people outside your department — a day-in-the-life doesn't give enough context | Get to know peer executives during the "honeymoon" period when you can ask the most basic questions without anyone raising an eyebrow |
| Accept the sadness; "augment it with pride that you did your best" | Charles Moon (CTO/Head of R&D across many organizations): "Be thoughtful regarding the internal politics since you may be surrounded by those who supported the previous technology roadmap. Most changes should be handled as positive enhancements due to technology/market advancements" |

## Worked Example

### The two role-play sessions

The author's technique for conducting due diligence as a conversation rather than an interrogation:
put both parties through **two role plays.**

**Role play 1 — pretend to be a new customer.** Have them take you through the onboarding process,
including any demos and training. "This is a wonderful exercise that puts everyone at ease because they
get to show off and be proud of their product." It serves multiple purposes:

- **Discovery of the language or terms they use with clients** — "this allows you to formulate questions
  using their language, thus reducing any intimidating questions you may ask by choosing the wrong
  phrase or word."
- **Understanding the features that actually matter to clients.** "Typically, things like 'We can do
  this, but no one uses it,' will kill off a whole realm of questioning that isn't necessary while
  learning a data point that may contradict a management proposition."
- **A natural hook for deeper probes.** "Say a chat portal is being demoed and they can upload a file.
  You can ask, 'Where does that file actually go?' and then ask whether and where it's backed up."

**The break.** "Once the client portion is done, it is a good natural break for lunch or coffee. Take
that opportunity to praise and thank them. Be genuine — be real."

**Role play 2 — pretend to be a new developer, engineer, or team member** being onboarded to the point
where you can start contributing. "Armed from your client onboarding, you now have the company vocabulary
at hand, so you can relate and ask more relevant and pointed questions." It gives them license to go
deeper in a way that is familiar, especially if they have hired recently. Through it you are introduced
to their processes, communication, task management, how code is developed and released, and the major
components powering the system.

**The bingo card.** "Keep a high-level 'bingo' card of topics you need to cover. Every time you get what
you need, mark that box. There is no specific order, no set of canned questions. But it gives you a cue
card to know whether, throughout all the conversations, you have indeed covered everything."

The standard to hold it to: **"Successful due diligence should be educational, enlightening, and fun for
all parties."**

### Translating a finding into a consequence

The house version first: it is discovered that **the electrical breaker for the hot water tank is not
working.** Don't lead with that detail — say instead that **there is no functioning hot water, it will
take three weeks, and cost approximately $2,000 to resolve.** "Now the buyer can do so much more with
that detail. They know they need to find an extra $2,000 (either additional investment or a discount from
the purchase price), and they won't be able to shower for at least three weeks."

The technology version. A technologist writes:

> The main database has no active standby and is configured in a standalone mode with a public IP address.

"We believe everything is clear as day. What we often overlook is the 'so what?' or consequences of what
we just stated." Rewritten for the report:

> In the event of a failure (including power loss), the company will no longer function and may take days
> to restore. This will require an investment of $X to determine and implement an acceptable solution to
> maintain business continuity.

The technical reasons and sources that led to that statement go into the appendixes.

### Google forgot to pay their domain bill

"Production systems go dark for the silliest of reasons, and today, in our modern, connected world, if you
don't pay your domain or SSL certificate, then the service will stop. One of the big boys, Google, with
their Blogger service, got caught out when someone didn't pay for the blogger.in domain and lost control
of it for a period of time."
(https://www.androidpolice.com/2020/07/16/someone-at-google-forgot-to-renew-a-blogspot-domain-still-offline-a-week-later)

## Key Takeaways

- Being acquired can be a huge distraction if not managed properly.
- Buying a smaller company can yield huge, nearly instant benefits if you integrate or use their offering
  as quickly as possible.
- Building an architecture that makes it easy to integrate "opens up your company for more strategic
  conversations."
- Due diligence is not an interrogation but a conversation designed around a specific goal.
- Contextualize results into language nontechnical people can process so high-level strategic decisions
  can be made.
- Taking over from an existing CTO is about listening and learning their struggles and challenges, not
  changing everything.
- When handing the reins over, set up everyone — including your team — for success. "You can be held
  responsible only for the moment, not the future in your absence."

## Connects To

- [Chapter 2 — Managing up](ch02-managing-up.md) — taking over from another CTO, board presentations, and
  the questions to ask the outgoing CTO.
- [Chapter 3 — Visionary planning](ch03-visionary-planning.md) — the elevator pitch behind the impromptu
  pitch, and long-term project timing.
- [Chapter 11 — Documentation](ch11-documentation.md) — the artifacts that make answering due diligence easy.
- [Chapter 8 — Technology decisions](ch08-technology-decisions.md) — IPR, data classification, and the
  API/microservice architecture that makes a company absorbable.
- [Chapter 12 — Security](ch12-security.md) — the security posture a diligence probes.
- [Chapter 1 — The Chief Technology Officer](ch01-the-chief-technology-officer.md) — the first 100 days,
  which taking the reins mirrors.
- [Chapter 15 — You, Inc.](ch15-you-inc.md) — succession planning and knowing when to step up or move on.
