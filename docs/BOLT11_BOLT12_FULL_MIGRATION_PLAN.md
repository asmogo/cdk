# Bolt11/Bolt12 Full Migration to Generic Architecture - Implementation Plan

## Current Status (Phase 1 Complete ✅)

We have successfully implemented trait-based field types for Bolt11 and Bolt12:
- `Bolt11MintRequestFields`, `Bolt11MintResponseFields`, `Bolt11MeltRequestFields`, `Bolt11MeltResponseFields`
- `Bolt12MintRequestFields`, `Bolt12MintResponseFields`, `Bolt12MeltRequestFields`, `Bolt12MeltResponseFields`

These are ready to use with the generic `MintQuoteRequest<M>` and `MeltQuoteRequest<M>` types.

## Phase 2: Complete Migration (Next Steps)

### Challenge
Currently, nut23.rs and nut25.rs contain the OLD struct definitions:
```rust
pub struct MintQuoteBolt11Request { amount, unit, description, pubkey }
pub struct MintQuoteBolt11Response<Q> { quote, request, state, ... }
// etc.
```

We want to REPLACE these with type aliases:
```rust
pub type MintQuoteBolt11Request = MintQuoteRequest<Bolt11MintRequestFields>;
pub type MintQuoteBolt11Response<Q> = MintQuoteResponse<Q, Bolt11MintResponseFields>;
```

### The Problem
1. **Field Mismatch**: Old structs have flat fields, new types nest fields in `method_fields`
2. **Construction Sites**: Code constructs old types directly: `MintQuoteBolt11Request { amount, unit, ... }`
3. **Conversions**: Many From/TryFrom implementations assume old structure
4. **Tests**: Tests expect old field layout

### Solution: Methodical Replacement

#### Step 1: Create Constructor Functions
Add to nut23.rs:
```rust
impl Bolt11MintRequestFields {
    pub fn from_parts(description: Option<String>, pubkey: Option<PublicKey>) -> Self {
        Self { description, pubkey }
    }
}

// Add similar for all field types
```

#### Step 2: Update Construction Sites
Change:
```rust
MintQuoteBolt11Request {
    amount,
    unit,
    description,
    pubkey,
}
```

To:
```rust
MintQuoteRequest::new(
    amount,
    unit,
    Bolt11MintRequestFields { description, pubkey }
)
```

Files to update:
- `crates/cdk/src/wallet/issue/bolt11.rs`
- `crates/cdk/src/test_helpers/mint.rs`  
- `crates/cdk-integration-tests/tests/fake_auth.rs`

#### Step 3: Update From/TryFrom Implementations
In `crates/cdk-common/src/mint.rs`:
```rust
impl From<MintQuote> for MintQuoteBolt11Response<QuoteId> {
    fn from(quote: MintQuote) -> Self {
        MintQuoteResponse::new(
            quote.id,
            quote.request,
            quote.unit,
            quote.state.into(),
            quote.expiry,
            Bolt11MintResponseFields { pubkey: quote.pubkey },
        )
    }
}
```

#### Step 4: Delete Old Struct Definitions
Remove from nut23.rs (lines ~35-150, ~240-310):
- `pub struct MintQuoteBolt11Request`
- `pub struct MintQuoteBolt11Response<Q>`
- `pub struct MeltQuoteBolt11Request`
- `pub struct MeltQuoteBolt11Response<Q>`

Keep only:
- Error enum
- QuoteState enum
- MeltOptions enum
- Type aliases
- Field implementations

#### Step 5: Update custom_handlers.rs
Remove special casing:
```rust
// OLD:
match method.as_str() {
    "bolt11" => { /* special handling */ }
    "bolt12" => { /* special handling */ }
    _ => { /* generic handling */ }
}

// NEW: All methods use same generic path
let quote_request = match method.as_str() {
    "bolt11" => deserialize_as::<MintQuoteRequest<Bolt11MintRequestFields>>(payload)?,
    "bolt12" => deserialize_as::<MintQuoteRequest<Bolt12MintRequestFields>>(payload)?,
    _ => deserialize_as::<MintQuoteRequest<NoAdditionalFields>>(payload)?,
};
```

#### Step 6: Fix Tests
Update all tests to use new construction:
- Cashu crate tests
- CDK tests
- Integration tests

### Files to Modify (In Order)

1. **crates/cashu/src/nuts/nut23.rs** - Remove old structs, add helpers
2. **crates/cashu/src/nuts/nut25.rs** - Remove old structs, add helpers  
3. **crates/cdk/src/wallet/issue/bolt11.rs** - Update construction
4. **crates/cdk/src/wallet/issue/bolt12.rs** - Update construction
5. **crates/cdk/src/wallet/melt/bolt11.rs** - Update construction
6. **crates/cdk/src/wallet/melt/bolt12.rs** - Update construction
7. **crates/cdk-common/src/mint.rs** - Update conversions
8. **crates/cdk-common/src/melt.rs** - Update conversions
9. **crates/cdk/src/mint/issue/mod.rs** - Update enum variants
10. **crates/cdk/src/mint/melt.rs** - Update enum variants
11. **crates/cdk-axum/src/custom_handlers.rs** - Unify handling
12. **All test files** - Update test construction

### Estimated Effort
- **Time**: 4-6 hours of focused work
- **Risk**: Medium (requires careful testing)
- **Impact**: High (complete architecture unification)

### Testing Strategy
After each file modification:
1. Run `cargo build -p <crate>` to catch compile errors
2. Run `cargo test -p <crate>` to catch logic errors
3. Fix incrementally before moving to next file

### Success Criteria
1. ✅ No old Bolt11/Bolt12 struct definitions remain
2. ✅ All code uses generic MintQuoteRequest/Response types
3. ✅ No special case bolt11/bolt12 logic in handlers
4. ✅ All tests pass (161+ tests)
5. ✅ Full workspace builds
6. ✅ Integration tests pass

## Alternative: Gradual Migration

If full migration is too risky, we can:
1. Keep old structs AND new field types
2. Add conversion traits between old and new
3. Migrate code file-by-file
4. Remove old structs in version 0.6.0

This is safer but prolongs technical debt.

## Recommendation

**Go for full migration now** because:
- Field implementations are ready and tested
- Codebase is relatively small (4 main construction sites)
- Unifies architecture immediately
- Eliminates confusion between old/new patterns
- Better developer experience

The work is mechanical and can be done systematically.
