# Chapter 26: AI Agents on the CLI

## Core Idea
The command line is becoming an intelligent, collaborative workspace. CLI agents understand natural language, maintain context about an entire codebase, and perform complex multi-step tasks — and the differentiator between them is less *what* they can do than the **quality, efficiency, and nuance** of the result.

## Frameworks Introduced
- **Claude CLI (Claude Code)** — engineered as a high-level coding agent with a deep, holistic understanding of project architecture. Its "agentic" nature lets it build a mental model of the repository for complex multi-step tasks; interaction is conversational, **resembling pair programming where it explains its plans before executing**. Built-in tools cover file ingestion, code structure analysis, and edit generation, with deep Git integration for branch and commit management. Extensibility is mediated by MCP for custom tools — private APIs, database queries, project-specific scripts. **The developer is the arbiter of the agent's functional scope: Claude is a reasoning engine augmented by user-defined tooling.**
  - Ideal for: professional developers on large-scale projects with significant refactoring or architecturally broad features.
- **Gemini CLI** — versatile, open-source, built on Gemini 2.5 Pro with a massive context window and **multimodal** capability (images and text). Open-source nature, generous free tier, and a "Reason and Act" loop make it transparent, controllable, and a strong all-rounder — especially within the Google Cloud ecosystem. Built-in tools: file system operations, a shell tool, web fetching and searching, multi-file reading for broader context, and a **memory tool** to save information for later sessions. Security foundation: **sandboxing** isolates model actions; **MCP servers** bridge safely to the local environment or other APIs.
- **Aider** — an open-source assistant acting as a true pair programmer that works directly on your files and commits to Git. Defining feature is **directness**: it applies edits, runs tests to validate them, and **automatically commits every successful change**. Model-agnostic, giving complete control over cost and capabilities. Its git-centric workflow suits developers who value efficiency, control, and a transparent auditable trail.
- **GitHub Copilot CLI** — extends Copilot into the terminal; its advantage is **native, deep GitHub integration**. It understands a project's context within GitHub and can be **assigned a GitHub issue, work on a fix, and submit a pull request** for human review.
- **Terminal-Bench** — an evaluation framework for AI agents executing complex tasks in a CLI. The terminal is identified as an optimal environment because it is **text-based and sandboxed**. `Terminal-Bench-Core-v0` comprises **80 manually curated tasks** spanning scientific workflows and data analysis. **Terminus**, a minimalistic agent, serves as a standardized testbed for comparing language models fairly. Designed for extensibility via containerization or direct connection; future work includes massively parallel evaluation and incorporating established benchmarks.

## Key Concepts
- **Mental model of the repository**: the property that separates an agent from an autocompleter.
- **Plan-before-execute conversation**: the agent explains its intended changes first.
- **Auto-commit on successful change** (Aider): every validated edit becomes a Git commit, producing an auditable trail.
- **Model-agnostic**: control over cost/capability trade-off by swapping the backing model.
- **Sandboxing**: isolating model actions to prevent risk.
- **Repository-aware Q&A**: answering "where is X defined and what environment variables does it need" with precise file paths.

## Mental Models
- **There is no single "best" tool** — a vibrant ecosystem where each agent has a specialized strength. Choose by the shape of your work, not by benchmark rank.
- **Claude for architecture, Gemini for versatility and multimodality, Aider for git-centric direct editing, Copilot for GitHub workflow integration.**
- **Many use cases overlap across all four**; the differentiator is result quality on a given task, which is what benchmarks like Terminal-Bench exist to measure.
- **MCP is the shared extensibility layer** — both Claude CLI and Gemini CLI use it to reach private APIs and local environments.

## Anti-patterns
- **Picking a CLI agent by feature list**: most can do most things; nuance of result is the actual differentiator.
- **Running an agent with shell access unsandboxed**: sandboxing exists precisely because the agent can execute.
- **Using an architecture-oriented agent for a one-line fix**, or a direct-edit agent for a codebase-wide refactor with subtle coupling.
- **No auditable trail**: without Git-committed increments, reviewing what an agent changed becomes archaeology.

## Reference Tables

| Agent | Core strength | Best for |
|---|---|---|
| **Claude CLI** | holistic architectural understanding; conversational plan-then-execute | large-scale refactoring, architecturally broad features |
| **Gemini CLI** | open-source, huge context, multimodal, free tier, ReAct loop | versatile all-round work, multimodal input, Google Cloud ecosystem |
| **Aider** | direct file edits + test validation + automatic Git commits; model-agnostic | TDD, precise bug fixes, auditable incremental change |
| **GitHub Copilot CLI** | native GitHub integration; issue → branch → PR | automated issue resolution, repository-aware Q&A |

| Agent | Example use cases |
|---|---|
| **Claude CLI** | "Our current user authentication relies on session cookies. Refactor the entire codebase to use stateless JWTs, updating the login/logout endpoints, middleware, and frontend token handling."<br>"Integrate this new weather API. Create a service module, add a component to display the weather, and update the main dashboard."<br>"Analyze `./src/utils/data_processing.js`. Generate comprehensive TSDoc comments for every function." |
| **Gemini CLI** | multimodal: `gemini describe component.png` → "Write the HTML and CSS to build a React component that looks exactly like this. Make sure it's responsive."<br>cloud: "Find all GKE clusters in the production project running versions older than 1.28 and generate a gcloud command to upgrade them one by one."<br>MCP: "Use the `get-employee-details --id=E90210` tool to fetch their name and team, then populate `welcome_template.md`."<br>refactor: replace `org.apache.log4j` with `org.slf4j` across all `*.java`, rewriting instantiation and `.info()`/`.debug()`/`.error()` calls to structured key-value format |
| **Aider** | TDD: "Create a failing test for a function that calculates the factorial of a number." → "Now, write the code to make the test pass."<br>bugfix: "The `calculate_total` function in `billing.py` fails on leap years. Add the file to the context, fix the bug, and verify against the existing test suite."<br>dependencies: update `requests` imports and deprecated calls across all Python files, then update `requirements.txt` |
| **Copilot CLI** | assign "Issue #123: Fix off-by-one error in pagination" → agent branches, codes, and submits a PR referencing the issue<br>"Where in this repository is the database connection logic defined, and what environment variables does it require?"<br>`gh? find all files larger than 50 MB, compress them, and place them in an archive folder` |

| Gemini CLI built-in tool | Function |
|---|---|
| File system | reading and writing |
| Shell | running commands |
| Web fetch / search | internet access |
| Multi-file read | broader context in one step |
| Memory | save information for later sessions |

## Worked Example
**The same repository, four agents, four natural fits.**

Consider a Python service with a leap-year bug in billing, an outdated logging library throughout, an open GitHub issue about pagination, and a pending migration from session cookies to JWTs.

- The **JWT migration** is architectural: endpoints, middleware, and frontend token handling all change together, and getting one wrong breaks auth. This is Claude CLI's case — it builds a mental model of the repo and explains its plan before touching files, which is exactly the review point you want before a coordinated multi-file change.
- The **logging library swap** is mechanical but wide: read every `*.java`, replace imports, rewrite instantiation and every `.info()`/`.debug()`/`.error()` call into structured key-value form. Gemini CLI's large context and multi-file read handle breadth.
- The **leap-year bug** is narrow and verifiable: add `billing.py` to context, fix, run the existing test suite. Aider applies the edit, runs the tests, and commits only on success — so the fix arrives as one reviewable commit with a green test behind it.
- The **pagination issue** already exists as GitHub Issue #123. Copilot CLI takes the assignment directly, branches, fixes, and opens a PR referencing the issue — no developer in the loop until review.

The point is not that each tool can only do its own row; all four could attempt any of these. It is that the *shape* of each task — architectural coupling, breadth, verifiability, workflow integration — maps to a different agent's strength.

## Key Takeaways
1. CLI agents understand natural language, hold codebase context, and automate multi-step development tasks.
2. There is no single best tool; each has a specialized strength and most use cases overlap.
3. Claude CLI excels at complex architectural work and explains plans before executing; MCP defines its functional scope.
4. Gemini CLI is open-source, multimodal, huge-context, sandboxed, and MCP-extensible — the strongest all-rounder.
5. Aider is direct and git-centric: edit, test, auto-commit, with a fully auditable trail and model-agnostic cost control.
6. GitHub Copilot CLI integrates natively with GitHub — issue assignment to pull request without manual intervention.
7. Terminal-Bench evaluates CLI agents on 80 curated tasks using the minimal Terminus agent as a standard testbed; the terminal suits agent evaluation because it is text-based and sandboxed.
8. Proficiency with these tools is becoming an essential development skill.

## Connects To
- **Ch 10 (Model Context Protocol)**: the shared extensibility layer for Claude CLI and Gemini CLI.
- **Ch 17 (Reasoning Techniques)**: Gemini CLI's Reason-and-Act loop.
- **Ch 19 (Evaluation and Monitoring)**: Terminal-Bench as agent evaluation.
- **Ch 23 (Agentic Interactions)**: vibe coding as the interaction style.
- **Ch 28 (Coding Agents)**: agents as team members in the development process.
- **Ch 18 (Guardrails)**: sandboxing and isolation for agents with shell access.
