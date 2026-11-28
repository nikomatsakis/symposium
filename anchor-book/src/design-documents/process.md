# When a change needs a design document

## Summary

A PR that changes an anchor or a decision includes a new design document explaining why. Everything else is a bug fix. This document records the [decision that establishes that rule](../authority.md#D-design-documents).

## Motivation

A reviewer can read an anchor to learn which rule Symposium follows. Months later, someone considering a different rule also needs to know what prompted the choice and which alternatives were considered. A design document keeps those answers in the book after the review conversation ends.

## Design

Create one document for the related changes in a PR and revise it as the proposal evolves. A later PR that changes those anchors or decisions gets its own document, so readers can trace why the rule changed again. Its **Summary** names the choice, **Motivation** gives the concrete case, **Design** links the decisions and affected anchors, and **FAQ** preserves the alternatives and rationale. Each affected decision links back to the document.

This change establishes a contributor decision. It changes how we review Symposium and leaves all runtime anchors as they were.

## FAQ

**What if a bug fix turns out to need a different rule?** Fixing the implementation to satisfy an existing anchor is a bug fix, even if it takes substantial work. Once the proposed fix requires editing the anchor or a decision, add a new design document to the PR and review the change to the rule.

**Does every anchor need its own document?** A single choice may change several anchors across chapters. Reviewing them together lets the reader see what that choice means throughout the system.

**Why keep the document after the PR merges?** Someone following a decision can find the motivation and alternatives directly from the book. A PR or discussion thread can supply further detail.
