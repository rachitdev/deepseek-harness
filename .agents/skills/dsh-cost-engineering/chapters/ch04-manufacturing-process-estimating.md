# Chapter 4 — Cost Estimating for Various Manufacturing Processes

## Core Idea

You cannot negotiate a cost you cannot build. Anyone in a company's cost engineering process needs at least rudimentary cost literacy in *every* basic manufacturing process, not just their own — an ECU assembly alone touches five (SMT, molding, stamping, plating, die casting). This chapter gives the main cost drivers and indicative rate ranges for thirteen processes.

The author's own caveat, stated twice: "the estimates shared are a rough guide only and do not necessarily represent every manufacturing scenario. There will be many outliers that will not fit within the general range of guidance numbers. **The main takeaway should be how cost is developed for each of the manufacturing processes.**" Estimators must build their own models on their own variables. (Recommended companion: *Realistic Cost Estimating for Manufacturing*, ed. Michael Lembersky, SME.)

## Frameworks Introduced

**The per-process cost driver table.** Every process is decomposed into the same rows — Raw Material, Scrap, Process step(s), Tooling — with a Description column (what physically happens and how many operators) and a Cost Basics column (dollar ranges and the unit the cost is expressed in). This is the reusable artifact: build one per process you buy.

**Process cost is per *mold*, not per part.** For casting, die casting and molding the rate is quoted per mold/shot; cost per part = cost per mold ÷ cavities. This makes **cavitation** and **pouring/cycle time** the two design levers that dominate process cost.

## Key Concepts by Process

- **Sand casting** — grey vs ductile iron differ by graphite content. Charge is ~50/50 pig iron and scrap metal, both exchange-traded (e.g. $300/ton pig + $400/ton scrap → ~$350/ton). Oxidation ("froth on the chicken soup") loses 2%–7%, use 5%. Induction furnaces of 30–60 tons run 24/7 to keep iron molten but do not produce 24/7 — that is a process characteristic, not supplier inefficiency, so the hourly rate must be built on real production hours (e.g. 20 hr/day × 6 days × 50 weeks = 6,000 hr/yr; $600,000/yr ÷ 6,000 = $100/hr). Ladle chemistry adjustment adds little cost. The automated DISAMATIC-type line makes sand molds from two pattern plates, inserts cores, pours, cools, shakes out, then de-gates, deburrs and sandblasts manually. System cost $5–10M, so **throughput is everything**: optimize design for maximum cavities per mold and shortest pouring time (the bottleneck).
- **Die casting** — molten Al or Mg forced under pressure; furnaces 20–40 tons, also 24/7. Presses 135–3,500 tons, usually one operator (sometimes shared across two presses) with a robot extracting parts. Drop-to-drop 35–65 s per mold; per-part = per-mold ÷ cavities. Gates and risers are re-melted, so the only loss is dross (oxidation).
- **Forging** — a billet hammered into die cavities with multiple hits; cold (small parts) or hot (large, heated billet). Losses from flash and surface oxidation can reach 30%. Account for secondary operations including inventory movement and storage.
- **Machining** — "probably the most complex of all manufacturing processes to cost estimate." Thousands of machine choices, and the choice is often driven by what is free on the floor rather than what is optimal. Cycle time comes from **speed and feed** (tool rotational speed; rate the tool is pushed through material), which depends on material, hardness, machine, tool and coolant. Speeds-and-feeds calculation is explicitly out of scope of the book. Cost the **gross weight less what is removed and sold as scrap**. Perishable tooling wears and is amortized into piece price.
- **Powder metal sintering** — four steps: mix and press the powder to shape, sinter (granules melt together in a furnace), size/trim to restore dimensions lost in sintering, then test. No offal because parts are pressed to shape; scrap comes from tool wear and wrong sintering temperature.
- **Stamping** — two families: **transfer die** (large parts, sheets loaded/unloaded across multiple presses by hand or robot) and **progressive die** (small parts, coil fed automatically through multi-station tooling). Offal is the dominant material issue — up to 70% of the working piece — so **nesting** is the key lever; scrapped material resells at 25%–80% of original value. Cycle times are very low (≤2 s) and process cost per part is small relative to material cost. Secondary ops: deburring, sand blasting, polishing, secondary bending on smaller offline presses, and welding into assemblies in booth cells.
- **Plating (electroplating)** — three methods: **barrel** (small parts tumbled in a rotating cage through chemical baths — no critical features survive), **rack** (larger parts hung on racks and dipped; most labor-intensive), **coil/reel-to-reel** (parts still attached to the strip run through small baths; allows vertical control of which section of the strip is plated). Thickness is controlled by current. A plating facility also needs ~$1M of water filtration and waste collection.
- **Molding** — the most common process worldwide; variants: injection, blow, compression, insert/over-molding. One step: resin pressed under heat and pressure between two tool halves, cooled, ejected to a bucket or removed by robot. Insert/over-molding needs a robot or operator at every press to load inserts. Glass fill (e.g. GF30 = 30% glass fibers) improves durability and temperature resistance and raises price. Gates and runners are 10%–60% of shot weight and are the main material loss unless reground (otherwise resale at ~10% of original value).
- **Painting** — manual spray booths or automated lines with robot nozzles and curing ovens. Decorative parts (car radio/HVAC bezels) may take several coats plus pad printing and laser etching. **The scrap driver**: 5%–30% cumulative, plus 10%–30% additional material for overspray, which must be water-collected, compressed into solid blocks and landfilled. "Although painting is often considered an afterthought in manufacturing, this can be a very difficult process and usually requires specialized experts to avoid high scrap rates."
- **Surface mount technology (SMT)** — stencil solder paste onto PCB panels → pick-and-place robots populate components → reflow oven → camera inspection. Post-SMT: de-paneling (manual press) and in-circuit testing (ICT, $50–150k machine, one operator, cycle time driven by circuit/component count). **PCB panel utilization** is a first-order cost driver: the more PCBs per standard panel, the cheaper each one.
- **Electronic component manufacturing** — active (microprocessors, MOSFETs, power supplies) vs passive (resistors, capacitors, inductors). Should-costing is **not** practised here; pricing is set by historical data and price/volume curves, and actual cost is largely irrelevant to purchasing decisions. IC Knowledge LLC's IC costing software can establish base pricing for negotiation.
- **Custom assembly** — inherently non-transferable (a brake pad line cannot assemble phones), with wide variation in automation level driven by local labor cost, capital restrictions, volume and design. Low-volume assemblies are usually manual because speed does not matter. Range: thousands to millions of dollars of investment, two to hundreds of steps, seconds to hours of assembly time. Estimating advice: **see as many assembly processes in person as possible**; failing that, design the process with simulated motion studies and historical data — but prior onsite experience is still needed for a decent starting point.
- **Additive manufacturing (3D printing)** — historically slow and limited to prototypes and tooling, but advancing fast (Carbon's light-and-oxygen resin process is producing end-use parts in the millions). The author expects another 5–10 years before full-material-spectrum, high-volume use, and possibly the eventual replacement of all the processes above — with less capital, energy and labor, and no tooling, so redesign becomes a CAD change. **Cost implications are explicitly not discussed** because the technology is moving too fast.

## Mental Models

- **Two levers dominate cavity-based processes**: cavities per mold and cycle time per mold. Everything else is second order.
- **Material loss has three distinct forms**: oxidation/dross (casting, 3%–7%), flash/offal (forging up to 30%, stamping up to 70%), and gates/runners (molding 10%–60%). Each has a different recovery value.
- **Furnace-type equipment must run when it is not producing.** Build hourly rates on actual production hours, not calendar hours, and do not treat the idle time as supplier inefficiency.
- **Weight-based pricing rules break on cavitation.** See the anecdote below — the same $/lb target is achievable on small parts (many cavities) and impossible on large ones (one or two).

## Anti-patterns

- **Pricing castings by $/lb regardless of design** (the chapter's Real-Life Anecdote — see Worked Example).
- **Assuming furnace idle time is inefficiency to be negotiated away.**
- **Costing machining on net weight**, ignoring the material removed (which also has resale value).
- **Ignoring nesting in stamping** — the difference between 10% and 70% offal is a design/layout decision, not a supplier margin.
- **Treating painting scrap as a single percentage** — it is cumulative and overspray adds 10%–30% more material.
- **Ignoring PCB panel utilization** when costing an SMT board.

## Reference Tables

**Table 4.1 — Sand casting cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Pig iron + steel scrap; sand | ~$0.15–$0.25/lb delivered to foundry (50/50 pig/scrap + special chemicals); sand ~$0.01/lb of iron poured (mostly recycled, but recycling and over-baking losses cost) |
| Scrap | ~5% oxidation loss, everything else re-melted; process scrap 2%–5% | ~5% of raw material cost; ~2%–5% of process cost |
| Core making | Molding press with 1 operator or robot; secondary coating with adhesive | ~$0.40 (brake caliper) – $1.50 (brake rotor) per core |
| Melting | Gas/electric furnace (~$0.5–1.0M), 2 operators, several tons/hr; ladle system | ~$0.05–$0.10/lb |
| Casting | DISAMATIC-type system (~$5–10M), 4–10 operators; 40–60 s cycle per mold | ~$2.50–$6.00 per mold; per part = per mold ÷ cavities |
| Tooling | Part-specific patterns for molding and core making | ~$100–$300k |

**Table 4.2 — Die casting cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Pre-melted alloyed ingots + internal scrap | ~$0.50–$1.50/lb Al; ~$3.00–$4.50/lb Mg |
| Scrap | 3%–5% oxidation (dross); process scrap 2%–3% | ~3%–5% of raw material; ~2%–5% of process cost |
| Melting | Gas/electric furnace (~$0.5–1M), 1 operator per 2–3 furnaces; ladle moved by Hilo | ~$0.05/lb |
| Casting | 135–3,500-ton presses ($350–$2,500k), 1 operator per 1–2 presses; 35–65 s per mold | ~$1.50–$4.50 per mold; per part = per mold ÷ cavities |
| Tooling | Part-specific dies | ~$100–$300k |

**Table 4.3 — Forging cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Pre-melted alloyed ingots formed into billets | ~$0.30–$1.00/lb steel |
| Scrap | Flash, surface oxidation and other losses up to 30%; process scrap 3%–5% | ~20%–30% of raw material cost; ~3%–5% of raw material and process cost |
| Forging | 100–16,500-ton presses ($350–$4,500k), 1 operator per press; 3–25 s per part | ~$50–$900/hr depending on press size |
| Tooling | Part-specific dies | ~$100–$300k |

**Table 4.4 — Machining cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Almost any material (iron, steel, Al, plastic); no other material consumed | Depends on material; cost gross weight less removed material (sold as scrap) |
| Scrap | Removed material recycled and resold; parts scrapped from tool wear/breakage | ~2%–3% of raw material and process cost |
| Machining | CNCs, transfer lines of multiple CNCs, machining centers; one or more spindles | Depends on machine type and cycle time from speeds and feeds (material, hardness, surface finish) |
| Tooling | Perishable diamond-tipped or other tools that wear | Highly dependent on tool count; usually amortized into piece price |

**Table 4.5 — Powder metal sintering cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Metal granules 1–3 mm diameter | ~$0.60–$1.50/lb |
| Scrap | No offal (pressed to shape); parts scrapped from tool wear or wrong sintering temperature | ~2%–3% of raw material and process cost |
| Pressing | 40–200-ton presses (~$500–$1,500k), usually single-cavity tool, 1 operator per 1–2 presses, 5–7 s/part | ~$0.10–$0.50 per part |
| Sintering | Large heating furnace (~$1M), 1–2 operators loading/unloading, ~5 s/part | ~$0.05 per part |
| Sizing | Small press (~$100–$300k), 1 operator | ~$0.05 per part |
| Tooling | One-cavity tool | ~$20–$60k |

**Table 4.6 — Stamping cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Pre-formed sheets or coils; thickness limited to ~5 mm for cold forming | ~$0.25–$0.50/lb steel; ~$0.50–$1.50/lb Al; ~$2.50–$4.50/lb copper and copper alloys |
| Scrap | Part scrap low; offal up to 70% depending on shape — nesting is key; all scrap resells at 25%–80% of original value | ~2%–3% of raw material and process cost; ~10%–70% offal (gross vs net weight) |
| Stamping | 20–1,500-ton presses (~$40k–$2M), 1–2 operators or robots; cycle times usually ≤2 s; multi-cavity tools possible | ~$0.01–$0.30 per part; process cost per part is very low relative to material |
| Tooling | Single or multiple cavity | ~$40–$200k |

**Table 4.7 — Plating cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Zinc, nickel, silver, gold, chrome bases | ~$0.75–$1.50/lb zinc; ~$5.00–$7.50/lb nickel; ~$1.00–$1.50/lb chrome; ~$15–$20/oz silver; ~$1,000–$2,000/oz gold. Cost per part depends on plating material, surface area and thickness |
| Scrap | No plating-material loss (electroplating is accurate); scrap from process issues and changeovers | ~2%–3% of raw material and process cost |
| Plating | Several baths incl. pre-cleaning; equipment $0.5–$3M with 1–4 operators (highest on rack plating, manual hang/un-hang); plus ~$1M water filtration and waste collection; cycle 0.5 s (coil) to 15 s (rack) per part; rack plating depends on rack size | ~$0.005–$0.50 per part |
| Tooling | Minimal — racks | ~$10–$20k for racks |

**Table 4.8 — Molding cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Thousands of resins and grades; additives tune flexibility, hardness, endurance, durability, temperature; glass fiber (e.g. GF30) most common; delivered as granules in bags, on pallets or into silos | ~$1.50–$2.50/lb depending on resin and quantity; prices rise sharply for small annual or per-shipment quantities |
| Scrap | Parts mold to net shape; resin solidified in delivery channels must be scrapped — 10%–60% of part finished weight; often reground and reused, otherwise resells at ~10%; bad-part scrap only 0.5%–2% | ~0.5%–2% of raw material and process cost; ~10%–60% of shot weight (gates and risers) |
| Molding | 40–600-ton presses ($100k–$1M); simple drop-to-bucket molding lets 1 operator run 4–6 presses; insert/over-molding needs an operator or robot per press; 15–65 s per mold | ~$0.05–$1.00 per part; per part = per mold ÷ cavities |
| Tooling | Highly design-dependent | ~$40–$200k per tool |

**Table 4.9 — Painting cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Thousands of colors; base and top coats | ~$40–$60/gallon base coat; ~$60–$150/gallon paint; ~$70/gallon top coat. Cost per part from surface area, thickness, cost per gallon |
| Scrap | Very significant; visual requirements drive multiple applications and laser etching, each generating rejects; overspray must be water-collected and landfilled | ~5%–30% of material and process, calculated **cumulatively** (at step 2, scrap covers only steps 1 and 2); overspray adds 10%–30% additional material per step |
| Painting | Manual single booth (~$500k, one sprayer) up to a fully enclosed automated line (~$1.5–3M) with multiple booths, robot nozzles and curing ovens; operators still hang/un-hang parts (2–8 depending on design and line speed); 15–60 s per part | ~$30–$200/hr or $0.02–$2.00 per part; ~$0.02 per pad print; ~$0.02–$0.10 per laser etch |
| Tooling | Fixtures to hold parts during painting | ~$20–$100k for all fixtures |

**Table 4.10 — SMT cost drivers**

| Driver | Description | Cost basics |
|---|---|---|
| Raw material | Mostly purchased components (micros, resistors, capacitors); the only true raw material is solder paste. **PCB panel utilization** is a key driver — more PCBs per standard panel means cheaper PCBs | ~$0.01 per square inch solder paste; ~$0.05–$0.12 per square inch PCB depending on board type (FR4, flex), copper content, through-holes and panel utilization |
| Scrap | Low rate, but high absolute cost because components are expensive; unused panel area is scrapped | ~0.5%–2% of material and process; ~5%–20% of PCB panel |
| SMT | Almost fully automated; line cost $1.5–$2.5M depending on number of placement machines; 2–3 operators mainly replacing tape reels; cycle time from component count, sizes and number of placement machines — each placement ≤0.5 s, so a 200-component board takes 15–30 s | ~$0.005–$0.02 per component placement |
| Tooling | Minimal — equipment handles standard panels; fixtures only for post-SMT operations | ~$20–$60k for all fixtures |

## Worked Example

**The $/lb casting rule that emptied the quote book** (chapter's Real-Life Anecdote, sand casting).

A casting buyer could not follow the trend in supplier pricing, so he imposed a rule on his suppliers: price = weight × a target $/lb, regardless of design. The target was set at roughly the average of existing pricing.

What that rule ignores is the physics of Table 4.1. Casting process cost is quoted **per mold** (~$2.50–$6.00), and cost per part is that figure divided by the number of cavities. Small parts fit many cavities per pattern, so their processing cost per piece is low; large parts fit only two cavities, sometimes one, so their processing cost per piece is high. A single $/lb target therefore has completely different meaning at the two ends of the size range.

Consequences, exactly as the book records them:
- Small castings comfortably met the $/lb target — and because the target was an average, their prices **drifted upward toward the average over time**.
- Large castings could not meet it at any design — and the buyer **could not find anyone to quote them**.

The fix, developed by the company's casting cost estimator, was a simple two-driver pricing model that split the estimate:
- **Material piece cost** driven by weight (Table 4.1: ~$0.15–$0.25/lb for the 50/50 pig-iron/scrap charge, plus ~5% oxidation loss and ~$0.01/lb of sand),
- **Processing piece cost** driven by design → cavitation and pouring time (mold rate ÷ cavities per mold).

That model "reflected the suppliers' costing realities" and restored quotability on large parts. The transferable lesson: a cost rule with one driver can only be right for a product family with one design geometry; the moment cavitation varies, the rule must have two drivers.

## Key Takeaways

1. Learn how cost is *developed* for each process; treat every dollar range in this chapter as an order-of-magnitude sanity check, not a quote.
2. For casting, die casting and molding, cost per part = cost per mold ÷ cavities — so cavitation and cycle time per mold are the design levers.
3. Distinguish the three material-loss modes and their recovery values: oxidation/dross, flash/offal, gates/runners.
4. Build furnace and other always-on equipment rates on real production hours.
5. Stamping is material-dominated (offal up to 70%, nesting is the lever); molding process cost is press-size and cavitation driven; painting is scrap-dominated and cumulative; SMT is component-count and panel-utilization driven.
6. Machining resists standard rates — cost it via speeds and feeds and gross-minus-removed weight.
7. Custom assembly can only be estimated well by people who have watched assembly lines in person.
8. Electronic components are priced on volume curves, not should-cost; additive manufacturing costing is deliberately out of scope.

## Connects To

- [ch02-cost-estimating-basics.md](ch02-cost-estimating-basics.md) — the bucket structure these drivers feed, and gross-vs-net weight
- [ch03-cost-based-negotiation.md](ch03-cost-based-negotiation.md) — using cavitation, cycle time, nesting and scrap data as negotiation evidence
- [ch05-cost-allocation-methods.md](ch05-cost-allocation-methods.md) — turning these process facts into defensible machine rates
- [ch06-cost-optimization-tools.md](ch06-cost-optimization-tools.md) — the benchmarking and functional-analysis moves that change these drivers
