---
id: negative_015
title: "OrderManagementWithDuplicateTerm"
type: domain
object: domain
expect: semantic.duplicate-model-entry
because: "each vocabulary Term is declared once; Place is declared twice"
---
# [negative_015] OrderManagementWithDuplicateTerm

## Bounded Context

The OrderManagement context owns the lifecycle of a customer order from the
moment a cart is converted into an order until the order is shipped or
cancelled.

## Ubiquitous Language

| Term | Description |
|---|---|
| Place | Convert a draft order into a binding purchase request |
| Place | Submit a cart |
