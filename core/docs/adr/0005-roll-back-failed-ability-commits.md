# ADR 0005: Roll Back Failed Ability Commits

## Status

Accepted

## Context

Ability commit is the point-of-no-return transition for an active activation. A synchronous commit action can fail while applying caller-owned commit consequences, but leaving the activation live and uncommitted would force most consumers to perform the same cleanup after every failed commit.

## Decision

A failed ability commit action is terminal. If the commit action returns an error, Flexweave removes the active activation, emits `RolledBack`, emits no `Committed` fact, and returns the commit action error.

## Consequences

Consumers can treat `RolledBack` as the cleanup signal for failed activations, including failed commit transactions. Retryable checks should happen before calling the commit command; the commit action represents the activation crossing the point of no return.
