# Document Flexweave public cost and determinism contracts

## Validation Verdict

Valid, with scope control.

Flexweave already documents deterministic order in several places, but it does not consistently document caller-visible cost contracts: which public calls allocate vectors, clone caller-owned payloads/tags, retain events, or linearly scan ordered storage.

This strengthens Flexweave if framed as public cost and determinism contracts. It would muddy the purpose if it becomes benchmark prose or locks in private implementation details beyond what callers need to rely on.

## Problem

Flexweave's deterministic ordered storage is an important design choice. Consumers need to know the cost implications when choosing hot-path APIs.

Today a caller has to inspect source code to answer:

- Does this allocate?
- Does this clone caller-owned generic payloads?
- Does this retain events?
- Does this scan linearly?
- What order does this preserve?
- Is there a streaming alternative?

## Evidence

- README and crate docs document deterministic mechanics primitives and deterministic iteration.
- `collect_where` documents creation order.
- `ObjectMap` is vector-backed, so public `DataStore` and `Attribute` behavior inherits linear put/contains/get/remove scans.
- `ObjectStore::exists` scans ids.
- `query::collect_where`, `AbilityStore::ids_with_tag`, `MechanicsDriver::tick`, and `SignalProjection` allocate output vectors.
- `MechanicsDriver::tick_with` streams, but this contrast is not elevated as a cost contract.
- Ability and effect event paths clone caller-owned tags/payloads.
- Signal projection clones definition fields, tags, signal payloads, and source payloads.
- `EventChannel` retained publication clones into a retained vector; `drain_retained` takes the vector.
- Tests prove deterministic order matters across objects, queries, signals, events, and mechanics.

## What Would Muddy Flexweave

Do not document unstable private details as hard guarantees.

Avoid benchmark claims unless benchmarked and maintained. Prefer stable public contracts:

- Allocates or streams.
- Clones or borrows.
- Retains or drops.
- Linear over what collection.
- Preserves which deterministic order.

## Proposed Scope

Add a focused reference page, for example:

`docs/reference/core-cost-and-determinism.md`

Link it from:

- `core/README.md`.
- Crate-level docs in `core/src/lib.rs`.
- Relevant module docs where appropriate.

Cover:

- `ObjectStore`.
- `DataStore`.
- `Attribute`.
- `DerivedAttribute`.
- `Tag` and `TagSet`.
- `AbilityStore`.
- `EffectPipeline`.
- `EventChannel`.
- `MechanicsDriver`.
- `Registry`.
- `SignalProjection`.

Suggested table columns:

- API or module.
- Allocation behavior.
- Clone behavior.
- Retention behavior.
- Lookup complexity shape.
- Deterministic order guarantee.
- Streaming/no-allocation alternative.

## Design Constraints

- State current behavior honestly without over-promising private implementation forever.
- Mark behaviors that are public guarantees separately from current implementation notes.
- Keep docs domain-neutral.
- Update docs if indexing or borrowed-event work changes cost profiles.

## Acceptance Criteria

- Public docs answer whether each major primitive allocates, clones, retains, scans, and what order it preserves.
- Docs distinguish guaranteed order from current implementation cost.
- Docs identify streaming alternatives such as `tick_with` where available.
- Retained event clone/allocation behavior is documented.
- Generic payload clone costs are documented for ability/effect/signal paths.
- Links from `core/README.md` and crate docs make the reference discoverable.

