---
title: Trace QSL declarations
type: how-to
description: Connect a QSL declaration to the requirement it formalizes.
---

Add a `formalizes` comment above a QSL declaration to record which requirement it represents.

## Before you start

Install Quire CLI 0.53.5 and use a traceability module whose trace model recognizes `formalizes` comments. The source file can use `.qsl` or `.native` with the QSL language.

## Steps

1. **Place the comment directly above the declaration.** List one or more requirement identifiers after the marker.

   ```qsl
   // formalizes: FR-001
   pre P1 using v on M::Server::request { }
   ```

2. **Include the source in the trace query.** The resulting trace relation identifies the declaration as QSL formalization evidence.

   ```sh
   quire trace --scope . --module /path/to/traceability-module \
     --file specs/server.qsl --format tsv
   ```

The `formalizes` relation is distinct from source-code `implements` claims and test `verifies` claims.

## Result

Trace output records a formalization claim for the declaration and the requirement identifiers in the comment.

## Next steps

- [Trace a requirement to source](https://github.com/agent-ix/quire-cli/blob/main/docs/how-to-trace-requirements.md)
