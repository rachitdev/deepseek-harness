# Chapter 5 — The Dialog Between Business and IT

## Core Idea
The intuitive assumption that IT planning should start from the business strategy is wrong. **A business strategy in the narrow sense — a 3–5 year plan with goals, objectives and KPIs — rarely provides an adequate foundation for architectural planning.** Instead, business and IT stakeholders converse through **five key discussion points** that form a "middle ground" meaningful to both sides. The chapter closes with the **EA uncertainty principle**, which bounds what any EA artifact can usefully do.

## Frameworks Introduced

### Four problems with strategy-based architectural planning (Figure 5.1)
1. **Business strategy is often vague, unknown or merely absent** — motherhood statements ("industry-leading service provider"), differently interpreted by executives, secretly disagreed with, miscommunicated, or missing altogether (sometimes deliberately, to stay flexible, or for political reasons).
2. **Business strategy rarely provides a clear direction for IT** — mission statements, values, market-share goals, financial targets, brand positioning and market-entry intentions give little or no guidance on what IT systems are required; architects are forced to make major decisions on guesswork.
3. **Business strategy is often unstable and frequently changes** — shifting priorities instantly destroy strategy-based plans; plans can become obsolete before completion.
4. **Business strategy often requires strategy-specific, non-reusable IT systems** — strategies last ~3–5 years, the systems built for them live ten years or longer. Systems are "an asset for the current business strategy, but a liability for the next strategies", producing the **"alignment trap"** where the landscape becomes increasingly complex, fragile and inefficient, and IT becomes a bottleneck rather than a strategic asset.

**The paradox:** even a stable, articulate business strategy is simultaneously *too abstract* (cannot say what systems are needed, let alone their details) and *too specific* (too short-sighted for fundamental IT capabilities lasting beyond the strategy). "A business strategy specifies neither what IT should do right now, nor what IT should provide after the next 3-5 years."

### The four operating models (Figure 5.3)
An **operating model** is the desired level of organization-wide **process standardization** and **data integration** across major business units. The two decisions are mutually independent.

| | Low data integration | High data integration |
|---|---|---|
| **High process standardization** | **Replication** | **Unification** |
| **Low process standardization** | **Diversification** | **Coordination** |

| Model | Business units | IT landscape | Key features | Strategic leverages |
|---|---|---|---|---|
| **Diversification** | Diverse and independent; different processes, no shared data | Local applications and databases owned by units; global IT services and infrastructure | Independence, flexibility, local autonomy in serving customers | Synergies between units; economies of scale from shared IT infrastructure and services |
| **Coordination** | Diverse but interdependent; different processes, some shared data | Local applications owned by units; global databases (customers, products, suppliers), IT services and infrastructure | Superior customer service, local innovation, transparency, cross-selling and upselling | Deep process expertise of units; common IT infrastructure for global data sharing |
| **Replication** | Similar but independent; same processes, no shared data | Globally standardized but locally owned applications and databases; global IT services and infrastructure | Efficient processes, consistent customer experience, capacity for global process innovation | Standardized processes and systems for expanding into new markets and offering new services |
| **Unification** | Similar and interdependent; same processes, some shared data | Global applications, databases, IT services and infrastructure shared by all units (often comprehensive ERP) | Efficient processes, consistent customer experience, integrated data, minimized costs | Significant economies of scale from global standardization and integration |

Growth patterns: diversification grows locally or by acquisition needing minimal integration; coordination grows by new segments/markets and acquisitions whose systems must adopt common data standards; replication grows by replicating best practices in new markets and "ripping and replacing" acquired competitors' processes and systems; unification grows by extending product lines and ripping and replacing acquired custom systems. Large companies often run **different operating models at different organizational levels** (e.g. diversification at the top with coordination, replication and unification inside the three units).

Key insight: **"A business strategy is based on the assumptions of what is going to change, while an operating model is based on the premise of what is not going to change."** Each operating model delineates a family of compatible business strategies — so the operating model is a *driver* of business strategy. Sticking with one produces a **digitized platform / operational backbone** reusable by all subsequent strategies; not sticking with any is riskier still, because such organizations "have to start the implementation of any business strategies from scratch every time."

## Key Concepts — the five key discussion points
- **Operating model** — global process standardization and data sharing requirements.
- **Business capabilities** — a general ability or capacity to perform a specific business activity; encompasses underlying processes, procedures, people, knowledge, incentives and IT systems ("Marketing campaign management", "Customer behavior tracking", "Financial reporting"). Stable, change only on major business-model transformation. Not all capability improvements can be achieved with IT.
- **Specific business needs** — a general idea to address a particular business problem using IT ("Accelerate the mortgage lending process", "Enable personalized discounts for customers", "Improve the precision of demand forecasting"). Essentially proposed initiatives with a necessary IT component.
- **Business processes** — sequences of concrete activities by specific actors producing valuable outcomes ("Issue an insurance policy", "Ship an ordered product", "Produce an annual report").
- **Business requirements** (system requirements) — detailed functional and non-functional specifications for concrete IT systems.

## Mental Models
- **The middle ground must be dual.** Purely business notions (competitive advantages, strengths, market segments, customer experience) give no actionable IT implications; purely IT notions (applications, databases, infrastructure, networks) are not traceable to business value. Only dual notions work.
- **A hierarchy, not a menu (Figure 5.5, the pyramid).** Operating model → (business strategy) → business capabilities → specific business needs → business processes → business requirements. Lower levels are more fundamental, global, long-lasting; higher levels more volatile, local, short-term.
- **Business capabilities substitute the business strategy in business–IT discussions.** "The same business strategy translated into a set of strategic business capabilities can offer a much more explicit, unambiguous and sound foundation for IT-related planning efforts."

## Anti-patterns
- Starting information systems planning directly from the business strategy.
- Chasing the latest strategy with strategy-specific systems — leads to legacy proliferation and the alignment trap.
- Discussing highly abstract points (operating model, capabilities) during project implementation — useless (too vague for the stage).
- Trying to plan five years of an entire organization in terms of specific business needs, processes or requirements — unachievable.

## Reference Tables

### Table 5.1 — Main properties of the five key discussion points
| Discussion point | Abstraction level | Planning scope | Planning horizon | Key business concerns | Key IT concerns |
|---|---|---|---|---|---|
| Operating model | Very abstract | Very broad (entire organization) | Very long-term (>3–5 years) | What operating model should be appropriate for the business of an entire organization? | What general structure of the IT landscape is required to enable the adopted operating model? |
| Business capabilities | Abstract | Broad (entire organization or major business units) | Long-term (3–5 years) | What business capabilities should be improved to execute the business strategy? | What IT capabilities are required to improve strategic business capabilities? |
| Specific business needs | Moderate | Moderate (separate business areas) | Mid-term (2–3 years) | What specific business needs should be addressed in the future? | What types of IT solutions are required to address the identified business needs? |
| Business processes | Detailed | Narrow (separate IT initiatives) | Short-term (1–2 years) | How should specific business processes be changed? | What systems, data and infrastructure are required to change the business processes as requested? |
| Business requirements | Very detailed | Very narrow (separate IT projects) | Very short-term (<1 year) | What specific business functionality should be provided? | What technical structure of IT systems is required to provide the necessary functionality? |

### Supporting EA artifacts per discussion point
| Discussion point | Typically supported by |
|---|---|
| Operating model | Architecture principles or other dual Considerations (sometimes only verbal agreement) |
| Business capabilities | Business capability maps / models and other dual Visions |
| Specific business needs | IT investment roadmaps and other dual Visions |
| Business processes | Solution overviews and other dual Outlines |
| Business requirements | Solution designs and other dual Designs |

### The EA uncertainty principle (Figure 5.6)
> Organizations can be either planned for **broader scopes and longer horizons in less detail**, or planned for **narrower scopes and shorter horizons in more detail**, but they **cannot be planned for broad scopes and long horizons in great detail**.

| Deviation | Zone | Why impractical |
|---|---|---|
| Highly abstract points/artifacts for narrow scopes and short horizons | **Useless Zone** | Lacking the implementation-level detail expected at that stage; "highly abstract descriptions of the local short-term future have no practical value" |
| Highly detailed points/artifacts for broad scopes and long horizons | **Unachievable Zone** | Inherent uncertainty is too high; "highly detailed descriptions of the global long-term future state are practically impossible to develop" |

Scope/horizon calibration used in Figure 5.6: entire organization ≈ 5 years · organization or its major units ≈ 4 years · separate business areas ≈ 3 years · separate IT initiatives ≈ 2 years · separate IT projects ≈ 1 year.

## Worked Example
**Why generic strategy statements fail as EA input.** Kotusev's own examples: the mission statements "Become the most trusted service provider in the market" and "Turn into a perfectly customer-centric organization", and the objectives "Increase the gross annual revenue by 10% over the next three years" and "Extend the market share in the retail segment to 20% by 2024" are, on their own, "virtually meaningless and useless from the IT standpoint."

**Operating model constrains strategy.** The whole family of price-leadership strategies is appropriate for **replication** but inappropriate for **coordination**, whose landscape consists of diverse locally customized applications with no economies of scale from global standardization. Conversely, the whole family of customer-intimacy strategies is appropriate for **coordination** but inappropriate for **replication**, whose landscape consists of siloed, locally owned databases with no global sharing of customer information.

**Principles as operating-model specifications.** The principle "Single Customer View" is an explicit specification of the data-sharing requirement of the **coordination or unification** model; the principle "Standardized Business Processes" is an explicit specification of the process-standardization requirement of the **replication or unification** model.

## Key Takeaways
- Never take the business strategy as the direct input to architectural planning.
- Use the five discussion points at their proper abstraction level, scope and horizon.
- Select an operating model and stick with it — it is the only stable foundation for a reusable digitized platform.
- Test every proposed artifact against the EA uncertainty principle before creating it.

## Connects To
- [Chapter 4 — Enterprise Architecture and City Planning](ch04-enterprise-architecture-and-city-planning.md) — the artifact types that carry each discussion point
- [Chapter 6 — Processes of Enterprise Architecture Practice](ch06-processes-of-ea-practice.md) — which process uses which discussion point
- [Chapter 11 — Visions](ch11-visions.md) — Business Capability Models and Roadmaps in depth
- [Chapter 17 — Architecture Functions in Organizations](ch17-architecture-functions-in-organizations.md) — how the operating model shapes the architecture function
