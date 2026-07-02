# ADR 0007: Split Ability Gate From Commit Action

## Status

Accepted

## Context

Activation checks and commit consequences answer different questions. Activation checks decide whether active ability state may be created, while commit consequences apply caller-owned point-of-no-return mutations such as costs, cooldown starts, or resource changes.

## Decision

Flexweave exposes a synchronous activation gate for begin commands and a separate synchronous commit action for commit commands. The activation gate receives read-oriented context and returns allow or block. The commit action receives mutable context and runs transactionally with the commit transition. Both extension points use small traits as their named public contract and closure blanket implementations for lightweight call sites.

## Consequences

Core can emit deterministic `Attempted` and `Rejected` facts inside the begin command without accepting async hooks. Both explicit gate blocks and gate errors emit `Rejected`; the command result carries the block reason or caller-owned gate error. Commit consequences have one transactional boundary and do not run during begin, end, cancel, revocation, or rollback. Core exposes convenience begin entry points for activations with no caller-owned gate and separate commit entry points for no-op commits and action-backed commits, while preserving the existing event-suffix naming convention rather than renaming event methods around the commit-action split.
