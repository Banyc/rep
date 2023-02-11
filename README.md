# `rep`

`rep` is a tiny utility that lets you easily enforce [representation/class invariants](https://en.wikipedia.org/wiki/Class_invariant) throughout your Rust data structures.

Representation invariants are logical assertions that must hold true for every mutation of your data structure. For example, in your GIS application, you may have the following rep invariant for a `LatLong`.

```rust
self.lat >= -90.0 && self.lat <= 90 && self.long >= -180.0 && self.long <= 180
```

Enforcing representation invariants is easy with `rep`:

1. Define a correct representation on independent fields (by deriving `CheckIndieFields` with macros)
1. Define a correct representation on interrelated fields (by either implementing `CheckFields` or using a default implementation)
1. Insert runtime checks (either manually or with a macro)

# Checking independent fields

`CheckIndieFields` must be derived first.

```rust
use rep::*;

#[derive(CheckIndieFields)]
pub struct Line {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32
}
```

## Examples

```rust
#[derive(CheckIndieFields)]
struct Circle {
    x: i32,
    y: i32,
    #[rep(assert_gt = 0)]
    #[rep(assert_le = 2048)]
    r: i32,
}
```

```rust
fn is_health_valid(h: u32) -> bool {
    h > 0 && h < 100
}

#[derive(CheckIndieFields)]
struct Player {
    #[rep(check)]
    position: Point,
    #[rep(assert_with = "is_health_valid")]
    health: u32
}
```

```rust
#[derive(CheckIndieFields)]
struct Parser {
    #[rep(assert_default)]
    unclosed_delims: (usize, usize, usize) // this is representing (parens, braces, brackets)
}
```

# Checking interrelated fields

`CheckFields` must be implemented first.

```rust
use rep::*;

impl CheckFields for Line {}
```

## Examples

```rust
fn is_health_valid(h: u32) -> bool {
    h > 0 && h < 100
}

#[derive(CheckIndieFields)]
struct Player {
    #[rep(check)]
    position: Point,
    #[rep(assert_with = "is_health_valid")]
    health: u32
}

impl CheckFields for Line {
    fn check_fields(&self, e: &mut RepErrors) {
        if self.x2 != self.y2 {
            e.add(String::from("self.x2 must equal self.y2"));
        }
    }
}
```

# Inserting runtime checks

`CheckRep` must be implemented first.

```rust
impl CheckRep for Line {}
```

By default, if a logger is present invariant violation will be logged instead of panicked.

## Examples

`#[check_rep]` macro automatically inserts calls to `check_rep` at start and end of all methods that are `pub` and mutate `&mut self`:

```rust
#[check_rep] // <-- this inserts calls to `check_rep` at the start and the end of `move_by`
impl Line {
    pub fn new() -> Self {
        let new_line = Self {
            x1: -1,
            y1: -1,
            x1: 1,
            y1: 1
        };

        new_line.check_rep();
        new_line
    }

    pub fn move_by(&mut self, x: i32, y: i32) {
        self.x1 += x;
        self.x2 += x;
        self.y1 += y;
        self.y2 += y;
    }
}
```

`#[check_rep]`, `#[require_rep]`, and `#[check_rep]` macros:

```rust
// this adds `check_rep` at start and end of all public mutating methods
#[check_rep]
impl Device {
    pub fn turn_on(&mut self) {}
    // require_rep, ensure_rep, check_rep add to start, end, start and end respectively
    #[require_rep]
    pub fn get_voltage(&mut self, p: Position) {}
    #[ensure_rep]
    pub fn actuate(&mut self, p: Position, v: Voltage) {}
    #[check_rep]
    fn do_something(&self) {}
}
```

# Usage

Just add the following to your `Cargo.toml` file.

```toml
[dependencies]
rep = { git = "<git url>" }
```
