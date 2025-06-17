As well as the built-in implementations, this library exposes two derive macros
for implementing the `ToKlvm` and `FromKlvm` traits on structs and enums.
These macros can be used with both named and unnamed structs and enum variants.

## Representations

There are multiple ways to encode a sequence of fields in either a struct or an enum variant.
These are referred to as representations and are specified using the `#[klvm(...)]` attribute.
Below are examples of derive macros using each of these representations.
Pick whichever representation fits your use-case the best.

Note that the syntax `(A . B)` represents a cons-pair with two values, `A` and `B`.
This is how non-atomic values are structured in KLVM.

### Proper List

This represents values in a nil terminated series of nested cons-pairs.

Note that if you mark the last field to [consume the rest of the list](#consume-the-rest), there is no nil terminator.

For example, with the list `[A, B, C]`, we build the list in reverse:

- Start with the nil terminator `()`
- Create the first cons pair `(C . ())`
- Then continue on with `(B . (C . ()))`
- Finally, the list is represented as `(A . (B . (C . ())))`

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(proper_list)]
struct Tiers {
    high: u8,
    medium: u8,
    low: u8,
}

// The KLVM representation for this is `(10 5 1)`.
// It can also be written as `(10 . (5 . (1 . ())))`.
let value = Tiers {
    high: 10,
    medium: 5,
    low: 1,
};

let a = &mut Allocator::new();
let ptr = value.to_klvm(a).unwrap();
assert_eq!(Tiers::from_klvm(a, ptr).unwrap(), value);
```

### List

The list representation is the same as proper list, except it does not check the nil terminator when parsing.
This allows it to be lenient to additional parameters that are in the KLVM object, which is especially useful
for preventing errors when parsing on-chain solutions and output conditions, where you wouldn't want strictness.
This is because puzzles often don't enforce additional solution parameters, and consensus allows additional
parameters to leave the door open for soft forks in the future.

### Curry

This represents the argument part of a curried KLVM program.
In Chik, currying commits to and partially applies some of the arguments of a program, without calling it.

The arguments are quoted and terminated with `1`, which is how partial application is implemented in KLVM.
Note that if you mark the last field to [consume the rest of the arguments](#consume-the-rest), there is no `1` terminator.

For example, the curried arguments `[A, B, C]` are encoded as `(c (q . A) (c (q . B) (c (q . C) 1)))`.

You can read more about currying on the [Chik blockchain documentation](https://docs.chiknetwork.com/guides/chiklisp-currying).

The following example is for demonstration purposes only:

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(curry)]
struct PuzzleArgs {
    code_to_unlock: u32,
    verification_level: u8,
}

// The KLVM representation for this is `(c (q . 4328714) (c (q . 5) 1))`.
let args = PuzzleArgs {
    code_to_unlock: 4328714,
    verification_level: 5,
};

let a = &mut Allocator::new();
let ptr = args.to_klvm(a).unwrap();
assert_eq!(PuzzleArgs::from_klvm(a, ptr).unwrap(), args);
```

### Transparent

If you want a struct to have the same KLVM representation as its inner struct (a newtype), you can use the `transparent` representation.

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(transparent)]
struct CustomString(String);

// The KLVM representation for this is the same as the string itself.
// So `"Hello"` in this case.
let string = CustomString("Hello".to_string());

let a = &mut Allocator::new();
let ptr = string.to_klvm(a).unwrap();
assert_eq!(CustomString::from_klvm(a, ptr).unwrap(), string);
```

## Optional Fields

You can only mark the last field in a struct or enum variant as optional.

This restriction is in place because if you were able to have multiple optional fields,
or an optional field that isn't at the end, it would be ambiguous.

### Optional Value

You can set a field as optional by marking it as `#[klvm(default)]`.
If the field isn't present when deserializing, it will default to the `Default` implementation of the type.
When serializing, it will check if it's equal to the default and omit it if so.

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(list)]
struct Person {
    name: String,
    #[klvm(default)]
    email: Option<String>,
}

// The KLVM representation of this is `("Bob" "bob@example.com")`.
// If `email` had been set to `None`, the representation would have just been `("Bob")`.
let person = Person {
    name: "Bob".to_string(),
    email: Some("bob@example.com".to_string()),
};

let a = &mut Allocator::new();
let ptr = person.to_klvm(a).unwrap();
assert_eq!(Person::from_klvm(a, ptr).unwrap(), person);
```

### Default Value

You can also specify the default value to check against manually.
This is useful if you want to override the `Default` trait, or if the `Default` trait isn't implemented.

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(list)]
struct Person {
    name: String,
    #[klvm(default = 18)]
    age: u8,
}

// The KLVM representation for this is `("Bob" 24)`.
// If `age` had been set to `18`, the representation would have been just `("Bob")`.
let person = Person {
    name: "Bob".to_string(),
    age: 24,
};

let a = &mut Allocator::new();
let ptr = person.to_klvm(a).unwrap();
assert_eq!(Person::from_klvm(a, ptr).unwrap(), person);
```

## Consume the Rest

You can consume the rest of the list items (or curried arguments, if using the `curry` representation) by using `#[klvm(rest)]`.
This is useful for types which are represented compactly, without a nil terminator. Or for extending a list of arguments with another.
You can also use it if you want to lazily parse the rest later.

Here's a simple example of a compact representation:

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(list)]
struct Point {
    x: i32,
    #[klvm(rest)]
    y: i32,
}

// The KLVM representation of this is `(5 . 2)` (with no nil terminator).
let point = Point {
    x: 5,
    y: 2,
};

let a = &mut Allocator::new();
let ptr = point.to_klvm(a).unwrap();
assert_eq!(Point::from_klvm(a, ptr).unwrap(), point);
```

And here's an example of lazily parsing the rest later:

```rust
use klvmr::{Allocator, NodePtr};
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(list)]
struct Items<T> {
    first: String,
    #[klvm(rest)]
    rest: T,
}

// The KLVM representation of this is `("First Item" 1 2 3 4 5)`.
// Notice how the list is not a separate argument, but rather the rest of the arguments.
let items = Items {
    first: "First Item".to_string(),
    rest: [1, 2, 3, 4, 5],
};

let a = &mut Allocator::new();
let ptr = items.to_klvm(a).unwrap();

// We parse `("First Item" . <rest>)`
let items = Items::<NodePtr>::from_klvm(a, ptr).unwrap();
assert_eq!(items.first, "First Item".to_string());

// Then parse rest into a separate list of `(1 2 3 4 5)`.
let rest: [u8; 5] = FromKlvm::from_klvm(a, items.rest).unwrap();
assert_eq!(rest, [1, 2, 3, 4, 5]);
```

## Enums

In Rust, enums contain a discriminant, a value used to distinguish between each variant of the enum.
In most cases, the KLVM representation of the enum will need to contain this discriminant as the first argument.
For convenience, this is the behavior when deriving `ToKlvm` and `FromKlvm` for enums by default.

### Simple Example

In this example, since the `atom` representation, the variants will be encoded as an integer.
Discriminants default to the `isize` type and the first value is `0`. Subsequent values are incremented by `1` by default.

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(atom)]
enum Status {
    Pending,
    Completed,
}

// The KLVM representation of this is just `0`.
let status = Status::Pending;

let a = &mut Allocator::new();
let ptr = status.to_klvm(a).unwrap();
assert_eq!(Status::from_klvm(a, ptr).unwrap(), status);
```

### Custom Discriminator

It's possible to override both the type of the discriminator, and the value.
The `#[repr(...)]` attribute is used by the Rust compiler to allow overriding the discriminator type.
As such, this attribute is also used to change the underlying type used to serialize and deserialize discriminator values.

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(atom)]
#[repr(u8)]
enum Status {
    Pending = 36,
    Completed = 42,
}

// The KLVM representation of this is `36`.
let status = Status::Pending;

let a = &mut Allocator::new();
let ptr = status.to_klvm(a).unwrap();
assert_eq!(Status::from_klvm(a, ptr).unwrap(), status);
```

### Variant Fields

Of course, you can also include fields on enum variants, and they will be serialized after the discriminator accordingly.
It's also possible to override the representation of an individual variant, as if it were a standalone struct.

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(list)]
enum SpendMode {
    AppendValue { value: i32 },
    ClearValues,
}

// The KLVM representation of this is `(42)`.
let mode = SpendMode::AppendValue {
    value: 42
};

let a = &mut Allocator::new();
let ptr = mode.to_klvm(a).unwrap();
assert_eq!(SpendMode::from_klvm(a, ptr).unwrap(), mode);
```

### Untagged Enums

Often, the discriminator isn't necessary to encode, and you'd prefer to try to match each variant in order until one matches.
This is what `#[klvm(untagged)]` allows you to do.

Note that if there is any ambiguity, the first variant which matches a value will be the resulting value.
For example, if both `A` and `B` are in that order and are the same type, if you serialize a value of `B`, it will be deserialized as `A`.

```rust
use klvmr::Allocator;
use klvm_traits::{ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(list, untagged)]
enum Either {
    ShortList([i32; 4]),
    ExtendedList([i32; 16]),
}

// The KLVM representation of this is `((42 42 42 42))`.
let value = Either::ShortList([42; 4]);

let a = &mut Allocator::new();
let ptr = value.to_klvm(a).unwrap();
assert_eq!(Either::from_klvm(a, ptr).unwrap(), value);
```

## Constant Values

Sometimes you may want to include constants inside of a struct without actually exposing them as fields.
It's possible to do this with `#[klvm(constant = ...)]`, however you must use an attribute macro to remove the constant fields.

This has to be done in the proper order, or it will not work.

The order is as follows:

- Derive `ToKlvm` and `FromKlvm`, so that the constant fields are serialized and deserialized properly.
- Use `#[apply_constants]` to remove them from the actual struct or enum, but keep them in the encoded output.
- Add any other derives you want after, so they don't see the constant fields.
- Write any `#[klvm(...)]` options you want to use.

Here is an example of this:

```rust
use klvmr::Allocator;
use klvm_traits::{apply_constants, ToKlvm, FromKlvm};

#[derive(ToKlvm, FromKlvm)]
#[apply_constants]
#[derive(Debug, PartialEq, Eq)]
#[klvm(list)]
struct CustomTerminator {
    value: u32,
    #[klvm(constant = 42, rest)]
    terminator: u8,
}

// The KLVM representation of this is `(100 . 42)`.
let value = CustomTerminator {
    value: 100,
};

let a = &mut Allocator::new();
let ptr = value.to_klvm(a).unwrap();
assert_eq!(CustomTerminator::from_klvm(a, ptr).unwrap(), value);
```

## Crate Name

You can override the name of the `klvm_traits` crate used within the macros:

```rust
use klvmr::Allocator;
use klvm_traits::{self as renamed_klvm_traits, ToKlvm, FromKlvm};

#[derive(Debug, PartialEq, Eq, ToKlvm, FromKlvm)]
#[klvm(list, crate_name = renamed_klvm_traits)]
struct Example;
```
