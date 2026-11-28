# Contribution guide

We primarily review **anchors and decisions**. An anchor states a rule someone can rely on. A decision records a choice about the design. Write your PR so that someone can read those changes, understand their impact, and challenge them without having to read the implementation.

## Give the anchors care

Read the main sentences with the details collapsed. Could a newcomer use them to predict what happens in their workspace? The [anchor style guide](anchor-style.md) shows how we revised real examples and how to choose a case that distinguishes two possible rules. The [review guide](review.md) shows how to run the criteria.

## Submit a design change

If your PR adds, changes, or removes an anchor or a decision, **include a new design document in the PR**. One document can explain several related changes. Use the [template](design-documents/template.md), list the document under **Design documents** in `SUMMARY.md`, and link each affected decision back to it.

Lead the review with the changed anchor sentences and decisions. Use the design document to explain what prompted the change, why you chose that rule, and which alternatives you considered. As the review evolves, revise the proposal and the anchors together. If implementation reveals that an agreed rule needs to change, bring that choice back for review.

A change that leaves the anchors and decisions intact follows the bug-fix path. Point reviewers to the rule involved and show how they can observe the fix. The [design document for this process](design-documents/process.md) explains the boundary.
