# Hiding the Dust Under the Carpet

## Index
- [Have You Seen This Code?](#have-you-seen-this-code)
  - The hidden cognitive cost of apparently modularized code that appears readable but suffers from poor modularization
  - The pain of navigating multiple files and the Shotgun Surgery code smell
  - This pain is not accidental, it stems from poor decomposition and cognitive overload

- [Cognitive Foundations](#cognitive-foundations)
  - [Working Memory Limits](#working-memory-limits)
    - Miller's Law (7±2) and modern updates
    - Chunking as cognitive compression
    - Why large undivided problems exceed mental capacity
  - [Comprehension and Abstraction](#comprehension-and-abstraction)
    - Abstraction reduces cognitive load
    - Hierarchical thinking is natural to humans
    - Decomposition helps create stable mental models

- [Consequences of Poor Decomposition](#consequences-of-poor-decomposition)
  - [In Software](#in-software)
    - Harder debugging (ripple effects everywhere)
    - Low reusability and duplication of effort
    - Fragile systems that break with small changes
    - Team-level slowdown (coordination overhead)
  - [In Other Systems Case Studies](#in-other-systems-case-studies)
    - Engineering disasters (complexity mismanaged)
    - Organizational failures (departments too interdependent)
    - Historical cases where lack of modularization led to collapse

<!-- [Formalization in Software](#formalization-in-software)
  [Coupling and Cohesion](#coupling-and-cohesion)
    - Definitions of coupling and cohesion
    - Why high coupling is dangerous
    - Why high cohesion is desirable
  [Metrics and Attempts](#metrics-and-attempts)
    - Attempts to measure complexity (McCabe, Halstead, etc.)
    - Limitations of metrics
    - Still useful to guide decomposition -->

<!-- [Benefits of Decomposition](#benefits-of-decomposition)
  [Proven Advantages](#proven-advantages)
    - Easier comprehension
    - Improved maintainability
    - Faster onboarding of developers
    - Reduced error rates
  [Historical Context](#historical-context)
    - Decomposition as old as math (Euclid, Polya)
    - Systems theory and modular design
    - Dijkstra and structured programming

[Decomposition Done Wrong](#decomposition-done-wrong)
  [Over Decomposition](#over-decomposition)
    - Too many small pieces create complexity
    - More glue code than actual logic
  [Wrong-Boundaries](#wrong-boundaries)
    - Artificial splits that increase coupling
    - "Hiding in another file" ≠ decomposition

[What Good Decomposition Looks Like](#what-good-decomposition-looks-like)
  - Clear separation of responsibilities
  - Minimal and meaningful dependencies
  - Results of one part can feed another, but not interdependence
  - Real examples of clean decomposition

[Business Reality and The Myth of No Time](#business-reality-and-the-myth-of-no-time)
  - Common excuse: "we don't have time to decompose"
  - Reality: poor decomposition increases long-term costs
  - Decomposition as a skill (gets faster with practice)
  - Why time pressure should not excuse bad structure -->

## Have You Seen This Code?

Picture this scenario: you encounter the following code:

```ts
function someFunction(){
  let someComputedValue: number | null | undefined;
  /** 300 lines of code
   * and somewhere within, someComputedValue is reassigned
   **/
  if(someComputedValue)
   return someComputedValue * 10

  return 0;
}
```

A fellow reviewer would ask you to modularize this because the function is doing too much. The refactored code then looks like this:

```ts
function someFunction(){
  const computedValue = someCalculation();

  if(computedValue)
   return computedValue * 10

  return 0;
}

function someCalculation(){
  let someComputedValue: number | null | undefined;
  /** 300 lines of code
   * and somewhere within, someComputedValue is reassigned
   **/

  return someComputedValue
}
```

I would argue that this refactoring provides no benefit at all. The main reason is that we've simply split one large procedure into two files, but it's still fundamentally the same procedure, we've just hidden the mess in the closet when someone asked us to properly clean the room, to use an analogy.

Is this more readable? Easier to reason about? It depends on several factors, but I believe this approach is similar to adding fold markers. For example, in VS Code we could accomplish the same visual organization with:

```ts
function someFunction() {
  // #region someCalculation - this section can be collapsed any time
  let someComputedValue: number | null | undefined;
  /** 300 lines of code
   * and somewhere within, someComputedValue is reassigned
   **/
  // #endregion

  if(someComputedValue)
   return someComputedValue * 10

  return 0;
}
```

Or perhaps a more preferable equivalent would be:

```ts
function someFunction() {
  
  // #region someCalculation - this section can be collapsed any time
  const someComputedValue = someCalculation();
  // #endregion

  if(someComputedValue)
   return someComputedValue * 10

  return 0;

  function someCalculation(){
    /** 300 lines of code
    * and somewhere within, someComputedValue is reassigned
    **/
  }
}
```

Consider this additional concern: in JavaScript, each function is actually a specialized object. Having many functions that aren't reusable can bloat the heap. I initially thought this was negligible, but in some projects I've worked on, it actually caused performance problems on mid-tier devices. The only exception would be if a compiler or transpiler could unwrap and inline the function's content when creating a production bundle, using heuristics like inlining only when the function isn't reused in multiple places.

There's another problem: when you find yourself performing what we call "shotgun surgery" (when a specification change requires you to modify several files or modules), chances are you have tight coupling that will grow in complexity, making this separation non-reusable.

This is just the beginning of many other issues that can tax your cognitive load with hidden details like side effects and mutations.

The code "works" most of the time initially, and that's the worst part: it invites complacency. You can end up working late hours every time someone requests new features, and the application grows rapidly without control.

This experience isn't about subjective aesthetics. The discomfort you feel is a measurable cognitive phenomenon that predicts mistakes and maintenance costs. The rest of this article explains, through a single coherent argument, why that feeling is an epistemic red flag: the code resists local reasoning because its parts are not independent in any meaningful sense.

## Cognitive Foundations

### Working Memory Limits

Human reasoning is constrained by working memory limitations. George Miller's classical work introduced the "7 ± 2" principle about the number of chunks people can hold simultaneously in their minds. Later research suggests an even smaller practical capacity in many tasks (approximately 3–5 meaningful items).
<!-- Citation is needed here -->

Software that bundles many responsibilities into one contiguous interface forces developers to hold numerous unrelated elements in working memory simultaneously. This cognitive overload produces forgetting, incorrect assumptions, and fragile edits.

Decomposition is cognitively effective because it externalizes chunking: a well-formed module functions as a named chunk in a developer's mental model, reducing the mental load required to reason about the entire system.

### Comprehension and Abstraction

Abstraction and hierarchical organization are not mere stylistic choices; they reflect how humans naturally think about complexity. Herbert Simon's concept of "near-decomposability" demonstrates why hierarchical structure makes complex systems intelligible and evolvable: subsystems interact more strongly within themselves than with external components, enabling local reasoning and local adaptation.

<!-- Needs citation -->

When code exposes higher-level operations ("process order", "validate policy") instead of lengthy sequences of micro-operations, developers can reason at the appropriate semantic level. Abstraction stabilizes mental models and transforms local correctness arguments (unit tests, invariants) into meaningful guarantees that compose reliably. Without this abstraction, reasoning must be flat and global, which quickly becomes intractable.

## Consequences of Poor Decomposition

### In Software

When modules are not properly decomposed, the following predictable problems occur:

• **Increased debugging costs and ripple effects.** Changes in one area force coordinated edits in many other areas, increasing the likelihood of regression. Empirical tools that measure co-change patterns demonstrate that poor architectural boundaries cause substantially more co-editing activity—a measurable indicator of hidden coupling.

• **Duplication and low reusability.** When responsibility boundaries are unclear, code that should be reusable gets copied with minor variations, creating "data clumps" and repeated logic that multiplies maintenance work.
<!-- Recent automated refactoring research documents the prevalence and cost of these data-clump patterns and shows measurable gains when they are corrected. -->

• **Fragility and brittleness.** Implicit state, temporal coupling, and hidden invariants generate failure modes that are difficult to test. Complexity that isn't localized becomes an emergent robustness risk: small perturbations can have far-reaching, non-local effects.

• **Organizational slowdown.** Conway's Law observes that system architecture mirrors organizational communication patterns. Poor technical boundaries force cross-team coordination, reducing throughput and increasing managerial overhead.

### In Other Systems (Case Studies)

The same structural pathologies appear outside software development. Diane Vaughan's sociological analysis of the Challenger disaster explains how coupled technical and organizational assumptions led to the normalization of deviance and ultimately a catastrophic decision. The technical failure (the O-ring) was deeply entangled with schedule pressures, authority structures, and unrecognized cross-dependencies. This serves as a canonical example of how poor decomposition of responsibility and insufficient explicit contracts can lead to disaster.

In finance and engineering, tightly coupled structures (opaque derivative networks, monolithic product designs) have amplified local shocks into systemic failures. These cross-domain cases strengthen the causal argument: when systems are not structured into semi-independent units, local problems inevitably cascade throughout the entire system.
