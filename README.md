# DNS Server in Rust

## TODO

- finish implementation + compare
- go through todos
- remove dns\_\* from files. We know it's DNS.
- refactor + naming
  - no side effects
  - unambigous names
  - add comments if really necessary
- sort out directory structure
- add tests?
  - maybe just important ones if necessary
- CI pipeline. Use GitHub or CircleCI.
- utilise the result type for returning values and errors
- are all the includes pragmatic?
  - can we reuse `type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;`?
  - are the file names correct?
  - should we include more stuff in one file?
    - alternatively, is there a way to reduce the amount of `use super::*`?
    - maybe use [`pub use`](https://www.reddit.com/r/rust/comments/6x49mu/what_are_some_rules_of_thumb_for_use/dmd07yr?utm_source=share&utm_medium=web2x&context=3)?
