# Refactoring Choices

Use an example only after confirming the underlying change problem. Preserve
behavior and compare callers and analysis with the same settings; no pattern
promises a particular score improvement.

| Observed problem | Candidate change | Check before choosing it |
|------------------|------------------|--------------------------|
| Distant modules share implementation details | Move related responsibilities closer or narrow the interface | Does the boundary reflect independent reasons to change? |
| A module wires many dependencies | Split distinct responsibilities if present | Is it an entrypoint whose fan-out is expected? |
| Callers depend on mutable representation | Expose the operation callers need | Does it preserve validation and error handling? |
| Many callers share a changing dependency | Stabilize the operation they depend on | Is the hub actually volatile? |
| Similar logic occurs in two modules | Share the rule when its meaning and ownership align | Are differences intentional or likely to evolve independently? |

## Encapsulating a Mutation

Before, callers choose how to mutate the representation:

```rust
pub struct Queue {
    pub entries: Vec<String>,
}

pub fn enqueue(queue: &mut Queue, value: String) {
    queue.entries.push(value);
}
```

An operation can keep that choice local:

```rust
pub struct Queue {
    entries: Vec<String>,
}

impl Queue {
    pub fn enqueue(&mut self, value: String) {
        self.entries.push(value);
    }
}
```

This is a design sketch. Check construction, existing external callers, and any
invariants before narrowing visibility. A trait or facade is useful only when it
expresses a meaningful boundary; it is not required for this example.
