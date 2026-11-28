---
id: positive_variant_set_001
title: "DiagnosticCause"
type: variant_set
object: variant_set
---
# [positive_variant_set_001] DiagnosticCause

A closed discriminated value: exactly one variant is selected and the payload
that variant requires travels with it.

## Variants

| Name | Payload | Description |
|---|---|---|
| revision_mismatch | value_object_001 | The expected and actual revisions differ |
| digest_mismatch | value_object_001 | The expected and actual digests differ |
| unavailable | | The comparison could not be made |
