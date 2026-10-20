# Chapter 15 — The CSVLOD Model Revisited

## Core Idea
Both dimensions of the CSVLOD taxonomy are actually **continuous axes**, turning the six-cell grid into a **coordinate plane** on which all 24 subtypes (and even non-EA documents) can be positioned. That plane makes visible the **decision paths of EA-enabled strategy execution** — the flow of planning decisions from business strategy to implemented systems. The chapter also states the model's **descriptive** limits and its known **exceptions**, and introduces **Enterprise Architecture on a Page**.

## Frameworks Introduced

### The continuous CSVLOD taxonomy (Figure 15.1)
| Axis | Extreme A | Extreme B |
|---|---|---|
| **What?** | **Generic** — intangible norms related to entire organizations, general concepts, essentially timeless; broad scope, less tangible and precise, less associated with time points; abstract suggestions, related to general ideas. Rules gravitate here. Closest examples: **Policies, IT Principles**. | **Specific** — tangible instances related to concrete projects, accurate details, exact time instants; narrow scope, tangible and precise, associated with time points; concrete suggestions, related to specific initiatives. Changes gravitate here. Closest examples: **Solution Overviews, Solution Designs**. |
| **How?** | **Business** — completely technology-neutral, pure business language: money, customers, capabilities, business goals, competitive advantages; more relevant to C-level business executives. Business-focused artifacts gravitate here. Closest examples: **Principles, Value Chains**. | **IT** — purely technical, IT-specific language: systems, applications, databases, platforms, networks; more relevant to ordinary IT specialists. IT-focused artifacts gravitate here. Closest examples: **Guidelines, Landscape Diagrams**. |

"The differences between the six general types of EA artifacts should not be considered as strict black-and-white distinctions."

### Seven non-EA documents mapped onto the plane (Figure 15.2)
| Document | Loosely related to |
|---|---|
| Corporate mission and values | Considerations |
| Strategic goals, objectives, KPIs, balanced scorecards (BSCs), strategy maps | Considerations |
| High-level strategic business plans; business motivation models (BMMs) | Visions |
| Programs of work | Visions |
| Business proposals | Outlines |
| Business cases | Outlines |
| Project management plans | Designs |

### The typical decision path (Figure 15.3)
The EA-based path from strategy planning to strategy implementation runs through **Considerations → Visions → Outlines → Designs**. Standards and Landscapes inform and influence decision-making but "rarely shape the decision path itself."

| Step | Process | Decision | Artifact |
|---|---|---|---|
| 1 | Strategic Planning | How the organization needs to work to execute its strategy | **Principles** |
| 2 | Strategic Planning | Which business capabilities become the focus of future IT investments | **Business Capability Models** |
| 3 | Strategic Planning | What initiatives will uplift those capabilities and when | **Roadmaps** |
| 4 | Initiative Delivery | How each planned initiative should be implemented | **Solution Overviews** |
| 5 | Initiative Delivery | Exactly how the approved solution should be implemented technically | **Solution Designs** |

These five decision types are aligned with the EA uncertainty principle and correspond closely to the five key discussion points: operating model, business capabilities, specific business needs, business processes, business requirements.

### An alternative decision path (Figure 15.4)
**Policies → Value Chains → Target States → Roadmaps → Options Assessments → Preliminary Solution Designs → Solution Designs.** Its logic and triggers:
| Substitution / addition | Trigger |
|---|---|
| Add **Policies** | Operating in a highly regulated industry, avoiding compliance risk |
| **Value Chains** instead of Business Capability Models | Warmer acceptance by the business audience |
| Add **Target States** between capabilities and Roadmaps | Rather radical transformation (switching operating model, consolidating systems after a merger) needing an explicit target platform architecture |
| Add **Options Assessments** before Solution Overviews | Seeking better early engagement with sponsors on how their needs will be addressed |
| Add **Preliminary Solution Designs** before Solution Designs | Reducing the risk of schedule and budget overruns; refining estimates and updating business cases |
| Drop **Principles** | Trivial/symbolic, or the imperatives are clearly shared without documentation |
| Drop **Solution Overviews** | Preparing them after Options Assessments seen as unnecessary and redundant |

"There is no single right way to organize an EA practice, but rather a multitude of possible options that should be applied selectively in different companies, environments and situations."

## Key Concepts — EA artifacts as decision points (Figure 15.5)
Most artifacts are decisions artifacts and can be read as **decision points** on the coordinate plane. Kotusev's mapping of subtype → the decision it supports:

| Artifact | Decision it supports |
|---|---|
| Principles | How business and IT need to work |
| Policies | What is not allowed to do |
| Conceptual Data Models | What data to possess |
| Analytical Reports | What technologies to adopt |
| Direction Statements | In what direction to move |
| IT Principles | How IT needs to work |
| Technology Reference Models | What technologies to use |
| Logical Data Models | How exactly to structure data |
| Patterns | What solutions to reuse |
| Guidelines | How exactly to use technologies |
| Value Chains | Which activities to focus on |
| Business Capability Models | Which capabilities to focus on |
| Context Diagrams | (current operational flows as planning context) |
| Target States | What IT platform to build |
| Roadmaps | What investments to make and when |
| Enterprise System Portfolios | (capabilities and constraints of the current landscape) |
| IT Roadmaps | What to improve in the IT landscape |
| Inventories / Landscape Diagrams | (facts on assets and connections) |
| Initiative Proposals | Whether to continue with IT initiatives |
| Options Assessments | Which options for IT solutions to choose |
| Solution Overviews | How to implement IT solutions |
| Preliminary Solution Designs | (refined estimates before construction) |
| Solution Designs | How exactly to construct IT systems |

This view lets artifacts be interpreted as **instruments for solving specific kinds of organizational problems** — the basis of the deliberate path to establishing an EA practice in Chapter 19.

## Mental Models
- **Proximity explains properties.** Business Capability Models and Roadmaps are both Visions and share all row, column and type properties — yet BCMs sit closer to the generic and business extremes (they only highlight which capabilities to uplift) while Roadmaps sit closer to the specific and IT extremes (they say what initiatives and when). "They still occupy the opposite corners within the Visions general type." Likewise Enterprise System Portfolios sit very close to Visions and are therefore somewhat Vision-like (high-level, occasionally usable with executives), whereas Landscape Diagrams are very distant from Visions and cannot be used for business discussion at all.
- **Mapping exercises pay off.** Periodically mapping the artifacts an organization maintains illustrates the "meta-architecture" of the practice, allows comparison with other companies, and helps identify missing or redundant artifacts and balance the whole set.

## Anti-patterns
- **Treating the model prescriptively.** "The CSVLOD model is purely descriptive in nature and provides only a 'food for thought' for architects, rather than a set of strict prescriptions."
- **Treating essential artifacts as mandatory.** Many successful practices skip some for sound reasons: no articulate global imperatives to document as Principles; Value Chains preferred over BCMs for familiarity; no Technology Reference Models because of heavy reliance on a strategic vendor's products which makes technology choices for them; Designs built directly from Options Assessments or even Initiative Proposals as more agile and lightweight; unique organization-specific artifacts used instead. **But** the eight essential artifacts are "a reasonable benchmark" — organizations "at least should be able to clearly explain why some of these artifacts are unnecessary or inappropriate for them, if any."
- **Counting artifacts as a maturity metric.** "Using a large number of various EA artifacts does not necessarily signify a mature EA practice, while using a small number of EA artifacts does not necessarily indicate an immature one, as long as all the six general types are adequately represented." Successful practices use **around 10–15** of the 24 subtypes.
- **Copying the book's sample artifacts verbatim.** The schematic representations in Chapters 9–14 "depict merely characteristic, typical or 'average' samples" and "should not be regarded as ideal or best possible specimens of EA artifacts to be compared against."

## Reference Tables

### What mature EA practices must have
All six general types present, though specific artifacts vary: **some Considerations** to maintain conceptual consistency · **some Standards** to define recommended approaches and technologies · **some Visions** to focus and guide future investments · **some Landscapes** to capture the current landscape structure · **some Outlines** to discuss initiatives at their early stages · **some Designs** to support implementation.

### Three classes of exceptions to the model
1. **Disposable architectural diagrams.** Temporary, no consistent meaning, created ad hoc — limited extracts from "true" artifacts, narrow slices, or modified versions of "master copies" adapted for particular needs. Used in discussions and presentations, included in stakeholder engagement packs, helpful for debating pros and cons of individual decisions and collecting feedback. "The CSVLOD model does not, and cannot, explain the meaning of all imaginable architectural drawings."
2. **Artifacts combining two adjacent general types.**
   | Combination | Example |
   |---|---|
   | Considerations + Standards | Principles and IT Principles merged into a common list of high-level directives |
   | Considerations + Visions | Principles placed inside Roadmaps for convenience |
   | Visions + Landscapes | Architectural initiatives placed in business-focused Roadmaps alongside regular business initiatives (common in small companies) |
   | Standards + Landscapes | Technology Reference Models and Enterprise System Portfolios merged into one artifact showing technical functions with technologies and business functions with applications. Also: some products (MS Excel) are both technology (general-purpose spreadsheets) and end-user application (macros and formulas automating processes), blurring the boundary |
   | Outlines + Designs | Designs built on the same document templates as Outlines, produced by filling in new technical sections in the previously approved Outlines |
3. **Artifacts combining two subtypes of one general type.** Broad IT Principles mixed with narrow Guidelines into a common list; tool-based architectural repositories combining the asset list, asset properties and asset connections — i.e. both **Landscape Diagrams and Inventories**.

"Even if some rare EA artifacts cannot be strictly related to any single general type... these artifacts can still be associated with multiple general types, and almost always specifically with two **adjacent** general types, which demonstrates the overall conceptual soundness of the model. Consequently, all practical EA artifacts can be explained within the boundaries of the CSVLOD model, no EA artifacts lay beyond the limits of the model."

## Worked Example
**Enterprise Architecture on a Page (Figure 15.6).** All 24 subtypes with their schematic graphical representations placed on a single page, colour-coded by prevalence (essential / common / uncommon) and structured around the CSVLOD model. It answers: What EA artifacts are useful? What information do they provide? What do they look like? For what purpose are they developed? How exactly are they used? What benefits do they bring? Positioned as "an intuitive visual model of enterprise architecture and as a convenient catalog of EA artifacts", a sense-making instrument, a printable poster, a common reference point for EA discussions, useful to aspiring and practising architects and to EA lecturers and students. Freely downloadable at **http://eaonapage.com**.

## Key Takeaways
- Both taxonomy dimensions are continuous; position tells you an artifact's properties.
- The typical decision path is Principles → Business Capability Models → Roadmaps → Solution Overviews → Solution Designs; real organizations customize it.
- The model is descriptive: essential artifacts are a benchmark, not a mandate; 10–15 artifacts is the pragmatic range.
- Exceptions exist but always resolve to two *adjacent* general types.

## Connects To
- [Chapter 8 — The CSVLOD Model of Enterprise Architecture](ch08-the-csvlod-model.md) — the formal model this chapter refines
- [Chapter 5 — The Dialog Between Business and IT](ch05-the-dialog-between-business-and-it.md) — the five discussion points the decision path mirrors
- [Chapter 19 — The Lifecycle of Enterprise Architecture Practice](ch19-the-lifecycle-of-ea-practice.md) — artifacts as instruments for problem solving, and the deliberate path
- [Chapter 18 — Instruments for Enterprise Architecture](ch18-instruments-for-enterprise-architecture.md) — how agility choices reshape the decision path
