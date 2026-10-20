# Patterns — The Practice of Enterprise Architecture (Kotusev)

The book's reusable moves, each as **When / How / Trade-offs**, grouped by the book's three parts. Chapter references point to the full treatment.

---

## Part I — Foundations and Mechanics (Ch 1–7)

### Classify an artifact on three axes before doing anything with it
**When** — any time you inherit, propose or redesign an EA artifact. (Ch 2)
**How** — fix its *contents* (format, detail, scope, domains, temporal state, storage, volume), its *meaning* (**decisions** or **facts**), and its *lifecycle* (**permanent** or **temporary**). Meaning dictates the development process; lifecycle dictates whether it needs a template and an archive rule.
**Trade-offs** — costs ten minutes up front. Skipping it produces the single most destructive error in the book: developing a decisions artifact the way a facts artifact is developed.

### Develop decisions artifacts collaboratively; develop facts artifacts alone
**When** — starting any new artifact or a substantive revision of an existing one. (Ch 2)
**How** — *Decisions*: schedule meetings with the real stakeholders, propose initial versions from collected opinions, run workshops to elaborate, then distribute for formal approval and sign-off. Architects are drivers and facilitators; success is measured by stakeholder involvement. *Facts*: read recent project documentation, interview support teams, search the CMDB, draft, then return to the source to double-check. Architects are sole developers; success is measured by accuracy.
**Trade-offs** — the collaborative process is slow, creative and politicized, and consumes senior people's calendars. The alternative is artifacts that are ignored and never acted upon. Even "harmless" unilateral format changes to a decisions artifact (changing colours) should be discussed first.

### Make artifacts dual
**When** — building any business-focused artifact that must cross the business–IT boundary. (Ch 1, 2)
**How** — *explicit duality*: give the artifact separate sections for each audience (business benefits, capability impact and process change alongside an architectural overview). *Implicit duality*: write one statement each side reads in its own terms — "All lines of business work with the shared list of customers" means cross-selling to executives and a single shared customer database to architects. Combine both.
**Trade-offs** — duality demands editorial discipline and shortens what you can say. Purely business notions give no IT implications; purely IT notions are untraceable to business value. Not every useful artifact needs to be dual — Standards and most Landscapes deliberately are not.

### Use the five discussion points instead of the business strategy
**When** — anyone proposes to "start from the business strategy". (Ch 5)
**How** — pick the discussion point that matches the scope and horizon at hand: operating model (entire organization, >3–5 years) → business capabilities (3–5 years) → specific business needs (2–3 years) → business processes (1–2 years) → business requirements (<1 year). Support each with its artifact type: dual Considerations, Business Capability Models, Roadmaps, Solution Overviews, Solution Designs respectively.
**Trade-offs** — abstract points are useless during implementation, and detailed points are unachievable for five-year organization-wide planning. Getting the level wrong wastes the meeting.

### Pick an operating model and stick with it
**When** — establishing the most fundamental IT requirements, or when strategies keep churning the landscape. (Ch 5)
**How** — decide process standardization and data integration independently: diversification, coordination, replication or unification. Encode the choice in Principles ("Single Customer View" specifies the data-sharing requirement of coordination or unification; "Standardized Business Processes" specifies the process-standardization requirement of replication or unification). Expect different models at different organizational levels in large companies.
**Trade-offs** — an operating model delineates a *family* of compatible strategies and forecloses others: price leadership fits replication and misfits coordination; customer intimacy fits coordination and misfits replication. The payoff is a reusable digitized platform; the alternative is starting every strategy from scratch.

### Run three processes in parallel, not one sequence
**When** — anyone tries to draw the EA practice as a numbered workflow. (Ch 6)
**How** — separate Strategic Planning (Considerations + Visions, continuous, unstructured, integrated with strategic management), Initiative Delivery (Outlines + Designs, linear, one instance per initiative, integrated with project management) and Technology Optimization (Standards + Landscapes, continuous, architects-only, integrated with nothing). Then deliberately build the three bidirectional flows between them.
**Trade-offs** — harder to explain to executives than a five-step ADM diagram, and it resists tidy process documentation. But only Technology Optimization can legitimately run on architects alone.

### Fold technical improvements into business initiatives
**When** — Technology Optimization produces rationalization suggestions. (Ch 6, 7, 12)
**How** — attach the suggestion to a regular business initiative at the Outlines stage (reuse this asset, integrate with that system, decommission the system this replaces, migrate data from this legacy store). Fall back to a standalone architectural initiative funded from the CIO's own budget only when no business initiative can carry it; last resort is "selling" it to a business sponsor.
**Trade-offs** — riding along is free but slower and subject to the host initiative's scope. Standalone architectural initiatives incur "unnecessary overhead expense" — ideally organizations "should avoid executing purely architectural initiatives altogether", though even mature practices periodically need some.

### Read the initiative portfolio as a quality signal
**When** — reviewing the program of work or diagnosing a practice. (Ch 7, 19)
**How** — classify every initiative as fundamental, strategic, local, urgent or architectural by origin and motivation. Push the mix toward fundamental and strategic; treat a portfolio dominated by urgent initiatives as evidence that Strategic Planning is not sifting needs.
**Trade-offs** — real portfolios always contain all five types because environments are uncertain and landscapes imperfect. Urgent and local initiatives can never be eliminated, especially in dynamic industries like retail.

---

## Part II — The CSVLOD Model and the Six Artifact Types (Ch 8–15)

### Aim for a pragmatic set of 10–15 subtypes
**When** — deciding what artifacts your practice should maintain. (Ch 8, 15, 19)
**How** — cover all six general types with *something*, then stop. Use the eight essential subtypes as a benchmark you must be able to argue against, not a mandate. Periodically map what you actually maintain onto the CSVLOD plane to spot gaps, redundancy and imbalance.
**Trade-offs** — "Using a large number of various EA artifacts does not necessarily signify a mature EA practice, while using a small number... does not necessarily indicate an immature one, as long as all the six general types are adequately represented."

### Apply the negation smoke test to Considerations
**When** — drafting or reviewing any Principle, Policy or Direction Statement. (Ch 9)
**How** — negate the statement. If the negation is absurd or meaningless, the Consideration is trivial and universal and should be cut. Meaningful Considerations "always imply certain trade-offs, can be potentially disagreed with and stay meaningful when negated" — and let you clearly distinguish appropriate planning decisions from inappropriate ones.
**Trade-offs** — the test rejects comfortable statements executives enjoy signing ("All IT systems should be effective and efficient", "Reuse Before Buy, Buy Before Build", "Data Is a Corporate Asset"). Note the subtlety: some apparently universal statements *are* meaningful because they signal willingness to pay above a default — "All IT systems should be designed for high availability" can mean 99.99% uptime and the infrastructure budget to match.

### Give every Principle a title, statement, rationale and implications
**When** — formalizing a Consideration. (Ch 9)
**How** — e.g. **Business Continuity** — *Statement*: "Critical business operations should not be interrupted even in case of severe emergency." *Rationale*: tie it to the business strategy. *Implication*: the concrete architectural consequence ("geographically dispersed, readily available reserve copies"). Keep to 5–10 organization-wide Principles, grouped by category, in a simple Word file; add unit-level Principles in large companies.
**Trade-offs** — implications commit money. Writing them honestly surfaces the trade-off between continuity, security, user convenience, timelines and cost — which is exactly the point, and exactly what makes sign-off slow.

### Build a hierarchy of Standards at the right organizational level
**When** — Standards are multiplying, or projects complain about bureaucracy. (Ch 10)
**How** — standardize only core strategic vendors organization-wide in large companies; add a kit of concrete products at medium scale; push unit-specific, technology-specific and implementation-level standards (naming, directory structure, coding conventions) down to units, IT subunits and project teams, where they may stay informal. More abstract Standards (IT Principles, TRMs) sit higher; more detailed (Patterns, Guidelines) sit lower. Review and clean up yearly.
**Trade-offs** — the operating model constrains the shape: diversification permits a wide tree with high autonomy; replication and unification imply narrow hierarchies. Over-standardize and you become the "architecture police"; under-standardize and you lose homogeneity, reuse and compliance.

### Co-author Standards with hands-on practitioners
**When** — introducing any new Guideline, Pattern or Technology Reference Model entry. (Ch 10)
**How** — bring in lead developers, senior infrastructure engineers and other subject-matter experts alongside architects; do not develop from scratch — study available industry standards, assess applicability and adapt. Introduce Standards **reactively and bottom-up**, alongside the project that first needs them.
**Trade-offs** — more people, more negotiation, slower. Standards written by architects alone "are often found too clumsy by project teams and eventually end up in 'ivory towers'."

### Colour-code status onto facts artifacts to make them decisions artifacts
**When** — you need one artifact to carry both the inventory and the strategy. (Ch 10, 12)
**How** — the technology list in a TRM, the asset list in an Inventory and the system list in an Enterprise System Portfolio are facts an individual architect may update. Overlay status — Current/Emerging/Unsupported/Retiring/Prohibited/Retired/Unknown for technologies; Reuse/Invest/Maintain/Decommission/Undecided for assets; Active/Strategic/Legacy/Inactive/Unclear for systems — and that overlay is a collective decision requiring consensus and committee endorsement.
**Trade-offs** — the artifact now has two update paths with different governance. Getting them confused means either paralysing routine inventory updates or letting a single architect quietly declare a system legacy.

### Heatmap a Business Capability Model to decide where money goes
**When** — prioritizing IT investment across the organization. (Ch 11)
**How** — choose an approach proportionate to the effort available: collective judgment of over/under-investment; intuitive strategic marking by executives; formal maturity assessment and gap analysis (relative against competitors and the industry, or absolute using CMM levels), optionally weighted by strategic importance; capability component analysis (people, process, information, systems, facilities); or strategy mapping. Then focus on low-maturity, high-importance capabilities and safely neglect high-maturity, low-importance ones.
**Trade-offs** — relative assessment may need external benchmarking. Component analysis often reveals a capability needs training or process work rather than a system — useful, but it removes IT initiatives from the pipeline. BCMs suit incremental improvement, not complex structural transformation.

### Sequence Visions from abstract to concrete
**When** — building out Strategic Planning. (Ch 11, 15)
**How** — Business Capability Models say *where* → Target States say *what the platform will look like* → Roadmaps say *when* and *in what sequence*. Colour-code Roadmap initiatives by approval status: Planned → Approved → Funded → Active. Add Target States only when the change is genuinely transformational (operating model switch, post-merger consolidation).
**Trade-offs** — Target States require more experienced architects and are found mainly in mature practices; adding them where change is merely evolutionary is wasted effort that a target system portfolio would have covered.

### Never lead the business with a future-state Landscape
**When** — architects propose an IT target state. (Ch 12)
**How** — enforce the order: business executives specify target capabilities in Visions → architects place new planned systems in Enterprise System Portfolios → implementation is approved on Outlines → new applications and databases appear in Landscape Diagrams. Never the reverse.
**Trade-offs** — this constrains architects' ability to act on obvious technical insight. But "all the attempts to create future-state Landscapes ahead of Visions essentially represent the attempts of IT to lead business" — such states cannot be understood, consciously approved or sponsored by executives, and IT executives lack the budget to fund them.

### Choose a Landscape abstraction level you can keep current
**When** — starting or expanding Landscape documentation. (Ch 12)
**How** — capture only architecturally significant details that change infrequently and are not expected to change without architects' involvement. Start patchy where a specific purpose demands it, then fill gaps as required. Get lower-level detail on demand from archived Designs, daily users, operations teams or reverse engineering. Trigger updates from Designs at project completion, discovered inconsistencies and CMDB change notifications.
**Trade-offs** — you lose completeness. But major changes involve architects and are trackable while minor ones happen unnoticed, so over-detailed Landscapes "tend to inevitably become stale" — and stale information undermines trust in the whole knowledge base. Architects who spend substantial time merely describing the environment "lose their credibility."

### Size Outlines to the decision, not to the solution
**When** — writing anything at the initiation step. (Ch 13)
**How** — focus on exactly three things: the anticipated business value and how it will be realized; potential risks, problems and negative consequences; the cost, estimated with reasonable accuracy. Avoid long narratives, sophisticated diagrams, technical detail and strict notations; simplified BPMN for process change is acceptable. Target 15–30 pages for a Solution Overview.
**Trade-offs** — under-specification risks a bad funding decision. Over-specification is worse: voluminous Outlines "complicate the assessment of proposed IT initiatives, distract attention from the most critical questions and eventually even reduce the quality of decision-making."

### Pair every Outline with a business case, every Design with a project plan
**When** — at the two Initiative Delivery gates. (Ch 13, 14)
**How** — the Outline supplies the time and cost estimates the business case discounts into NPV/IRR/ROI; the business case supplies the financial motivation the Outline compares options against. The Design supplies the component counts and person-hour estimates the project plan schedules and resources; the plan's feasibility and availability constraints reshape the Design. Approve each pair together.
**Trade-offs** — two documents, two owners, two review cycles at each gate. Not all benefits quantify — customer satisfaction and security enhancements resist the financial half.

### Score options, including "Do nothing"
**When** — several credible ways exist to meet a business need. (Ch 13)
**How** — score each option on business functionality, technical feasibility, time/effort/cost, security and risks, strategic alignment and bottom-line impact; sum weighted scores. Ask sponsors to weight the business requirements first so options can be scored against them. Some organizations require at least three conceptual options.
**Trade-offs** — a weighted total invites false precision and can be gamed by weight selection; keep the advantages, disadvantages and risks narrative alongside the number. Skip Options Assessments entirely when no reasonable alternatives exist.

### Always specify technologies, external interactions and security in a Design
**When** — sizing any Design, at any level of desired agility. (Ch 14)
**How** — whatever else you cut, keep these three. "All these project-level decisions can have substantial consequences for an entire organization; none of them can be neglected or simply left at the discretion of the project teams." Use Kotusev's architecture/design distinction to decide the rest: architecture is concerned with fitness for purpose and choices dictated by needs; design is concerned with engineering optimization and choices compliant with architectures.
**Trade-offs** — 25–50 pages is the "average" Solution Design; agile deliveries run leaner and waterfall deliveries run heavier. Fine-grained Designs invite analysis paralysis and are read as rigid and bureaucratic by project teams.

### Update Designs to "as-implemented" and archive them
**When** — at project release. (Ch 14, 12)
**How** — revise the Design to reflect what was actually built, add lower-level details, store it in a searchable repository. It becomes passive reference material for user training, operations, support and troubleshooting — complementing Landscapes, which say what systems exist and how they connect while Designs say how each works internally. Also update Landscapes, and retrieve the archived Outline and business case a few years later for a post-implementation benefit review.
**Trade-offs** — costs effort at the least motivated moment in a project. Benefit reviews are what discourage exaggerated business proposals, enforce business ownership of claimed benefits, and eliminate systematic estimation bias.

### Position an artifact on the continuous plane to predict its behaviour
**When** — an artifact does not fit a single CSVLOD cell, or you want to know why two artifacts of the same type behave differently. (Ch 15)
**How** — place it on the generic↔specific and business↔IT axes. Business Capability Models sit near the generic/business corner and Roadmaps near the specific/IT corner *within* Visions — which is why one says where and the other says when. Enterprise System Portfolios sit close to Visions and can occasionally be shown to executives; Landscape Diagrams sit far away and cannot.
**Trade-offs** — descriptive, not prescriptive; it explains and predicts but prescribes nothing. Genuine hybrids always resolve to two **adjacent** general types.

### Customize the decision path
**When** — designing how strategy will actually reach implementation. (Ch 15)
**How** — start from the typical path (Principles → Business Capability Models → Roadmaps → Solution Overviews → Solution Designs), then substitute or add by trigger: add Policies in regulated industries; use Value Chains instead of BCMs for warmer business acceptance; insert Target States for radical transformation; insert Options Assessments for earlier sponsor engagement; insert Preliminary Solution Designs to de-risk estimates; drop Principles when imperatives are clearly shared; drop Solution Overviews when Options Assessments already suffice.
**Trade-offs** — every addition lengthens the path from need to system. "There is no single right way to organize an EA practice, but rather a multitude of possible options that should be applied selectively."

---

## Part III — People, Instruments and Lifecycle (Ch 16–19)

### Staff by scope × domain expertise
**When** — designing or growing an architecture function. (Ch 16, 17)
**How** — solution architects (narrow scope, narrow expertise, 60–80% of the population) → domain architects (broad scope, narrow expertise) and business area architects (narrow scope, broad expertise) → enterprise architects (broad × broad) → architecture managers outside the matrix. Size at **one architect per 20–30 IT employees**, i.e. 3–5% of the IT workforce; one per 20 during aggressive transformation, one per 30 in maintenance mode. Rotate architects between positions and tiers.
**Trade-offs** — domain architects drive global consolidation while business area architects drive local flexibility; the tension is deliberate and the operating model decides the balance. Rotation costs continuity but "removes the boundaries between different tiers of architecture and avoids the 'ivory tower' syndrome".

### Shape the function to the degree of decentralization
**When** — the organization is federated, multinational or post-merger. (Ch 17)
**How** — very centralized (unification, replication) → only domain architects. Centralized (coordination) → domain architects for data, integration, infrastructure, security plus some business area architects. Decentralized (diversification) → business area architects plus domain architects for infrastructure and security only. Very decentralized → only business area architects. Keep enterprise, domain and business area architects co-located at headquarters; only ordinary solution architects can be safely dispersed.
**Trade-offs** — splitting senior architects across sites "may undermine overall architectural consistency and result in considerable discrepancies in technology selection, investment prioritization and solution assessment approaches", and is justifiable "arguably only in the cases of extreme decentralization".

### Decide before the committee meets
**When** — running any architecture governance body. (Ch 17)
**How** — peer-review every decisions artifact with other, often more senior, architects and secure tentative approval from its direct stakeholders first. "Significant EA-related decisions should not be new and surprising for participants of governance committees." The committee's job is authorization, not debate. Record minutes with participants, agenda items, associated artifacts and approval decisions to follow up.
**Trade-offs** — much of the real work moves offline and becomes less visible. But a committee used as a debating forum stalls, and architects act only as advisors in most procedures — they cannot authorize or reject investment decisions on managers' behalf.

### Implement all four governance types, in as many committees as your size warrants
**When** — establishing governance. (Ch 17)
**How** — strategic (Considerations, Visions), technology (most Standards, some Landscapes), investment (Outlines and business cases), project (Designs). Small → one committee doing all four with different invitees per topic. Medium → two, a business committee chaired by the CIO and an IT committee chaired by the architecture manager, with escalation upward. Large → four separate committees. Very large and decentralized → multiple per type, aligned with units and domains, plus global committees as escalation points.
**Trade-offs** — separate committees cost more meeting overhead but "help set clearer agenda for concrete meetings, minimize deviations from the agenda, focus discussions on relevant issues". Regardless of structure, "all the four types should be implemented".

### Resolve by impact × deviation
**When** — a submitted artifact does not fully align. (Ch 17)
**How** — no deviations → endorsement, always. Minor deviations → exemption if impact is critical or moderate, rejection if negligible. Major deviations → escalation if impact is critical, rejection otherwise. Log every exemption and its reason, and register the resulting architecture debt.
**Trade-offs** — strictness is a dial set by the desired level of architectural flexibility. Logged exemptions enable post-implementation analysis and organizational learning; unlogged ones are just erosion.

### Gate architect involvement with a checklist
**When** — projects outnumber the architects available. (Ch 18)
**How** — have project managers score projects at inception on business criticality, duration, new assets, new infrastructure, unproven technologies, personal data, payments, authentication and other special requirements. Set explicit thresholds: e.g. one "Yes" → a solution architect must be invited; more than two "Yes" → presentation to the Design Authority is required. Then let teams self-assess compliance later against principle and standards checklists, contacting architects only where conformance fails.
**Trade-offs** — the thresholds are arbitrary and need tuning; a badly calibrated checklist either floods architects or lets significant projects through unseen. Done well, it "helps avoid wasting architects' precious time on trivial IT projects" and concentrates the function on value-adding work.

### Start every decisions artifact as a straw-man
**When** — before formalizing any planning decision. (Ch 18)
**How** — produce a quick draft one format lighter than the eventual artifact: PowerPoint where the artifact will be Word; photographed whiteboard drawings where the artifact will be PowerPoint. Use it to discuss principal decisions and available alternatives, then formalize.
**Trade-offs** — throwaway work by design. It "helps involve relevant stakeholders at the early stages, collect timely feedback, exclude unfeasible alternatives as soon as possible and avoid spending time on elaborating inappropriate planning options."

### Template the artifacts you produce repeatedly
**When** — an artifact type will have many instances. (Ch 18)
**How** — template all temporary artifacts (Outlines, Designs) first, then permanent artifacts produced per business area (Roadmaps, Target States). Refine templates from stakeholder feedback so they accumulate the organization's own best practices. Do not bother templating single-instance artifacts like Business Capability Models or Enterprise System Portfolios.
**Trade-offs** — templates standardize look, feel and stakeholder experience at the cost of some flexibility; the fix is continuous optimization rather than rigidity.

### Price the architecture debt into the investment decision
**When** — an initiative wants an exemption. (Ch 18, 17)
**How** — estimate the corrective work and its cost from the Outline, add it to total costs (subtract from the business case), record the debt in the register with actions, estimate, deadline and minor/medium/major band, and set governance thresholds by debt size (e.g. <\$100k at design-committee discretion; moderate escalates to investment or technology committee; >\$1m to the strategy committee). Feed the register into Technology Optimization as a source of rationalization suggestions.
**Trade-offs** — the monetary value "often cannot be easily quantified in a straightforward manner and may be assessed based only on 'guesstimation'". Even so, the metaphor is what makes long-term consequences legible to non-IT-savvy leaders: "all debts must be paid".

### Introduce tools last, and only against a stated need
**When** — someone proposes buying an EA tool. (Ch 18, 19)
**How** — start with Visio for Landscape Diagrams and Excel for Inventories; migrate to a repository only when maintaining consistency becomes too burdensome and a clear business case exists. If a populated CMDB integrated with ITSM already exists, use it as the primary source of current-state information instead. Remember EA tool capabilities correlate with **facts** artifacts, not decisions artifacts.
**Trade-offs** — you forgo impact analysis and automated visualization for a while. But "if an EA practice does not work as expected, software tools are the last candidates to blame", and unfamiliar tools divert architects "from solving real organizational problems to purely technical questions of secondary importance".

### Choose modeling notation by artifact type, and cut it down
**When** — deciding how to draw something. (Ch 18)
**How** — ArchiMate for Landscape Diagrams and Designs; UML for Solution Designs; simplified BPMN for process change in Outlines and full BPMN in Designs; ARIS where already established. Use intuitive ad hoc notations for Visions, and plain text for Considerations and Standards. Use reduced versions of every language.
**Trade-offs** — informality sacrifices unambiguous interpretation, but "less formal modeling approaches allow some interpretive flexibility and ambiguity which is often conducive to reaching agreements". The opposite failure is fatal: "the obsession with correct modeling can easily guarantee its failure."

### Master one artifact type at a time, through seven checkpoints
**When** — establishing or extending an EA practice. (Ch 19)
**How** — for each type in turn: secure the executive mandate (CIO for IT-focused, chief business executives for business-focused); obtain stakeholder commitment to use it for decisions; appoint architect owners responsible for usability and templates; organize consistent development/update/review/re-approval processes; establish or reuse a governance body with scheduled meetings; introduce enforcement (governance procedures, downstream peer-review, project supervision) for decisions artifacts; and confirm the benefits are felt and acknowledged before moving on. In large companies, pilot in one unit before propagating.
**Trade-offs** — slow: around two years to reasonable maturity in a small organization, several years in a large one. Introducing many types at once "can overwhelm the involved stakeholders, exceed their change capacity and undermine the entire effort."

### Choose an entry point by asking "What is our biggest business problem in relation to IT?"
**When** — starting a practice from scratch. (Ch 19)
**How** — high run-costs, hard-to-change systems, legacy risk → start with **Standards and Landscapes** (Technology Optimization). Business cannot see what IT does, investments lack strategic impact → start with **Considerations and Visions** (Strategic Planning). Projects overrun and fail to deliver benefits → start with **Outlines and Designs** (Initiative Delivery). Alternatively follow the historical path (Outlines/Designs → Standards/Landscapes → Considerations/Visions), which is the least risky.
**Trade-offs** — the deliberate path is faster and buys early wins and credibility, but is riskier and requires architects experienced across artifact types. Starting from the wrong place is actively destructive: an EA effort begun with Standards and Landscapes when executives wanted strategic alignment "may be viewed only as an additional burden, useless paperwork and futile bureaucracy", and sponsorship is likely to be discontinued.

### Convert existing documents rather than replacing them
**When** — introducing a new artifact type into a functioning organization. (Ch 19)
**How** — add architectural sections to the non-architectural proposal papers already used to justify IT projects, turning them into Initiative Proposals or Solution Overviews. Adopt the value chains executives already use for strategic communication and turn them into architectural Value Chains rather than introducing unfamiliar Business Capability Models.
**Trade-offs** — you inherit the existing document's format compromises. In return you "ease their organizational acceptance, flatten the learning curve and facilitate smooth adoption", with minimal process disturbance.

### Assess maturity by outcomes, not by inventory
**When** — reporting on or benchmarking a practice. (Ch 19)
**How** — ask senior business executives five questions: Do you understand what IT is doing? How IT contributes to your business goals? Where your IT budget is spent? How IT is transforming your business? What business value IT brings? Then check the six signs: all six general types mastered (10–15 subtypes), predictable and repeatable processes, periodic optimization of artifacts and templates, direct dialog without engagement-manager intermediaries, a lower ratio of urgent and local initiatives than peers, a higher percentage of strategic investments than peers.
**Trade-offs** — none of these is objectively measurable, and maturity is uneven by process and by business unit — so discuss the maturity of each process, or of each unit, rather than of the practice as a whole. "All the typical EA artifacts, processes and governance bodies might be in place, but the actual engagement between business and IT stakeholders can still remain rather poor."

### Use consultants for knowledge, never for documents
**When** — engaging an EA consultancy. (Ch 19)
**How** — three productive shapes: initiative-based (consultants as solution architects filling technology gaps or absorbing workload peaks; outcome is working systems); strategic (consultants as enterprise, business area or domain architects; outcome is knowledge transfer to internal teams); developmental (consultants as architecture managers establishing or improving the practice; outcome is practice improvement plus knowledge transfer). Keep most of the architecture team as permanent employees.
**Trade-offs** — external architects cost more than internal ones. Three shapes to refuse: project-based documentation-oriented engagements (the BSP pattern — "can only waste money, discredit the very word 'architecture' and even undermine all further attempts"); wholesale outsourcing of the practice (you "start to implement the business strategies of these consultancies, rather than their own"); and hiring consultants to build facts artifacts, which is simply economically inefficient.

### Measure to diagnose, never to target
**When** — a mature practice wants quantitative control. (Ch 18)
**How** — collect investment-quality indicators at investment-committee approval, technical-deviation indicators at design-committee endorsement, and complexity indicators from Landscapes and Standards. Report yearly or quarterly. Use trends and anomalies to ask *why* — why do so many investments add little value, why do so many projects need exemptions — then change the decision process.
**Trade-offs** — "Any attempts to quantify complex phenomena and reduce them to a small set of numbers inevitably simplify the reality behind them and may be deceptive." Never make an investment or technology choice in order to move an indicator, and never "establish measurements for the sake of measurements". These are features of advanced practices and are wasted on emerging ones.
