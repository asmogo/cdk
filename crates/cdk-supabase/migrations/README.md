# Supabase Migrations

These SQL files define the CDK wallet database schema for Supabase.

## Running Migrations (Operator / Admin Only)

Migrations must be run by an operator with admin access to the Supabase project.
**Never run migrations from a client application.**

### Option 1 — Supabase CLI (recommended)

```bash
supabase db push
```

Point the Supabase CLI at the project containing these files.

### Option 2 — Supabase Dashboard SQL editor

Use the Rust SDK helper to get the full concatenated SQL:

```rust
let sql = SupabaseWalletDatabase::get_schema_sql();
// or via FFI:
let sql = supabase_get_schema_sql();
```

Paste the output into the **SQL Editor** in the Supabase Dashboard and run it.

### Option 3 — Manual file-by-file

Run each `*.sql` file in order (001, 002, 003, …) using the Supabase Dashboard
SQL editor or `psql` with the service role connection string.

## Client-Side Compatibility Check

After an operator has run migrations, client applications should call
`check_schema_compatibility()` before first use:

```rust
db.check_schema_compatibility().await?;
```

This reads the `schema_info` table (readable by all authenticated users) and
returns an error if the database schema is older than what the SDK requires.

## Adding New Migrations

1. Create a new file `NNN_description.sql` (e.g. `005_add_foo.sql`).
2. At the end of the file, update the `schema_info` version:
   ```sql
   INSERT INTO schema_info (key, value) VALUES ('schema_version', 'N')
   ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;
   ```
3. Bump `REQUIRED_SCHEMA_VERSION` in `crates/cdk-supabase/src/wallet.rs`.
