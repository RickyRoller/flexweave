# ADR 0003: Keep Ability Core Synchronous

## Status

Accepted

## Context

Ability activation can involve animation waits, input waits, targeting, network authority, resource mutation, cooldowns, and other caller-owned runtime behavior. Flexweave still needs deterministic lifecycle state and facts, but awaiting arbitrary runtime hooks inside core makes lifecycle transitions depend on executor behavior and blurs whether lifecycle events are facts or negotiation points.

## Decision

Flexweave ability core remains synchronous. Core owns deterministic activation ids, active ability state, lifecycle commands, and lifecycle facts. Caller-owned asynchronous ability behavior lives in runtime adapters keyed by `AbilityActivationId`. Core activation may consult a synchronous activation gate, and commit may run a synchronous commit action transactionally with the Flexweave commit transition, but start, end, cancel, waits, tasks, animation, input, networking, resources, costs, and cooldown semantics are adapter responsibilities. Core does not provide instant activation helpers; runtimes compose begin, commit, end, cancel, and rollback commands into their own convenience flows. Cancellation remains an infallible primitive removal command and does not accept a caller participant.

## Consequences

`Canceled` becomes a primitive cancellation fact: active ability state was removed by an explicit cancellation command. It no longer means caller-owned cancel behavior ran. Revocation remains deterministic owner cleanup and removes active activations whether or not they are committed. ADR 0001 remains valid for distinguishing cancellation, revocation, and rollback cleanup facts, but its hook-backed cancellation wording is superseded by this boundary.
