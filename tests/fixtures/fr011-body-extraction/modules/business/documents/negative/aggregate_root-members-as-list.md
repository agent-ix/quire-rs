---
id: negative_014
title: "OrderWithMemberList"
type: aggregate_root
object: aggregate_root
expect: semantic.feature-not-extractable
because: "the Members section holds only the declared Member table; a bullet list there is a form the manifest does not declare"
---
# [negative_014] OrderWithMemberList

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| order_id | UUID | 1..1 | identity |
| status | OrderStatus | 1..1 | |
| lines | OrderLine | 0..* | |

## Invariants

### PlacedPaidShippedOrDeliveredOrderCarriesAtLeastOneLine

```quire
(self.status = OrderManagement::OrderStatus::Placed or self.status = OrderManagement::OrderStatus::Paid or self.status = OrderManagement::OrderStatus::Shipped or self.status = OrderManagement::OrderStatus::Delivered) implies size(self.lines) >= 1
```

## Members

- OrderLine 0..*
- Money 3..3
