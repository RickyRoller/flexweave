# ADR 0002: Remove Attribute Policy Definitions

## Status

Accepted

## Context

Attribute policy definitions exposed metadata that looked like runtime mutation
behavior without being bound to the mutation path. Actual mutation behavior is
owned by caller-registered `AttributeMutationHooks`, so a policy definition type
created a second apparent source of truth without a runtime role.

## Decision

Flexweave removes `AttributePolicyDefinition` instead of keeping an inert
authoring surface. Attribute mutation behavior stays in `AttributeMutationHooks`,
and attribute channel definitions do not declare policy payload schemas.

## Consequences

- Callers keep runtime mutation behavior explicit in hooks.
- Flexweave avoids validating attribute policy metadata that has no runtime
  binding.
- A future attribute policy surface should include an executable runtime path or
  stay outside core.
