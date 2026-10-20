# Chapter 10 — Contract management

## Core Idea

"Contracts, agreements, and licensing are definitely not the most glamorous side of being a CTO. We
didn't sign up to manage legal documents — we're engineers!" But they come with the territory of a
growing technology team, and the chapter's governing rule is a boundary: **legal documents are not
code.** "They don't have a logical structure or a single computational value. A good CTO knows this
is not their area of expertise and allows their legal counsel to articulate the spirit of what they
are looking to turn into 'legalese.'"

Always seek proper legal advice before signing or producing such a document. "Too many times, a
headstrong engineer has inadvertently signed something they shouldn't have, believing they
understood everything contained within."

Storage convention: the **CFO keeps all paper (hard) copies**, signed and sealed; you keep a
**digital version** in your usual file space (Google Drive, Dropbox, Office 365) for fast retrieval
and reference.

## Frameworks Introduced

- **The service agreement as a book** — beginning, middle, and end.
- **The statement of work (SOW)** — an eleven-point structure.
- **The service-level agreement (SLA)** — reading someone else's, and defining your own from seven
  inputs.
- **Vendor evaluation** — five factors before price.
- **Software license tracking** — six fields, reviewed every six months.
- **The support-contract gap exercise.**

## Key Concepts

- **Service agreement structure.** *Beginning*: commencement date, any prerequisites that must be
  completed, and the costs — and if the engagement produces something, "spell out clearly the
  assignment of ownership of the produced work, usually tied to the payment of each monthly
  invoice." *Middle*: deliverables, responsibilities, reports, availability — "not too detailed or,
  conversely, too vague." *End*: how the agreement concludes and how things are handed over, with
  provisions for either party to terminate early.
- **Termination is not necessarily negative.** Support contracts for legacy systems no longer in use
  get terminated — "that is one of the reasons for upgrading or replacing them: to reduce the
  continued spend." Notice periods typically run **two to eight weeks, with a month common for
  contracting resources**.
- **NDA definition.** "A mini contract that exists between two parties for them to be able to
  exchange information confidentially to advance discussions without fear of loss or misuse of IPR."
  Usually limited in scope with an associated time span, typically signed by your CFO or CEO. Keep a
  lawyer-validated NDA ready in your arsenal. Caveat: "just because a party is under an NDA doesn't
  mean you should discuss everything — the conversation should still be restricted to as little
  information as possible and only to what is relevant for the contract at hand."
- **SOW granularity varies by engagement type.** For rapid on-demand developers it may simply say
  *"Java development services, as dictated by the project manager"* without listing tasks — but with
  a minimum time frame and a **no-direct-recruitment clause without prior approval for a period of
  up to two years after termination**. For support contracts it may limit which software versions are
  covered, how often upgrades happen, and how offline time is managed with the business.
- **The SOW balance.** "Detailed enough for both parties to know exactly what is expected of them but
  still adaptable to a changing and evolving environment. Neither party should feel beholden to the
  other, so beware of any agreements with too long a commit period."
- **What an SLA is for.** "To provide the confidence and faith that you can rely on a given service."
  Counterfactual: if Amazon had no SLA, or an uptime of only 50%, "that means they could be down for
  up to two weeks in every month, with no recourse for the customer."
- **More nines cost more.** "Finding that balance will depend largely on your need and may not be the
  same for all. Some parts of your ecosystem will be able to tolerate more downtime than others."
- **SLAs also cover time to resolution** — how long between a bug or issue being reported and a
  successful resolution. "The more important a service is, the more you will want to demand from the SLA."
- **100% availability is unattainable** — "too many variables are in play."
- **Price is not the winning attribute** when choosing a vendor.
- **License compliance traps.** "Are you using private or student licenses for commercial purposes?
  This situation comes up a number of times in audits — companies believe they are at least taking
  steps in the right direction, so their conscience is clear, but the eyes of the law don't see it
  like that."
- **Why licensing got complicated.** It was simple when software was licensed per server; multicore
  processors and virtualization broke that — a single hardware server could host 10 virtual servers,
  and a public cloud server may run only for a period of time. "The vendors naturally felt they were
  getting shortchanged and adapted accordingly."
- **The zero-cost development trap.** "Although the decision to use Microsoft's SQL Server is zero
  cost at development, as you move into production, you can be hit with a significant license cost —
  the equivalent of a salary in some cases." Most vendors give full development access freely
  because "that is their way to get you hooked or locked in."
- **Cloud license bundling.** Amazon and Azure offer virtualized commercial offerings where the
  license cost is built into the per-minute price — a Windows Server EC2 instance costs a few cents
  more per minute than a Linux one, which "removes all worries of whether you are legal from a
  licensing viewpoint."
- **Open source still has commercial support.** "All major open source software bodies have their own
  ecosystem to provide you a wide range of support."
- **Buy support for the gaps**, not for everything — "it can be very expensive." But hardware is
  different: if you manage significant hardware servers, "this is one area you definitely want to
  make sure you have the support needed to cope with failures. Physical items break — even the ones
  that don't have moving parts." A good support partner replaces items *before* they are due to fail.

## Mental Models

- **"Assumption is the mother of all mistakes"** — attributed in the book to philosopher Eugene Lewis
  Fordsworthe, and applied to service agreements.
- **The service agreement as a book** — beginning, middle, end.
- **You are only as strong as your weakest link** — your own SLA cannot exceed the worst SLA among the
  third-party services you depend on.
- **Evaluating a vendor is not unlike hiring a team member** — define what you need first, then tender,
  interview, and check references.

## Anti-patterns

- **Signing without counsel** because the document seemed comprehensible.
- **Assuming you own what you paid for.** See the Worked Example — the deal that died over a missing
  IPR assignment clause.
- **Letting the vendor learn on your contract.** "Their skills must complement your needs from the
  moment they start the engagement."
- **The single-expert vendor.** "Be cautious if there is only one expert in the company. What happens
  if they go on vacation or, worse, leave?"
- **Plucking an SLA number from thin air.** "It should be based on something real and deliverable and
  be defendable and achievable."
- **Both underspending and overspending on licenses** — a common simultaneous failure at the same
  company, driven by vendor licensing complexity that changes often.
- **Dumpster diving for spare parts.** Clients stuck with legacy kit scouring second-hand sites like
  eBay — "there is quite the subculture in trading old hardware as companies try to eke out as much
  life as possible." If you're there, "be sure to get the replacement on the road map."

## Reference Tables

### Statement of work — the structure

| # | Section |
|---|---|
| 1 | Scope of work: areas that will be worked on, areas that won't be |
| 2 | Definition of deliverables: source code, library, or service |
| 3 | When the assignment of the IPR of the rendered work occurs |
| 4 | Access required to fulfill the contract |
| 5 | Flexibility/restrictions on personnel, or subcontracting the work |
| 6 | Authorization steps for any out-of-scope work, additional licenses, or travel |
| 7 | Warranty on any rendered work |
| 8 | Time frame of contract |
| 9 | Extension/renewal/termination procedures |
| 10 | Limitations on solutions offered |
| 11 | Payment terms |

References the book supplies: NDA example — https://www.lawdepot.com/contracts/non-disclosure-agreement/ ;
SOW examples — https://www.projectmanager.com/blog/statement-work-definition-examples

### Reasons to engage a third-party company

| Reason |
|---|
| Staff augmentation |
| Specific project |
| Support |
| System management |

### Seven inputs to defining your own SLA

| Input |
|---|
| Of all the third-party services you rely on, what has the worst SLA in that list? |
| How much downtime do you need to do releases or system upgrades? |
| Are you able to offer an SLA for a full 24 hours each day or only during core business hours? |
| What is the time for resolution on a given reported issue? |
| Do you have the necessary support staff in place to cover support, especially during vacations? |
| What has been the track record of your offering thus far? |
| What is the weakest part, or single point of failure, in your offering that would affect uptime? |

### Vendor evaluation factors

| Factor | What to check |
|---|---|
| Alignment of skills | Their skills must complement your needs from day one. May involve interviewing the specific resources who will work on your project |
| Depth of knowledge | Cursory knowledge won't do — you engaged them because your team lacks the expertise. "Testing the company with some exercises will give you the confidence they can handle any problem you throw at them" |
| Accessibility | Out-of-hours coverage (common for support contracts); specific times of year needing more attention. Rate how they handle requests during and outside regular hours, response time, communication style, updates, messaging |
| Scalability | Can they grow and adapt as you grow? Beware the single expert — you rely on them sharing knowledge with their team for continuity of service |
| References/affiliations | Ask for references from clients with requirements similar to yours. If they support a given product, does that product have recommended service vendors or partner programs, and are they part of it? |

Plus price and contract tenure. When you get to the fine print, **bring in your CFO or CEO and legal
counsel** — "a second pair of eyes who isn't as close to the deal will be a welcome addition, because
they will challenge each point and translate the legalese of the various clauses to see how truly
workable they are."

### Software license tracker — minimum fields

| Field |
|---|
| Date of acquisition |
| Renewal date/cost |
| Contact details |
| Contact details they have for you, such as email/phone/address |
| Notes on internal usage |
| License keys/codes |

Review cadence: **at least every six months**, plus any time a new purchase is made or a renewal is
done. A shareable spreadsheet is enough if your organization isn't large enough to justify dedicated
license-management software.

## Worked Example

### The AWS EC2 SLA, decoded

AWS states in its SLA for the Elastic Compute Cloud (EC2) service that they will have a **monthly
uptime of 99.99%**, and if they fail, there will be a **credit in the bill of 10%**.

The book does the arithmetic: 99.99% "equates to roughly anything more than **six minutes of
downtime a month**; beyond that, Amazon owes you 10% of your EC2 bill."
(https://aws.amazon.com/compute/sla/)

That single figure is what you inherit as the ceiling on any SLA you offer your own users, since
"you are only as strong as your weakest link."

### The service agreement that killed a deal

A private equity company was doing due diligence on a software company it intended to purchase. **The
target had no developers of its own** — all development had been outsourced to a third-party
development company. Everyone assumed the target owned all the work it had paid for.

**The problem: the signed agreement had no transfer of the IPR of the final product — it wasn't
theirs to sell.**

It was treated as an oversight. Given a great multiyear relationship, squaring away the legal side
looked routine. "However, people do funny things when they see large dollar signs. Assignment wasn't
forthcoming. It got a little ugly with more demands."

**The private equity firm walked away** — "all because someone never read the fine print of the
service agreement."

### The support-contract gap exercise (as printed)

> **Identify all the major components of your enterprise**
>
> For the sake of this exercise, we will consider a major component to be a piece that is critical to
> customer service and continuity. Now let's assume something goes wrong with that component: who do
> you call for help?
>
> If you have someone experienced and knowledgeable on your team, great. What if you don't? Or what
> happens if they can't resolve it (or are not available, say, on vacation)? You have now put the
> business at risk of not being able to service clients.

The conclusion the exercise drives to: **buy support for the gaps, the areas where you are a bit
thin** — not for everything, and not where you have a strong internal team who knows what they are
doing and where you are not relying on a single person.

## Key Takeaways

- The SOW details the service to be rendered and protects both parties by making expectations explicit.
- The SLA defines the level of service that can be expected *and* the consequences of missing it.
- Tender after you have defined what you want; evaluate vendors like candidates, references included.
- A software license defines what you can and cannot do with a piece of software, hardware, or service —
  track every one, and re-check the fine print whenever the enterprise changes significantly (reach
  out to your account representative, "who will be more than happy to take your call").
- Support contracts are the safety net for systems you must maintain; hardware support is the one you
  shouldn't go without.
- Contract checklist worth carrying: IPR assignment stated on payment or completion for every
  contracting resource; an NDA ready to go; understanding of every third-party SLA you depend on; your
  own SLA defined; a documented vendor-evaluation methodology; a handle on all third-party licenses and
  their restrictions; support contracts for major components.

## Connects To

- [Chapter 4 — Building a team](ch04-building-a-team.md) — the outsourcing, onshore, and offshore
  resource types these contracts govern, and the tender process for onshore firms.
- [Chapter 8 — Technology decisions](ch08-technology-decisions.md) — the buy side of build vs. buy, and
  open source licensing as its own license family.
- [Chapter 13 — Housekeeping](ch13-housekeeping.md) — support contracts, hardware management, and budget
  oversight in steady state.
- [Chapter 11 — Documentation](ch11-documentation.md) — license and audit tracking as a documentation type.
- [Chapter 14 — Company growth](ch14-company-growth.md) — where an IPR gap surfaces as a due diligence
  finding.
