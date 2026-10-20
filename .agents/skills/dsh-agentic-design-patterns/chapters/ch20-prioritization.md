# Chapter 20: Prioritization

## Core Idea
In dynamic environments with many possible actions, conflicting goals, and finite resources, an agent needs an explicit process for ranking tasks by significance, urgency, dependencies, and defined criteria — otherwise it is efficient at nothing in particular.

## Frameworks Introduced
- **Prioritization pattern** — four elements:
  1. **Criteria definition** — the rules or metrics for task evaluation: **urgency** (time sensitivity), **importance** (impact on the primary objective), **dependencies** (whether the task is a prerequisite for others), **resource availability** (readiness of tools or information), **cost/benefit analysis** (effort vs expected outcome), and **user preferences** for personalized agents.
  2. **Task evaluation** — assessing each candidate against those criteria, from simple rules to complex scoring or LLM reasoning.
  3. **Scheduling / selection logic** — the algorithm that picks the optimal next action or task sequence, possibly via a queue or an advanced planning component.
  4. **Dynamic re-prioritization** — modifying priorities as circumstances change (a new critical event, an approaching deadline), giving adaptability and responsiveness.
  - When to use: an agentic system must autonomously manage multiple, often conflicting, tasks or goals under resource constraints in a dynamic environment.
- **The three levels of prioritization**:
  - **High-level goal prioritization** — selecting an overarching objective.
  - **Sub-task prioritization** — ordering steps within a plan.
  - **Action selection** — choosing the next immediate action from available options.
- **Prompt-encoded prioritization policy**: give the agent an explicit procedure (create → analyze for priority/assignee → default when missing → report) plus a fixed priority vocabulary and a roster of assignable workers.

## Key Concepts
- **P0 / P1 / P2**: a three-level priority vocabulary — P0 highest, P1 medium, P2 lowest.
- **Urgency keyword mapping**: "urgent", "ASAP", "critical" → P0. An explicit mapping in the prompt makes an ambiguous request actionable.
- **Reasonable defaults**: when priority or assignee is missing, the agent must make a sensible default (P1, 'Worker A') rather than stall or ask.
- **Pydantic `args_schema` on tools**: typed, validated, self-documenting tool arguments.
- **`model_copy(update=...)`**: type-safe partial update of a Pydantic model.
- **`handle_parsing_errors=True`** (AgentExecutor): keeps the loop alive when the LLM emits malformed tool syntax.
- **`ConversationBufferMemory(memory_key="chat_history", return_messages=True)`**: gives the PM agent continuity across requests.
- **Mirrors human team organization**: managers prioritize by considering input from all members.

## Mental Models
- **Prioritization is what separates an agentic system from an automated script**: the script runs a fixed order; the agent decides the order.
- **Criteria before scoring.** Most bad prioritization is not bad arithmetic — it is unstated criteria.
- **Re-prioritization is the hard part.** A one-time ranking is a schedule; continuous re-ranking under new information is prioritization.
- **Encode the defaults.** An agent that stalls on missing information is worse than one that picks a documented default and reports it.

## Anti-patterns
- **No defined criteria**: the agent optimizes for whatever the prompt last emphasized.
- **Static priorities**: a critical event arrives and the agent keeps working the old queue.
- **Unconstrained priority vocabulary**: without "must be one of P0, P1, P2" the agent invents levels and the downstream system cannot compare them.
- **No validation on tool arguments**: an invalid priority silently corrupts the task record. Validate and return a clear error.
- **Blocking on missing information**: specify defaults so the agent proceeds.
- **Sequencing tools implicitly**: state the order ("create first to get a `task_id`, then assign") or the agent will try to assign priority to a task that doesn't exist yet.

## Code Examples
Typed task model and manager:
```python
from pydantic import BaseModel, Field
from typing import Optional, Dict

class Task(BaseModel):
    """Represents a single task in the system."""
    id: str
    description: str
    priority: Optional[str] = None      # P0, P1, P2
    assigned_to: Optional[str] = None

class SuperSimpleTaskManager:
    """An efficient and robust in-memory task manager."""
    def __init__(self):
        self.tasks: Dict[str, Task] = {}    # dict for O(1) lookup/update/delete
        self.next_task_id = 1

    def create_task(self, description: str) -> Task:
        task_id = f"TASK-{self.next_task_id:03d}"
        self.tasks[task_id] = Task(id=task_id, description=description)
        self.next_task_id += 1
        return self.tasks[task_id]

    def update_task(self, task_id: str, **kwargs) -> Optional[Task]:
        """Safely updates a task using Pydantic's model_copy."""
        task = self.tasks.get(task_id)
        if task:
            update_data = {k: v for k, v in kwargs.items() if v is not None}
            self.tasks[task_id] = task.model_copy(update=update_data)
            return self.tasks[task_id]
        return None
```

Tools with validated argument schemas:
```python
from langchain_core.tools import Tool

class CreateTaskArgs(BaseModel):
    description: str = Field(description="A detailed description of the task.")

class PriorityArgs(BaseModel):
    task_id: str = Field(description="The ID of the task to update, e.g., 'TASK-001'.")
    priority: str = Field(description="The priority to set. Must be one of: 'P0', 'P1', 'P2'.")

class AssignWorkerArgs(BaseModel):
    task_id: str = Field(description="The ID of the task to update, e.g., 'TASK-001'.")
    worker_name: str = Field(description="The name of the worker to assign the task to.")

def assign_priority_to_task_tool(task_id: str, priority: str) -> str:
    """Assigns a priority (P0, P1, P2) to a given task ID."""
    if priority not in ["P0", "P1", "P2"]:
        return "Invalid priority. Must be P0, P1, or P2."
    task = task_manager.update_task(task_id, priority=priority)
    return (f"Assigned priority {priority} to task {task.id}." if task
            else f"Task {task_id} not found.")

pm_tools = [
    Tool(name="create_new_task", func=create_new_task_tool,
         description="Use this first to create a new task and get its ID.",
         args_schema=CreateTaskArgs),
    Tool(name="assign_priority_to_task", func=assign_priority_to_task_tool,
         description="Use this to assign a priority to a task after it has been created.",
         args_schema=PriorityArgs),
    Tool(name="assign_task_to_worker", func=assign_task_to_worker_tool,
         description="Use this to assign a task to a specific worker after it has been created.",
         args_schema=AssignWorkerArgs),
    Tool(name="list_all_tasks", func=task_manager.list_all_tasks,
         description="Use this to list all current tasks and their status."),
]
```

The prioritization policy, in the prompt:
```python
pm_prompt_template = ChatPromptTemplate.from_messages([
    ("system", """You are a focused Project Manager LLM agent. Your goal is to manage
project tasks efficiently.

When you receive a new task request, follow these steps:
1. First, create the task with the given description using the 'create_new_task' tool.
   You must do this first to get a 'task_id'.
2. Next, analyze the user's request to see if a priority or an assignee is mentioned.
   - If a priority is mentioned (e.g., "urgent", "ASAP", "critical"), map it to P0.
     Use 'assign_priority_to_task'.
   - If a worker is mentioned, use 'assign_task_to_worker'.
3. If any information (priority, assignee) is missing, you must make a reasonable default
   assignment (e.g., assign P1 priority and assign to 'Worker A').
4. Once the task is fully processed, use 'list_all_tasks' to show the final state.

Available workers: 'Worker A', 'Worker B', 'Review Team'
Priority levels: P0 (highest), P1 (medium), P2 (lowest)
"""),
    ("placeholder", "{chat_history}"),
    ("human", "{input}"),
    ("placeholder", "{agent_scratchpad}"),
])

pm_agent = create_react_agent(llm, pm_tools, pm_prompt_template)
pm_agent_executor = AgentExecutor(
    agent=pm_agent, tools=pm_tools, verbose=True, handle_parsing_errors=True,
    memory=ConversationBufferMemory(memory_key="chat_history", return_messages=True))
```

## Reference Tables

| Criterion | Question it answers |
|---|---|
| Urgency | how time-sensitive is this? |
| Importance | how much does it affect the primary objective? |
| Dependencies | is this a prerequisite for other tasks? |
| Resource availability | are the required tools/information ready? |
| Cost/benefit | is the effort justified by the expected outcome? |
| User preferences | what does this particular user value? |

| Level | Decision |
|---|---|
| High-level goal prioritization | which overarching objective to pursue |
| Sub-task prioritization | ordering steps within a plan |
| Action selection | which immediate action next |

| Domain | Priority rule |
|---|---|
| Automated customer support | urgent (system outage) over routine (password reset); high-value customers preferentially |
| Cloud computing | critical applications at peak demand; batch jobs to off-peak to optimize cost |
| Autonomous driving | braking to avoid collision > lane discipline > fuel efficiency |
| Financial trading | market conditions, risk tolerance, profit margins, real-time news |
| Project management | deadlines, dependencies, team availability, strategic importance |
| Cybersecurity | threat severity, potential impact, asset criticality |
| Personal assistant | user-defined importance, upcoming deadlines, current context |

## Worked Example
**The Project Manager agent handling two very different requests.**

*Request 1 (fully specified):* "Create a task to implement a new login system. It's urgent and should be assigned to Worker B."

The agent follows the prompt's procedure exactly. Step 1 creates the task and returns `TASK-001` — required before anything else, because the other two tools take a `task_id`. Step 2 finds "urgent" in the request and maps it to **P0** via the explicit keyword rule, and finds "Worker B" in the roster. Two tool calls. Step 4 lists the final state.

*Request 2 (underspecified):* "Manage a new task: Review marketing website content."

No priority, no assignee. Without step 3, the agent would either ask a clarifying question (halting an autonomous workflow) or invent something inconsistent. The documented defaults — **P1** and **Worker A** — let it proceed and produce a complete, well-formed record. Note that "Review Team" is in the roster and would be the smarter assignment here; the prompt's default is deliberately dumb-but-predictable, which is the right trade when nobody is watching.

Three design details carry the example: the **tool ordering is stated explicitly** because the dependency is invisible to the model otherwise; the **priority vocabulary is closed and validated in code**, so a hallucinated "P5" returns an error instead of corrupting state; and the **defaults are in the prompt**, so ambiguity produces a decision rather than a stall.

## Key Takeaways
1. Prioritization lets agents function effectively in complex, multi-faceted environments.
2. Agents rank tasks against established criteria: urgency, importance, dependencies, resource availability, cost/benefit, user preferences.
3. The pattern has four parts: criteria definition, task evaluation, scheduling/selection logic, dynamic re-prioritization.
4. Dynamic re-prioritization is what makes the agent responsive to real-time change.
5. Prioritization applies at three levels: strategic goals, sub-tasks within a plan, and immediate action selection.
6. Use a closed priority vocabulary, validate it in code, and map natural-language urgency terms to it explicitly.
7. Specify tool ordering and sensible defaults in the prompt so ambiguity produces a decision, not a stall.
8. Self-managing its own workflow is what separates a true agentic system from an automated script.

## Connects To
- **Ch 6 (Planning)**: prioritization orders the steps a plan produces.
- **Ch 11 (Goal Setting and Monitoring)**: goals supply the importance criterion; monitoring triggers re-prioritization.
- **Ch 16 (Resource-Aware Optimization)**: prioritizing critical tasks under resource constraint.
- **Ch 2 (Routing)**: action selection is routing among candidate next steps.
- **Ch 5 (Tool Use)**: typed tool schemas and validated arguments.
- **Ch 7 (Multi-Agent Collaboration)**: assigning tasks to specialized workers.
