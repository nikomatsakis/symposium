# Bootstrapping the design anchors

## Summary

The first Symposium anchors were bootstrapped from the existing code and Niko Matsakis’s understanding of how Symposium worked. They were meant to describe behavior as implemented. Whether that behavior was intended, or was the right design to keep, remained open.

## Motivation

Niko wanted to review the design without having to read the code. That required a starting point that a reader could challenge by running Symposium and observing what happened.

## Design

We took revision `ab351035` as the baseline. Niko identified the concepts and interactions worth pinning down. The implementation and its tests supplied the concrete behavior for the [initial anchors](../design.md). The [test audit](../review/test-audit.md) records places where the tests exercised more than those anchors said.

## FAQ

**Does a bootstrap anchor mean we would choose that behavior again?** No. It tells us what we understood the implementation to do at the baseline. If an anchor captured an accident or misunderstood the code, a later design document can identify it and explain the correction.
