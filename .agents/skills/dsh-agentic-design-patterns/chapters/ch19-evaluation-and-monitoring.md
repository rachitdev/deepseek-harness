# Chapter 19: Evaluation and Monitoring

## Core Idea
Agents are probabilistic, so pass/fail software tests are insufficient. Evaluation must be continuous and multi-faceted — measuring not just the final output but the **trajectory** taken to reach it, plus latency, token cost, drift, and subjective quality.

## Frameworks Introduced
- **Continuous evaluation and monitoring**: define metrics, establish feedback loops, implement reporting so performance stays aligned with expectations in operational environments.
  - When to use: live production deployments where real-time performance and reliability matter; systematic comparison of agent versions; regulated or high-stakes domains needing compliance, safety, and ethical audits; environments where performance may drift; and whenever behavior (trajectory) or subjective quality must be assessed.
- **The five evaluation targets**:
  - **Agent Response Assessment** — is the output pertinent, correct, logical, unbiased, accurate? Metrics: factual correctness, fluency, grammatical precision, adherence to user intent.
  - **Latency Monitoring** — time to process requests and generate outputs. Critical in real-time or interactive environments.
  - **Token Usage Tracking** — billing depends on input + output tokens; efficient usage directly reduces operational expense and reveals prompt-engineering opportunities.
  - **Trajectory Analysis** — the sequence of steps: tool selection, strategies, task efficiency.
  - **Subjective Quality (LLM-as-a-Judge)** — nuanced, human-like evaluation of qualities like helpfulness.
- **The six trajectory comparison methods** — comparing actual actions against the ground-truth trajectory:
  - **Exact match** — perfect match to the ideal sequence.
  - **In-order match** — correct actions in order, extra steps allowed.
  - **Any-order match** — correct actions in any order, extra steps allowed.
  - **Precision** — relevance of predicted actions.
  - **Recall** — how many essential actions were captured.
  - **Single-tool use** — checking for one specific action.
  - Metric selection depends on requirements: high-stakes may demand exact match; flexible situations use in-order or any-order.
- **Test files vs evalset files** (ADK):
  - **Test files** — JSON, a single simple session with multiple turns. Ideal for **unit testing** during active development: rapid execution, simple session complexity. Each turn contains the user query, expected tool-use trajectory, intermediate agent responses, and final response. Can be organized into folders with a `test_config.json` defining criteria.
  - **Evalset files** — multiple, potentially lengthy sessions. For simulating complex multi-turn conversations and **integration tests**. Comprises multiple "evals", each a distinct session with one or more turns.
- **From Agents to Advanced Contractors** (*Agent Companion*, Gulli et al.) — an evolution from probabilistic, brittle agents to deterministic, accountable ones, built on **four pillars**:
  1. **The Formalized Contract** — a detailed specification as the single source of truth. Not "analyze last quarter's sales" but "a 20-page PDF report analyzing European market sales from Q1 2025, including five specific data visualizations, a comparative analysis against Q1 2024, and a risk assessment based on the included dataset of supply chain disruptions." Defines deliverables, precise specifications, acceptable data sources, scope of work, and even expected computational cost and completion time — making the outcome **objectively verifiable**.
  2. **Dynamic Lifecycle of Negotiation and Feedback** — the contract starts a dialogue, not a command. The agent can respond: "The specified XYZ database is inaccessible. Please provide credentials or approve the use of an alternative public database, which may slightly alter the data's granularity." Resolves ambiguity and flags risks **before** execution.
  3. **Quality-Focused Iterative Execution** — prioritizes correctness over latency, with self-validation. For a code contract: generate multiple algorithmic approaches, compile and run them against the contract's unit tests, score each on performance, security, and readability, and submit only the version passing all criteria.
  4. **Hierarchical Decomposition via Subcontracts** — a primary contractor acts as project manager, generating new formal subcontracts. "Build an e-commerce mobile application" decomposes into "designing the UI/UX", "developing the user authentication module", "creating the product database schema", "integrating a payment gateway" — each a complete independent contract assignable to a specialized agent.
- **The AI "Contract" as an enterprise control instrument**: a dynamic agreement codifying objectives, rules, and controls for AI-delegated tasks.

## Key Concepts
- **Concept drift**: performance degrading as input data distribution or the environment shifts.
- **Anomaly detection in agent behavior**: unusual actions signalling an error, a malicious attack, or emergent undesired behavior.
- **Ground-truth trajectory**: the ideal action sequence — e.g. for a product query: intent determination → database search tool → result review → report generation.
- **LLM-as-a-Judge**: an LLM evaluating another agent's output against a rubric.
- **Where to store telemetry**: printing to console is insufficient. Structured log files (JSON), time-series databases (InfluxDB, Prometheus), data warehouses (Snowflake, BigQuery, PostgreSQL), or observability platforms (Datadog, Splunk, Grafana Cloud).
- **Sophisticated response metrics**: String Similarity (Levenshtein, Jaccard), Keyword Analysis, Semantic Similarity (cosine similarity over embeddings), LLM-as-a-Judge, and RAG-specific metrics (faithfulness, relevance).
- **ADK's three execution modes**: `adk web` (interactive evaluation and dataset generation), `pytest` integration via `AgentEvaluator.evaluate` (CI/CD), `adk eval` (automated CLI for build verification; specific evals can be selected by listing them comma-separated after the eval set filename).

## Mental Models
- **The trajectory is the unit of evaluation, not the answer.** A right answer reached by a wrong path will fail differently next time.
- **Evaluating a multi-agent system is assessing a team project** — many steps and handoffs, which is an *advantage*: you can check quality at each stage.
- **Formal contract over prompt.** Ambiguity is what makes agents brittle in production; specification is the fix.
- **Match the strictness of the trajectory metric to the stakes.** Exact match for high-stakes, any-order where the path genuinely doesn't matter.

## Anti-patterns
- **Exact string match as an accuracy metric**: "The capital of France is Paris." vs "Paris is the capital of France." scores 0.0 despite identical meaning. Use semantic similarity.
- **Printing metrics to the console**: not persistent, not queryable, not alertable.
- **Traditional pass/fail testing only**: standard code yields predictable results; agents are probabilistic and need qualitative assessment.
- **Evaluating only individual agents in a multi-agent system**: misses cooperation failures, plan deviation, wrong-agent selection, and scalability regressions.
- **Static test cases in a changing environment**: environments are not static; evaluation methods and test cases must adapt over time.
- **Brief, underspecified instructions in production**: suitable for demos, brittle where ambiguity leads to failure.
- **Treating LLM-as-a-Judge as complete**: it may overlook intermediate steps and is limited by the judging LLM's capabilities.

## Code Examples
The naive accuracy metric — and why it fails:
```python
def evaluate_response_accuracy(agent_output: str, expected_output: str) -> float:
    """Calculates a simple accuracy score for agent responses."""
    # Very basic exact match; real-world would use more sophisticated metrics
    return 1.0 if agent_output.strip().lower() == expected_output.strip().lower() else 0.0

agent_response = "The capital of France is Paris."
ground_truth   = "Paris is the capital of France."
score = evaluate_response_accuracy(agent_response, ground_truth)   # -> 0.0  ❌
```
Both sentences convey the same meaning; strict character comparison returns 0.0. This is the motivating failure for semantic metrics.

Token usage monitor:
```python
class LLMInteractionMonitor:
    def __init__(self):
        self.total_input_tokens = 0
        self.total_output_tokens = 0

    def record_interaction(self, prompt: str, response: str):
        # In a real scenario, use the LLM API's token counter or a tokenizer
        input_tokens  = len(prompt.split())     # placeholder
        output_tokens = len(response.split())   # placeholder
        self.total_input_tokens  += input_tokens
        self.total_output_tokens += output_tokens

    def get_total_tokens(self):
        return self.total_input_tokens, self.total_output_tokens
```

LLM-as-a-Judge with an explicit five-criterion rubric:
```python
LEGAL_SURVEY_RUBRIC = """
You are an expert legal survey methodologist and a critical legal reviewer. Your task is
to evaluate the quality of a given legal survey question. Provide a score from 1 to 5 for
overall quality, along with a detailed rationale and specific feedback.

1. **Clarity & Precision (Score 1-5):**
   * 1: Extremely vague, highly ambiguous, or confusing.
   * 3: Moderately clear, but could be more precise.
   * 5: Perfectly clear, unambiguous, precise in its legal terminology and intent.
2. **Neutrality & Bias (Score 1-5):**
   * 1: Highly leading or biased, clearly influencing the respondent.
   * 3: Slightly suggestive or could be interpreted as leading.
   * 5: Completely neutral, objective, free from leading language or loaded terms.
3. **Relevance & Focus (Score 1-5):**
   * 1: Irrelevant to the stated survey topic or out of scope.
   * 5: Directly relevant and well-focused on a single concept.
4. **Completeness (Score 1-5):**
   * 1: Omits critical information or provides insufficient context.
   * 5: Provides all necessary context for the respondent to answer thoroughly.
5. **Appropriateness for Audience (Score 1-5):**
   * 1: Jargon inaccessible to the target audience, or overly simplistic for experts.
   * 5: Perfectly tailored to the assumed legal knowledge of the target audience.

**Output Format:** JSON with keys:
* 'overall_score': integer 1-5
* 'rationale': concise summary of strengths and weaknesses
* 'detailed_feedback': bullet list per criterion, suggesting specific improvements
* 'concerns': list of legal, ethical, or methodological concerns
* 'recommended_action': e.g. "Revise for neutrality", "Approve as is", "Clarify scope"
"""

class LLMJudgeForLegalSurvey:
    def __init__(self, model_name: str = 'gemini-1.5-flash-latest', temperature: float = 0.2):
        # flash for speed and cost; pro for highest quality.
        # Low temperature is better for deterministic evaluation.
        self.model = genai.GenerativeModel(model_name)
        self.temperature = temperature

    def _generate_prompt(self, survey_question: str) -> str:
        return (f"{LEGAL_SURVEY_RUBRIC}\n\n---\n"
                f"**LEGAL SURVEY QUESTION TO EVALUATE:**\n{survey_question}\n---")

    def judge_survey_question(self, survey_question: str) -> Optional[dict]:
        try:
            response = self.model.generate_content(
                self._generate_prompt(survey_question),
                generation_config=genai.types.GenerationConfig(
                    temperature=self.temperature,
                    response_mime_type="application/json"))
            if not response.parts:
                logging.error("LLM response was empty or blocked. Safety Ratings: "
                              f"{response.prompt_feedback.safety_ratings}")
                return None
            return json.loads(response.text)
        except json.JSONDecodeError:
            logging.error(f"Failed to decode LLM response as JSON. Raw: {response.text}")
            return None
```
- **What it demonstrates**: `response_mime_type="application/json"` enforces the output contract at the API level; the empty-`parts` branch catches content-moderation blocks that would otherwise look like a parse failure.

## Reference Tables

| Evaluation method | Strengths | Weaknesses |
|---|---|---|
| **Human Evaluation** | captures subtle behavior | difficult to scale, expensive, time-consuming; subjective human factors |
| **LLM-as-a-Judge** | consistent, efficient, scalable | intermediate steps may be overlooked; limited by LLM capabilities |
| **Automated Metrics** | scalable, efficient, objective | potential limitation in capturing complete capabilities |

| Trajectory comparison | Requires | Use when |
|---|---|---|
| Exact match | perfect sequence match | high-stakes, prescribed procedure |
| In-order match | correct actions in order, extras allowed | order matters, efficiency doesn't |
| Any-order match | correct actions, any order, extras allowed | independent steps |
| Precision | relevance of predicted actions | penalizing unnecessary tool calls |
| Recall | essential actions captured | ensuring nothing critical is skipped |
| Single-tool use | one specific action present | verifying a required capability was invoked |

| ADK artifact | Scope | Purpose |
|---|---|---|
| Test file (JSON) | one session, multiple turns | unit testing during active development |
| `test_config.json` | folder of test files | defines evaluation criteria |
| Evalset file | multiple long sessions ("evals") | integration testing, complex multi-turn conversations |

| ADK execution mode | Command | Fits |
|---|---|---|
| Web UI | `adk web` | interactive evaluation, dataset generation |
| Programmatic | `pytest` + `AgentEvaluator.evaluate` | CI/CD integration tests |
| CLI | `adk eval` | automated build generation and verification |

| Application | What is measured |
|---|---|
| Performance tracking in live systems | accuracy, latency, resource consumption (e.g. chatbot resolution rate) |
| A/B testing | competing agent versions or strategies in parallel |
| Compliance and safety audits | automated audit reports, verified by HITL or another agent; KPIs and alerts |
| Enterprise systems | the AI "Contract" codifying objectives, rules, controls |
| Drift detection | output relevance/accuracy over time under concept drift |
| Anomaly detection | unusual actions signalling error, attack, or emergent behavior |
| Learning progress | learning curve, skill improvement, generalization |

| Multi-agent question | Failure it detects |
|---|---|
| Are agents cooperating effectively? | Flight-Booking Agent passes wrong dates to Hotel-Booking Agent → hotel booked for the wrong week |
| Did they create a good plan and stick to it? | Hotel Agent books before flight confirmed; or an agent gets stuck endlessly searching for the "perfect" rental car |
| Is the right agent chosen for the right task? | a General Knowledge Agent answering "it's usually warm in summer" instead of the Weather Agent's live data |
| Does adding more agents improve performance? | a new Restaurant-Reservation Agent creating conflicts and slowing the system — a scalability problem |

## Worked Example
**The contractor model applied to a financial analysis task** — the clearest illustration of what "specification instead of prompt" buys.

A conventional agent receives: *"Analyze last quarter's sales."* Every failure mode follows from that sentence's ambiguity — which quarter, which markets, what format, how long, compared against what, using which data.

The contract version specifies: *a 20-page PDF report analyzing European market sales from Q1 2025, including five specific data visualizations, a comparative analysis against Q1 2024, and a risk assessment based on the included dataset of supply chain disruptions* — plus acceptable data sources, scope of work, expected computational cost, and completion time. Every one of those is **objectively verifiable**, which means the output can be checked without a human judging whether it "seems right".

Then the lifecycle runs:
- **Negotiation.** The agent inspects the terms before working and returns: "The specified XYZ database is inaccessible. Please provide credentials or approve the use of an alternative public database, which may slightly alter the data's granularity." A conventional agent would have silently substituted a source or produced a worse report.
- **Iterative execution.** For the code-generation analogue: generate multiple algorithmic approaches, compile and run each against the contract's unit tests, score on performance, security, and readability, submit only what passes everything.
- **Subcontracting.** "Build an e-commerce mobile application" becomes four independent contracts — UI/UX design, user authentication module, product database schema, payment gateway integration — each with its own deliverables and assignable to a specialist.

The shift is from *hoping* the output is right to *verifying* it against terms both sides agreed to before work began.

## Key Takeaways
1. Evaluating agents goes beyond traditional tests to continuous measurement of effectiveness, efficiency, and requirement adherence in real environments.
2. Applications: live performance tracking, A/B testing, compliance audits, drift and anomaly detection, learning progress.
3. Exact string matching is inadequate; use semantic similarity, LLM-as-a-Judge, and RAG-specific metrics like faithfulness and relevance.
4. Monitor latency and token usage, and persist both to real storage — logs, time-series DBs, warehouses, or observability platforms.
5. Trajectories are crucial: compare actual actions against a ground-truth path with an appropriately strict comparison metric.
6. ADK offers test files for unit testing and evalset files for integration testing, runnable via web UI, pytest, or CLI.
7. Multi-agent evaluation asks about cooperation, plan adherence, correct agent selection, and whether adding agents actually helps.
8. Each evaluation method trades off differently: human captures nuance but doesn't scale; LLM-as-a-Judge scales but misses intermediate steps; automated metrics are objective but partial.
9. Formal contracts — verifiable deliverables, negotiation, self-validation, subcontracting — turn unpredictable tools into accountable systems fit for high-stakes domains.

## Connects To
- **Ch 11 (Goal Setting and Monitoring)**: internal goal tracking vs external continuous measurement.
- **Ch 4 (Reflection)**: the critic is an evaluator run inline; LLM-as-a-Judge is the same idea run offline.
- **Ch 18 (Guardrails)**: observability, structured logging, traceability, compliance audits.
- **Ch 9 (Learning and Adaptation)**: benchmarks as selection pressure for self-improving agents.
- **Ch 7 (Multi-Agent Collaboration)**: team-level metrics for cooperation and handoffs.
- **Ch 6 (Planning)**: contracts decompose into subcontracts as plans decompose into steps.
- **Ch 13 (Human-in-the-Loop)**: humans verify audit reports and supply ground truth.
