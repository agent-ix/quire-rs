---
id: quant_codec
title: "QuantCodec"
type: interface
object: interface
---
<!-- interface authoring skeleton (spec-objects-architecture). Fill every
     section with substantive content. Contract (manifest body_extraction
     asserts):
     - Frontmatter MUST carry id, title, type: interface, object: interface.
     - "## Operations" (H2): the AUTHORITY. One `### <name>` per operation
       with its parameter table and a `Returns:` line where it answers with a
       value.
     - "## Contract" (H2, required): MUST contain a fenced ```yaml code block
       holding the language-neutral operations contract. It is the derived,
       human-facing view of the same declarations; FR-006 fixes the mapping
       (name -> OperationDecl.name, inputs[] -> params[], output -> returns)
       and requires the two sections to agree on names, params, and returns.
     - Implementations are NOT enumerated here: each implementation is its
       own FR linked via an `implements` relationship edge.
     - Boundary: `interface` covers contracts WITHIN the system; a contract
       with an external system is an `external_contract`.
     - An interface's features are fields and operations (QSpec FR-152,
       FR-208), ordered as one sequence across both; Interface.json admits
       `fields` and `operations` and requires `featureOrder`.
     - Keep headings unique per level. -->
# [quant_codec] QuantCodec

## Contract

```yaml
name: QuantCodec
associated_types: [Bytes]
operations:
  - name: prepare_ip_query
    inputs: [query_vector, dimension]
    output: Bytes
    semantics: one-time query-side transform (the PreparedQuery form), amortized across candidates
  - name: score_ip_candidate
    inputs: [prepared_query, candidate_payload]
    output: Decimal(18,9)
    semantics: single-candidate approximate inner-product score; exact fallback allowed
  - name: score_ip_batch
    inputs: [prepared_query, candidate_payloads]
    output: Decimal(18,9)
    semantics: block-kernel entry point; one score per payload, same order
invariants:
  - payload_len is constant for a built index
  - batch and single-candidate scores agree within quantization error
dispatch: by codec_kind label recorded at index build
```

## Operations

The operations the QuantCodec declaration exposes. Each operation owns one
`### <name>` heading with its parameter table and a `Returns:` line where it
answers with a value. This section is the authority; the `## Contract` fence
above is its derived view.

### prepare_ip_query

Apply the one-time query-side transform, producing the prepared form every
candidate is then scored against.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| query_vector | Bytes | 1..1 | nonEmpty |
| dimension | Integer | 1..1 | min: 1 |

Returns: Bytes[1..1]

### score_ip_candidate

Score one candidate payload against the prepared query; an exact fallback is
permitted when the quantized path cannot answer.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| prepared_query | Bytes | 1..1 | nonEmpty |
| candidate_payload | Bytes | 1..1 | nonEmpty |

Returns: Decimal(18,9)[1..1]

### score_ip_batch

Score a block of candidate payloads in one call, answering one score per
payload in the order given.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| prepared_query | Bytes | 1..1 | nonEmpty |
| candidate_payloads | Bytes | 1..* ordered | |

Returns: Decimal(18,9)[1..* ordered]
