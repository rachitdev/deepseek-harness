# Chapter 4 — Exercise: Design an Ecosystem in 60 Minutes

*Pages 82–98 ("Exercise: Start Thinking in Ecosystems. Now!")*

## Core Idea

"The basic principle of design thinking for business growth can be understood best through experience." This is a timeboxed 60-minute dry run that walks a team from an already-validated MVP to a designed, business-modelled minimum viable ecosystem. It exists to make the **transition from an MVP to an MVE** concrete, and the book explicitly recommends it as training material for design thinking, strategic design, and ecosystem design courses.

The setup deliberately starts *after* the first two design lenses: a team of experts has already solved a customer problem, the solution was selected from various options, and problem/market fit was tested by an MVP. The exercise's task is only: **design a suitable ecosystem that can be tested in the form of an MVE in the market.**

The design challenge is drawn from health care — the e-prescription — but the book notes it could equally have come from media, financial services, agriculture, or pharmaceuticals.

## Frameworks Introduced

**The six-step timeboxed sequence** (plus a reflection step). Total ≈ 60 minutes:

| Step | Activity | Time |
|---|---|---|
| 0 | Read the design challenge, persona, and critical items diagram / MVP | 6 min |
| 1 | Brainstorming: who are the actors currently involved in a conventional prescription? | 5 min |
| 2 | Visualize: draw the current system = ACTUAL state, with flows and problem areas highlighted | 8 min |
| 3.1 | (Re)design: 1–2 variants of a superior business ecosystem with value flows (data, money, information) | 6 min |
| 3.2 | Benefits per actor from participating (preferred variant) and their motivation | 6 min |
| 4.1 | Core business model of the overall system: how are setup costs and operations financed? | (grouped) |
| 4.2 | Actors' business models and other incentives for indispensable actors | 8 min |
| 5 | MVE requirements: how actors interact in a minimum version, which technologies enable it | 6 min |
| 6 | Storytelling: why should actors become part of the ecosystem? | (grouped) |
| 7 | Reflection: typical lessons learned from the dry run | 5 min |

**Reflection prompt set** used in step 7 — twelve prompts arranged as pairs: target image / funding; stakeholders and management / scaling; value streams / critical experiences and functions for customers; initiation / capabilities of the actors in the system; operatives / new technologies; mindset / collaboration.

## Key Concepts

- **ACTUAL before TARGET.** Step 2 requires drawing the existing system before redesigning it. The book calls this an "iterative procedure from the ACTUAL to the TARGET ecosystem" and treats it as the reusable move (see also the redesign guidance in Ch 13, where creating today's ecosystem first and optimizing it in iteration is recommended especially for radically restructuring existing ecosystems).
- **The whole ecosystem journey, including background interactions.** Step 1 instructs the team to consider actors who touch the prescription for handing in, clearing, and billing — "obvious interactions *and* interactions running in the background."
- **Advantage per actor is a gate, not a nicety.** Steps 3.2 and 4.2 exist because "ecosystems only work if all actors in the system benefit from it."
- **Storytelling as an actor-acquisition tool.** Step 6 asks what individual benefits should be highlighted per actor and what customer/user needs are satisfied.
- **Timeboxing.** Every step carries a time indication; the exercise is designed to prove that a simple business ecosystem can be developed in 60 minutes.

## Mental Models

- **The MVE follows the logic of the MVP**: the system is tested with a small budget and a very limited number of actors (e.g. **one per role**), then optimized until the most promising system can be validated.
- **Design thinking tools carry forward into ecosystem design.** The exercise explicitly uses the critical items diagram to define the critical functions and experiences that the ecosystem must support.
- **Solution ideas are not model solutions.** The book states outright: "They aren't by any means model solutions; there are countless ways to solve this ecosystem challenge."

## Anti-patterns

The current-state analysis in step 2 surfaces the anti-patterns the redesign must remove:
- Susceptibility to fraud and high transaction costs.
- Long processing times due to intermediaries.
- Data inconsistency and discrepancy, isolated technology platforms.
- No or inefficient information for patients.

## Reference Tables

**The design challenge and persona (given inputs).**

> *"How can a better digital health care experience be designed for patients that simultaneously allows for cost reductions in the health care system?"*

Persona — current situation of a typical patient (user profile):

| Field | Content |
|---|---|
| **Jobs to be done** | When at the doctor's office, patients want to know whether prescribed drugs are incompatible with other active ingredients so they can take the medication without worries. When leaving the doctor's office, they want to get the drug from the closest pharmacy so pain is quickly alleviated and healing begins as soon as possible. If they have an annual prescription, they want to receive medicine during vacation or when on the move. |
| **Pains** | Prescription on paper; various doctors and treatments; get medicines on the move or on vacation; order and availability of drugs |
| **Gains** | Owns smartphone; interacts with her health insurer via digital channels |

Tested MVP value proposition (from the critical items diagram and vision prototype):

> "The electronic prescription allows patients to hand in their prescription for the medicine they need at **any** pharmacy and receive a reminder for the correct dosage. In addition, the information helps to avoid incompatibilities with other medicines."

MVP elements: the patient's view of the app with expanded functionality; patient; drug vending machine.

## Worked Example

**The e-prescription ecosystem, filled in** — the book's own solution ideas at every step.

**Step 1 — Actors in the conventional prescription system:** doctors; patients; pharmacies; online pharmacies; pharmaceutical companies; intermediaries for billing / failure of payments / fraud; insurance companies; pharmacies; accident insurance companies; health insurance companies; health insurer.

**Step 2 — ACTUAL state problems:** susceptible to fraud and high transaction costs; long processing times due to intermediaries; data inconsistency and discrepancy with isolated technology platforms; no or inefficient information for patients.

**Step 3.1 — Redesign prompts** applied to the variants: Which actors are missing? Which ones might be eliminated by new technologies? Is the system sustainable? Do all actors have advantages? Which actors are mandatory? What might be their concerns? The solution idea introduces **data ownership via blockchain**.

**Step 3.2 — Advantages per actor:**

| Actor | Advantages |
|---|---|
| **Patients** | Always have up-to-date prescriptions with them; quick preorder from pharmacy; quick pharmacy finder; display which drugs are in stock and where they are available; easier to use |
| **Doctor** | Paperless prescription; forgery-proof prescription; transparent overview of drugs with a wide range of analysis options |
| **Pharmacies** | No expenditure with the billing; no fee accounting; improved liquidity; loyalty programs and pharmacy finder integrated in the patient app |
| **Health insurers** | Prevention of multiple billing → savings |

Fraud prevention, layered by where it bites:
- *At the doctor's office*: with two-factor authentication, creation of prescriptions by third parties (e.g. the doctor's assistant) can be prevented; prescriptions created without authorization can be identified and analyzed by the doctor (date, patient, medicine).
- *At doctor's and pharmacy*: fraud mainly exists with expensive drugs — especially chemotherapeutics, painkillers, and other addictive drugs; multiple billing of prescriptions becomes impossible.
- *At doctor's, pharmacy and pharmaceutical industry*: secret agreements favoring some drugs in prescriptions can be detected and analyzed.

**Step 4.1 — Core business model of the overall system:**

| Potential revenue | Potential expenses |
|---|---|
| Advertising of pharmacies | Incentives for doctors to participate in the system |
| Provision, connection to health insurance companies | Operation of blockchain and patient app |
| Financial participation of health and accident insurers | Integration in the software used at the doctor's office |
| Integration of pharmaceutical companies (legally problematic) | Marketing costs for raising awareness of actors |
| Transaction fee of pharmacies | Marketing to end consumer (patient) |
| | Further development of new added value for patients |

**Step 4.2 — Incentives for actors who must be convinced:**

| Actor | Incentives |
|---|---|
| **Doctors** | Free web-based software for the doctor's office for easy and fast entry of prescriptions; suggestions for treatments and information on intolerances; real-time view of cost assumption for different drugs |
| **Health and accident insurers** | Better treatment of patients / quicker recovery owing to optimized prescription; cost savings with checking of cost acceptance and with billing; potential for further services for patients |
| **Hospitals** | Updated view of prescriptions and dosing (if the patient makes it visible) |

**Step 5 — MVE requirements and enabling technologies:**
1. Doctor writes a prescription / the relevant formulation on the **blockchain** with the patient's ID.
2. Patient receives a **QR code** on their mobile application and goes to the pharmacy to pick up the medicine.
3. Pharmacy hands the drugs to the patient after verifying the patient ID and QR scan.
4. Health insurer and accident insurer: the invoice for the prescription is directly transmitted and paid; or information on the estimated treatment cost is stored directly on the blockchain.
5. Consequence: **centralized billing and clearing offices can gradually be replaced by blockchain.**

**Step 6 — Storytelling, per actor:**
- *Patients*: "The prescription is always ready for collection at the pharmacy around the corner." "Indication of intolerances at the touch of a button."
- *Pharmacies*: "No cash-flow problems due to real-time billing."
- *Health insurance companies*: "Reduced costs, lower mortality rate, and improved customer loyalty."

**Step 7 — Typical lessons learned from the dry run:**
- A clear target image of the ecosystem helps in communication with potential actors.
- Building and orchestrating an ecosystem requires financial resources.
- Ecosystems only work if all actors in the system benefit from it.
- Well-designed ecosystems can be scaled faster.
- Outlining value streams helps to understand the overall system.
- Focusing on individual key experiences and functions based on customer needs boosts the probability of success.
- Buy-in of all actors is important to build the ecosystem.
- New ideas and approaches emerge as soon as you know the capabilities of the actors and can start a dialog.
- Try, fail fast, and learn — from the first prototype to the MVP all the way to the MVE — is the work motto.
- New technologies (e.g. the cloud, AI, and blockchain) help make systems efficient.
- Business ecosystem design needs a new mindset that consciously gets away from traditional ways of thinking.
- Collaboration between business and technology is essential.

**Context note the book supplies:** although there was no fixed date for a nationwide launch of digital prescriptions at time of writing, the e-prescription is "definitely in the offing" — pharmacies need new hardware and software, the market of patient apps is already booming, and software firms and data centers need to change their processes and products (source cited: DAZ Online, deutsche-apotheker-zeitung.de).

## Key Takeaways

- The exercise is the book's cleanest demonstration of MVP → MVE: an existing validated MVP is the *input*, not the output.
- Six steps in 60 minutes: brainstorm actors → draw ACTUAL → redesign variants with value flows → advantages per actor → core plus actor business models → MVE requirements → storytelling → reflect.
- Blockchain in this case is not decoration: it is what lets the redesign *eliminate* an actor class (centralized billing and clearing offices).
- The revenue/expense table is deliberately honest, including "legally problematic" options — business model exploration precedes legal filtering.
- Fraud prevention is decomposed by locus (doctor / pharmacy / pharma industry), which is what makes the incentives concrete enough to convince each actor.

## Connects To

- [Chapter 3 — Ecosystems as Growth Lever](ch03-ecosystems-as-growth-lever.md): the roles and value streams being applied here.
- [Chapter 11 — Lens 1 — Design Thinking](ch11-lens-design-thinking.md): persona / user profile and critical items diagram, the tools that produced this exercise's inputs.
- [Chapter 12 — Lens 2 — Lean Start-Up](ch12-lens-lean-start-up.md): the MVP that the exercise takes as given, and the five questions for the transition to the MVE.
- [Chapter 13 — Lens 3 — Ecosystem Design](ch13-lens-ecosystem-design.md): the full four-loop, ten-step procedural model that this exercise compresses.
