# Git registries

A Git registry has the same plugin layout as a path registry. Its additional responsibility is obtaining and refreshing the repository.

> **`A-git-registry`** (draft): **A Git registry lists plugins from its cached checkout**
> Property: Refresh obtains the repository in Symposium’s cache. Listing delegates to the path manager over that checkout. Before the checkout exists, the registry reports that it cannot be read.
> Authority: `R-ecosystems`
> - Criterion `unfetched` [test]: List a Git registry whose checkout is missing and observe an unreadable-registry result.
