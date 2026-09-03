# AXIOM Constitution

These are non-negotiable rules for the AXIOM language. They may be amended only by explicit decision of the project maintainers with documented rationale.

## Rule 1: AXIOM Is General-Purpose

AXIOM must remain capable of expressing any computation. No domain-specific restrictions may be added that prevent AXIOM from being used for a legitimate general-purpose task.

## Rule 2: The Type System Is Sound

If a program type-checks, it must not exhibit type errors at runtime. Type soundness is a hard constraint. "Mostly type-safe" is not acceptable.

## Rule 3: No Hidden Control Flow

Exceptions, implicit goroutine creation, hidden allocation, implicit type coercion across boundaries — these are forbidden. If something happens, it is visible in the source code or the type signature.

## Rule 4: No Mandatory Runtime Cost

Features that impose runtime cost must be opt-in, not opt-out. If a programmer does not use a feature, they should not pay for it at runtime.

## Rule 5: No Paid Dependencies

The core AXIOM toolchain (compiler, runtime, standard library, package manager) must not depend on any paid service, API, or proprietary software. Free and open-source dependencies only.

## Rule 6: Compilation Must Be Deterministic

Given the same source code and the same compiler version, the output must be identical. No randomness, no timestamps, no environment-dependent behavior in compilation.

## Rule 7: Errors Must Be Useful

Compiler errors must include source locations, clear explanations, and, when possible, suggestions for fixes. "Error on line 42" is not acceptable.

## Rule 8: The Language Must Be Bootable

AXIOM must eventually be able to compile itself. The bootstrap path must be planned from the beginning, even if it is not implemented in the MVP.

## Rule 9: No Backwards-Compatible Breaking Changes After 1.0

After version 1.0, the language may not break backwards compatibility. Additions are permitted. Removals and changes to semantics are forbidden.

## Rule 10: The Specification Is Authoritative

The language is defined by its specification, not by its implementation. If the implementation disagrees with the specification, the implementation is wrong.
