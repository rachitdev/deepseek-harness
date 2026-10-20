# Chapter 2 — Managing up

## Core Idea

Technologists communicate inward comfortably — "birds of a feather flock together" —
and stumble when they turn outward to a world with different words, language, and
competing objectives. The secret of working with each level up and across is **empathy**:
knowing to whom you communicate, their motives, desires, and concerns. A single misplaced
phrase can send a team into a frenzied panic. The CTO's job here is bridging the divide
with the CEO, the CFO, peers, the board, the company at large, and the outgoing CTO whose
seat you may be taking.

## Frameworks Introduced

- **The office of the CEO** — stop thinking of the CEO as a person; think of them as an
  office. Everything you say is a sound bite they will reuse, in their language, to whoever
  asks them next (board, PE partners, external investors).
- **Three types of CEO** — nontechnical; nontechnical who thinks they know technology;
  technical.
- **The three-bucket burn rate** — Development / Production / Support.
- **PIP** — Problem, Impact, Prevention — the reporting format for outages and breaches.
- **Four kinds of internal politics** — fiefdoms, bullying, passive aggression,
  information hoarder — plus five mitigations.
- **Change management set** — boiling the frog, the excitement committee, the valley of
  anguish, burn the ships.

## Key Concepts

- **Lead with data.** "Emotions, feelings, and gut instinct can all be argued subjectively.
  Data is much harder to dispute." Hiring case = show the productivity gain. Project case =
  show the real financial impact (money saved in unnecessary support or licenses).
- **Sophie's choice moment.** When no path is good, bring the CEO into the decision, lay out
  the consequences of each path so they are fully vested with you. "Such a big decision
  should not be just your responsibility."
- **Think slow, act fast.** Resist the social pressure to say yes in a big meeting. Take
  requests under advisement, take the space to think through the permutations — then execute
  with no second guesses and no procrastination.
- **Build for input/change.** A yes/no proposal has already cut your success odds by 50%
  before you start. Offer relevant options and a phased, incremental approach with wins along
  the way.
- **No such thing as bad news.** Own it, present without emotion or blame, never name the
  team member responsible. Rule of thumb: **if it affects clients or revenue, it goes to the
  CEO immediately.** "Your worst nightmare is when your CEO tells you about a problem you
  already knew about."
- **Your department is a cost center.** It does not directly generate income; in accountancy
  terms you cost the company money. For every dollar you ask for, show the value that dollar
  returns.
- **Office politics defined** — "the absence of data that is replaced by emotion."
- **The secret to a good CFO relationship is no surprises.** They want predictability and
  dependability on your numbers.
- **Burn rate litmus test.** "If you don't know your burn rate at this precise moment, then
  that is your litmus test."

## Mental Models

- **The orchestra.** The company as an orchestral movement; your string section matters but
  many sections combine, with the CEO conducting. Sometimes the CEO "is just waving their
  arms around, throwing everyone off."
- **Metaphors as translation layer.** Use metaphors for both technical and nontechnical
  audiences — with a technical person a metaphor frees them from thinking too much in the weeds;
  a nontechnical person doesn't feel inferior and can still contribute to the resolution.
- **The carpenter's lament.** "The one before you was an idiot, and the one that comes next
  will think you were an idiot."
- **Poisoned apple.** A technical CEO looks like nirvana, but they may dismiss real challenges
  as trivial and question every decision — not malice, just their comfort place.
- **Boiling the frog.** Drop a frog into boiling water and it jumps out; raise the heat slowly
  and it stays. Introduce change so people barely notice the temperature change. Agile's
  small-release-often mentality lends itself beautifully to this.
- **Burn the ships** (Milbery). Identify the moment the old system is switched off and cannot
  be used. "The last thing you want is to be maintaining two systems."
- **Ugly baby.** Never disparage the current way of working — "as ugly as a baby may be, the
  mother believes it is beautiful."
- **A software demo can speak a thousand slides** (Devin Mathews: "Don't just tell, show").

## Anti-patterns

- Going into too much detail too quickly; the listener usually only wants the result or
  implication — how does this affect our customer?
- Dismissing or underestimating the nontechnical CEO, the most common type.
- "Quick win" arguments — throwing out an opinion as fact. Kedric Griffin (senior director of
  engineering, EAB): "Faith comes from time, trust, and confidence in you... It only takes
  getting caught here once to lose all the confidence and faith you had built up."
- Being divorced from your department's financials — "a dereliction of duty."
- Disparaging the outgoing CTO, in any circle you think is safe. It also cuts off the ability
  to lean on their historical knowledge later.
- Rewriting something because you don't know how it works. "Don't fix what isn't broken."
- Making many changes fast after taking over a loyal team — you become the enemy who ousted
  their beloved leader.
- Overengineering a peer's fix (recommending a CRM when moving them to a shared Google Sheet
  removes the pain without disrupting their workflow).
- Board mistakes: hiding behind a laptop, correcting your CEO in the room, hanging around
  after your slot, opening up in the informal drinks.
- Blindsiding the business with change without the CEO and department leads behind it.

## Reference Tables

### Types of CEO

| Type | Signature | How to handle |
|---|---|---|
| Nontechnical | Traditional management or finance background; knows the technology only as it relates to the customer. The most common type | Your role exists to take the logistics and headaches off their desk so they can focus on the business of business. Do not take them lightly |
| Nontechnical who thinks they know technology | Talks confidently with buzzwords that sound plausible to the untrained ear; nods knowingly, giving false security | Course-correct discreetly and in private, without making them look wrong in front of their team |
| Technical | Speaks your language, but may dismiss real challenges as trivial and question every decision — architecture, production, delivery, hiring | Healthy challenge is fine; recognize the smothering as them returning to their comfort zone. A good one lifts you up and develops your career |

### Know your CEO — the discovery list

| Question |
|---|
| Their background (Google, LinkedIn) and previous experience |
| What motivates them; what makes them happy/sad |
| What success looks like for them |
| Detailed/micromanager or big-picture type |
| Have a conversation |
| Plus: ask openly, "Is there anything I should know about Person X that will help me be successful?" |

### The three expenditure buckets

| Bucket | What it contains |
|---|---|
| Development | Staff costs (salary and computer); software tool licenses (IDE, project management tools); software runtime licenses (database, libraries); hardware/cloud for development and testing; third-party vendors such as contractors; training (courses, books, conferences) |
| Production | Staff costs (client setup or onboarding of client data); hardware/cloud costs; software runtime licenses; third-party services (domains, APIs); security audits |
| Support | Cost of things going wrong, client help, and maintenance (monitoring log files, validating backups). Should track production costs. Break down to an hourly cost so you can price a ticket type |

Staff costing rule: use **120%–130% of salary**, not salary alone, to cover benefits,
equipment, furniture, and coffee.

### Table 2.1 — Example monthly production breakdown (as printed)

| Area | Monthly |
|---|---|
| AWS cost | $15K |
| DevOps (one person) | $2K |
| Software license (JIRA and GitHub) | $0.5K |
| Support (two persons at 10K each) | $20K |

### Questions for the outgoing CTO

| Question | What it buys you |
|---|---|
| What kept you up at night? | Ongoing concerns with no immediate solution — usually things you'd never think of |
| What pressure did the business place on you? | Seasonality, or a stronger voice than others directing output |
| Who is the most important client? | The one the company bends over backward for; avoids stepping over legacy goodwill |
| Which client placed the most overhead on your department? | Time sink or complexity; often a bespoke feature that became a headache |
| Who in your team did you rely the most on? | Possibly the most important question — who keeps the lights on |

### Board presentation: dos and don'ts

| Do | Don't |
|---|---|
| Know the agenda and the overall message your CEO is conveying | Disagree with or correct your CEO/CFO outside your area — note it, follow up after |
| Keep the presentation on point; explain complicated initiatives relatably | Be intimidated into a rash decision — commit to an answer within a couple of days |
| Be prepared to field questions from left field (you don't know what else was discussed) | Blame, disparage, or name anyone when reporting bad news |
| Take notes with pen and paper to show engagement | Take your laptop — or use it sparingly for a demo and close the lid |
| Ask the CEO or chairperson if it's okay to leave once you're done | Hang around and be told to leave |
| Be on your guard in the informal sessions | Open up — "these people are very adept at making you open up" |

### Internal politics: types and mitigations

| Type | Description |
|---|---|
| Fiefdoms | Term from the early 1800s: control asserted by one person over an area. In business, people serving their own goals first in a semi-loose alliance. An unofficial club, hard to prove, but felt |
| Bullying | Distinct from assertive: inconsistent decisions or actions, intimidating, irrational, very rude |
| Passive aggression | Not dealing with the underlying issue directly: dragging feet, voicing approval to the group then undermining the initiative outside |
| Information hoarder | Believes sharing valuable information diminishes their role and puts them at risk of termination |

| Mitigation | Move |
|---|---|
| Recognize it in yourself | Ask a trusted colleague or friend for feedback before expecting change in others |
| Call it out | In the moment: "Wanna try that again, this time without the aggression/sarcasm?" Humor and directness disarm |
| Avoid ambiguity | Ambiguity gives them room to change their mind but pretend it was their intent all along |
| Clarify and confirm | Concede nothing until they specify precisely what they are angry about and you confirm you understood |
| Welcome criticism | Ask for a specific example; pushing back for detail removes the urge to be defensive |

Plus the **24-hour rule** (Jeff Hanner, retired data warehouse manager): you have 24 hours
from the infraction to address it head on; after that it is never to be mentioned again.

## Worked Example

### The "missing mailbox" translation

A key third-party API stopped working, hurting clients. The cause was on the vendor's side:
a DNS change resolving to the wrong address. Williamson knew the incident would be explained
to many people of differing backgrounds, so he told the CEO that the vendor **had updated
their mailbox and accidentally put the wrong number on it** — so all deliveries were going to
an unknown box, and while they worked on getting the right numbers on it, deliveries were
being held back.

The CEO instantly understood. Getting into DNS resolution and time-to-live values would not
have advanced the situation. The metaphor then propagated: it was repeated across client
success and sales, and the status update he received was simply, *"Has the mailbox got the
right number yet?"*

### The password-reset ROI, built from support cost per hour

Because support desk costs had been broken down to an **hourly cost**, and because ticket
data was tracked, the team discovered that **60% of all ticket requests were password reset
requests**. Putting a real financial amount on that made the budget decision trivial: install
a password reset mechanism. It showed **ROI in the first month of going live**.

The enabling discipline: support desks often fail to track useful data points. One commonly
overlooked field is **the actual client (or product line) the ticket was raised against**,
which frequently unearths that one particular client consumes far more support than expected.

### Daily-rate costing for future budgets

The book's worked arithmetic for making future budgets easy:

- Typical web developer salary: **$100K**
- Add 20% overhead: **$120K**
- Per month: **$10K**
- Per day (assume 20 working days per month): **$500**
- A future project demanding **14 days** of a web developer: **14 × $500 = $7,000**

No hunting around, no guessing at a rate. "Your budget suddenly got a lot more informed."

## Key Takeaways

- See your CEO as your partner, not your boss; meet one-on-one on a regular cadence and
  revisit "how can I be successful for you?" at least quarterly.
- Communicate so the CEO can relay it upstream in their own language without getting bogged
  down in detail.
- Work the CFO relationship until your numbers are trusted; know your burn rate today.
- Lunch with each peer at least once a month; learn their language and use it without imposing
  yours; look for the simple fix to their human-intensive pain.
- Taking over from a CTO: celebrate, never disparage; make changes slowly enough that the team
  wonders how they managed without you.
- A board invitation is an honor, not a chore — and don't just tell, show.
- Change is not a software install: it challenges established business processes, employee
  training, client onboarding, and inherited company know-how. Budget time to bring people along.

## Connects To

- [Chapter 1 — The Chief Technology Officer](ch01-the-chief-technology-officer.md) — the role
  definition these relationships operate inside.
- [Chapter 3 — Visionary planning](ch03-visionary-planning.md) — budgets and ROI are developed
  in full there; the elevator pitch is the CEO/board version of this chapter's communication work.
- [Chapter 9 — Development](ch09-development.md) — agile's small-release-often as the delivery
  form of boiling the frog.
- [Chapter 12 — Security](ch12-security.md) — the "we are under attack" communication plan is
  PIP under pressure.
- [Chapter 14 — Company growth](ch14-company-growth.md) — handing over and taking the reins,
  the fuller version of taking over from another CTO.
