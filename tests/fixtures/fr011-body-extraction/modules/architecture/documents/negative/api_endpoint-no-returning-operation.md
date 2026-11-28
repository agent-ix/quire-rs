---
id: negative_001
title: "EndpointThatNeverAnswers"
type: api_endpoint
object: api_endpoint
expect: semantic.record-invalid
because: "ApiEndpoint.json requires at least one operation that declares a return"
---
# [negative_001] EndpointThatNeverAnswers

## Endpoint

`POST /artifacts/{artifact_id}/touch` marks an artifact as recently accessed
and answers with nothing at all.

## Operations

### touch_artifact

Mark the artifact as recently accessed; declares no return value, which is what
makes this record invalid.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| artifact_id | UUID | 1..1 | |
