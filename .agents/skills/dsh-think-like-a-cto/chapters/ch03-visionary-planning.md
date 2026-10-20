# Chapter 3 — Visionary planning

## Core Idea

Keeping the lights on is not enough — you have to make sure they stay on and shine bright
for a long time. The CTO owns a **grand vision**: a technology vision statement focused on
the output of your group that supports the company's overall vision and mission. It states
what you are working toward, why you are heading that way, and **what you and the business
will be able to do once you get there** — the part most people forget.

Horizon rule: **two to five years out.** Anything longer is hard for people to relate to,
because the business may change, pivot, or be sold before the vision is realized. "A good
vision is one that can be delivered and celebrated. You never want to be the leader for whom
'tomorrow never comes.'"

Scale rule: **if your vision can be executed in a few months, it isn't a vision, it's a
project.** A vision is a multiproject endeavor, typically revolutionary, sometimes
evolutionary during transformational change.

## The running case company — HomeMax PLC

The chapter introduces a fictional company used throughout: **HomeMax PLC**, 40+ years old,
providing **home warranty insurance for residential homeowners**. Running on legacy technology
built in a mixture of **RPG and Cobol**, driven through terminal software against a **Db2
database on AS/400s**. Never had a formal CTO — the founder directed a small team of
developers. Recently **purchased by a private equity firm** keen to expand their market, who
brought in their first CTO to drive the technology transformation.

- HomeMax company vision statement: *"Gaining peace of mind about your most valuable asset."*
- HomeMax technology vision statement: *"A self-driven, customer data–centric platform."*

## Frameworks Introduced

- **What/Why vs. How** — two audiences for one vision. The business (plus board, investors,
  clients) gets the what and why, jargon-free. Your own group gets the how, with meat on the bones.
- **The eight-step elevator pitch exercise** (see Worked Example).
- **Pillars of the vision** — foundational, replaceable components you can watch and swap.
- **Fast / Good / Cheap** — pick two; used to build three budget models.
- **Vision-planning "keep in mind" checklist** — seven questions.
- **PIP-style progress communication + narrative control** against feature creep.

## Key Concepts

- **Vision statement vs. mission statement.** The vision statement is the goal of the mission,
  the end result; the mission statement is the action-oriented version, detailing why the
  organization exists, what services and products it offers, and what makes it unique.
- **The vision test for your team:** "Do they believe in your vision?" If the people down in the
  trenches on client problems, bugs, and features can't get behind it, either the vision is wrong
  or you haven't broken it down to a point that feels real.
- **Every decision as a vector.** For every decision you and your team make, ask whether it takes
  you a step closer or a step farther from your vision — the new hire, the vendor contract, the
  technology stack.
- **Pillars.** The biggest components you rely on to support the vision's execution — "not the
  science or business rules you have to create, but the areas that ideally you would like to
  outsource... the commodities that your vision relies on." Any given pillar can be replaced as
  long as the others keep holding the vision up.
- **Splinter team.** A small team spun up for a few days to dive deep into a very narrow area
  and fetch the intel for a more informed decision — the response to a pillar shifting.
- **Commoditization / "cloud."** When a process or technology reaches a state of acceptability
  in role, innovation, and competition. Web and mail servers; more recently databases —
  database-as-a-service versus a team of DBAs doing backups, optimizations, and failover.
- **Sunk cost.** Like a poker player who sees the hand won't win: it doesn't matter how much you
  have bet so far, pull out. The book's advice when a cloud provider nullifies your bespoke system:
  at first stick with what you have, let the service shake out, confirm it has everything you need — then switch, reduce operational burden, and move on.
- **Cloud-deflation paradox.** Cloud services strive to drive costs down, but the savings are
  usually lost as usage increases: because we think it's cheaper we use more than we need,
  believing it's well within budget. Build inflation into multi-year budgets anyway.
- **Naming the vision.** "We name things to give them personality, to give them emotion." Real
  names cited: **Project Gandalf** (a legacy transformation in two phases, Gandalf-the-Gray then
  Gandalf-the-White), **Codename Borg** (a large data warehouse), and **the Gump Platform** (an
  IoT device management system whose primary requirement was to keep running all the time —
  Forrest Gump was advised to just keep running). Large warehouse projects tend to be called the
  Death Star.
- **Business fatigue and the valley of anguish.** The point where the business forgets it was
  going to take this long, and the old legacy system starts looking attractive — "better the
  devil you know."
- **Feature creep.** Smaller features keep being added because each seems too small to matter —
  the straws added to the camel. By delivery time expectations are nowhere near reality even
  though you hit every goal you set.

## Mental Models

- **A CTO without a vision is a captain without a destination** — you have to know where you are
  going to know when you have arrived.
- **How do you eat an elephant? One mouthful at a time.** Break the vision into bite-size chunks
  the business can feast on and feel progress from. "Whatever they say at the start, they will
  never have the same patience as you to wait for delivery."
- **Skip before you jump.** Warm the business up with smaller steps (move old servers to the
  cloud rather than renewing hardware) instead of leaping from traditional servers to cloud-native.
- **Building a house.** Report progress as foundation → plumbing → electric cabling → drywall →
  internal doors, so non-software people get a genuine sense of progress during the invisible
  early months.
- **The Telephone game.** "Business is just one big Telephone game" — how client requests reach
  you distorted; use your counterpart relationship to ask the basic question, *what are you
  trying to solve?*
- **Starter's guilt.** The CTO's version of buyer's remorse — the feeling that the whole
  project hinges on one decision. "No decision is irreversible or, at least, can't tolerate a
  small course correction."
- **Warren Buffett on schedule compression:** "You can't produce a baby in one month by getting
  nine women pregnant."

## Anti-patterns

- **Bandwagon visions.** The CTO whose vision was "We're integrating machine learning into our
  analysis" — no thought about what it meant to the business or how clients benefit, the "how"
  never given to his own group, who were "left chasing a shadow, trying to solve a problem that
  didn't exist." He had jumped on the bandwagon because it sounded cool and thought it would help
  his career.
- **Decision paralysis.** "When it comes to technology, any decision is better than making no
  decision. What is right at the time, given all the data and reasoning, will always be the right path."
- **Un-named projects.** No name, no emotional investment.
- **Jumping on client requests.** Offer no solutions, no promises, no assurances; just listen.
- **Solving the symptom rather than the problem** (the daily emailed sensitive-data file; see
  Worked Example).
- **Budget omissions.** Leaving out in-house personnel costs "because the business already pays
  for them" — ignoring the opportunity cost of what they are *not* working on. And leaving out
  hardware, software licenses, third-party services and cloud resources, or failing to phase them
  across the project timeline.
- **Sticker-shock budgets.** Never put together numbers you think your CEO wants to see. "No good
  ever comes from such a poor budgetary start."
- **Overbudgeted, underutilized resources.** They lull you into thinking you're under budget when
  in fact someone was on vacation or working elsewhere.
- **Being too polite to correct misconceptions.** Better small disappointments now than defending
  a project that failed to live up to hype it never claimed.
- **Celebrating too loudly too early** with the business. Celebrate sprints internally; save the
  bigger celebration for delivered value the business can benefit from.

## Reference Tables

### Table 3.1 — Visionary statement examples (as printed)

| Statement | What | How | Success |
|---|---|---|---|
| Deprecate all hardware and move exclusively to the cloud. | Remove the overhead of maintaining physical assets so we can move to a pay-as-you-go model that will accelerate the onboarding of new clients. | Stop purchasing hardware, start learning cloud alternatives, and identify the areas where we can have early wins without putting the business at risk. | When the last server is turned off and the reliance on the office/data center is zero. |
| Move toward an API-driven architecture. | Make our data/services available in a secure and reliable fashion, which lets our customers directly integrate with us without transferring files. | Identify the touchpoints our clients need to send and receive data, as well as how we move data around internally, to free ourselves up from data overhead. | Client interacting without needing to send or attach files; self-servicing using the API. |
| Replace the legacy system. | Move the system to a modern, supported architecture that will enable us to react to the requests that clients have been asking for with significantly less effort. | Extract all the business rules and responsibilities of this system, and then start designing one or more systems to take over in a phased approach. | When the legacy system is finally turned off, supporting no more clients. |
| Empower the client with a modern web and mobile interface. | Remove our call center from managing all interactions, instead letting customers do it themselves. | Using a modern web framework, talking securely to backend APIs. | Seeing a dramatic reduction in the number of calls being actively answered. |

### Client discovery questions

| Question |
|---|
| What system/service is the primary point of contact with our company? |
| If you exchange data, where do you source that data? |
| What happens to the exchanged data once you receive it? |
| Typically, how do you use the service each day? |
| What is your team relying on to be available for them to be successful? |
| What aspect of your processes are you looking to improve or change in the near future? |

### Vision planning — keep in mind

| Question | Why it matters |
|---|---|
| What is the overall problem you are trying to solve? | Makes the "why" easy to answer when the business asks you to justify the vision |
| What are the benefits to the company and/or client? | Flexibility to client wants, or opening up new opportunities |
| Is this time sensitive? | If there are better times of year, say so and use it for greater planning |
| What are your criteria for success? Are they aligned with the business? | "Remember, your pain isn't necessarily their pain" |
| Who are the main people you need for success? | How crucial are they to day-to-day running; how will you backfill or minimize their absence |
| Is this principally a modernization/reimagination project? | Clients and the business may be quite happy — think harder about why you're changing the status quo |
| What impact will this have on the continuity of the business? | Large initiatives are always a distraction; even extra resources need managing |

### Three budget models from Fast / Good / Cheap

| Model | What it represents | What you vary |
|---|---|---|
| Best case (good) | Every cost captured as accurately as possible, reviewed line by line with the CFO | Baseline |
| Faster (fast + good) | "If money were no object, how fast can you deliver this vision?" — the question private equity often poses, because their timeline is tight and money is the only lever they control | Increased personnel costs, short-term contractors; also negotiate features out |
| Cheap (fast + cheap) | Often not about cost but limited in-house resources critical elsewhere | Remove a significant number of features, naming exactly which ones |

Budget hygiene: simple spreadsheet, months across the top, expenses as rows broken out
monthly. Review all expenditure **at the end of every month** against invoices; review the
budget **with the CFO at least once a quarter**; keep the alternative scenarios up to date so
you can answer instantly when the funding tap opens.

### ROI by project type

| Project type | How to compute ROI |
|---|---|
| Replace/upgrade legacy | Cost savings: software licenses and support contracts no longer needed, hardware upgrades avoided, and — importantly — reduced management costs. The easiest to estimate |
| Change how the business operates / open new markets | Harder; reach out to peers for reduced costs or increased-revenue opportunity |
| Strategic bets | Sometimes a collective gut feeling — the whole management team behind it — so ROI is not a justifiable metric |
| HomeMax example | Reduction in call center head count, plus reduced cost of buying, maintaining, and hosting physical hardware |

## Worked Example

### The eight-step exercise for building your vision's elevator pitch

Run exactly as the book gives it:

1. **Write up to five items that describe the benefits of your vision for either employees or
   clients.** These are the gains the business or client will be able to use on delivery — for
   example, greater insight for a client to measure success by comparing year-over-year data.
2. **Write up to five internal features of your vision.** Features internal to the company or
   your department, more technology focused — for example, reduce the overhead while increasing
   capacity of data management by moving to a cloud-managed database.
3. **Step away from the list and leave it for at least one night.** "Everything requires time to
   cook, so let your brain process these items while you sleep."
4. **You have 10 items — remove all repeated or implied items to get down to a list of six.**
   Having done step 3 this is easy: find the items saying the same thing in different words, or
   implied ones (moving to the cloud implies reducing hardware reliance).
5. **Step away from the last six for at least an hour.** A final check that you have the right six.
6. **Rank the six in order of importance** from a business perspective.
7. **For each of the six, create a technical sentence and a nontechnical sentence.** Start with
   the technical one — it feels more natural — then write a business-oriented version with no
   buzzwords or jargon.
8. **Find the opener, the grabber.** The opening sentence that engages the listener and buys you
   the next 20 seconds. Either a problem statement ("We need to onboard a client in hours instead
   of the months it currently takes") or a parallel to something they know ("Imagine becoming the
   Uber of home warranty by empowering our clients to self-serve for HomeMax").

Then practice out loud with trusted people, technical and nontechnical. The **opener stays the
same**; swap the technical and business versions of each item by audience. Success is measured
by seeing the listener get as excited as you are and engage you with questions seeking more detail.

### The resulting HomeMax pitch (as printed)

> By letting our clients manage their own data on their mobile device or browser, we are going
> to reduce the overall burden on the call center while increasing the data quality and promoting
> engagement. Moving to a scalable, modern architecture, we are opening the opportunity for
> greater integration with third parties to further offer products that would complement our
> offering, for example, offering a yearly home service to address the smaller items before they
> become big problems.

What the pitch encodes: reduced overall running costs; the biggest area of concern (data
quality); and by teasing the cloud, positioning for new markets and products.

### HomeMax client answers to the discovery questions (homeowner's perspective)

- I can only interact through the call center helpline.
- If I have to change or update my information, I have to read it out over the phone.
- I have no idea whether the agent has received my data until I get a confirmation at renewal time.
- I need to interact only when something has gone wrong and I need to start a claim. I also need
  to call regarding the status of a claim.
- I want the call agent to answer in a timely fashion.
- I want to be able to make changes myself, upload photos to support my claim, and look up the
  status without having to make a single call.

**The CTO's read:** the customer expects to drive the experience at their own pace and time. So
the company is *not* yet at the stage of needing new products or features — it needs to remove
the friction of interacting via the phone.

### Solve the problem, not the symptom

A client insisted on receiving a daily file **by email** — a file full of sensitive data crossing
an inherently insecure transport. The new CTO didn't feel he had the authority to refuse. Given
the talking points to ask *why* they needed it by email, the answer emerged: email was simply a
convenient way for the user to save the information to a network share on their side. Since the
user was already in the portal each morning, the CTO **added a file download feature**. Data was
then logged and transferred over a secure channel, and no one lost face.

## Key Takeaways

- Separate the what from the how so you can speak to different audiences without diluting either.
- A good vision has easily measurable success metrics — firm them up so there is no dispute when
  you reach them.
- Know your immediate client and build a relationship with your counterpart; it validates the
  vision and short-circuits the Telephone game.
- Weave the vision into the company's calendar — seasonal no-go periods can remove three or four
  months of engineering time per year, and context switching after a two-month gap costs more
  than people expect.
- Design for change: identify pillars, watch them commoditize, spin up splinter teams, and accept
  taking steps back to leap forward — but treat framework and language changes as a different,
  more disruptive class of decision.
- Budget with the CFO, keep three scenarios alive, and update monthly.
- Control the narrative: in the absence of information people fill in the blanks. Reset
  expectations in every periodic update — what you are delivering *and what you are not*.

## Connects To

- [Chapter 2 — Managing up](ch02-managing-up.md) — the valley of anguish, the CFO relationship,
  and the change-management moves this chapter's planning depends on.
- [Chapter 4 — Building a team](ch04-building-a-team.md) — the resources the budget pays for.
- [Chapter 8 — Technology decisions](ch08-technology-decisions.md) — how pillar and stack
  decisions get made, including cloud vs. on-prem and build vs. buy.
- [Chapter 9 — Development](ch09-development.md) — project planning, sprints, and the delivery
  machinery under the vision.
- [Chapter 13 — Housekeeping](ch13-housekeeping.md) — budget oversight in steady state.
- [Chapter 14 — Company growth](ch14-company-growth.md) — the impromptu pitch, a close cousin of
  the elevator pitch.
