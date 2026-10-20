# Chapter 21: Exploration and Discovery

## Core Idea
Agents that proactively venture into unfamiliar territory, experiment with new approaches, and generate new knowledge — the pattern for uncovering **"unknown unknowns"** rather than optimizing within a predefined solution space.

## Frameworks Introduced
- **Exploration and Discovery pattern**: proactive search for novel information and possibilities, distinct from reactive behavior or optimization inside a known space.
  - When to use: open-ended, complex, or rapidly evolving domains where the solution space is not fully defined; tasks requiring generation of novel hypotheses, strategies, or insights; when the objective is uncovering unknown unknowns rather than optimizing a known process.
- **Google AI co-scientist** — a multi-agent framework on Gemini emulating collaborative, iterative science, with a **supervisor agent** coordinating specialists in an **asynchronous task execution framework** allowing flexible scaling of compute. The six agents:
  - **Generation agent** — produces initial hypotheses through literature exploration and simulated scientific debates.
  - **Reflection agent** — acts as peer reviewer, assessing correctness, novelty, and quality.
  - **Ranking agent** — an **Elo-based tournament** comparing, ranking, and prioritizing hypotheses through simulated debates.
  - **Evolution agent** — refines top-ranked hypotheses by simplifying concepts, synthesizing ideas, and exploring unconventional reasoning.
  - **Proximity agent** — computes a proximity graph clustering similar ideas, assisting exploration of the hypothesis landscape.
  - **Meta-review agent** — synthesizes insights from all reviews and debates to identify common patterns and provide feedback, letting the system continuously improve.
  - Operating loop: **"generate, debate, and evolve"**, mirroring the scientific method, with **test-time compute scaling** allocating more computation to iteratively reason and enhance outputs.
- **Agent Laboratory** (Samuel Schmidgall, MIT License) — an autonomous research workflow in four phases:
  1. **Literature Review** — LLM agents autonomously collect and critically analyze scholarly literature via external databases (arXiv), synthesizing and categorizing to establish a knowledge base.
  2. **Experimentation** — collaborative experimental design, data preparation, execution, and analysis, using Python for code generation/execution and Hugging Face for model access, with iterative refinement based on real-time outcomes.
  3. **Report Writing** — synthesizing findings with literature insights, structured to academic conventions, integrating LaTeX for formatting and figures.
  4. **Knowledge Sharing** — **AgentRxiv**, a decentralized repository where autonomous agents deposit, retrieve, and build on research outputs, fostering cumulative progress.
- **Tripartite agentic judgment** — three reviewer agents with distinct evaluative stances, collectively mimicking the multi-faceted nature of human peer review:
  - a harsh but fair reviewer expecting good experiments that lead to insights;
  - a harsh, critical but fair reviewer looking for impact in the field;
  - a harsh but fair **open-minded** reviewer looking for novel ideas not proposed before.
- **The academic-hierarchy agent roles** (Agent Laboratory): **Professor Agent** (research director: sets the agenda, defines questions, delegates); **PostDoc Agent** (executes: literature review, experiment design and implementation, writes and runs code, produces papers); **Reviewer Agents** (peer review of quality, validity, scientific rigor); **ML Engineering Agents** and **SWEngineer Agents** (paired dialogic collaboration producing simple data-preparation code).

## Key Concepts
- **Unknown unknowns**: what exploration targets, as opposed to optimizing known variables.
- **Elo rating for hypotheses**: a tournament ranking that turned out to be **concordant with accuracy** — the system's internal Elo tracked real correctness.
- **Test-time compute scaling**: across 200+ research goals, scaling inference compute **consistently improved hypothesis quality** as measured by Elo.
- **Scientist-in-the-loop**: the design philosophy — augmentation, not automation. Researchers guide the system in natural language, give feedback, contribute ideas, and direct exploration.
- **Structured review output**: a THOUGHT (note-taking, specific not generic) followed by a strict JSON review with Summary, Strengths, Weaknesses, Originality/Quality/Clarity/Significance (1–4), Questions, Limitations, Ethical Concerns, Soundness/Presentation/Contribution (1–4), Overall (1–10), Confidence (1–5), and a binary Accept/Reject **Decision** — deliberately banning Weak Accept / Borderline / Strong Reject.
- **Exploration-exploitation dilemma**: the underlying decision-theoretic tension.

## Mental Models
- **Generate, debate, evolve** is the loop. Generation alone produces volume; the debate and evolution stages produce quality.
- **Rank by tournament, not by score.** Pairwise comparison in simulated debate ranks hypotheses more reliably than absolute scoring — and the resulting Elo turned out to track accuracy.
- **Three critics with different stances beat one critic with a rubric.** Rigor, impact, and novelty pull in different directions; separate agents keep all three represented.
- **Force a binary decision.** Banning "Borderline Accept" removes the reviewer's escape hatch and makes the judgment usable.

## Anti-patterns
- **Optimizing inside a predefined space when the goal is discovery**: static, pre-programmed information is insufficient for genuine innovation.
- **A single reviewer**: collapses rigor, impact, and novelty into one perspective.
- **Allowing hedged decisions**: "Borderline Accept" is not an output a system can act on.
- **Generic review comments**: the THOUGHT section explicitly demands specificity to the current paper.
- **Trusting the literature base as complete**: the co-scientist's knowledge is constrained by **open-access literature**, potentially missing critical paywalled work.
- **Ignoring the absence of negative results**: rarely published, but crucial to experienced scientists — a structural blind spot.
- **Deploying discovery agents without safety review**: research goals and generated hypotheses both need checking against unsafe or unethical use.

## Code Examples
Three reviewers with distinct stances:
```python
class ReviewersAgent:
    def __init__(self, model="gpt-4o-mini", notes=None, openai_api_key=None):
        self.notes = notes if notes is not None else []
        self.model = model
        self.openai_api_key = openai_api_key

    def inference(self, plan, report):
        reviewer_1 = ("You are a harsh but fair reviewer and expect good experiments "
                      "that lead to insights for the research topic.")
        review_1 = get_score(outlined_plan=plan, latex=report,
                             reward_model_llm=self.model, reviewer_type=reviewer_1,
                             openai_api_key=self.openai_api_key)

        reviewer_2 = ("You are a harsh and critical but fair reviewer who is looking "
                      "for an idea that would be impactful in the field.")
        review_2 = get_score(..., reviewer_type=reviewer_2)

        reviewer_3 = ("You are a harsh but fair open-minded reviewer that is looking "
                      "for novel ideas that have not been proposed before.")
        review_3 = get_score(..., reviewer_type=reviewer_3)

        return f"Reviewer #1:\n{review_1},\nReviewer #2:\n{review_2},\nReviewer #3:\n{review_3}"
```

The review output contract:
```
Respond in the following format:

THOUGHT:
<THOUGHT>

REVIEW JSON:
```json
<JSON>
```

In <THOUGHT>, first briefly discuss your intuitions and reasoning for the evaluation.
Detail your high-level arguments, necessary choices and desired outcomes of the review.
Do not make generic comments here, but be specific to your current paper.
Treat this as the note-taking phase of your review.

In <JSON>, provide the review in JSON format with the following fields in order:
- "Summary": A summary of the paper content and its contributions.
- "Strengths": A list of strengths of the paper.
- "Weaknesses": A list of weaknesses of the paper.
- "Originality": A rating from 1 to 4 (low, medium, high, very high).
- "Quality": A rating from 1 to 4 (low, medium, high, very high).
- "Clarity": A rating from 1 to 4 (low, medium, high, very high).
- "Significance": A rating from 1 to 4 (low, medium, high, very high).
- "Questions": A set of clarifying questions to be answered by the paper authors.
- "Limitations": A set of limitations and potential negative societal impacts.
- "Ethical Concerns": A boolean value indicating whether there are ethical concerns.
- "Soundness": A rating from 1 to 4 (poor, fair, good, excellent).
- "Presentation": A rating from 1 to 4 (poor, fair, good, excellent).
- "Contribution": A rating from 1 to 4 (poor, fair, good, excellent).
- "Overall": A rating from 1 to 10 (very strong reject to award quality).
- "Confidence": A rating from 1 to 5 (low, medium, high, very high, absolute).
- "Decision": Must be one of: Accept, Reject.

For the "Decision" field, don't use Weak Accept, Borderline Accept, Borderline Reject,
or Strong Reject. Instead, only use Accept or Reject.
This JSON will be automatically parsed, so ensure the format is precise.
```

The PostDoc agent carrying prior-round context forward:
```python
class PostdocAgent(BaseAgent):
    def __init__(self, model="gpt4omini", notes=None, max_steps=100, openai_api_key=None):
        super().__init__(model, notes, max_steps, openai_api_key)
        self.phases = ["plan formulation", "results interpretation"]

    def context(self, phase):
        sr_str = ""
        if self.second_round:
            sr_str = (f"The following are results from the previous experiments\n"
                      f"Previous Experiment code: {self.prev_results_code}\n"
                      f"Previous Results: {self.prev_exp_results}\n"
                      f"Previous Interpretation of results: {self.prev_interpretation}\n"
                      f"Previous Report: {self.prev_report}\n"
                      f"{self.reviewer_response}\n\n\n")

        if phase == "plan formulation":
            return (sr_str, f"Current Literature Review: {self.lit_review_sum}")
        elif phase == "results interpretation":
            return (sr_str,
                    f"Current Literature Review: {self.lit_review_sum}\n"
                    f"Current Plan: {self.plan}\n"
                    f"Current Dataset code: {self.dataset_code}\n"
                    f"Current Experiment code: {self.results_code}\n"
                    f"Current Results: {self.exp_results}")
        return ""
```
- **What it demonstrates**: the second-round context carries code, results, interpretation, report, **and the reviewer response** — the critique is an input to the next round, which is what makes the loop cumulative rather than repetitive.

Paired engineer agents, each told the other's role:
```python
# Software engineer directing:
"You are a software engineer directing a machine learning engineer, where the machine "
"learning engineer will be writing the code, and you can interact with them through dialogue.\n"
"Your goal is to help the ML engineer produce code that prepares the data for the provided "
"experiment. You should aim for very simple code to prepare the data, not complex code. "
"You should integrate the provided literature review and the plan and come up with code to "
"prepare data for this experiment.\n"

# ML engineer being directed:
"You are a machine learning engineer being directed by a PhD student who will help you write "
"the code, and you can interact with them through dialogue.\n"
"Your goal is to produce code that prepares the data for the provided experiment. You should "
"aim for simple code to prepare the data, not complex code. ..."
```

## Reference Tables

| Application | What is explored |
|---|---|
| Scientific research automation | design and run experiments, analyze, form new hypotheses — novel materials, drug candidates, principles |
| Game playing and strategy | game states, emergent strategies, vulnerabilities (AlphaGo) |
| Market research and trend spotting | unstructured data — social media, news, reports — for trends and opportunities |
| Security vulnerability discovery | systems and codebases for flaws and attack vectors |
| Creative content generation | combinations of styles, themes, data for art, music, literature |
| Personalized education | learning paths and content by progress, style, weak areas |

| Co-scientist agent | Role |
|---|---|
| Generation | initial hypotheses via literature exploration and simulated debate |
| Reflection | peer review of correctness, novelty, quality |
| Ranking | Elo-based tournament through simulated debates |
| Evolution | refine top hypotheses — simplify, synthesize, unconventional reasoning |
| Proximity | proximity graph clustering similar ideas |
| Meta-review | synthesize all reviews to find patterns and feed improvement |

| Agent Laboratory phase | Tools |
|---|---|
| Literature Review | arXiv and external databases |
| Experimentation | Python code generation/execution, Hugging Face |
| Report Writing | LaTeX formatting and figure generation |
| Knowledge Sharing | AgentRxiv decentralized repository |

| Co-scientist limitation | Consequence |
|---|---|
| Reliance on open-access literature | may miss critical paywalled prior work |
| Limited access to negative results | rarely published, but crucial to experienced scientists |
| Inherits LLM limitations | factual inaccuracies and hallucinations |

## Worked Example
**The AI co-scientist's validation results** — worth knowing in detail, because they are the strongest published evidence that a multi-agent discovery loop produces real findings.

*Benchmarks and expert review.* On the GPQA benchmark the system's internal Elo rating was **concordant with the accuracy of its results**, reaching 78.4% top-1 on the difficult "diamond set" — meaning the tournament ranking was tracking truth, not just confidence. Across **200+ research goals**, scaling test-time compute consistently improved hypothesis quality by Elo. On 15 curated challenging problems it outperformed other state-of-the-art models *and* human experts' "best guess" solutions. Biomedical experts rated its outputs more novel and impactful than baselines, and six expert oncologists judged its drug-repurposing proposals — formatted as NIH Specific Aims pages — to be high quality.

*Wet-lab validation.* Three end-to-end results:
- **Drug repurposing (AML).** The system proposed novel candidates including **KIRA6**, which had *no prior preclinical evidence* for use in acute myeloid leukemia. In vitro experiments confirmed KIRA6 and other suggestions inhibited tumor cell viability at clinically relevant concentrations across multiple AML cell lines.
- **Novel target discovery (liver fibrosis).** It identified novel epigenetic targets; experiments in human hepatic organoids showed drugs targeting those modifiers had significant anti-fibrotic activity — and one identified drug is already FDA-approved for another condition, opening a repurposing path.
- **Antimicrobial resistance.** Asked to explain why certain mobile genetic elements (cf-PICIs) appear across many bacterial species, the system's top-ranked hypothesis **in two days** was that cf-PICIs interact with diverse phage tails to expand host range — mirroring an unpublished, experimentally validated discovery an independent group had reached after **more than a decade** of research.

*Safety.* All research goals are reviewed for safety on input and generated hypotheses are checked to prevent unsafe or unethical research. A preliminary evaluation using **1,200 adversarial research goals** found the system robustly rejected dangerous inputs. Access is staged through a Trusted Tester Program.

## Key Takeaways
1. Exploration and Discovery lets agents actively pursue new information — essential in complex, evolving environments where the solution space is undefined.
2. Google's AI co-scientist shows agents autonomously generating hypotheses and designing experiments, with six specialized agents under a supervisor in a generate-debate-evolve loop.
3. Elo-based tournament ranking through simulated debate proved concordant with accuracy; scaling test-time compute consistently improved hypothesis quality.
4. Agent Laboratory's phases — literature review, experimentation, report writing, knowledge sharing via AgentRxiv — automate the research lifecycle while keeping humans in charge.
5. Three reviewers with distinct stances (rigor, impact, novelty) approximate human peer review better than one.
6. Force structured, specific, binary review output; ban hedged decisions.
7. The design philosophy is augmentation with a scientist-in-the-loop, not full automation.
8. Real limitations remain: open-access-only literature, no negative results, and inherited LLM hallucination.
9. Discovery agents need safety review of both goals and generated hypotheses.

## Connects To
- **Ch 9 (Learning and Adaptation)**: AlphaEvolve and OpenEvolve as evolutionary discovery systems.
- **Ch 7 (Multi-Agent Collaboration)**: the co-scientist and Agent Laboratory are hierarchical multi-agent systems.
- **Ch 6 (Planning)**: Deep Research as iterative investigation under a plan.
- **Ch 4 (Reflection)** and **Ch 19 (Evaluation and Monitoring)**: the Reflection agent and tripartite judgment.
- **Ch 20 (Prioritization)**: tournament ranking prioritizes which hypotheses to evolve.
- **Ch 13 (Human-in-the-Loop)**: the scientist-in-the-loop paradigm.
- **Ch 18 (Guardrails)**: safety review of research goals and generated hypotheses.
