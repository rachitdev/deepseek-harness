# Chapter 25: Building an Agent with AgentSpace

## Core Idea
AgentSpace is Google's platform for an "agent-driven enterprise": unified search across an organization's entire digital footprint, an **enterprise knowledge graph** for context, and a **no-code interface** for building agents that reason, plan, and execute multi-step actions.

## Frameworks Introduced
- **The AgentSpace platform** — four capabilities:
  - **Unified enterprise search** across documents, emails, and databases, using Gemini to comprehend and synthesize across varied sources.
  - **Specialized agents** that are not merely chatbots — they reason, plan, and execute multi-step actions autonomously (research a topic, compile a report with citations, generate an audio summary).
  - **Enterprise knowledge graph** mapping relationships between people, documents, and data, so the AI understands context and delivers relevant, personalized results.
  - **Agent Designer**, a no-code interface for creating custom agents without deep technical expertise.
- **The build flow**:
  1. Access AI Applications from the Google Cloud Console and choose **Agentspace** (vs Custom search).
  2. **Add an action** by connecting a service — Google sources (Calendar, Gmail) or third-party (Workday, Jira, Outlook, ServiceNow).
  3. Choose a prompt from Google's **prompt gallery** or author your own.
  4. Configure advanced features: connected data stores, Google Cloud Knowledge Graph or a Private Knowledge Graph, the web interface, and analytics.
  5. Use the chat interface.
- **Prompt definition fields**: Name, Display name, Title, Description, Prompt type (e.g. *User query*), the query text itself, Activation behavior (e.g. *New session*), Icon, and Enabled state.

## Key Concepts
- **A2A support**: AgentSpace supports a multi-agent system where agents communicate and collaborate through the open **Agent2Agent Protocol**, enabling more complex orchestrated workflows.
- **Security as a foundational component**: role-based access controls and data encryption protecting sensitive enterprise information.
- **Private Knowledge Graph**: leverages internal organizational data for enriched results and more contextually accurate query annotations — note it may take **up to 24 hours to regenerate data** after enabling.
- **Google Cloud Knowledge Graph**: expands search by incorporating external data sources, broadening scope and relevance.

## Mental Models
- **Search plus action, over one corpus.** The differentiator is not the agent — it is that the agent sits on a unified index of the organization's own data with a graph of who relates to what.
- **No-code lowers the author, not the ceiling.** Agent Designer abstracts the technical complexity, but the resulting agents still participate in A2A multi-agent workflows.
- **Connected actions are the capability boundary.** What the agent can *do* is exactly the set of services connected to it.

## Anti-patterns
- **Enabling the Private Knowledge Graph and expecting immediate results**: regeneration can take a day.
- **Building an agent with no connected actions**: it becomes a search interface, not an agent.
- **Skipping role-based access controls on enterprise data**: unified search across everything is exactly why access control matters.

## Reference Tables

| Connectable service | Category |
|---|---|
| Calendar, Gmail | Google sources |
| Workday, Jira, Outlook, ServiceNow | Third-party sources |

| Prompt gallery examples (Google-made) | Purpose |
|---|---|
| `goog_analyze_data` | Analyze Data |
| `goog_book_time_off` | Book Time Off |
| `goog_chat_with_content` / `goog_chat_with_documents` | Chat with content or documents |
| `goog_create_jira_ticket` | Create Jira Ticket |
| `goog_deep_research` | Deep Research |
| `goog_draft_email` | Draft Email |
| `goog_explain_technical_documentation` | Explain Technical Documentation |
| `goog_find_information` | Find Information |
| `goog_generate_code` | Generate Code |
| `goog_generate_image` | Generate Image |
| `goog_generate_marketing_copy` | Generate Marketing Copy |
| `goog_help_me_analyze` | Analyze/Visualize Data |

| Configuration tab | Controls |
|---|---|
| Connected data stores | your own data |
| Actions | connected services |
| Prompt gallery | Google-made and custom prompts |
| Configurations | Autocomplete, Search UI, Control, Assistant, Knowledge Graph, Feature Management |
| Integration | web interface exposure |
| Analytics | usage monitoring |

## Worked Example
**Creating a custom prompt in Agent Designer.** The Create-prompt form asks for:

- **Name**: `write`
- **Display name**: `writing assistant`
- **Title**: `My personal writing assistant`
- **Description**: `Help me to write concise sentences`
- **Prompt type**: `User query`
- **User query**: `You are a writing assistant who helps me to write concise sentences`
- **Activation behavior**: `New session`
- **Icon** and **Enabled** toggle

Note the separation of concerns in those fields: `Name` is the programmatic identifier, `Display name` and `Title` are what users see in the gallery, `Description` explains selection, and only `User query` is the actual instruction sent to the model. `Activation behavior: New session` determines that invoking this prompt starts a fresh conversation rather than continuing the current one — which is what makes a "specialized agent" specialized rather than a mode toggle on a running chat.

## Key Takeaways
1. AgentSpace unifies enterprise search across documents, emails, and databases, powered by Gemini.
2. Its agents reason, plan, and execute multi-step actions — research, report with citations, audio summary — not just chat.
3. An enterprise knowledge graph maps people, documents, and data so results are contextual and personalized.
4. Agent Designer is a no-code interface for custom agents.
5. Agents connect to Google and third-party services (Calendar, Gmail, Workday, Jira, Outlook, ServiceNow) — connected actions define capability.
6. A2A support enables multi-agent collaboration and orchestrated workflows.
7. Role-based access controls and data encryption are foundational, not optional, given unified access to enterprise data.
8. Hands-on learning is available via the "Build a Gen AI Agent with Agentspace" lab on Google Cloud Skills Boost.

## Connects To
- **Ch 15 (Inter-Agent Communication / A2A)**: the protocol AgentSpace agents collaborate over.
- **Ch 14 (Knowledge Retrieval / RAG)**: unified search and connected data stores.
- **Ch 5 (Tool Use)**: connected service actions.
- **Ch 6 (Planning)**: the built-in Deep Research prompt.
- **Ch 18 (Guardrails)**: role-based access control and encryption.
