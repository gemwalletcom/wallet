# Error Handling

## Principle

Prefer plain `Error` enums with `From` impls over `thiserror` macros. Use the `?` operator for propagation.

## Error Type Pattern

Define plain error enums with `Display`, `Error`, and constructor methods:

```rust
// good — crates/primitives/src/signer_error.rs
#[derive(Debug, Clone)]
pub enum SignerError {
    InvalidInput(String),
    SigningError(String),
}

impl std::fmt::Display for SignerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            SignerError::SigningError(msg) => write!(f, "Signing error: {}", msg),
        }
    }
}

impl std::error::Error for SignerError {}

impl SignerError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn signing_error(message: impl Into<String>) -> Self {
        Self::SigningError(message.into())
    }
}
```

## Constructor Methods over Verbose Enums

```rust
// bad — verbose enum construction
return Err(SignerError::InvalidInput("missing field".into()));

// good — constructor method
return Err(SignerError::invalid_input("missing field"));
```

## From Impls for `?` Operator

Add `From` conversions so callers can use `?` instead of manual `map_err`:

```rust
// good — From impls enable ? operator
impl From<serde_json::Error> for SignerError {
    fn from(error: serde_json::Error) -> Self {
        SignerError::InvalidInput(error.to_string())
    }
}

impl From<HexError> for SignerError {
    fn from(error: HexError) -> Self {
        SignerError::InvalidInput(error.to_string())
    }
}

// Then callers can write:
let data: MyStruct = serde_json::from_str(input)?;  // auto-converts error
```

```rust
// bad — manual map_err when From impl exists
let data = serde_json::from_str(input)
    .map_err(|e| SignerError::InvalidInput(e.to_string()))?;

// good — ? with From impl
let data: MyStruct = serde_json::from_str(input)?;
```

## Don't Let Error Plumbing Bury the Logic

When a block does repeated validation, inline `.ok_or_else(|| Error::invalid_input(format!("{ctx} ...")))?` makes every line read as error handling instead of logic. Capture the shared context in a local closure, and use guard clauses for boolean checks instead of `cond.then_some(()).ok_or_else(...)?`.

```rust
// bad — the chain/format context repeats on every line and drowns the logic
let (unlocking_script, hash) = address
    .unlocking_script()
    .zip(address.public_key_hash())
    .ok_or_else(|| SignerError::invalid_input(format!("{} UTXO address type is unsupported", chain.get_chain())))?;
(hash == sender_hash)
    .then_some(())
    .ok_or_else(|| SignerError::invalid_input(format!("{} UTXO address does not match sender", chain.get_chain())))?;
let vout = u32::try_from(utxo.vout).map_err(|_| SignerError::invalid_input(format!("invalid {} UTXO index", chain.get_chain())))?;

// good — build the chain-scoped message once; pick the constructor by context
let message = |reason: &str| format!("{} {reason}", chain.get_chain());

let (unlocking_script, hash) = address
    .unlocking_script()
    .zip(address.public_key_hash())
    .ok_or_else(|| SignerError::invalid_input(message("UTXO address type is unsupported")))?;
if hash != sender_hash {
    return SignerError::invalid_input_err(message("UTXO address does not match sender"));
}
let vout = u32::try_from(utxo.vout).map_err(|_| SignerError::invalid_input(message("UTXO index is invalid")))?;
```

Use the `_err` Result-returning constructor (`invalid_input_err`) for guard-clause early returns; use the bare `invalid_input` inside `ok_or_else`/`map_err` combinators. Don't hand-roll `return Err(SignerError::invalid_input(...))` when an `_err` constructor exists.

## Don't Repeat a Message for an Unreachable Branch

If an earlier guard already bounds a value, a later "can't fail" conversion must not duplicate the guard's message — surface the real library error so a future limit change isn't masked by a stale, wrong message.

```rust
// bad — at <=80 bytes PushBytes never overflows, so this message is unreachable and misleading
if data.len() > MAX_OP_RETURN_BYTES {
    return SignerError::invalid_input_err("Bitcoin memo is too large");
}
let push = PushBytesBuf::try_from(data.to_vec()).map_err(|_| SignerError::invalid_input("Bitcoin memo is too large"))?;

// good — the length check owns the rule; the conversion surfaces the actual error if it ever fires
let push = PushBytesBuf::try_from(data.to_vec()).map_err(SignerError::from_display)?;
```

## JSON Parameter Extraction

Use the `primitives::ValueAccess` trait instead of manual `.get().ok_or()` chains:

```rust
// Trait provides: get_value(key), at(index), string()
use primitives::ValueAccess;

// bad — manual extraction
let tx = params.get("transactions")
    .ok_or("Missing transactions")?
    .as_array()
    .ok_or("Expected array")?
    .get(0)
    .ok_or("Missing first element")?
    .as_str()
    .ok_or("Expected string")?;

// good — chained ValueAccess methods
let tx = params.get_value("transactions")?.at(0)?.string()?;
```

Add accessor methods on parent types to avoid pattern-matching boilerplate at call sites (e.g., `TransactionLoadInput::get_data_extra()`).

## Database Patterns

- Separate database models from domain primitives
- Use `as_primitive()` methods for conversion
- Diesel ORM with PostgreSQL backend
- Support transactions and upserts

## Async Patterns

- Tokio runtime throughout
- Async client structs returning `Result<T, Error>`
- Use `Arc<tokio::sync::Mutex<T>>` for shared async state
