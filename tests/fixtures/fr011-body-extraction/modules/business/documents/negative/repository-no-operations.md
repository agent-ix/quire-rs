---
id: negative_005
title: "OrderRepositoryWithoutOperations"
type: repository
object: repository
expect: semantic.record-invalid
because: "Repository.json requires at least one operation; a prose-only Operations section yields an empty array"
---
# [negative_005] OrderRepositoryWithoutOperations

## Operations

This repository describes its operations in prose and declares none, so the
extracted `operations` array is empty.
