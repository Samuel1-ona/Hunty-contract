# ADR 003: Answer Hashing Scheme

## Status

Accepted

## Context

Clue answers must remain secret on-chain while allowing trustless verification.

## Decision

Use **SHA-256** over a normalized plain-text answer:

1. Trim leading/trailing ASCII whitespace
2. Lowercase ASCII letters
3. Hash with `env.crypto().sha256`

Plain-text answers are never stored or returned by `get_clue` / `list_clues`. Incorrect submissions emit `AnswerIncorrect` events for analytics.

## Consequences

- Creators cannot recover answers from chain data (must store off-chain backup)
- Case and whitespace insensitive matching improves UX
- Unicode normalization is not applied (ASCII hunts only)

## Alternatives Considered

- **Keccak256**: equally viable; SHA-256 chosen for SDK native support and familiarity
- **Plain-text storage**: rejected — leaks answers to all players
