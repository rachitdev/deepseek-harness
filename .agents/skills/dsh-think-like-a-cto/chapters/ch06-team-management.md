# Chapter 6 — Team management

## Core Idea

You have to think in teams "because you can't do everything yourself." A team is an entity
that should produce a given output, at a quality and level of reliability you can depend on,
because that output is the input to another process or system. **Choosing a team is not unlike
evaluating a piece of software** — you expect it to do a job so you are free to focus on the
output. "Imagine if every time you opened up Microsoft Excel to review your budget, you had to
manage the code base, tweaking and compiling it before you could see your numbers."

A good team can think for itself, take on responsibility, and be given the latitude to make its
own decisions in pursuit of its purpose. That requires four things the chapter supplies in
order: a **charter**, a **structure**, a **title ladder**, and a **reporting framework** —
followed by one-on-ones, training, and a communication policy.

## Frameworks Introduced

- **The team charter** — purpose, customer, resources, skill requirements, key metrics.
- **Four team structures** — product centric, lifecycle centric, customer/vertical centric, hybrid.
- **Table 6.1 title ladder** — Junior → Associate → Senior → Lead/Manager → Architect →
  Divisional Manager/Vice President → Fellow.
- **Table 6.2 rules of engagement** — three autonomy buckets: no reporting required / keeping you
  informed / decision required.
- **Table 6.3 one-on-one meeting format** — in-scope vs. out-of-scope topics.
- **The no-update update** (Kelley Powell).
- **Seven learning channels** — online libraries, certifications, bootcamps, user groups, internal
  training, conferences, university courses.
- **Meeting taxonomy** — engineering meeting, town hall, team meeting, ad hoc, plus interactive
  meeting rules.

## Key Concepts

- **The charter litmus test.** "If you were to poll every member of a given team now about what
  their purpose is in one sentence, how many different responses would you get? If everyone gives
  you something completely different, then you don't have a team — you have a collection of people
  drawing a salary."
- **Team knowledge must survive any one person.** "A solid, strong team should and must tolerate
  the loss of any single member without adversely affecting the output of the team as a whole."
  The three necessities: onboarding/training materials specific to the team's goal (a release team
  would train on Jenkins/SonarQube); a knowledge base (wiki, document store — or videos, which are
  sometimes quicker to produce); defined processes/structure, documented not inferred (coding and
  formatting standards, how to use version control when working on tickets).
- **Two immovable data points on structure.** (1) There is only one CTO, responsible for everything
  the group does, good and bad — the structure must give you visibility so you are never
  blindsided. (2) **More than five to seven direct reports becomes impractical.** Exception:
  support and testing teams, where many people at a similar level do a similar task with little
  day-to-day management — and that manager should be experienced and will most likely not be from
  a technical background.
- **Plan as if you are double the size you are now.**
- **What a title embodies:** responsibilities, experience, salary, seniority, industry recognition.
  "No matter who may profess that a title doesn't matter to them, they really do care, even if
  their bravado won't let them admit it." Would you rather the person fastening your seatbelt was
  introduced as "Captain" or "Junior Pilot"?
- **Titles are near-irreversible.** "Once a title has been assigned to a person, removing that, or
  reassigning it, can be seen as a demotion, may have to involve HR, and will have demoralizing
  effects."
- **Do not confuse the experience with the title** — "operating at a Junior level for five years
  does not make someone a Senior."
- **Metric swim lanes.** Resist having teams compete — you'd be comparing apples to oranges.
  Compare each team only against its historical or projected self.
- **Accountability defined.** "Accountability means taking responsibility for the error, getting to
  the root of the problem, and putting in the necessary practices, processes, and even tools to
  prevent them from being repeated. If nothing changes or is repeated time and time again, then
  accountability means removing those people responsible from that position."
- **Their failure is your failure.** "It is not good enough to point the finger and blame a weak
  manager. They are your manager." And private equity "doesn't have the time to tolerate too many
  mistakes... and you will never see it coming."
- **Transparency ≠ telling everyone everything immediately.** You will learn confidential
  information from other C levels not yet ready to share. Transparency is "being respectful and
  thoughtful enough to take the time to communicate information in a forthcoming way."
- **Email vs. chat.** Chat systems (Slack, Mattermost, Cliq, Teams) are real-time and transient —
  not a historical record. "Do not use them as a way to broadcast critical information that you
  wish everyone to read. Stay with email." Create a group alias (engineering@xxx.com) and keep it
  current as part of on/offboarding.
- **Email never gets deleted.** Companies keep copies in audit vaults that cannot be manipulated
  (trivial to set up with G Suite or Office 365). "The company owns that email; it may be used for
  or against you."

## Mental Models

- **The LEGO people planning board.** Like a general devising battle strategies, the author lays
  LEGO figures on a large sheet of paper to visualize team composition and draws the lines of
  responsibility between them — it shows quickly whether something was missed or an area
  overprovisioned. ("If you're using the Star Wars LEGO set, remove Darth Vader — no one wants to
  be assigned to that role.")
- **Would I be happy working for me?** Your managers largely model their style on their only real
  frame of reference — their own boss, which is now you. "For the record, there have been times
  throughout my career, I would have to answer NO to that one."
- **"Spider," From the Earth to the Moon.** An engineer sheepishly admits he dropped a rounding
  error that kept crashing the lunar module, expecting to be fired. The lead Grumman engineer:
  *"Did you hide this? Did you try to cover this up? No. It's bad. This is very bad. But go home,
  get some sleep, and let's get this resolved."* Had he shouted and cursed, would the next person
  have spoken up? "We may have never gotten to the moon, in that case."
- **Write as if it will be published and live forever** (Jeremy Geelan — journalist, BBC producer,
  publisher). Before pressing Send, reread and ask whether you'd be happy for this to be reprinted.

## Anti-patterns

- **The micromanager.** Oversees every aspect, details exactly how things should be done, checks
  every step, gets in the way — and justifies it as maintaining quality. "They are a bottleneck to
  scalability and always are a huge detriment to team morale." In PE consulting work, if they can't
  get out of their own way, they are let go.
- **Making changes to inherited teams before you understand why things are the way they are.**
- **Pulling a group of people together and telling them they are a team.** "It never works. They
  have to be given the tools and structure to be successful."
- **Four title mistakes:** too rapid a promotion (never as a result of a single win or event);
  **title in lieu of salary increase** ("all this does is give them a reason to start looking
  around for roles with a similar title for the money they should be paid"); rewarding longevity
  with a title — "one of the most common problems new CTOs find themselves inheriting"; being too
  clever with titles.
- **Vanity titles.** The author's own "Chief Knowledge Officer" at a well-funded Silicon Valley
  startup ~20 years ago — in reality he was an Evangelist. "We kept getting overlooked because our
  potential customers could never figure out who they should be speaking to."
- **Metrics for the sake of graphs.** A good metric shows progress holistically and feeds the report
  you compile for management; it should fall out of the team's core tool (JIRA, SonarQube, Jenkins),
  not become a project in itself.
- **Goal-less teams.** "A team that has no goals will never know whether it is falling short or
  exceeding your expectations. That is unfair to the team and opens you up for subjective emotional
  rulings."
- **Using the one-on-one for project updates**, new tasks, discipline, or gossip.
- **"My door is always open."** Be cautious — people will use it. Never get sucked into an on-the-spot
  conversation; ascertain whether it needs more than a one-sentence answer and ask them to book 15
  minutes. "Simply dismissing someone or listening to someone while you still type at your keyboard
  is the worst thing you can do."
- **60-minute default meetings.** "Too many people default the meeting length to 60 minutes; this is
  a mistake, and you will slowly get your time sucked away."
- **Using the group distribution list for informal topics** — it destroys the precedent that this
  email matters.

## Reference Tables

### Table 6.1 — Title ladder: technology (as printed)

| Title | Experience | Notes |
|---|---|---|
| Junior | 0–2 years | Typically entry level; recent graduate starting position |
| Associate | 2+ years | Has industry experience but is still learning their trade |
| Senior | 5+ years | Knows their trade very well, to great depth |
| Lead/Manager | 5+ years | Ready to take on the responsibility of managing the output of others |
| Architect | 10+ years | Has a deep knowledge in a number of disciplines within their sphere and responsible for thinking broader |
| Divisional Manager/Vice President | 10+ years | Responsible for more than one team that is focused in a defined area; usually involved with budgets |
| Fellow | 15+ years | Special title, like a lifetime achievement, or a university tenure, to recognize long-term continued efforts |

Each title should ideally be tied to a discipline — "Senior Developer," "Support Manager."
Cautionary notes: at IBM "everyone, it would appear, is a Vice President of something"; and
"Director" carries a legal meaning in the United Kingdom, where **Managing Director** is used
instead of Chief Executive Officer and a Director is legally listed as a steward of the company
with the company registration.

### Table 6.2 — Rules of engagement (as printed)

| No reporting required | Keeping you informed | Decision required |
|---|---|---|
| Interviewing potential candidates | Any problem or issue concerning a client | Final interview of top candidates under review |
| Internal processes pertinent to the team execution | Any production problem | HR consultation or termination considerations |
| Team one-on-one meetings; PTO management | Security concerns or issues | Purchase or licensing of tools |
| Project execution | Changes in how the team interacts with other teams | Change of technology or tools |
| | Any problems with individual team members | |
| | Timelines; updates | |

### Table 6.3 — One-on-one meeting format (as printed)

| In-scope topics | Out-of-scope topics |
|---|---|
| Personal goals | Project updates |
| Areas for improvement | New tasks you want them to address |
| Concerns they have regarding their execution | Disciplinary talks |
| Analysis of a recent event | Gossip about other members |
| Company updates | |

Cadence: weekly or every second week, **15 to 30 minutes**, strictest confidence — "some even
think of them like mini therapy sessions." Never postpone or cancel them.

### Four team structures

| Structure | When it fits |
|---|---|
| Product centric | Each product line has its own development, testing, support, under one manager reporting to you. Test for: different technology stacks; nonoverlapping client bases; independent release cycles; uniquely challenging support issues; significant revenue disparities. Runs as a mini-ecosystem; also good when the long-term goal is to merge or integrate a product line |
| Lifecycle centric | The most popular model for in-house software — an assembly line: development, testing, release, DevOps/infrastructure, support. Works when the SDLC is well defined and adhered to. Early on people serve many roles; carve out teams as deeper skills are needed |
| Customer/vertical centric | Large clients or verticals with unique requirements on the same underlying platform — e.g. a Salesforce-based offering configured differently for health care, financial, and education markets |
| Hybrid | A bit of everything at once — common in large organizations with historical products, and during a reimagination project with evolving legacy products. "Teams will flex — what works one day may not make the same sense in a year's time" |

Review your structure **once a year** and ask whether it is serving the business optimally.

### Common reporting metrics

| Metric |
|---|
| Open/closed tickets |
| Support interactions |
| Release metrics (quantity and time it takes) |
| Performance/load metrics |
| Technical debt paydown |
| Goal/sprint progress |

Frequency: weekly or every two weeks. "Anything longer than that, then too much can happen
without you getting a real line of sight to head off any potential problems."

### Meeting types

| Type | Cadence | Length | Content |
|---|---|---|---|
| Engineering meeting | Weekly or biweekly | ≤30 minutes | Celebrating major releases, customer acquisitions or losses, process improvements, structural changes. Keep the format steady so people know what to expect |
| Town hall | As and when the need arises | — | One person presents a specific topic, then open Q&A. Good for a new initiative, a change in product direction, or inviting another leader to explain their group's work or a decision's impact |
| Team meetings (team leads) | Once a week | ≤60 minutes | High-level issues per team; table detail and schedule a follow-up with only the necessary parties. Equal time per lead, clear agenda order, room at the end for any other business |
| Ad hoc | — | Default 15 or 30 minutes | One or two named agenda items so everyone knows what to expect |

### Interactive meeting rules

| Rule |
|---|
| The agenda should be distributed in advance, including any reading materials |
| Have only the people who will contribute; do not include people simply for keeping them in the know |
| No interruptions are permitted; cell phones are off the table |
| No catering — the meeting should not be long enough that you need snacks |
| Laptops are used only for presentations; it is too easy to become a distraction while checking emails or messages |

Minutes should be taken at a minimum, meaning noting any action items people agreed to.
Further reading the book recommends: **Cameron Herold, *Meetings Suck*** (ISBN-10 1619614146).

### Group-wide email topics

| Topic | Guidance |
|---|---|
| Hires/additions | Quick introduction, brief summary of accomplishments, the role, and the start date — send **before** the person arrives so people are more welcoming |
| Terminations/resignations | Give the date; invite others to share best wishes. Even with terminations, wish them the best: *"We've agreed mutually that our journey together has come to its conclusion"* is all you need |
| Company announcements | Follow-up that contextualizes what a broad announcement means for your group |
| Group announcements | Major milestones and successes; recognizing exceptional team members; also failures or missteps, without finger-pointing, plus the remedies |
| Industry trends that may impact the company | Use cautiously — major shifts only, not every interesting link (use a public Slack channel for those) |

### Email writing tips

| Tip |
|---|
| Use your natural voice; don't try to be too formal |
| Do not use any profanity, innuendos, or slurs |
| No personal attacks no matter what |
| Be honest: no conjecture, no hyperbole |
| Keep it professional. Keep it on point. Keep it honest |

### Education and training channels

| Channel | Notes |
|---|---|
| Online libraries/resources | Publisher subscriptions to back catalogs; Khan Academy and Coursera for free formal courses; Udemy for a few dollars a course. Vet them with your team lead. Ideal for self-paced learners |
| Platform/software certifications | Amazon, Microsoft, Cisco, Oracle. Prices have fallen to a few hundred dollars (from thousands). Proves training was completed and tested — good for their résumé and your assurance |
| Bootcamps | Intense immersive week, or spread over weeks in evenings or lunchtimes; in-person classroom with a teacher plus networking. Priced for individuals, so no significant budgetary hurdle |
| User groups | Monthly, evening, pizza and beer, guest speaker. Great networking. Mostly unfunded — offering your office space and sponsoring food also showcases your company |
| Internal training | Subject matter experts run sessions; or brown bag lunches — "it is always a goodwill gesture to buy lunch for everyone who attends" |
| Conferences | Pack many sessions at differing levels. Expensive with travel; reward top performers with the trip and have them present one or two summary sessions on return, or send someone with a specific learning task |
| University/college courses | Thousands of dollars over years. Consider sponsoring all or part with class time during the working day; you can require a designated number of years' service after graduation or repayment of the investment. Be thoughtful so as not to create discontent with those who don't receive it |

Standing economics: "Training and retaining existing employees is so much cheaper than
terminating, with resultant training and onboarding new employees."

## Worked Example

### A complete charter for a support team on an online product (as printed)

- **Purpose** — To provide end users with live support with everything associated with the online
  user experience, including helping out the account managers with any initial onboarding issues.
  - The team is responsible for any data changes that may need to happen but will not make any code
    changes to fix any bugs or add features. They are responsible for filing any bug/feature tickets
    for the development team.
- **Customer** — Any user of the product, including internal and external users.
- **Resources**
  - Knowledge base of the internal data structures via an internal wiki
  - Read-only access to the production data stores
  - Elevated access to the production data stores for key individuals
- **Skill requirements**
  - Familiarity with the web, including online video/chat systems
  - Strong written and oral communications
  - SQL skills
- **Key metrics**
  - Time-to-close resolutions
  - Total interactions in a given period

The charter "justifies the existence of the team and how they will contribute to the overall
success of the group." The team leader then produces a plan to fulfil it, including any resources,
budget, and tools required — and **you leave that leader to execute it**, held accountable but
supported and guided.

### The no-update update

Taught to the author by **Kelley Powell** (CEO/author), then Head of Client Operations at Royall &
Co, for situations with a lot of visibility. Powell insisted on updates **even when there was no
resolution yet** — just that things were still being worked on — via email or text, **every 30 or
60 minutes depending on the severity** of the issue.

The author's own adaptation: he asks for updates **on the top and bottom of the hour**, so people
can watch the clock rather than remember when the last update was. "It doesn't need to be time
consuming or detailed — just a quick note to say things are still being worked on."

Paired rule, from his own practice on client visibility: he must be made aware of **any problem
related to client disaffection or security breaches or concerns, no matter the time of day** — and
the whole team had his cell phone, with instructions to call if he did not acknowledge an email or
message. "I never wanted to be in a position where my CEO was asking me about something that I had
no idea about."

### Coaching in the one-on-one, by question rather than instruction

When the author has witnessed something that could have been done better, he starts by asking how
*they* felt it went — did they get their message across? Did the room understand exactly what they
were asking? Then, **instead of saying how you would do it**, he suggests different ways they could
have done the same thing and asks whether those would yield different results. The aim: mentorship
"in a format that doesn't come over as patronizing or lecturing."

## Key Takeaways

- Every team needs a written charter; new joiners should know their team's collective role and their
  own part in it within days.
- There is no perfect structure — evaluate product/lifecycle/vertical/hybrid against your needs and
  revisit yearly.
- Titles are badges worn outside your organization; use industry-standard ones and never grant them
  in place of money or for longevity.
- Define autonomy explicitly in three buckets so team leads never guess whether to escalate.
- Agree metrics, then hold leads accountable to them — accountability is root-cause plus prevention,
  and ultimately replacement if nothing changes.
- Protect the one-on-one; it is where soft-skill correction happens early and cheaply.
- Fund continual learning across several channels; it is cheaper than replacing people.
- Openness and transparency make people trust you and give them more confidence in your decisions.

## Connects To

- [Chapter 5 — Interviewing, choosing, and onboarding](ch05-interviewing-choosing-onboarding.md) —
  where the new team member comes from and how they are introduced.
- [Chapter 7 — Annual reviews](ch07-annual-reviews.md) — the skill matrix, handling failure, and
  termination that follow from the title ladder and accountability model here.
- [Chapter 4 — Building a team](ch04-building-a-team.md) — the discipline matrix that the training
  program is aimed at, and the resource types the structures are built from.
- [Chapter 2 — Managing up](ch02-managing-up.md) — the same communication discipline aimed upward.
- [Chapter 11 — Documentation](ch11-documentation.md) — the knowledge base and defined processes a
  team charter depends on.
- [Chapter 9 — Development](ch09-development.md) — the SDLC that the lifecycle-centric structure
  mirrors.
