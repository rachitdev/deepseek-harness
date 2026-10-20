# Chapter 14 — The Measuring Toolkit: Analytics, Statistics, Dashboards

*(printed pp. 223–255)*

## Core Idea

"The measurements toolkit aims to provide a simple and easy-to-understand approach to statistics
and the basic concepts of data analytics… To keep things simple, mathematics and theorems are
covered in as little depth as possible."

The target competence: "With this introduction it should be possible to handle concepts such as
**outcome, event, random variable, expectation, mean, variance, and probability distribution**."

The chapter's framing question sits at the centre of the data ecosystem model: **"asking the
right questions."** — "Having a powerful design thinking/innovation team in place is fundamental
to asking and answering the right questions."

## Frameworks Introduced

### The five key types of statistics and data analytics

| Type | The question it answers |
|---|---|
| **Descriptive** | "What happened?" |
| **Diagnostic** | "Why did this happen?" |
| **Dashboard** | "What is happening now?" |
| **Predictive** | "What will happen?" |
| **Prescriptive** | "What should be done?" |

### Spectrum of Applied Statistics & Analytics (low → high statistical and analytical sophistication)

| What happened? (Descriptive) | Why did this happen? (Diagnostic) | What is happening now? (Dashboard) | What will happen? (Predictive) | What should be done? (Prescriptive) |
|---|---|---|---|---|
| Descriptive statistics; data clustering; appropriate exploitation metrics (financial information, business performance, ratios) | Sensitivity analysis; design of experiment | Strategic dashboard; tactical dashboard; analytical dashboard; operational/static dashboard | Linear and logistic regression; neural networks; support vector machines | Simulation (e.g. **Monte Carlo**); optimization (e.g. linear/nonlinear programming) |

### The Data Ecosystem — asking the questions across EXPLORE/EXPLOIT

Axes: **internal focus ↔ external focus** × **EXPLORE ↔ EXPLOIT**, with *asking the questions* at
the centre.

| | EXPLORE | EXPLOIT |
|---|---|---|
| **Internal focus** | Reinvent the way of collaborating | Optimize operations, measurement systems, and processes |
| **External focus** | Create radical new and viable market opportunities | Improve and scale existing products, services, and experiences |

### The Data Analytics Process — two parts

**I. Data Analysis Process**
| Step | Activities |
|---|---|
| **1. Define the question** | Define and prioritize question · Where is the problem? · Estimate the size of opportunity |
| **2. Design experiment** | Brainstorm hypotheses · Interview users/experts/decision makers · Observe and understand |
| **3. Set up experiment** | Build, clean up, and merge data sets |
| **4. Conduct experiment** | Test data sets · Analyze, iterate, and re-define with the available sets of data |
| **5. Visualize and share data** | Present findings and align with decision makers and users of data · Create knowledge transfer |

**II. Operationalization**
| Step | Activities |
|---|---|
| **1. Implement** | Enable real-time analytics and establish meaningful metrics · Incorporate metrics in decision and review processes |
| **2. Manage change** | Define and implement change management procedures and communication plan |
| **3. Capture impact** | Implement findings (new or adjusted processes, tools, capabilities) · Measure impact on innovation, business success, customer satisfaction |
| *(and back)* | **ADAPTION** |

### Data sources — the three parties

| Type | Definition | Examples the book gives |
|---|---|---|
| **First-party** | Collected by the company itself | Transactional data; CRM information; customer satisfaction surveys; evidence from focus groups, direct observations, insights from exploration interviews |
| **Second-party** | Initial data from other companies, "either exchanged as part of a cooperation or acquired from a data marketplace. As a rule, such data is structured and reliable" | Websites, apps, social media activities such as online purchase histories or shipping data |
| **Third-party** | "Collected and aggregated from numerous sources"; often contains a large number of unstructured data points (big data) | Big-4 consulting companies, Gartner; government open-data initiatives such as **Open Data Singapore** |

### The Four Levels of Measurement

| Level | Definition | The book's innovation example | Analyzed how |
|---|---|---|---|
| **Nominal** | "Simply categorizes variables according to qualitative labels (or names). These labels and groupings don't have any order or hierarchy to them, nor do they convey any numerical value" | Variable "Kind of Innovation": process innovation, service innovation, product innovation | Descriptive statistic: frequency distribution and mode. Non-parametric statistical tests |
| **Ordinal** | "Also categorizes variables into labeled groups, and these categories have an order or hierarchy" | Variable "innovativeness": low / medium / high innovativeness. Also education level, seniority level | Non-parametric statistical tests |
| **Interval** | "A numerical scale which labels and orders variables, with a known, evenly spaced interval between each of the values" | **Insights collected by the teams** — "the difference between 10 and 20 insights is the same as the difference between, say, 50 and 60 insights." Also team confidence, IQ score, income ranges | — |
| **Ratio** | "Exactly the same as the interval scale, with one key difference: the ratio scale has what's known as a **true zero**" | **The absolute number of ideas** — "If there is no idea, it truly is 0 ideas — compared to the change of innovation culture (interval data), where a value of zero degrees doesn't mean there is no innovation culture; it simply means that it has not changed" | — |

Capability matrix: nominal = categories and labels variables. Ordinal adds *rank categories in
order*. Interval adds *known, equal intervals*. Ratio adds *a true or meaningful zero*.

> "The level of measurement is important because it **determines the type of statistical analysis
> that can be performed** … Certain statistical tests can only be performed if more precise
> measures have been used. Therefore, it is important to plan in advance how the data will be
> collected and measured."

### Measures of Association

| Method | When to use it |
|---|---|
| **Pearson's correlation coefficient** | Both variables measured on an interval/ratio (continuous) scale; measures the strength of the **linear** relationship |
| **Spearman rank-order correlation coefficient** | Strength of a **monotonic** (constant direction) relationship between two variables on an ordinal or rank scale; also when one variable is interval and the other ordinal |
| **Chi-square test** | Standard measure of association (contingency) between two variables; assesses the **significance** of the association — "not a measure of the **strength** of the association" |
| **Relative risk and odds ratio** | For categorical variables / cohort data; "a relative risk of 1 means that there is no association, while a relative risk greater than 1 indicates an association" |

Additional methods named: point-biserial correlations; Durbin-Watson tests for serial correlation.

> **"The only danger with all measured associations is the tendency to infer causality. Whenever
> one variable causes changes in another variable, there is an association. However, it does not
> always follow from the presence of an association that causality exists."**

Terminology note the book supplies: "While the terms *correlation* and *association* are often
used interchangeably, **correlation in the narrow sense refers to linear correlation and
association refers to any relationship between variables.**"

### Linear Regression Assumptions (all must be tested)
1. Relationship between the variables is sensible
2. Relationship between the variables is linear
3. Independent variables are not highly associated (**multicollinearity**) — *only relevant for
   multiple regression models*
4. No presence of extreme outliers exists
5. Residuals are random and normally distributed
6. Residuals are independent of one another (time series data)
7. Variance of residuals is fairly constant at different levels of the predicted variable
   (**homoskedasticity**)

**Residual-plot diagnostics the book prints:**
| What you see | What to do |
|---|---|
| Rugby-ball shape | Suggests the need for **more explanatory variables** |
| U shape (or n shape) | Suggests **non-linear relationship**; try a term with a power |
| Increasing variance (heteroskedasticity) | Suggests the need to **transform one of the variables**; try a logarithmic transformation |
| Residual plot with an extreme value | Try **removing the extreme value** |

### Forecasting vs Predictive Analytics — the book's distinction
| | Question | Scope |
|---|---|---|
| **Forecasting** | *"How many customers will visit the shop next month?"* | "Anticipating the behavior of **many** customers/users on a **longer** timeline" |
| **Predictive** | *"Will the top customer buy at one of the competitors?"* | "Anticipating the behavior of **one** customer/user on a **shorter** timeline" |

**Forecasting methods:**
| Method | Description |
|---|---|
| **Qualitative** | "Used when historical data is not available… subjective and based on the opinions and judgments of consumers and experts. Usually used to make medium- or long-term decisions" |
| **Quantitative** | "Predicting future data based on historical data… appropriate if numerical data from the past is available and it is reasonable to conclude that one of the characteristics in the data will persist" |
| **Average method** | "All future values are predicted to be equal to the average of the previous data" |
| **Naïve method** | "Allowing the previous month's actual values to be used as a projection for that period without making adjustments or attempting to identify causal factors. Used for economic, financial time-series, and **health metrics**" |
| **Drift method** | "Allowing the projections to increase or decrease over time, setting the amount of change over time (called **drift**) to the average change observed in the historical record" |

### Exponential Smoothing matched to the product life cycle
"Nine common data profiles can be predicted using common exponential smoothing techniques… ranging
from a constant data level to a more complex damped trend with a multiplicative seasonal
influence." Grid: **nonseasonal / additive seasonal / multiplicative seasonal** ×
**one-parameter (simple) / two-parameter (Holt) / three-parameter (Winters)**.
"A good first step is to **plot the data series to be forecast** and then choose the exponential
smoothing method that best fits the data."

| Life-cycle phase | Recommended approach |
|---|---|
| **Initial phase** (before meaningful market data is available) | Qualitative analyses and market experiments |
| **On market, rapidly gaining acceptance** | **Three-parameter** exponential smoothing (level, trend, seasonal components) |
| **Mature phase / sales stability** | **Two-parameter** exponential smoothing models (or econometric models) with level and seasonal components |
| **Declining market share** | **Three-parameter** methods become relevant again |

### The Sigmoid Curve (S-curve)
"The circle that's marked is the **inflection point**, the place where decisions are made which
will determine the future course of the cycle." Applied to "the business as a whole, a product, a
service, an experience, or even how an offering is marketed or sold."

> "The more quickly the team of decision makers identifies the approaching inflection point, the
> more prepared they will be to respond appropriately and put strategies in place to ensure the
> business follows the upward curve, and not the downward."

**The second-curve rule:** "To keep on growing, the successful growth and innovation teams must
develop a **second curve out of the first**. Here, it is strongly recommended that **the new curve
starts before the first one peaks, in a time when all the evidence is that there is no need for
change.**"
Why it is hard: "In many organizations, the transition from one curve to another is confusing,
because a culture of EXPLORE and radical innovation is facing a culture trimmed to efficiency and
scale."
The curve is drawn with `new technologies / radical` versus `existing technologies / incremental`.

### The Basic AI Canvas

| Field | Question |
|---|---|
| **Description of business problem or forecast problem to solve** | What is the problem the team wants to solve? |
| **Prediction** | What does the team need to know to make a decision? |
| **Judgment** | How are different outcomes and errors valued? |
| **Outcomes** | What are the team's key metrics for task success? |
| **Action** | What does the team want to achieve? What added value does it have for the customer? Can activities be derived from it? |
| **Training** | What data does the team need to train the predictive/prescriptive algorithm? |
| **Input** | What data must be available to run the predictive algorithm? |
| **Feedback** | How can the team use the outcomes to improve the algorithm? |

## Key Concepts

- **Descriptive vs inferential statistics.** "Descriptive statistics focus on describing the
  visible characteristics of a data set… inferential statistics focus on making predictions or
  generalizations about a larger data set, based on a sample."
- **Population and sample.** "A **population** is the entire group to draw data from… in
  statistics it can apply to any group from which information will be collected." A **sample** is
  a representative group of a larger population. **Note the book adds:** "A random sample is
  representative of a population. It will never be 100 percent accurate and, therefore, it is
  important to incorporate a **margin of error**."
- **Random sampling steps:** define the population · decide the sample size (*big sample size =
  more representative results*) · randomly select a sample (e.g. random number generator or
  defined algorithm) · analyze the data sample.
- **Hypothesis / null hypothesis.** "The hypothesis is a test of two concepts: (1) the **null
  hypothesis** that there is no significant effect or relation, and (2) the **alternative
  hypothesis** that there is a significant effect or relation. The null hypothesis is assumed
  unless evidence can be given with a high likelihood of being true… Likelihood is assessed as the
  probability that the test statistic is different from zero with a preset degree of confidence."
- **Confidence intervals.** "Confidence intervals are **not fixed targets**. The true value is
  fixed and the interval might end up around it." They estimate a population parameter (e.g. the
  mean) from sample data and give "not a single mean value, but a range of values… often expressed
  as a percentage."
- **Sampling approaches:** non-probability and probability sampling. "If the intention is to make
  the samples representative of the population, **probability sampling is recommended**… because
  it allows inferences to be made from the sample to the population."
- **The data-cleaning warning:** **"80 percent of data cleaning problems are caused by poor
  data-gathering techniques."** Cleaning "is time consuming and usually represents the largest
  work package for the team."

## Mental Models

- **Honest communication is the last step.** "It is also important to highlight gaps in the data
  or to point out findings that may be open to interpretation. **Communicating honestly is the
  most important part in this last step** of the initiated data analytics process." And: "it's
  very important that **all the evidence collected is presented, not just selected data points.**"
- **Selection methods do not replace human decision making.** "The process of multiple linear
  regression modeling can be assisted and automated by the use of selection methods, but in the
  end, this should not replace human decision making."
- **Less is more on dashboards.** "From time to time, companies experience true **dashboard
  fever**… it is indispensable to question at regular intervals whether the dashboards still serve
  their purpose and, if necessary, to **assign an expiration date to the metrics.**"

## Anti-patterns

- **Off-the-shelf strategic dashboards.** "Out-of-the-box innovation measurement tools and
  strategic dashboards provide mainly standard metrics and very often **vanity metrics to provide
  shiny readings of performance** on innovation activities" — e.g. `17 # of prototypes`,
  `8 # of innovations realized`.
- **Assuming operationalization is easy.** "From a distance, operationalization seems simple:
  bring artificial intelligence into production… and the magic happens by itself. In practice,
  however, operationalizing such models can be challenging."
- **Data-visualization mistakes named:** using incorrect chart types · presenting data in a
  misleading way · not plotting enough data · using different scaling or too much color · 3D
  charts ("3D charts might create false representations since our eyes perceive objects as closer
  and larger than they truly are" — use a column chart instead) · pie charts with unwelcome color
  and size combinations (sort the data in descending order instead) · shaky data comparisons
  ("not only erode credibility, but also lead to wrong measurements for future activities" — set
  the baseline to zero).
- **Indices that only predict predictable outcomes.** "Typically, an index provides a number based
  on a set of indicators and a formula. When useful, the index should serve as the basis for
  specific decisions by predicting observable outcomes. **However, if it only predicts predictable
  outcomes, it is better to omit it from the forecasting algorithm altogether.**"

## Reference Tables

### Typical activities during data cleaning
- **Bringing structure to the data** — correct typos and layout problems
- **Fill in gaps in data series** — complete missing important data
- **Clean up major errors, duplicates, and outliers** — especially when merging from numerous sources
- **Remove unwanted data points** — extract observations not relevant to the planned analysis

**5 tips for data cleaning (as printed):** Cleaning your data isn't rocket science · Bad data =
bad experiences · Good data = greater efficiency · Duplicate data can hurt the bottom line ·
Dirty data destroys trust.

### The four operationalization challenges
| Challenge | The book's description |
|---|---|
| **Variability of integration** | "Integrating data and scoring pipelines with the multitude of systems requires a lot of integration work that is time consuming and very technical" |
| **Model monitoring** | "The accuracy of the metrics and model predictions must be constantly monitored, and the models must be adjusted and retrained over time as the data changes" |
| **Ability to scale** | "Initial models often rely on a smaller subset of the total available data set. For example, in a churn model, the models may be developed on **less than 50 percent** of the available data, but in production, the models must be scaled to handle **100 percent**" |
| **Portability to other systems** | "Tools used in the data analytics phase are significantly different from production environments… requires porting the code to platforms and systems that were not considered during model development" |

### Common methods in data collection
Surveys and questionnaires · interviews · direct observation · focus groups · existing documents
and records. Sources named: statistical methods, surveys, polls, interviews, **Delphi technique**,
focus groups; financial reports, sales reports, government reports, mission/vision statement,
internet.

### Tools named
| Category | Tools |
|---|---|
| **Objectives / KPI dashboards** | Dashbox; DashThis; open-source (Grafana, Freeboard, Dashbuilder) |
| **Data collection** | Data management platforms (Salesforce, SAS, Xplenty; open-source Pimcore, D:Swarm) |
| **Data cleaning** | OpenRefine (open source); Python libraries / R packages e.g. Pandas; Data Ladder (professional) |
| **Data visualization (no coding)** | Google Charts; Tableau; Datawrapper; Infogram; Power BI / Qlik |
| **Python and R** | Plotly; Seaborn; Matplotlib |
| **Also named for visualization** | Tableau, Power BI, Qlik, DataStudio |

### Applied statistics, algorithms, and innovation use cases
| Algorithm | Use case the book gives |
|---|---|
| **Linear regression** | Understanding drivers for sales (prices of competitors, distribution, advertisement) to optimize price points and product price elasticities |
| **Logistic regression** | Classification tasks for binary outcomes based on product size, shape, and color |
| **Linear/quadratic discriminant analysis** | Predict churn rate or a sales lead's likelihood of closing |
| **Decision trees** | Decision tree for hiring new talents for design thinking teams; or understand critical functions and experiences most likely to drive purchase |
| **Naïve Bayes** | Analyze sentiment to assess product or service perception in the market; create classifiers to filter relevant data |
| **Support vector machine** | Predict how likely a potential customer/user is to click on an online ad |
| **Reinforcement learning** | Optimize pricing in real time for an online auction of a product with limited supply |
| **AdaBoost** | "Adaptive boosting is a low-cost way to classify images with lower accuracy than deep learning" |
| **Gradient-boosting trees** | Combine results from all decision trees to generate product demand forecasts, or predict pricing based on features |
| **Simple neural network** | Classify data or find relationships between variables in regression problems — e.g. predict whether registered users will pay a particular price |
| **K-means clustering** | Segment customers into groups by distinct persona characteristics, like age segments, to prevent churn |
| **Gaussian mixture model** | Segment customers to better assign marketing campaigns with **less-distinct** persona characteristics or product/service preferences |
| **Hierarchical clustering** | Splits or aggregates cluster data to inform product/service usage or development by grouping customers based on mentioned keywords in social media activities |
| **Recommender system** | Cluster behavior prediction to recommend other services or products based on preferences of other customers/users |
| **Deep learning (convolutional neural networks)** | Understand brand perception and usage through images; create joint marketing opportunities based on **logo detection** on social media images and videos |

### Dashboard design rules
- Key questions: *"What story should be told based on the data, measurements, and metrics, and
  based on the interests of the decision makers and the innovation teams using them?"* and
  *"Does the design hierarchy of the dashboard reflect these intentions?"*
- **The F-pattern** — "defined as the propensity for humans to consume web pages starting in the
  upper-left and moving to the bottom-right, is also observed for dashboards."
- Eye-tracking finding: "innovation teams and decision makers tend to spend **more time looking
  at the top portion** of a dashboard." The **fixation duration metric** counts "the total amount
  of time all team members and decision makers looked at a certain area."
- **Distributed attention:** "When big numbers are shown on a screen with another noticeable
  element, like a map, distributed attention is observed. Most of the team members and decision
  makers look at the big number **and** the map."
- Placement rule: "If it is known that a particular metric should be particularly eye catching,
  and a columnar dashboard has been created, then that metric **should not be placed on the right
  or bottom** of the dashboard."
- A design-thinking-process dashboard is arranged by phase: `OVERVIEW · EMPATHIZE · DEFINE (THE
  PROBLEM) · IDEATE · PROTOTYPE · TEST`, each with its own metrics band.

## Worked Example

### Descriptive statistics on 286 innovations — the book's fully worked data set

**Data set: 286 innovations.** Distribution by kind of innovation:

| Kind of innovation | Count |
|---|---|
| Process innovations | **130** |
| Service innovations | **91** |
| Product innovations | **39** |
| Business model innovations | **13** |
| Business ecosystem innovations | **13** |
| **Total** | **286** |

**Central tendency** (computed over the five category counts):
| Measure | Definition the book gives | Value |
|---|---|---|
| **Mean** | "The average value of all the data points" — total value of all the measurements divided by the number of values | **57.2** |
| **Median** | "The central or middle value in the data set" | **39** |
| **Mode** | "The value that appears most often in the data set" | **13** — "because it appears twice, which is more than any of the other data points" |

**Variability:**
| Measure | Definition | Value in this data set |
|---|---|---|
| **Minimum / maximum** | The extreme values | **13** and **130** |
| **Range** | "Measures the size of distribution value" | **130 − 13 = 117** |
| **Standard deviation** | "High standard deviation suggests that the values are [widely spread]" | — |
| **Kurtosis** | "Measures the tails of a given distribution containing extreme values (many outliers are indicated as high kurtosis)" | — |
| **Skewness** | "Measures the symmetry of data sets (long left-hand tail = negative skewness, long right-hand tails = positive skewness)" | — |

Distribution shapes and their mean/median/mode ordering: **positive skew** · **symmetrical
distribution** (mean = median = mode) · **negative skew**.

### The customer-experience diagnosis — a walk through the whole analysis process

The chapter carries one narrative example across all five steps:

1. **Define the question.** Leadership asks: *"Why is there no market success with the new
   product?"* The book notes the tempting glib answer — "no one applied design thinking to
   understand the real customer problem" — and rejects it: "this may not get to the heart of the
   problem and would **not be evidence based**. The big challenge is to understand the company and
   its aspirations (North Star) well enough to find the appropriate problem, and also to formulate
   it correctly."
2. **Design experiment.** Observation suggests "the sales process works very well, and customers
   can be excited about the product at first, but later **the customer experience leaves much to
   be wished for during installation**. Could this be the reason that the word of this experience
   has spread and is keeping other customers from buying the product?"
3. **Set up experiment.** Identify which first-, second-, or third-party data sources can test the
   hypotheses; then clean them.
4. **Conduct experiment.** The exploratory analysis overturns the hypothesis: "if it is found that
   the customers are more likely to talk badly about the **price** instead of experience, there
   might be a **correlation between price and customer satisfaction**, and perhaps the initial
   hypothesis presents less of a problem than the market price for the newly launched product."
5. **Visualize and share.** Present *all* the evidence, flag gaps, use dashboards and interactive
   visualizations. "Depending on what is communicated, a whole company, organization, or
   innovation team might decide to restructure, launch a high-risk product, or even redesign the
   entire innovation process and measurement system."

Diagnostic analysis in the same example "would help to establish correlations between the problem
(difficulty in attracting repeat customers) and the factors that might be responsible (e.g.
**methodological approach for innovating, project costs, speed of delivery, and quality of
personas**)." A separate diagnostic finding: "retail customers are churning faster than other
customers. This could indicate that the team or organization is losing customers because they lack
expertise in that area. **That would be a very useful finding!**"

### The innovation-performance hypothesis framework

The chapter reproduces a conceptual framework with four testable hypotheses:

```
Innovative activities  →  Innovative performance  →  Firm performance
  Product innovation        Product                    Product performance
  Process innovation        Process                    Market performance
  Market innovation         Market                     Finance performance
  Organizational innovation Organizational
```

| Hypothesis | Statement |
|---|---|
| **H1** | "The higher the level of innovation activities is, the greater the innovative performance is." |
| **H2** | "The greater the innovative performance is, the greater the production performance is." |
| **H3** | "The higher the level of innovative performance is, the greater the market performance improvement is." |
| **H4** | "The higher the level of innovative performance is, the greater the finance performance improvement is." |

The regression framing the chapter gives: "To predict future innovation success (an **output
variable**), you might compare the recent capabilities at companies and innovativeness (both
**input variables**) to determine how innovation success might happen in companies with a certain
set of capabilities."

## Key Takeaways

1. Route every analysis question to one of five types: descriptive, diagnostic, dashboard,
   predictive, prescriptive.
2. Fix the level of measurement (nominal / ordinal / interval / ratio) **before** collecting —
   it determines which tests are available.
3. Data cleaning is the largest work package; 80% of its problems come from poor gathering.
4. Association ≠ causation, always. Chi-square gives significance, not strength.
5. Test all seven linear-regression assumptions; read residual plots for the fix.
6. Match forecasting method to life-cycle phase; naïve method suits health metrics.
7. Watch the S-curve inflection and start the second curve *before* the first peaks.
8. Give dashboard metrics expiration dates; put the eye-catching metric top-left.

## Connects To

- [Chapter 7 — From Observations to Real Metrics](ch07-from-observations-to-metrics.md) — the IMS maturity ladder this toolkit powers
- [Chapter 9 — Defining the North Star Metric(s)](ch09-north-star-metrics.md) — S-curve inflection points
- [Chapter 10 — Working with Exploration Metrics](ch10-exploration-metrics.md) — the data-analytics process supporting MVMs
- [Chapter 11 — Selecting Exploitation Metrics](ch11-exploitation-metrics.md) — health metrics and the naïve forecasting method
- [Chapter 17 — Data-Driven Innovation: Analytics & the Hybrid Model](ch17-data-analytics-hybrid-model.md)
- [Chapter 18 — Design Thinking & Artificial Intelligence](ch18-design-thinking-and-ai.md) — the Basic AI Canvas taken further
