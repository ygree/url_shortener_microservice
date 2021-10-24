# Ideas

In order to work around async and ownership limitations in Rust need to leverage `#derive[(Clone)]` and `svc.clone()`.
Typically, in within the future we need to use a cloned service to be able to close over it making sure it's bind to the current future life-time.

There is a nice `BoxFuture` type alias to remove some boilerplate.

Don't be afraid of the type signatures. They are informative and useful, plus enforced by the compiler.

In order to avoid future allocations could use a type of another future reference and a future combinator type that is known from the library or its docs. This way we can construct a resulting type. So, it's known to the compiler.
If still can't write the future type then can still avoid allocation by implementing a new future manually.
For reference implementation use tower source code, e.g. timeout and its ResponseFuture.
Pin is needed for creating self-referencing structures.
pin_project.

