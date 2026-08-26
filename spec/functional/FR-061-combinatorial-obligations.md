---
id: FR-061
title: "Combinatorial Obligations from Declared Configuration Dimensions"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-053"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
---
# FR-061: Combinatorial Obligations from Declared Configuration Dimensions

## Description

Config-space bugs hide in interactions no single-dimension test exercises, and *"we tested the
configurations"* is unquantifiable without a declared space. t-way coverage gives a defensible
number — but nothing lets a spec **declare** its configuration dimensions, so no obligation can
demand a strength over them.

A module SHALL be able to declare that an obligation source reads its table as a **configuration
space**, minting one obligation over the interaction of every row.

### A new source kind, not a new mechanism

[FR-053](./FR-053-obligation-record.md)'s `ObligationSource` is already a declaration-driven minter
carrying a `parameters` map whose own documentation names *"a t-way strength"*. The statement hash,
the suspect link and the parameter carriage are inherited unchanged.

**What differs is arity.** A t-way obligation is a statement about the interaction of *every* row,
so no single row can carry it — which is why this needs a declaration rather than another column.
The engine still knows no dimension, no value and no strength: a module says which column holds
each.

### What the number counts

For strength `t`, the obligation is over every **t-way value tuple**: for each set of `t` distinct
dimensions, the product of their value counts, summed over all such sets. Dimensions of sizes 2, 3
and 2 at `t = 2` give `2·3 + 2·2 + 3·2 = 16` pairs.

This is deliberately **not the size of a covering array**. The minimum array size is NP-hard to
compute and depends on the generator that produces it; the number of tuples to cover is a property
of the declared space alone — which is what an obligation must be able to restate from the spec at
any later time, without knowing what tool ran.

### Forbidden combinations are first-class

Real spaces have them: a feature unavailable on a target, a codec absent from a build. A covering
array over an unconstrained product demands combinations that **cannot exist**, so counting them
makes the target permanently unreachable — the fastest way to get a coverage number ignored.

An exclusion forbids every **wider** tuple containing it, not only the exact one. A two-value
constraint has to bite at strength 3 as well, or a space would become *less* constrained as strength
rises, which is backwards.

### Three ways to declare nothing, all rejected

Each of these reads as a declared configuration space and demands coverage of nothing, which is
worse than being absent because it looks answered:

- **Strength 0** — rejected at module load.
- **Fewer than two real dimensions** — mints no obligation. A dimension with one value is a
  constant, not an axis; it takes part in no interaction.
- **A strength above the dimension count** — counts 0 tuples rather than the full product. There is
  no 3-way interaction among two dimensions, and reporting one would put an obligation on the spec
  that no run could ever discharge.

### Why this is in quire and not quoin

ADR-0011 **invariant 2**: every capability either *produces* obligations (quire) or
*discharges and audits* them (quoin and the consumer). Minting "2-way over these dimensions" is
derivation from the spec corpus alone, which is what FR-053 owns. `agent-ix/quoin#90` as originally
filed put an obligation **producer** in quoin, which breaks the invariant on the first ticket that
tests it.

Computing achieved coverage from a run record, and the gap list, stay in `agent-ix/quoin#90`.
Generating or executing the combinations is invariant 1's line and belongs to consumer CI.

### Both minting paths, or the feature is unreachable

An obligation is minted by two functions: `obligation::for_document`, which
single-document validation calls, and `obligation::derive`, which the corpus
rollup behind `quire coverage` calls.

**The combinatorial branch shipped in only the first.** A module declaring a
configuration matrix therefore minted one obligation *per dimension row* — the
exact shape this source exists to replace — and the `coverage --json` contract
quoin reads never carried a combinatorial obligation at all. CR-076.

The two paths must agree on more than arity. They mint the same id, the same
`strength`/`dimensions`/`tuples` parameters and the same statement hash, because
a binding made while validating a document has to match the obligation the
rollup reports. TC-934 asserts that identity rather than just the count.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-061-AC-1 | The t-way tuple count is the sum, over every set of `t` dimensions, of the product of their value counts. | Test (TC-925) |
| FR-061-AC-2 | A strength of 0, or one exceeding the dimension count, yields 0 tuples rather than an error or the full product. | Test (TC-926) |
| FR-061-AC-3 | A declared forbidden combination is excluded from the count. | Test (TC-927) |
| FR-061-AC-4 | An exclusion forbids every wider tuple containing it, so a two-value constraint still bites at strength 3. | Test (TC-928) |
| FR-061-AC-5 | The hashed statement carries every declared value and the strength, so any change to the space — or to the strength — changes the hash and suspects every binding over it. | Test (TC-929) |
| FR-061-AC-6 | Declared cells parse as authored: backticks and whitespace tolerated, a repeated value counted once, a single-assignment "exclusion" rejected. | Test (TC-930) |
| FR-061-AC-7 | `obligation::for_document` mints **one** obligation for the whole table, with its id from `id_format` and `strength`/`dimensions`/`tuples` in `parameters`. | Test (TC-931) |
| FR-061-AC-8 | A space with fewer than two real dimensions mints nothing. | Test (TC-932) |
| FR-061-AC-9 | A combinatorial source declaring strength 0 fails at module load, naming the source. | Test (TC-933) |
| FR-061-AC-10 | The corpus path mints the same one obligation as the single-document path — same id, same parameters, same statement hash — so a binding made against one matches the other. | Test (TC-934) |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-061-CON-1 | The engine SHALL contain no dimension name, no value, and no default strength. Every one is read from the module's declaration and the document's table. | Design | Inspection |
| FR-061-CON-2 | quire SHALL NOT generate or execute combinations (ADR-0011 invariant 1). A covering-array skeleton is a Generator's job and is not scoped here. | Design | Inspection |
| FR-061-CON-3 | Computing **achieved** t-way coverage from a run record, and the gap list, stay in `agent-ix/quoin#90`. This FR mints the obligation; it does not discharge it. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-053](./FR-053-obligation-record.md) (the minter this extends), [FR-055](./FR-055-published-output-contract.md) (how quoin reads it)
- **Downstream**: `agent-ix/quoin#90` (the coverage view and gap list)
