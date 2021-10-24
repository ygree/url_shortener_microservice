# Ideas

In order to work around async and ownership limitations in Rust need to leverage `#derive[(Clone)]` and `svc.clone()`.
Typically, in within the future we need to use a cloned service to be able to close over it making sure it's bind to the current future life-time.

There is a nice `BoxFuture` type alias to remove some boilerplate.

Don't be afraid of the type signatures. They are informative and useful, plus enforced by the compiler.