// Source: ~/claudecode/openclaudecode/src/types/utils.rs

/// DeepImmutable is a TypeScript conditional type that recursively makes all
/// properties readonly. In Rust, immutability is the default, so this type
/// serves as documentation rather than an active constraint.
///
/// TypeScript:
///   T extends (...args: never[]) => unknown ? T
///     : T extends readonly (infer U)[] ? ReadonlyArray<DeepImmutable<U>>
///     : T extends object ? { readonly [K in keyof T]: DeepImmutable<T[K]> }
///     : T
///
/// In Rust, all values are immutable by default unless marked `mut`.
/// For shared ownership of immutable data, use `Arc<T>`.
/// For interior mutability patterns, use `RefCell<T>`, `Mutex<T>`, or `RwLock<T>`.
/// For functions, use `Box<dyn Fn(...)>` or `fn(...)` directly.
pub type DeepImmutable<T> = T;

/// Permutations type in TypeScript generates all possible orderings of a union.
/// In Rust, this doesn't have a direct type-level equivalent, but can be
/// implemented at runtime for enums.
pub type Permutations<T> = T;
