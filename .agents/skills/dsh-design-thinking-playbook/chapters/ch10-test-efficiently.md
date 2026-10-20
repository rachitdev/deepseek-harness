# Chapter 10 (Book §1.10) — How to Test Efficiently

## Core Idea
"We always receive valuable feedback when we test prototypes with customers in the real
world, namely potential users or in the environment of the users." The chapter's whole
discipline follows from one rule about intent: **"Our primary goal in a test interview is
to learn, not to give reasons for or sell the prototype. This is why we don't explain (too
early) how it works."**

Testing is not validation theatre. "Not infrequently, decisive change proposals appear
during this phase that could enhance the quality of the end result substantially. In
particular, the fresh views of people who were not involved in the development of the
prototype and thus are much freer in their assessment can pay off quite well in the end.
**They can see prototypes through the eyes of a customer or user.**"

And the operating slogan: **"fail fast — fail often."** "Early and frequent failure is
indeed an important element of design thinking and contributes significantly to realizing
market opportunities in the end."

## Frameworks Introduced

**The four-step test sequence** ("A test can be broken down into footsteps")
1. **Test preparation** — Where? How? What? Who?
2. **Conducting the test** with potential users in the real context.
3. **Document results** — the feedback-capture grid.
4. **Infer learnings.**

**Test preparation — three defining questions.** "The best way to start is to define
clear-cut learning goals or hypotheses that we want to test:"
- What do we want to learn?
- What do we want to test?
- With whom do we want to conduct the test, and where?

"In the end, the test should show what parts of an idea we should **keep**, what we should
**change**, and what we should **discard**." In early phases the goal might also be to
understand the problem. **"Before embarking on the actual test series with various users,
an initial test with one person should be carried out to exclude any errors"** — with
enough time left to implement improvements before the rest.

**Question maps.** "We formulate simple, clear, and open questions that we can explore in
greater depth at the end. **They should not be hypothetical but tie in to the real
situation of the test person.** We do not ask many questions, but rather focus on the core
about which we want to gain insights. Courage and focus are important." Moderator
follow-up: *"Tell us what you think while you do that."*

**The three test roles** ("Never go hunting alone")
- **The moderator** — "helps the user to cross over from reality to the prototype
  situation and explain the context, so that the user has a better understanding of the
  scenario," and poses the questions.
- **The actor** — "must take on certain roles in the scenario in order to create the right
  prototype experience — usually a service experience."
- **The observer** — "to watch in a focused way everything the user does in the situation.
  **If we have only one observer on the team, it is best to film everything** so we can
  look at the interaction together later in more detail."

**The feedback-capture grid** — four quadrants (see Reference Tables), with an alternative
four-quadrant set.

**The experiment grid** — a six-step structure for defining and documenting experiments
(see Worked Example).

**A/B testing** — the quantitative complement (see Reference Tables).

## Key Concepts
- **Test several variants, not one.** "We achieve the best results when we test multiple
  ideas or variants of one idea that we have described as a scenario beforehand. This way,
  the feedback will be far more differentiated. If we only have one solution ready, the
  user's response… might be rather vague." With several arrangements, "he can make
  comparisons, evaluate, and formulate his feedback far more precisely."
- **Test in context.** "It has become second nature for us to test the prototype in a
  context, namely in its natural environment."
- **Observe misuse.** "We actively observe how the users use (**and misuse!**) what we
  have given them. We do not immediately correct what our test person is doing."
- **Probing questions are the most valuable part.** "Can you say more about how it feels to
  you?" · "Why?" · "Show us why this would (not) work for you." **"Ideally, we answer
  questions with questions: 'What do you think this button is for?'"**
- **Not a survey.** "Resist the temptation to conduct a marketing or voice-of-the-customer
  survey!"
- **Not a pricing study, yet.** "Particularly in phases… in which there are still many
  iterations, the issue is not, for instance, to find out how much the customer would be
  willing to pay… Instead, we try to find out **whether our idea matches the context and
  life of our user and, if so, how does it fit.**"
- **Receiving feedback.** "When we receive feedback, we should see it as a **gift** and
  express our gratitude. We listen to the feedback and do not have to answer in any way…
  we should avoid justifying ourselves and simply listen well. At the end, we ask again if
  [we] haven't understood something."
- **Quantitative validates qualitative.** "Whenever possible, we collect and analyze
  quantitative data to validate the qualitative results."
- **"Customer trial," not "friendly user test."** The German-language term "friendly user
  test" "doesn't quite hit home. After all, the specific purpose of the test is to identify
  weaknesses in the design and get suggestions for improvement — **which are not
  necessarily 'friendly.'**"
- **Non-experts still matter.** Dedicated selection of technically accomplished community
  participants "can boost the quality of the feedback, but you should always consider the
  feedback of nonexperts as well; because they are **less professionally blinkered**, they
  often have a fresh viewpoint."
- **Halve it.** "We think about how the test (or the experiment) could be conducted in
  **half the time and with half the resources**."

## Mental Models
- **Ask "why," even when you think you know.** "In tests with users, it is important to ask
  'why' in order to learn the real motivation, even if we think we know the answer."
- **Explaining the prototype destroys the data.** Every early explanation replaces the
  user's interpretation with yours.
- **Early phase = several variables at once; late phase = one variable.** "In the early
  phases… often, several variables are tested at the same time. For the testing in later
  project stages, other types of experiments with customers (e.g. online tests, A/B
  testing) can be conducted. Here we usually focus on a **single test variable or
  assumption.**"
- **Documentation records why each decision was made.** "Documentation helps when tracing decisions
  later or showing an investor the success of an MVP."
- **Testing feeds the persona, not just the prototype.** "The insights serve to improve our
  prototypes **and adapt the persona**."

## Anti-patterns
- Suggestive questions. "Ask simple and open questions in testing; **never suggestive
  questions**."
- "Avoid the pitfall of relating too quickly or too much about how the prototype works."
- Interrupting or steering. "Let the test persons think aloud and don't interrupt them.
  Don't try to influence them by steering them in one direction or selling the prototype as
  a great solution."
- Over-long tests. "Don't design the test to be too long. Concentrate on what is
  essential."
- Testing with only the builders present — "Involve neutral people… namely, people who did
  not build the prototype."
- Skipping the pilot test with one person, so avoidable errors contaminate the whole
  series.
- Changing more than one variable in an A/B test. "Only one variable at a time should be
  changed to find out why one variant is better liked."
- Confusing regular visitors in a live A/B test — "make sure that regular visitors don't
  get confused by making version B available only to **new visitors**."
- Having digital documentation tools available and forgetting to use them.

## Reference Tables

**The feedback-capture grid**

| Quadrant | What goes in it |
|---|---|
| **Like** | "Things that somebody likes or that are worth mentioning" |
| **Wishes** | "Constructive criticism" |
| **Questions** | "cropping up during the experience" |
| **Ideas** | "generated during the experience or presentation" |

Guiding questions: *What do we like? What wishes do we have? What questions have cropped
up? Which initial ideas and solutions have we found?* "Filling in the four quadrants is
pretty easy: We write each piece of user feedback in the suitable quadrant category."

**Alternative quadrant set:** "I like…", "I wish…", "What if…", "What is the benefit?"
"This method can be easily applied to groups consisting of **two to over 100 people**. The
simple structure helps to formulate constructive feedback."

**A/B testing — the expert tip**

| Aspect | The book's guidance |
|---|---|
| Best suited to | "Simple prototypes" — two versions of a landing page, or of a single element such as a value proposition or test button |
| What can be examined on a Web site | Titles and descriptions of the offers, text volume, style, promotion offers, length of forms, and boxes |
| Timing | "Both versions [tested] concurrently or in tandem and within a predefined, appropriate time period" |
| Decision basis | Final measurement and evaluation "on the basis of clearly predefined criteria" |
| Early-stage variant | Test person experiences variant A, then what they like / would change; repeat with B. Or observe and question one test group about A and another about B |
| Live variant | Distribute page views to A and B with an A/B testing tool; check the conversion rate directly. "Calculators are available to check the statistical relevance" |
| Variables | Change only one at a time |
| Possible outcomes | A wins, B wins, **or no statistically relevant preference at all** — and "perhaps possibilities can be inferred… as to how to combine the best of the two variants" |

**Selection criteria for a Web-based feedback tool (A–D)**

| | Criterion | Note |
|---|---|---|
| **A** | Does the tool offer the possibility for **uploading various types of prototypes**? | Single · variants · sequence |
| **B** | Is there a possibility for **drawing up a scenario**? | "This will give responding users the opportunity to see and understand the situation" |
| **C** | Does the tool enable **predefined and open questions**? | "It pays to spend much time on formulating the question, because it directly affects the feedback and its quality" |
| **D** | Can you choose the **feedback community**? | "Ideally, it should not be limited to one's own organization… but instead include the possibility of inviting additional, freely definable respondents" |

Sample question set the book gives: **1.** Evaluate the prototype with 1 star (poor) to 5
stars (really awesome). **2.** What do you like about the prototype? **3.** What would you
change in the prototype?

Peter's use of such a tool: feedback on **prototype variants, procedures, and images or
links through URLs**, plus A/B testing, with an unlimited number of prototypes. "On the
same day he sets up the tool, Peter receives some initial feedback. **Within only two days,
he can give a valid assessment of the prototype variants**, based on which he can develop a
new product function."

**The multi-variant test scenario the book illustrates** — a wearable in the shape of a
belt that "when crossing the road… warns children or people who are peering into their
mobile phones," with three variants:

| Variant | Form |
|---|---|
| (1) | Vibration |
| (2) | Acoustic warning signal |
| (3) | Voice: *"Watch out! You're crossing the road!"* or *"Caution, bus from the left."* |

## Worked Example — Peter on Bahnhofstrasse, and the experiment grid

**The field test.** Peter's prototype is "an app for monitoring metabolic diseases with the
option of receiving help from a team of doctors online." He takes it out of his innovation
and co-creation lab to **Bahnhofstrasse in Zurich** — "Where else would he find the
clientele for such an upscale and expensive managed service?"

He approaches a woman in her mid-thirties leaving an exclusive handbag shop, heading for
her Bentley. **"She doesn't look sick or anything but Peter doesn't want to start the test
with false assumptions."** His sequence: *first offer help, come across as a likable
fellow, and build up empathy* — he carries her shopping bags, asks if she would like to
participate in the next big innovation, and two minutes later they are in the bar
kitty-corner from the shop.

She likes "the naive questions Peter asks with the attitude and behavior of a
**'greenhorn'**," and talks about herself and, more, about the illnesses of the older
gentlemen with whom she spends summer nights in Monte Carlo.

**The insight that only fieldwork produces:** *there is no Wi-Fi coverage on yachts on the
high seas, so getting online support from doctors there would be impossible.* A
context-killing constraint on the core value proposition, invisible from the lab.

(The chapter also records the cost: after one and a half bottles of Champagne, Priya
happens to walk past the bar — "at least now we know what triggered Peter's little marital
crisis with Priya.")

**The experiment grid — the book's filled-in template**

| Field | Prompt |
|---|---|
| **Step 1: Hypothesis** | "We believe that…" |
| **Step 2: Test** | "To verify this, we will…" — the experiment can be a prototype we want to test with customers/users, an interview, a survey, and so forth |
| **Step 3: Metrics** | "And measure…" — "a certain volume of positive feedback or just a specific value" |
| **Step 4: Criteria** | "We are on the right track if…" — the criterion that shows whether we are on the right (or wrong) track |
| **Step 5: Carry out** | Conduct the experiment; document with photos or videos |
| **Step 6: Learnings** | "We have learned the following:" — insights gained, conclusions drawn, and what measures we will undertake |

The grid pairs **Experiment 1 → Learnings 1**, so the learning progress is documented
alongside the experiment definition.

## Key Takeaways
- Define scenarios and a clear goal before the test.
- Involve neutral people — people who did not build the prototype.
- Ask simple, open, non-suggestive questions; always ask "why."
- Keep tests short and focused on the essential.
- Take stakeholder groups (e.g. developers) along so they experience user feedback
  firsthand.
- Let test persons think aloud; don't interrupt, steer, or sell.
- Don't reveal too much, too early, about how the prototype works.
- Document the tests, and **always schedule enough time after a test to integrate the
  findings into a new prototype**.
- Use Web-based tools for simple prototypes.
- **Perform many qualitative tests with no more than five test persons in each one.**

## Connects To
- [Chapter 2 — Process awareness](ch02-process-awareness-micro-macro-cycle.md) — Test as a micro-cycle step and as a permanent macro-cycle habit.
- [Chapter 4 — How to discover user needs](ch04-discover-user-needs.md) — the questioning technique behind the test interview.
- [Chapter 5 — How to build empathy with the user](ch05-build-empathy.md) — the empathy Peter builds before he tests.
- [Chapter 9 — How to create a good prototype](ch09-create-a-good-prototype.md) — step 6 of the prototyping workshop hands over to this chapter.
- [Chapter 19 — Lean business model thinking](ch19-lean-business-model-thinking.md) — hypotheses, MVPs and validated learning.
- [Chapter 18 — Systems thinking and complexity](ch18-systems-thinking-complexity.md) — situation analysis, where the quantitative half of testing scales up.
- Companion skill **dsh-design-thinking-toolbox** — mechanics of the feedback-capture grid, the experiment grid, A/B testing and structured user tests.
