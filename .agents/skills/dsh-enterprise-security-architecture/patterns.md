# Patterns — Enterprise Security Architecture (SABSA)

The book's reusable moves, grouped by architecture layer, each as **When / How / Trade-offs**. Chapter references in headings.

---

## Cross-Layer: Framing and Method

### Refuse the security question until "secure" is defined (Ch 1)
**When** — Anyone asks "have a look at this system; do you consider it to be secure?"
**How** — Do not answer. Ask what *secure* means to them: what assets, what events they fear, what business outcome they are protecting. Only then assess. The book's position is that everything downstream of an undefined "secure" is invalid.
**Trade-offs** — Reads as evasion to an impatient sponsor, and costs you the quick win of a technical opinion. It also converts a five-minute favour into a requirements engagement — which is the point, but it must be sold as such.

### State the enablement objective alongside the control objective (Ch 1)
**When** — Writing any control objective, or defending a control to a business audience.
**How** — For every control, write the positive reading. Brakes look like a control (they slow the car) but they *enable* the car to be driven faster, because the driver has confidence. If you cannot state the enablement reading of a control, you have not understood the requirement.
**Trade-offs** — Some genuinely defensive controls have strained enablement readings; forcing the framing can produce marketing rather than architecture. Use it as a comprehension test on yourself first, a communication device second.

### Populate the 36 cells; use the matrix as the document structure (Ch 3)
**When** — Scoping, structuring or reviewing an architecture engagement for completeness.
**How** — Walk each layer × each question. Structure the architecture document with a chapter per row and a section per cell. A gap is then visible rather than implicit. Cells need not be filled in strict sequential order.
**Trade-offs** — The authors themselves footnote that completeness does not guarantee consistency: "there may remain a potential issue regarding consistency and lack of conflict between all the various cells." A fully populated matrix can still contain contradictions.

### Decompose top-down until only simple pieces remain (Ch 5, Ch 10)
**When** — Any complexity that resists direct analysis: a trust relationship, a multi-domain environment, a business process.
**How** — Eat the elephant one piece at a time. For trust: Complex Two-Way Trust → Trust Type X, Y, Z → … until only simple one-way claimant/relying-party relationships remain. For domains: unravel overlaid, overlapping domains into a set of well-defined simple domains. Use the **black box** — accept a defined function with a defined interface now, open it later.
**Trade-offs** — Decomposition can be carried past the point of usefulness, producing a model nobody will maintain. And a black box deferred forever becomes an unexamined assumption.

### Draw the system boundary wider than the application (Ch 5)
**When** — Chartering any security project, especially one funded by a single department.
**How** — Ask what the enclosing system is and where the environment starts. Threats live outside the boundary; vulnerabilities live inside it. The IBFS trading-system case is the canonical failure: the encryption project met its brief perfectly and was still the wrong system, because the boundary was drawn at the application.
**Trade-offs** — A wide boundary makes the project harder to fund, since business units pay for projects tied to initiatives they conceived. This is the tension the blueprint pattern (below) exists to manage.

### Model the architecture as a blueprint, deliver as many funded projects (Ch 7)
**When** — Planning an enterprise-wide security infrastructure.
**How** — Do not attempt it as one project — "you would be wise not even to try to take this route." Produce a blueprint and road map that guides a series of separately business-funded implementation projects, and enforce it through **architecture governance via the Architecture Board**, whose teeth are budget approval.
**Trade-offs** — Fragmented implementation risks becoming piecemeal design; this works only if the architecture is created, documented, referred to *and enforced*. Without governance power over money, the blueprint is advisory and will be ignored.

### Insert review-and-validate steps when starting in the middle (Ch 7)
**When** — You inherit contextual or conceptual work that may not fit the model.
**How** — Add two process steps: review and validate the business requirements / contextual architecture, and review and validate the conceptual architecture. If the review finds inadequacies, propose rework with a business case built on your findings.
**Trade-offs** — This costs schedule and can read as territorial. If you are ordered to proceed anyway, the book's instruction is to document your advice and lodge it where it can be retrieved and authenticated later.

---

## Contextual Layer — the Business View (Ch 4, Ch 9)

### Interview for business drivers, including the sceptic (Ch 4, Ch 8)
**When** — Gathering contextual requirements.
**How** — Interview across the business, including the executive hostile to the programme. Everything a business executive complains about is a security requirement: margin pressure, joint-venture governance, legacy integration, agent attrition from password fatigue, insider-dealing walls, "prove it to a regulator." Route each complaint into a matrix cell. Ask senior business people business questions, and keep the promise not to ask technical ones.
**Trade-offs** — Interview-derived requirements are unevenly reliable and reflect who was in the room. The Business Attributes Profile exists partly to normalise this variance.

### Assess vulnerability greenfield (Ch 9)
**When** — Populating the Business Risk Model.
**How** — Rate vulnerability as if no controls existed (column 9), then rate the target vulnerability with planned controls (column 12). The delta is what the controls actually buy. People will try to explain vulnerabilities away with controls already in place — do not let them.
**Trade-offs** — Greenfield ratings look alarmist out of context and can be quoted against you. Always publish columns 9 and 12 together with the mitigated risk category.

### Assess threat binarily, grade impact and vulnerability (Ch 9)
**When** — Any commercial risk assessment.
**How** — Identify threats without quantifying their probability; threats exist outside your span of control, historical loss data is unreliable, and "observation of past events is not necessarily a good guide to how the future will be." Grade impact and vulnerability H/M/L, since both are inside your control. Map the product to risk categories A (red) / B (amber) / C (green) / D (blue).
**Trade-offs** — Loses the apparent precision that finance-trained executives expect, and cannot feed a capital calculation. The book's defence is that the precision was never real.

### Build threat scenarios, not threat lists (Ch 9)
**When** — A named threat needs a real risk decision, not a checkbox.
**How** — Fill the Table 9-3 frame: specific threat · threat agent · capability (finance, equipment, software, facilities, expertise, literature, experience) · motivation (personal gain / group gain) · opportunity · catalysts · inhibitors · amplifiers.
**Trade-offs** — Substantially more work per threat than a list, so reserve it for threats that will drive expensive control decisions.

### Apply the P > D + R test to any control design (Ch 9)
**When** — Deciding whether a protective control is sufficient, or where to spend next.
**How** — Measure Protection time, Detection time and Reaction time. If P > D + R the attack fails; if P < D + R the exposure is E = D + R − P. Design to reduce D and R, not only to increase P.
**Trade-offs** — All three values are estimates, and P especially is unknowable against an unknown attacker. Its value is as a design heuristic that redirects budget from hardening towards monitoring and response.

---

## Conceptual Layer — the Architect's View (Ch 6, Ch 10)

### Build the Business Attributes Profile as the normalising layer (Ch 6, Ch 9)
**When** — Immediately after contextual requirements gathering, and before any control decisions.
**How** — Select attributes from the 85-attribute taxonomy, rewrite each definition in your enterprise's own terms, assign a metric type (hard/soft), a measurement approach and a performance target. Route every business impact estimate *through* the profile so one operational manager's opinion cannot contaminate the senior view of priorities.
**Trade-offs** — It is an extra abstraction layer that sponsors initially find indirect. The taxonomy is explicitly incomplete — expect to add one or two attributes per engagement.

### Measure the virtue, not the absence of the vice (Ch 6)
**When** — Anyone asks for "security metrics."
**How** — Risk is the negative of a Business Attribute. "Security" is not measurable; *Available*, *Responsive*, *Non-repudiable*, *Access-controlled* are. Report against the attribute's performance target.
**Trade-offs** — Hard where you can, soft where you must — and be honest about which. The book's model of honesty is the *Protected* attribute footnote: a penetration test is a hard metric only if a penetration is *achieved*; failure to penetrate does not mean penetration is impossible.

### Report metrics on an independent line (Ch 6)
**When** — Designing the security management dashboard.
**How** — Think carefully about who receives which metric. If a metric exposes a specific manager's failings, delivering it to that manager gives them the chance to cover up. Treat the reporting line the way internal audit reports are treated.
**Trade-offs** — Bypassing a manager damages the working relationship you need for delivery. The book offers no resolution beyond deliberate design of the reporting line.

### Decompose trust to claimant and relying party (Ch 10)
**When** — Designing any PKI, federation, or inter-organisational trust arrangement.
**How** — Ask "who is the customer for trust?" The answer is always the relying party. Decompose complex two-way trust into simple one-way relationships. A trust broker must contract with and be paid by the relying party, and must carry liability cover — "when the wheels start coming off, the trust broker has to be responsible and liable, otherwise what was the value of the trust service?"
**Trade-offs** — This inverts the classic PKI commercial model (selling certificates to claimants), which the book calls a flawed assumption "that has contaminated thinking ever since." What a broker can actually sell is a trusted *transaction*.

### Set registration assurance to match transaction value (Ch 10)
**When** — Designing enrolment for any service.
**How** — Choose from the eight levels of Table 10-1 — self, e-mail, credit card, telephone confirmation (out of band), postal, personal, transferred, delegated. Level of trust attachable to a transaction depends on the certificate, which depends directly on the strength of registration. High-value transaction and order-execution systems need strong registration *and* strong authentication (smart cards, biometrics).
**Trade-offs** — Strong registration is expensive and adds friction at exactly the point where user acquisition is measured. Delegated registration scales but its strength is the weakest link across every tier.

### Model security domains, and remember the universe (Ch 10)
**When** — Any architecture spanning organisations, networks, sites or applications.
**How** — Define each domain's elements, boundary and interfaces explicitly; identify the domain security authority and check it really does set and implement the policy; consider sub-domains and super-domains; cover all boundaries and interaction points. Classify inter-domain relationships as isolated, independent, or sub-/super-domain. "There is always a surrounding domain called 'the universe' … completely hostile, with no security policy, no domain authority and containing untrusted security elements."
**Trade-offs** — Domain modelling is time-consuming and produces diagrams non-specialists find abstract. Skipping it is worse: jump to physical design and "you will almost certainly design a physical network that does not fully meet the requirements."

### Progress from eggshell to honeycomb to egg-and-honey (Ch 10)
**When** — The current architecture is a single hard perimeter with a soft interior.
**How** — Segment internally into many small domains (honeycomb), then re-establish a perimeter over the segmented interior (egg-and-honey). A role is a honeycomb cell. Remember that a secure pipe with no regulator at its end merges two domains into one.
**Trade-offs** — Segmentation multiplies boundary management, ACL maintenance and troubleshooting cost. And a firewall secures nothing if the perimeter also contains unauthorised modems and other bypasses.

### Set every lifetime and deadline deliberately (Ch 10)
**When** — Populating the conceptual *When* cell.
**How** — Set registration, certification, key, policy, rule, password, token, message, stored data, secrecy, session and time-out lifetimes explicitly. The binding rules: certificate life must **never** exceed registration life; digital signature keys need **two** lifetimes (signing period and verification period, the former expiring first); passwords need a **minimum** lifetime (usually 24 hours) to stop users cycling back.
**Trade-offs** — Long lifetimes reduce operational load and increase cryptanalytic and dormancy exposure; short ones do the reverse. Stagger token issue across the business year to smooth the workload rather than shortening lifetimes to manage it.

---

## Logical Layer — the Designer's View (Ch 11)

### Express requirements as generic security services (Ch 11)
**When** — Translating policy into design.
**How** — Policy says *what*, not *how* — which is exactly why requirements are expressed as generic, reusable security services from the Table 11-2 catalogue, classified by defensive strategy. Assemble them as common building blocks rather than per-application.
**Trade-offs** — Generic services need a service management layer and an owner, and the catalogue is explicitly not complete: "no list of security services will ever be complete, and you should not take this one to be so."

### Build the hierarchical policy architecture from the audience down (Ch 11, Ch 14)
**When** — Writing or restructuring security policy.
**How** — Ask two questions: what is the purpose of this policy, and **whose behaviour are you trying to influence?** If the community is too hard to define, the scope is wrong — go back to the beginning. List the groups, write a policy per group, and confine content to what is relevant to that audience. Put an **over-arching Business Operational Risk Management Policy** at level 1, with information security, physical security and business continuity as level-2 siblings beneath it.
**Trade-offs** — Multiple audience-specific policies mean more documents to maintain and version. The alternative — repeating the same operational-risk statements under three discipline headings — is worse.

### Keep physical elements out of logical diagrams (Ch 11)
**When** — Producing the logical domain architecture.
**How** — No servers, firewalls, routers, VPNs or cables in a logical diagram. Model network domains and middleware domains as two orthogonal views — plan view and elevation view — of the same infrastructure.
**Trade-offs** — Engineers find pure logical diagrams unsatisfying and will ask "but where does it run?" Answer that in the physical layer, not by contaminating the logical one.

### Design roles as logical sub-domains, and minimise their number (Ch 11)
**When** — Designing RBAC.
**How** — A role is a logical sub-domain of an application domain. Aim to minimise the number of roles while keeping enough granularity to segregate job types and privilege profiles. Define mutually exclusive roles where duties must be segregated. Govern real-time association with pre-condition rules (illegal combinations, transitivity limits, delegation controls), post-condition rules, and context-based rules (time of day, day of week, workstation location, access method).
**Trade-offs** — Too many roles drives up administration cost at target servers; too few destroys segregation. Uncontrolled inheritance and transitivity produce a directory "in which everything gains access to everything else."

### Treat the directory as the pivot and protect it accordingly (Ch 11)
**When** — Any infrastructure design.
**How** — "Without directory services almost no other service can remain operable" — protect its integrity and availability almost at all costs. Impose a schema (attribute types with OIDs, object classes with mandatory and optional attributes) to maintain data quality, reduce polyinstantiation, and slow **directory entropy**.
**Trade-offs** — A strict schema constrains applications that want ad hoc attributes, and schema change becomes a governed process. That is the cost of not drifting to chaos.

### Design for detection where prevention is impossible (Ch 11, Ch 12)
**When** — Specifying integrity and session protection.
**How** — Accept that session authentication and MACs do not stop changes; they make changes **immediately detectable**, so action can be taken. Design the follow-on action. Same for nonces: they allow replay to be detected, not prevented, "which implies some recovery action to follow."
**Trade-offs** — Detection-based designs need a monitoring and response capability that costs more to operate than a mechanism. If there is no response, the detection is decorative.

---

## Physical Layer — the Builder's View (Ch 12)

### Map service to mechanism, not requirement to product (Ch 12)
**When** — Selecting technology.
**How** — One logical service, many candidate mechanisms; Table 12-1 is the join. Choose the mechanism that implements the service the requirement asked for, then choose products at the component layer.
**Trade-offs** — Slower than buying the product a vendor recommends, and the intermediate mapping document is one more artefact to keep current. It is the defence against piecemeal acquisition.

### Apply the cryptography test before reaching for cryptography (Ch 12)
**When** — Anyone proposes encryption as the answer.
**How** — Cryptography does exactly four things: confidentiality, integrity, authenticity, non-repudiation. If the requirement is not a variation of one of these, cryptography is the wrong answer. Then check the other two constraints besides algorithm strength: the **key management scheme** and the **cryptographic protocols** (replay, man-in-the-middle, spoofing, dictionary).
**Trade-offs** — "To make a proper cryptographic assessment of a secure system is a job for an expert" — this test screens out the obvious errors but does not substitute for review. Both of the chapter's vulnerability case studies are protocol failures around mechanisms that worked exactly as designed.

### Build the key hierarchy on the exposure equation (Ch 12)
**When** — Designing key management.
**How** — Key exposure = samples × time, and the two trade off. So bulk data keys change often and key-encrypting keys live long. Split key components across people so duress on one employee cannot extract the whole key. Never reuse a key across functions, parties and channels. Never store file-encryption keys in another file on the same system — "then no advantage has been gained."
**Trade-offs** — Hierarchy and splitting add ceremony, staffing and recovery risk. And there must be a physical written backup of any passphrase-derived master key: lose the passphrase, lose the data, period.

### Compute passphrase entropy, not character count (Ch 12)
**When** — Deriving a key from a human-memorable secret.
**How** — English text carries roughly 1.3 bits per character, so a 128-bit key needs roughly a 99-character passphrase. A 16-character ASCII string does not yield a 128-bit key.
**Trade-offs** — Passphrases of that length are not memorable for most users, which is why the pattern usually terminates in a token or an HSM rather than a human.

### Encapsulate privileged operations (Ch 12, Ch 16, Ch 17)
**When** — Routine operations require super-user rights.
**How** — The book's "highly controversial" recommendation: analyse all routine operations tasks fully, write captive scripts or programs that perform exactly those tasks, grant operators access to the tools rather than the raw privilege, and lock the actual super-user password in a safe under dual control. The privileged human should meet the system through a tool that cannot express the wrong action.
**Trade-offs** — Requires up-front analysis of every routine task and a break-glass procedure for the unanticipated. Operators experience it as a loss of autonomy. The alternative is operators sharing a super-user password and issuing raw command-line syntax.

### Apply the six resilience principles (Ch 12)
**When** — Designing for availability at platform, network or site level.
**How** — Apply them to host platforms (redundant components, clustering, RAID) and to network topology (dual local connections, geographic separation, diverse routing) alike, and remember database recovery mechanisms and two-phase commitment for distributed databases.
**Trade-offs** — Resilience multiplies cost and complexity, and complexity is itself a risk — the *Simple* Business Attribute exists because "complexity only adds further risk."

---

## Component Layer — the Tradesman's View (Ch 13)

### Choose the encoding by consumer: binary for embedded, text for applications (Ch 13)
**When** — Selecting a data representation for a security protocol or interface.
**How** — ASN.1 where the router or embedded device needs encoding efficiency; XML where the business application needs flexibility and human legibility. ASN.1 nests inside XML where both are needed.
**Trade-offs** — ASN.1 encoding rules (BER/DER) are compact but opaque and need tooling; XML is verbose, parseable by everything, and tunnels straight through conventional firewalls unless you deploy an XML firewall.

### Prefer internationally standardised interfaces (Ch 13)
**When** — Selecting any component.
**How** — The whole component layer is an inter-operability problem, and the answer is compatible — preferably internationally standardised — interfaces. Write the internal standards the external bodies do not cover.
**Trade-offs** — Standards-compliant components may lag proprietary ones in features, and printed standards inventories "go out of date very quickly." The cost of the alternative is that "systems integration is very difficult."

### Cross-examine every claimed security property of a protocol (Ch 13)
**When** — A vendor, a padlock icon or a design document asserts a protocol "secures" something.
**How** — Ask, property by property: does it authenticate the counterparty as trustworthy? Is confidentiality end-to-end or hop-by-hop? Does it protect transaction integrity? Does it give non-repudiation? Does it leave an audit trail? Can the application tell whether it is even switched on? The SSL padlock fails most of these; IPSec fails them as an application mechanism.
**Trade-offs** — This makes you the person who says no to a widely accepted control. The underlying principle is that **the pipe is not the transaction** — a channel security mechanism cannot substitute for application security.

---

## Operational Layer — the Facilities Manager's View (Ch 15, Ch 16, Ch 17)

### Run the whole risk cycle as a feedback control loop (Ch 5, Ch 15)
**When** — Standing up operational risk management.
**How** — Follow AS/NZS 4360 with Communicate & Consult and Monitor & Review running alongside every step. Maintain a **risk register** covering *all* risk types with a named **owner** per risk — "assigning ownership of risk is critical" — its own access control (Risk Manager / Risk Owner / Risk User / Restricted Risk User) and a transactions table for audit. Retain closed entries for historical analysis.
**Trade-offs** — Registers bloat: screen entries or the system gets "bogged down with too many risk items." And a register with no owner per risk is a list, not a management instrument.

### Pick the control type from what you need to reduce (Ch 15)
**When** — Choosing a mitigation.
**How** — Risk = Threat × Vulnerability × Impact. Deterrent controls reduce **threat**, preventive reduce **vulnerability**, corrective reduce **impact**, detective **trigger** the others. Then select a risk strategy from the eight: reduce, transfer, avoid, delay, compensate, spread, accept, reject.
**Trade-offs** — **Every mitigation creates a risk.** The book's own example is fire escape doors: easily opened for evacuation versus locked against unauthorised access — "there have been many tragic events … where this conflict has led to multiple deaths and injuries."

### Report risk in traffic lights, and risk-adjust the numbers (Ch 15, Ch 17)
**When** — Reporting upward to executives or the board.
**How** — Colour-code A/B/C/D red/amber/green/blue. Where numbers exist, use **risk-adjusted** KPIs, not raw ones: a zero-control, zero-loss year is a loan, not a saving. Lead with the bottom line, not the derivation.
**Trade-offs** — Colour bands hide the variance inside each band, and a risk-adjusted number invites argument about the adjustment. Both beat a logically-ordered argument with the conclusion at the end.

### Audit against a written standard, continuously (Ch 16)
**When** — Establishing the assurance programme.
**How** — Write the policies and standards first — you cannot conduct an audit without a written definition of what the auditors expect to find. Then audit configuration against target configuration on a cycle, because **entropy is the default**: configuration drifts away from standard by natural process, and audit is the restoring force, not a one-off certification event.
**Trade-offs** — Continuous audit is a standing cost with no visible output when it finds nothing. Treating certification as assurance is cheaper and wrong — the ISMS auditor's three questions are completeness, relevance **and implementation**.

### Escalate testing independence phase by phase (Ch 16)
**When** — Planning the test strategy for any development.
**How** — Five phases with defined entry and exit criteria, and rising independence at each: the engineer, then other engineers, then the QA team, then users (UAT, six-point problem scale from Cosmetic to Complete Failure), then operations (OAT, nine-category verification checklist). **You cannot test your own programs.** The users test functionality; the operators test operability — different questions, different sign-offs.
**Trade-offs** — Speed versus assurance is a genuine risk trade, not a quality failure — two operational risks in direct conflict. Exhaustive white box testing beyond unit level is not a realistic option: "once the integration process begins and complexity grows it would be an infinitely difficult task."

### Contract penetration testing rather than commissioning it (Ch 16)
**When** — Buying adversarial testing.
**How** — Select from the twelve levels according to what you actually need tested. Use professionals; rotate firms; and **restrict in the contract what testers may do after a successful penetration**. Recruit a tester mindset unlike the operator's — nobody who had run a physical casino would think to bet a negative stake.
**Trade-offs** — An impenetrable test environment does not guarantee an impenetrable live one, and penetration testing proves presence of a flaw, never its absence. The book also warns against ex-hackers and convicted criminals.

### Treat personnel management as a security control (Ch 17)
**When** — Designing operational security around people.
**How** — Screening, supervision, disciplinary process, leaver process and **enforced annual leave** are security mechanisms that happen to live in HR. Put security responsibilities in the job description, the contract of employment, the induction and the appraisal — if it is not there, it is not anyone's responsibility. Segregate duties, apply dual control, and segregate the guards from the guarded.
**Trade-offs** — These controls run through a function that does not report to security and has its own priorities. **The person who never takes leave is a finding** — continuous control over records is what an ongoing fraud requires — but acting on it requires HR partnership, not a security memo.

### Pre-agree the incident/problem decision criteria (Ch 17)
**When** — Standing up operational support.
**How** — Incident management wants service restored now; problem management wants the system kept down to investigate. They are **designed to conflict**. Security incidents — "not known problems but the result of new threats" — sit exactly on the fault line. Agree decision criteria in advance rather than declaring a winner.
**Trade-offs** — Pre-agreed criteria will occasionally be wrong for a specific incident, and someone must have authority to override them under pressure. The alternative is improvising the governance during the outage.

### Verify the backup, not the backup job (Ch 17)
**When** — Reviewing continuity arrangements.
**How** — **A backup you have not restored is a rumour.** Verify content, verify the restore, verify the restore *time* against the SLA, and verify the drive and media technology will still exist in seven years. Back software up from a **master copy of the latest release**, never from the running system, which would bake alterations into the backups.
**Trade-offs** — Restore testing consumes production-grade capacity and staff time on a schedule that competes with delivery. Over long retention periods the binding constraint is not media but the machines that read it.

### Outsource implementation, never policy (Ch 8, Ch 14, Ch 17)
**When** — Considering managed security services or ICT outsourcing.
**How** — Outsource the operational implementation of policy already made. Never outsource policymaking, risk assessment, requirements specification or authorisation granting. Settle the security management arrangements — a security target document in the SLA and an agreed definition of "secure" — **before** signing. Use the owner / custodian / user model to make the contract legible: owner decides, custodian enforces, user consumes.
**Trade-offs** — Retaining policy while outsourcing operations means retaining the expertise to set and check it, which erodes the headcount saving that motivated the deal.
