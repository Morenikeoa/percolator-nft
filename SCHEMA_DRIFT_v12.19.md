# SCHEMA_DRIFT_v12.19.md — NFT mirror lag

Created: 2026-05-16
Status: **DEFERRED** — requires wrapper redeploy verification before landing.

## Summary

The NFT mirror at `src/slab_types.rs` is at v12.17 layout. The wrapper has
moved to v12.19 SBF layout. NFT compiles cleanly because every byte-layout
assertion compares the mirror to its own constants — there is no
cross-validation against the live `percolator-prog` crate (intentionally,
to avoid solana-program version conflicts).

## Confirmed drift (verified 2026-05-15)

| Constant | NFT mirror value | Live wrapper SBF | Delta |
|---|---|---|---|
| `EXPECTED_SLAB_HEADER_SIZE` | 72 | **136** | +64 (insurance_authority+insurance_operator added per ML8) |
| `EXPECTED_MARKET_CONFIG_SIZE` | 512 | **480** | -32 (max_insurance_floor + _iw_padding2 removed) |
| `EXPECTED_RISK_PARAMS_SIZE` | 184 | **168** | -16 (added 5 fields, removed min_initial_deposit; net trim) |
| `EXPECTED_ACCOUNT_SIZE` | 408 | **360** (V12_19 SBF, native u128 alignment) | -48 |
| `ENGINE_OFF` | 584 (V12_17) | **616** (V12_19 SBF) / **600** (test-side) | +32 |
| `SLAB_LEN` | header + config + RiskEngine only | + RISK_BUF_LEN(160) + GEN_TABLE_LEN(MAX_ACCOUNTS*8) | ≥ +2208 |

Detection entry in `src/cpi.rs:172` uses `V12_17_ENGINE_OFF: usize = 584` —
no v12.19 layout entry exists. `detect_layout` will fall through to
`UnrecognizedSlabLayout` for any v12.19 slab.

## Why deferred

1. **Wrapper deploy issue (per ~/.claude/projects/-Users-khubair/memory/MEMORY.md):**
   v12.19.1 hotfix deployed mainnet with wrong slab tier (4096 vs intended
   256), breaking new market creation AND existing small-tier markets with
   InvalidSlabLen 0x4. Until that's corrected, mainnet has no live v12.19
   small-tier slabs to validate the NFT mirror against.

2. **No cross-program verification:** NFT mirror has no Cargo dep on
   `percolator-prog` (intentional, avoids solana-program version conflicts).
   Adding one for verification only would create a maintenance burden.
   Alternative: build-script that emits the constants and compares.

3. **Risk profile:** mirror schema drift can mask itself as compile-pass
   (every assertion is `mirror == self-declared mirror constant`). Landing a
   wrong v12.19 mirror could mis-read positions or PNL on a future v12.19
   small-tier deploy. Verification needs an actual live v12.19 slab to read
   from.

## Migration plan (when unblocked)

### Step 1: Add v12.19 layout entry alongside v12.17

```rust
// src/cpi.rs additions
const V12_19_HEADER_LEN: usize = 136;
const V12_19_CONFIG_LEN: usize = 480;
const V12_19_ENGINE_OFF: usize = 616; // BPF native; 600 on test side
const V12_19_RISK_PARAMS_SIZE: usize = 168;
const V12_19_ACCOUNT_SIZE: usize = 360;
```

### Step 2: Update SlabHeader to add insurance_authority + insurance_operator

```rust
// src/slab_types.rs
pub struct SlabHeader {
    pub magic: u64,
    pub version: u32,
    pub bump: u8,
    pub _padding: [u8; 3],
    pub admin: [u8; 32],
    pub _reserved: [u8; 24],
    pub insurance_authority: [u8; 32], // NEW: +32
    pub insurance_operator: [u8; 32],  // NEW: +32
}
```

Bump `LAYOUT_REVISION` to 3. Update `EXPECTED_SLAB_HEADER_SIZE` to 136.

### Step 3: Trim MarketConfig opaque blob to 480

Just change the constant:
```rust
pub const EXPECTED_MARKET_CONFIG_SIZE: usize = 480;
```

The actual MarketConfig struct in NFT is opaque (`pub struct MarketConfig(pub [u8; EXPECTED_MARKET_CONFIG_SIZE])`), so the trim is safe — NFT never reads its fields.

### Step 4: Update RiskParams + Account structs

Mirror current wrapper `percolator-prog/src/percolator.rs:2799-2828` (header)
and the engine's `percolator/src/percolator.rs` RiskParams + Account.

### Step 5: Add SLAB_LEN tail (RiskBuf + GenTable)

```rust
pub const RISK_BUF_LEN: usize = 160;
pub const GEN_TABLE_LEN: usize = MAX_ACCOUNTS * 8; // u64 per slot
pub const SLAB_LEN: usize = ENGINE_OFF + RISK_ENGINE_SIZE + RISK_BUF_LEN + GEN_TABLE_LEN;
```

### Step 6: Update `detect_layout` in src/cpi.rs

Add v12.19 detection:
- Probe at V12_19_ENGINE_OFF + 32 + 24 for max_accounts
- Choose v12.17 vs v12.19 layout based on which probe returns valid value

### Step 7: Add real cross-check test

Create `tests/integration_v12_19.rs` that uses LiteSVM to spin up a real
percolator-prog v12.19 small-tier market and reads a position via the NFT
mirror. Assert the position bytes match what the wrapper exposes. This is
the only actual verification possible.

### Step 8: Verify

- `cargo build` clean
- `cargo test` all passing
- Manual: load a known mainnet v12.19 slab fixture and assert `read_position`
  returns expected values

## Activation gating

The NFT mirror update should land BEFORE:
- The wrapper v12.19 hotfix is correctly redeployed (with `--features small`)
- Any market creation on the corrected v12.19 mainnet
- Any NFT mint/transfer/burn against a v12.19 market

In the meantime, NFT continues to work correctly against v12.17 slabs that
existed before the hotfix.

## Cross-references

- Wrapper SBF constants: `percolator-prog/tests/drift_detection.rs:143-145`
  (`HEADER_LEN_EXPECTED`, `CONFIG_LEN_EXPECTED = 480`)
- Wrapper SlabHeader struct: `percolator-prog/src/percolator.rs:2799-2828`
- Wrapper MarketConfig struct: `percolator-prog/src/percolator.rs:2836+`
- Engine RiskEngine struct: `percolator/src/percolator.rs` (search `pub struct RiskEngine`)
- Engine RiskParams: `percolator/src/percolator.rs` (search `pub struct RiskParams`)
- Wave 12-B in `~/wrapper-engine-deep-audit/WAVE_12_PLAN.md`

## Estimated effort

3-4 hours focused work + 1 hour verification = 1 session
