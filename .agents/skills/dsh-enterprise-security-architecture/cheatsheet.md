# Cheatsheet — Enterprise Security Architecture (SABSA)

Dense reference. Thresholds and product names are as printed in the 2005 edition — see Scope & Limits in [SKILL.md](SKILL.md).

---

## 1. Layer-by-layer decision tables

### Which layer am I working at?

| The question in front of me | Layer | Deliverable | Chapter |
|---|---|---|---|
| What does the business value and fear? | Contextual | Business model, Business Risk Model, process/organisation/geography/time models | 9 |
| What must be true, in normalised terms, and why? | Conceptual | Business Attributes Profile, control objectives, strategies, entity & trust model, domain model, lifetimes | 10 |
| What services deliver it, under what policy? | Logical | Business information model, security policies, security services, entity schema, domain definitions, processing cycle | 11 |
| What mechanisms and machines implement it? | Physical | Business data model, rules/practices/procedures, mechanisms, user interface, platform & network infrastructure | 12 |
| Which products, standards and protocols? | Component | Data structures, security standards, products & tools, identities & ACLs, nodes & protocols, step timing | 13 |
| Who runs it on a Tuesday, and how do we know it works? | Operational | Continuity assurance, operational risk management, service management, user support, site/network/platform security, operations schedule | 15, 16, 17 |

### Six questions × what each asks

| | Asks |
|---|---|
| **What?** | The assets to be protected at this layer |
| **Why?** | The motivation for applying security, in this layer's terms |
| **How?** | The functions needed to achieve security at this layer |
| **Who?** | The people and organisational aspects at this layer |
| **Where?** | The locations where security is applied |
| **When?** | The time-related aspects relevant to this layer |

### Layer sequencing rules

| Rule | Detail |
|---|---|
| Sequence | Layers develop in order, contextual → component |
| Exception | Operational architecture develops **in parallel** — but cannot start before the conceptual layer exists |
| Natural break | Between conceptual and logical. Get sign-off before committing design resource |
| Milestones | End of Strategy and Concept (buy-in); end of Design (agreement) |
| Pre-existing inputs | Business process model (contextual) and business information model (logical) must come from the wider enterprise architecture programme — SABSA does not create them |
| Held back | Detailed functional requirements specifications — each project defines its own at detailed design |

---

## 2. The Business Attribute profiling procedure (Figure 6-5)

Per business driver, six steps then collection:

1. **Business Driver** — state it in the business's own words.
2. **Select Business Attribute(s)** — from the 85-attribute taxonomy (7 classes: User, Management, Operational, Risk Management, Legal & Regulatory, Technical Strategy, Business Strategy). Add your own where there is a gap; the taxonomy is "almost certainly not complete" and one or two attributes are typically added per assignment.
3. **Define Business Attribute** — customise the generic definition to *this* enterprise and *this* business.
4. **Define Metric Type** — hard or soft.
5. **Define Measurement Approach** — the specific instrument.
6. **Define Performance Target** — the number or condition that counts as success.
7. **Collect & Evaluate Metrics** — into the security management dashboard, on an independent reporting line.

### Metric type selection

| Choose | When | Typical measurement approach |
|---|---|---|
| **Hard** | Objective number available | Response time; error rate per transaction/batch/period; percentage up-time; MTBF / MTTR / max time to repair; refresh rates; unauthorised access attempts per period; search tree depth; efficiency ratio (INPUT VALUE)/(OUTPUT VALUE) |
| **Soft** | Qualitative or subjective only | SA-CMM review (independent audit against a Security Architecture Capability Maturity Model); focus groups; satisfaction surveys; market surveys; **red team review**; independent design/legal/forensic expert review |

**Honesty rule** (the *Protected* attribute footnote): a penetration test is a hard metric only if a penetration is **achieved**. Failure to penetrate does not mean penetration is impossible.

### Two-way traceability

| Direction | Proves |
|---|---|
| Down the model | **Completeness** — every requirement is met by something |
| Up the model | **Justification** — every implemented component traces to a business requirement |

Any "why are we doing this?" is answered by walking upward. This is the *Traceable* attribute made operational.

---

## 3. Risk modelling steps

### The generic six-step commercial risk assessment (Ch 9)

1. **What are your business assets?** Identify and value them.
2. **What possible threats put them at risk?** Identify — do not quantify probability.
3. **If each threat materialised, what would be the business impact?** Identify and quantify by relating back to the asset list.
4. **If the impact is significant, what vulnerabilities could let the threat exploit your assets?** Identify and quantify.
5. **Can you reduce those weaknesses with additional controls?** Identify control strategies and their **total cost of ownership**.
6. **Cost/benefit analysis** — reduction in potential business impact against cost of additional control.

### Business Risk Model — the 13 columns (Table 9-8)

| # | Column | Layer |
|---|---|---|
| 1 | ID | Contextual |
| 2 | Business Driver | Contextual |
| 3 | Business Attributes | Contextual |
| 4 | Business Requirement | Contextual |
| 5 | High-Level Threat | Contextual |
| 6 | Business Impact | Contextual |
| 7 | Impact Value (H/M/L) | Contextual |
| 8 | Potential High-Level Vulnerability | Contextual |
| 9 | **Green Field** Vulnerability Value (H/M/L) | Contextual |
| 10 | Green Field Risk Category (A/B/C/D) | Contextual |
| 11 | High-Level Control Objectives | **Conceptual** |
| 12 | Target Vulnerability Value (H/M/L) | **Conceptual** |
| 13 | Mitigated Risk Category (A/B/C/D) | **Conceptual** |

Columns 10–13 are the key interface between contextual and conceptual architecture. **Split the row** when several threats or several vulnerabilities apply — never compress them into one.

### Rating scales

| Impact | Meaning | | Vulnerability | Meaning |
|---|---|---|---|---|
| **H** | Could do great damage to the business | | **H** | Easily exploited by the threat |
| **M** | Could do significant damage | | **M** | Possible for the threat to exploit |
| **L** | Could do only minimal damage | | **L** | Very difficult to exploit |

### Risk categories (Table 9-9, impact × vulnerability)

| Cat | Colour | Meaning | Required action |
|---|---|---|---|
| **A** | Red | Severe risk | Immediate corrective action to reduce vulnerability or impact or both. Highest priority |
| **B** | Amber/Yellow | Significant risk | Plan and execute corrective actions to reduce vulnerability or impact |
| **C** | Green | Acceptable risk | Acceptable — vulnerability is at its lowest or impact is minor — but **monitor for creep into B** |
| **D** | Blue | Negligible risk | No action needed |

### Threat scenario frame (Table 9-3)

Specific threat · Threat agent · **Capability** (finance, technical equipment, software, facilities, expertise, literature, experience) · **Motivation** (personal gain: financial, revenge, knowledge, power, peer recognition, curiosity, antisocial traits, terrorising, status / group gain: political, criminal, religious, terrorising, competitive advantage) · Opportunity · **Catalysts** · **Inhibitors** · **Amplifiers**.

| Inhibitors | Catalysts | Amplifiers |
|---|---|---|
| Fear of capture | External triggering events | Peer pressure |
| Fear of failure | Changed personal circumstances creating a "need" | Fame |
| Insufficient access | Step change in level of access | Easy access, high opportunity |
| High technical difficulty | New technologies and tools lower difficulty | Low technical difficulty |
| High cost of participation | Step change in cost | Low cost of participation |
| Sensitivity to adverse public opinion | Dramatic shift in public opinion | Belief in sympathetic public opinion |

### Threat domains (from Basel II) and agents (Table 9-1)

| Domain | Losses caused by | Agents |
|---|---|---|
| **People** | Malicious or negligent violation of internal policies; human errors | Current, past and prospective employees |
| **Processes** | *Unintentional*: deficient procedure, absent procedure, failure to follow procedure | Employees, customers, suppliers, service providers, agents, partners, public |
| **Systems** | *Unintentional*: unforeseen breakdown, insufficient resilience | Fair wear-and-tear; inadequate design or poor implementation |
| **External** | Natural and man-made disasters; malicious, negligent or legitimate third-party actions | Natural events, accidents, malicious/negligent/legitimate third parties |

### Control type selection (Figure 15-13)

Risk = **Threat × Vulnerability × Impact**

| Need to reduce | Use | |
|---|---|---|
| Threat | **Deterrent** control | |
| Vulnerability | **Preventive** control | |
| Impact | **Corrective** control | |
| — | **Detective** control | Detects a problem and **triggers the others** |

### Eight risk management strategies

Reduce/mitigate · Transfer (insurance) · Avoid · Delay · Compensate (offset against benefits) · Spread · Accept · **Reject** — "not a real risk at all!"

### Time-Based Security

| Symbol | Meaning (all in time) |
|---|---|
| **P** | Protection Value — time the system resists attack |
| **D** | Detection Value — time to raise the alarm |
| **R** | Reaction Value — time for management to react |
| **E** | Exposure Value — time during which the attack can cause damage |

**P > D + R → secure, the attack fails. P < D + R → exposed, the attack succeeds. E = D + R − P.** Design to reduce D and R.

---

## 4. Selection matrices

### Registration assurance (Table 10-1) — pick to match transaction value

| Level | Strength | Fits |
|---|---|---|
| Self-registration | None — no attempt to verify name ownership | Public services where you only want to count users |
| E-mail registration | Weak — proves possession of an address in a domain | Certain low-assurance applications |
| Web-based credit card | Weak — a card number is easy to fake | Low-assurance, small payments, tolerable fraud |
| Telephone confirmation | Augments the above — **out of band** callback | Eliminates certain impersonations |
| Postal registration | Set by RA policy — copies or originals, how many | In the USA, adds mail-fraud exposure as a deterrent |
| Personal registration | As rigorous as required — passport, birth certificate, proof of address | **High-value transaction and order-execution systems**, paired with smart cards or biometrics |
| Transferred registration | Only as strong as the original registration and the database's maintenance | Reuse of existing client databases |
| Delegated / multi-tiered (LRAs) | **Weakest link across every tier** | Scale across domains |

### Lifetimes and deadlines (conceptual *When*)

| Item | The book's value / rule |
|---|---|
| Registration lifetime | Typically **1–3 years**; some much longer (a lifetime social security number) |
| Certification lifetime | **Never exceed registration life.** All verifiers must check expiry |
| Cryptographic key lifetime | Expiry date **or** usage threshold, whichever comes first |
| Digital signature keys | **Two lifetimes** — signing period and verification period; signing must expire first. Archive keys as long as the data's archive life |
| Policy lifetime | Review perhaps **every 3–4 years**; frequent change confuses and undermines confidence |
| Rule lifetime | Changes **frequently** — technology, new attack modes, exploits |
| Password lifetime | Domain policy, plus a **minimum lifetime, usually 24 hours**, so users cannot cycle back; better still, a history file blocking several generations |
| Token lifetime | Renew smart cards every **2–3 years**; distribute the initial password through **another channel**; **stagger issue across the business year** |
| User session lifetime | **A few hours**, matching working-hours patterns |
| System session lifetime | Generally **24 hours**; for dial-up, bounded by call duration |
| Inactivity time-out | **10–15 minutes** frequently used; either close the session or suspend it holding context |
| Response time-out | Local policy. Ask: who is waiting, what is the business activity, how long will they be patient? Server must detect repeats of already-serviced requests |
| Stored data lifetime | Phases: memory-cached → on-line disk → off-line archive. Over decades the binding constraint is **retaining the technology to read the media** and media degradation |

### Inter-domain relationships (Ch 10)

| Relationship | Definition | Consequence |
|---|---|---|
| **Isolated** | No data objects and no activities in common | Cannot interact |
| **Independent** | No data objects in common; each constrained only by its own policy; authorities not obliged to harmonise | May enter into **agreements** to coordinate sharing |
| **Sub-/super-domain** | A's elements and activities ⊆ B's; A's jurisdiction **delegated from B**; A's policy does not conflict with B's | A may add policy only if B permits |

**Secure interaction between domains** requires the authorities to agree: secure interaction rules → common security services → common security mechanisms → the security information items to be exchanged.

**Always remember the universe**: the super-domain of every domain you define — completely hostile, no policy, no authority, untrusted elements.

### Trust classification

| Term | Meaning |
|---|---|
| **Unconditionally trusted** | Can misbehave **without detection** — the dangerous category |
| **Conditionally trusted** | Can violate policy, but not without detection |

Trust within a domain is **not necessarily mutual and not necessarily transitive**. The customer for trust is **always the relying party**.

### Cryptographic strength (Table 12-2 — 2005 values, method not thresholds)

| Strength | Symmetric | Asymmetric | Hash / MAC |
|---|---|---|---|
| Weak | DES 40-bit; RC4 40-bit | RSA 256-bit | — |
| Medium | DES 56-bit; CAST 64-bit | RSA/DH/DSA 512-bit | ANSI X9.9 MAC 32-bit |
| Strong | Triple DES 112-bit; IDEA 128-bit; RC4 128-bit | RSA/DH/DSA 1024-bit | MD5 128-bit; SHA-1 160-bit |
| Very strong | AES 192/256-bit | RSA 2048-bit; ECC 300-bit | SHA-256/384/512 |

**Algorithm strength is not the only indicator.** Also assess (a) the **key management scheme** and (b) the **cryptographic protocols** against replay, man-in-the-middle, spoofing and dictionary attacks.

### RAID selection as the book presents it

| Level | Description |
|---|---|
| RAID 0 | Striping, **no parity** — "no true redundancy" |
| RAID 1 | Mirroring |
| RAID 2 | Striping + Hamming codes for detection and correction |
| RAID 3 | **Byte-wise** striping, parity on a **separate** disk; any single drive reconstructable |
| RAID 4 | As RAID 3 but **block-wise**; different performance/cost profile |
| RAID 5 | As RAID 4 with **parity distributed** across drives; suits **multi-user, many short random writes** |

### Penetration testing — the twelve levels

Internal network penetration · Static analysis (logs, firewall rule sets) · On-host auditing · Vulnerability assessment (automated scanning) · External network penetration · Application testing (manipulating data inputs, looking for unwanted functionality) · Source code analysis · Wireless and mobile telephony · PABX auditing · War dialling · Remote access server (RAS) testing · Social engineering.

### Testing phases and independence (Ch 16)

| Phase | Tested by | Question |
|---|---|---|
| Unit | The engineer | Does this module work? |
| Integration | Other engineers | Do the modules work together? |
| System / QA | The QA team | Does the system meet the specification? |
| **UAT** | **Users** | Does it do what the business needs? (six-point scale: Cosmetic → Complete Failure) |
| **OAT** | **Operations** | Is it operable? (nine-category verification checklist) |

Every phase needs defined **entry and exit criteria**. You cannot test your own programs. Presence is testable; absence is not.

---

## 5. Tells and smells — symptom → diagnosis → fix

| Symptom | Diagnosis | Fix |
|---|---|---|
| "Have a look at this system; is it secure?" | No definition of *secure* exists | Refuse to answer; establish what secure means to them first (Ch 1) |
| A technical report on "weak widgets and accessible files" with no business framing | Contextual layer skipped | Go back and populate the contextual row (Ch 1, Ch 3) |
| Isolated point solutions that will not integrate; exploding admin workload | Piecemeal/tactical acquisition, no architecture | Architecture blueprint + Architecture Board with budget authority (Ch 2, Ch 7, Ch 8) |
| Risk assessment feels faulty and no one can name critical functions | No business process model, no process culture | The Missing Taxis pattern — get the process model before assessing (Ch 5, Ch 7) |
| People explain away every vulnerability with a control they already have | Greenfield discipline lost | Re-rate as if no controls existed; publish columns 9 and 12 together (Ch 9) |
| Someone is estimating a probability for a threat with zero recorded occurrences | Quantitative threat assessment attempted where data does not support it | Return to qualitative H/M/L; threats are binary (Ch 9, Ch 15) |
| Impact estimates change depending on who is in the room | Impact assessed without routing through the Business Attributes Profile | Normalise via the profile (Ch 15) |
| Metrics with no performance target | Steps 6 of the metrics procedure skipped | Set targets, or stop collecting (Ch 6) |
| A manager receives the metric that exposes their own failings | Reporting line not designed | Treat as internal audit reporting (Ch 6) |
| Servers, firewalls and VPNs appear on the "logical" diagram | Logical and physical layers merged | Redraw; keep physical elements out (Ch 11) |
| A physical network that does not meet requirements | Logical domain model skipped | Build the logical domain model first (Ch 11, Ch 12) |
| Everything in the directory can reach everything else | Uncontrolled inheritance and transitivity | Pre-condition and post-condition rules on roles (Ch 11) |
| Role administration cost at target servers is climbing | Too many roles | Minimise roles while preserving segregation granularity (Ch 11) |
| The vendor says the product "provides authentication" | Service claimed, mechanism unexamined | Inspect the mechanism. The named case: SNMPv1's "authenticator" is the plaintext community name shared by every device — "Not a strong mechanism!" (Ch 11) |
| An encrypted password block works when replayed verbatim | Protocol failure — no nonce | Add a nonce; and note nonces **detect** replay, they do not prevent it (Ch 10, Ch 12) |
| A 16-character passphrase claimed to yield a 128-bit key | Entropy confused with character count | ~1.3 bits per English character → ~99 characters for 128 bits (Ch 12) |
| File-encryption keys stored in another file on the same system | "No advantage has been gained" | External key store, HSM, or split components (Ch 12) |
| Operators share a super-user password and type raw commands | Privilege not encapsulated | Captive scripts for analysed routine tasks; raw password in a safe (Ch 12, Ch 17) |
| Editors, compilers and linkers on a production platform | Development/production not segregated | Remove them; enforce the five segregation rules (Ch 12, Ch 16) |
| A 30-day password change cycle across systems with differing syntax rules | Policy guarantees written-down passwords | Rationalise the estate or the policy (Ch 12) |
| The padlock icon is cited as evidence of a trustworthy counterparty | Channel security mistaken for transaction security | Cross-examine each claimed property; **the pipe is not the transaction** (Ch 13) |
| XML passing cleanly through the firewall | Network-level firewall cannot inspect XML | XML firewall; sign all XML documents (Ch 13) |
| Policy exists but lives only in the filing cabinet | Shelf-ware written for the auditor | Rewrite from "whose behaviour are you trying to influence?" (Ch 14) |
| A 60-cell information handling matrix and classifications drifting upward | Military classification adopted commercially | **Classify systems, not documents** — nine policies instead of sixty regimes (Ch 14) |
| Knowledge work has become impossible; analysts cannot connect information | Least privilege applied as a *knowledge* principle | Need-to-share at the knowledge layer, least privilege at the platform layer (the 9/11 pattern) (Ch 14) |
| The IT director is accountable for information security | Wrong accountability | Not the IT director; and do not call the role "IT Security Manager"; the CISO is not responsible for the security of business information (Ch 14) |
| The steering group has junior delegates and takes no decisions | No budget authority | Money buys attendance — give it budget authority (Ch 14) |
| Incident reports spike right after an awareness campaign | Expected and healthy | Anticipate it; never tie report counts to bonuses or league tables (Ch 14) |
| Nobody reports mistakes | Blame culture | Institute a no-blame culture in risk event reporting (Ch 14, Ch 15) |
| Risk register has grown unmanageable | Entries unscreened | Screen entries; retain closed ones separately for historical analysis (Ch 15) |
| A risk with no named owner | Ownership not assigned | "Assigning ownership of risk is critical" — one owner per risk (Ch 15) |
| A zero-loss year presented as a saving | Cost not risk-adjusted | Risk-adjusted cost is the only real cost; a zero-control zero-loss year is a loan (Ch 15) |
| Encryptors turned off to fix throughput, and nobody noticed | No configuration audit against a target | Audit actual against target configuration on a cycle — entropy is the default (Ch 16) |
| Certification presented as assurance | Completeness and relevance checked, **implementation** not | Ask the third question; and note ISO/IEC 17799 does not certify an ISMS — BS 7799-2:2002 does (Ch 16) |
| Emergency patches accumulating in live systems | No follow-up change request or regression test | Close every emergency change through the normal process (Ch 16, Ch 17) |
| A planned release with no reversal procedure | Change control incomplete | No release without a documented back-out (Ch 16, Ch 17) |
| An employee who never takes annual leave | Continuous control over records — the fraud pattern | Enforced annual leave as a security control (Ch 17) |
| An outsourcing contract signed before security was discussed | Policy control conceded | Security target document in the SLA, agreed definition of "secure", before signature (Ch 14, Ch 17) |
| A backup that has never been restored | An unverified assumption | "A backup you have not restored is a rumour" — verify content, restore, restore time, and future media readability (Ch 17) |

---

## 6. Process templates

### SABSA Lifecycle

**Strategy and Concept** (contextual + conceptual) → *sign-off/buy-in milestone* → **Design** (logical + physical + component + operational) → *agreement milestone* → **Implement** → **Manage and Measure** → feeds back into a new Strategy and Concept phase.

### Chain of derivation

Business drivers → Business Attributes → Business Risk Model + Business Attributes Profile → **Control objectives** → security strategies and architectural layering → logical security services → physical security mechanisms → component products, tools and standards → operational procedures.

### Multi-tiered defensive strategy (Figure 10-2)

**Prevent → Contain → Detect and notify → React → Collect evidence → Assure.** Select services from the Table 11-2 catalogue against each tier; a service may serve several tiers.

### The eight logical steps of incident response (Ch 11)

Data collection → data normalisation and collation → data analysis → incident assessment and conclusions → presentation → response alternatives → response decision → response action management.

Two governing decisions: **(1) automated or manual?** — decidable by error type or count; one instance may be handled automatically, multiple instances may need a human. **(2) Are the error conditions correlated?** — "in most cases of system failure, it becomes progressive and may require rapid action to contain damage."

Preparation: develop worst-case scenarios and establish a **minimum sustainable configuration** you can implement quickly — physical domain segmentation, human intervention, firewalls at critical domain boundaries, alternate communications links.

### Hierarchical policy architecture (Figure 14-1)

| Level | Documents | SABSA layer |
|---|---|---|
| 1 | **Over-arching Business Operational Risk Management Policy** (all employees) | Logical |
| 2 | Information Security \| Physical Security \| Business Continuity (\| Health and Safety, etc.) | Logical |
| 3 | CA and RA Security Policies | Logical |
| 4 | Infrastructure Security Policies \| Line-of-Business Application Security Policies | Logical |
| 5a | Anti-virus, Remote Access, Network Security, Platform Security, Acceptable Use \| per-application policies | Logical |
| 5b | Security Rules, Practices and Procedures | **Physical** |
| 6 | Security Standards (external and internal) | **Component** |
| 7 | Implementation Guidelines (sparsely populated by design) | **Operational** |

Split level 4 "driven by the granularity and variability of business risk across these sub-domains."

### Rules / practices / procedures / guidelines

| Form | Nature |
|---|---|
| **Rule** | Absolute and unequivocal; interprets policy in definite terms a mechanism can enforce; changes often |
| **Practice** | What is habitually done; **portable across platforms** |
| **Procedure** | A prescribed step sequence for a specific platform or context; **not portable** |
| **Guideline** | Advice, no obligation |

### Risk register fields (Table 15-2)

Risk ID · Risk Name · Risk Type (**all** types, not just operational) · Threat Description · Business Impact Description & Rating (H/M/L) · Vulnerabilities & Overall Likelihood (H/M/L) · Risk Category Rating (A/B/C/D) · **Owner** · Date Identified / Last Updated / Date Closed · Actions Planned (URL) · Current Status Report (URL). Plus a transactions table recording all additions, changes and deletions.

| Register role | Privileges |
|---|---|
| **Risk Manager** | Full access; **the only role with Delete** |
| **Risk Owner** | Read all; Create/Write/Update on owned entries |
| **Risk User** | Read all |
| **Restricted Risk User** | Read only entries flagged unrestricted |

### Owner / custodian / user (Figure 14-5)

**Owner decides** (policymaker for all risk management aspects, ultimate authority) · **Custodian enforces** (holds but does not own) · **User consumes**. This is what makes an outsourcing contract legible.

### Operational Prevention Process (Figure 17-1)

The operational row is where the Process, People, Location and Time columns collapse into one job. Every logical security management service from Chapter 11 has an operational procedure running in parallel with it — **a service with no procedure is not deployed**.

### Change control vs change management

| | Nature | Scope |
|---|---|---|
| **Change control** | A **record** — log, evaluate, approve, track (seven points) | Individual changes to production |
| **Change management** | A **project** — people, process, culture, organisation | A major change programme |

Confusing them means either bureaucratising trivia or running an enterprise transformation off a ticket.

### Capability maturity levels (Carnegie Mellon)

**Initial** (ad hoc, occasionally chaotic; success depends on individual effort and heroics) → **Repeatable** → **Defined** → **Managed** → **Optimising**. CobiT's CMM has six levels, 0–5.

**You do not have to reach level 5.** Use the levels to decide where on the scale you want to be, and why. The SSE-CMM (ISO/IEC 21827, 22 process areas) is "a little on the heavy side" at 336 pages — build a simplified internal version.

### Metrics reporting

Security Management Dashboard (Figure 6-1) fed by the Business Attributes Profile, presented through the **Balanced Scorecard** perspectives (financial, customer, internal process, learning and growth). Colour beats numbers in the boardroom; bottom line first, not the derivation order.
