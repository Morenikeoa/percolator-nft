//! v16 portfolio layout — vendored Percolator v16 type definitions (NFT side).
//!
//! ## What this file is
//!
//! The v16-sync replacement for [`crate::slab_types`] (which vendored the v12 /
//! v12.17 slab layout). It vendors the byte-image of the on-chain
//! `PortfolioAccountV16Account` and its sub-structs from
//! `percolator/src/v16.rs`, byte-for-byte, plus compile-time `size_of!` /
//! `offset_of!` assertions that fail to compile if the vendored layout drifts.
//!
//! In v16, ONE `PortfolioAccountV16` is its own Solana account (not a slab
//! entry). The wrapper stores it as:
//!
//! ```text
//!   [ 16-byte wrapper header ][ PortfolioAccountV16Account ][ source-domain tail ]
//!     MAGIC u64 @0                fixed 2907-byte POD          variable, ignored
//!     VERSION u16 @8              starts at HEADER_LEN=16       by the NFT
//!     kind   u8  @10
//! ```
//!
//! See `percolator-prog/src/v16_program.rs` `portfolio_wire` (the wrapper's own
//! decode): `data.get(HEADER_LEN .. HEADER_LEN + size_of::<PortfolioAccountV16Account>())`
//! then `bytemuck` cast. [`decode_portfolio`] below mirrors that exactly.
//!
//! ## Layout-offset bug class — closed by construction
//!
//! The v12 NFT hardcoded an `ENGINE_OFF` derived from a chain of slab-size
//! arithmetic; a stale value (off by hundreds of bytes) silently mis-decoded
//! every position (commit `3d7d185`). v16 removes that entire surface:
//!
//! 1. The body offset is a single constant [`HEADER_LEN`] = 16 — no slab
//!    tiering, no `RISK_BUF`, no per-account `GEN_TABLE` tail.
//! 2. The NFT never computes a field offset by hand. It `bytemuck`-casts the
//!    fixed window to [`PortfolioAccountV16Account`] and reads fields by name —
//!    field access *is* the offset.
//! 3. Compile-time `assert!` on every sub-struct size + the key field offsets
//!    catch any vendoring drift; the LiteSVM integration test (later sub-phase)
//!    is the ground truth: it creates a real portfolio through the wrapper and
//!    decodes it through this module.
//!
//! Sizes/offsets below were confirmed by an empirical `size_of`/`offset_of`
//! probe run against the engine crate (then reverted), NOT hand-computed.
//!
//! ## BPF / host byte-identity
//!
//! Every 16/8/4/2-byte scalar uses a `#[repr(C)]` byte-array wrapper
//! ([`V16PodU128`] = `[u8; 16]`, etc.), align 1. The whole struct is therefore
//! packed with zero padding and is byte-identical on host (`cargo check`) and
//! SBF (`cargo build-sbf`) with no `#[cfg(target_arch)]` gating — matching the
//! engine's own `V16Pod*` POD wrappers (`percolator/src/v16.rs:3181-3255`).

#![allow(dead_code)] // wired into cpi.rs / processor.rs in the next NFT sub-phase

use core::mem::{align_of, offset_of, size_of};

// ════════════════════════════════════════════════════════════════════════════
// LAYOUT REVISION
// ════════════════════════════════════════════════════════════════════════════

/// Bump whenever the assertions below are intentionally re-vendored against a
/// new engine layout. Stamped into every minted NFT so re-vendoring invalidates
/// older NFTs rather than silently decoding with the wrong offsets.
///
/// Revision 3: v16 account-local layout (`PortfolioAccountV16Account`,
/// 2907-byte fixed head). Supersedes revision 2 (v12.17 slab layout in
/// `slab_types.rs`).
pub const LAYOUT_REVISION: u32 = 3;

// ════════════════════════════════════════════════════════════════════════════
// WRAPPER ACCOUNT HEADER — percolator-prog/src/v16_program.rs:44-51, 450-471
// ════════════════════════════════════════════════════════════════════════════

/// `"PERCV16\0"` little-endian. Byte 0..8 of every wrapper account.
pub const MAGIC: u64 = 0x5045_5243_5631_3600;
/// Wrapper account-format version. Byte 8..10.
pub const VERSION: u16 = 16;
/// Account-kind discriminant for a portfolio. Byte 10.
pub const KIND_PORTFOLIO: u8 = 2;
/// Bytes consumed by the wrapper header before the engine POD begins.
pub const HEADER_LEN: usize = 16;

// ════════════════════════════════════════════════════════════════════════════
// ENGINE CONSTANTS — percolator/src/v16.rs
// ════════════════════════════════════════════════════════════════════════════

/// `ProvenanceHeaderV16.layout_discriminator` must equal this (v16 guard).
pub const V16_LAYOUT_DISCRIMINATOR: u16 = 16;
/// `ProvenanceHeaderV16.version` must equal this.
pub const V16_ACCOUNT_VERSION: u16 = 1;
/// Number of leg slots in a portfolio's `legs` array.
pub const V16_MAX_PORTFOLIO_ASSETS_N: usize = 16;
/// `ceil(V16_MAX_PORTFOLIO_ASSETS_N / 64)`.
pub const V16_ACTIVE_BITMAP_WORDS: usize = 1;
/// Largest active-leg count the wrapper permits per portfolio (CU envelope).
/// An NFT's `asset_index` must be `< WRAPPER_MAX_PORTFOLIO_ASSETS`.
pub const WRAPPER_MAX_PORTFOLIO_ASSETS: u16 = 14;

// ════════════════════════════════════════════════════════════════════════════
// EXPECTED SIZES — empirically verified against the engine crate
// ════════════════════════════════════════════════════════════════════════════

pub const EXPECTED_PROVENANCE_HEADER_SIZE: usize = 100;
pub const EXPECTED_PORTFOLIO_LEG_SIZE: usize = 144;
pub const EXPECTED_HEALTH_CERT_SIZE: usize = 121;
pub const EXPECTED_CLOSE_PROGRESS_SIZE: usize = 184;
pub const EXPECTED_RESOLVED_PAYOUT_RECEIPT_SIZE: usize = 66;
pub const EXPECTED_PORTFOLIO_ACCOUNT_SIZE: usize = 2907;

// ════════════════════════════════════════════════════════════════════════════
// POD SCALAR WRAPPERS — byte arrays, align 1 (percolator/src/v16.rs:3181-3255)
// ════════════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct V16PodU16 {
    pub bytes: [u8; 2],
}
impl V16PodU16 {
    pub fn new(value: u16) -> Self {
        Self { bytes: value.to_le_bytes() }
    }
    pub fn get(self) -> u16 {
        u16::from_le_bytes(self.bytes)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct V16PodU32 {
    pub bytes: [u8; 4],
}
impl V16PodU32 {
    pub fn new(value: u32) -> Self {
        Self { bytes: value.to_le_bytes() }
    }
    pub fn get(self) -> u32 {
        u32::from_le_bytes(self.bytes)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct V16PodU64 {
    pub bytes: [u8; 8],
}
impl V16PodU64 {
    pub fn new(value: u64) -> Self {
        Self { bytes: value.to_le_bytes() }
    }
    pub fn get(self) -> u64 {
        u64::from_le_bytes(self.bytes)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct V16PodU128 {
    pub bytes: [u8; 16],
}
impl V16PodU128 {
    pub fn new(value: u128) -> Self {
        Self { bytes: value.to_le_bytes() }
    }
    pub fn get(self) -> u128 {
        u128::from_le_bytes(self.bytes)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct V16PodI128 {
    pub bytes: [u8; 16],
}
impl V16PodI128 {
    pub fn new(value: i128) -> Self {
        Self { bytes: value.to_le_bytes() }
    }
    pub fn get(self) -> i128 {
        i128::from_le_bytes(self.bytes)
    }
}

const _: () = assert!(size_of::<V16PodU16>() == 2 && align_of::<V16PodU16>() == 1);
const _: () = assert!(size_of::<V16PodU32>() == 4 && align_of::<V16PodU32>() == 1);
const _: () = assert!(size_of::<V16PodU64>() == 8 && align_of::<V16PodU64>() == 1);
const _: () = assert!(size_of::<V16PodU128>() == 16 && align_of::<V16PodU128>() == 1);
const _: () = assert!(size_of::<V16PodI128>() == 16 && align_of::<V16PodI128>() == 1);

// ════════════════════════════════════════════════════════════════════════════
// ProvenanceHeaderV16Account — percolator/src/v16.rs:3301
// ════════════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ProvenanceHeaderV16Account {
    pub market_group_id: [u8; 32],
    pub portfolio_account_id: [u8; 32],
    pub owner: [u8; 32],
    pub version: V16PodU16,
    pub layout_discriminator: V16PodU16,
}

const _: () = assert!(size_of::<ProvenanceHeaderV16Account>() == EXPECTED_PROVENANCE_HEADER_SIZE);
const _: () = assert!(align_of::<ProvenanceHeaderV16Account>() == 1);
const _: () = assert!(offset_of!(ProvenanceHeaderV16Account, market_group_id) == 0);
const _: () = assert!(offset_of!(ProvenanceHeaderV16Account, portfolio_account_id) == 32);
const _: () = assert!(offset_of!(ProvenanceHeaderV16Account, owner) == 64);
const _: () = assert!(offset_of!(ProvenanceHeaderV16Account, version) == 96);
const _: () = assert!(offset_of!(ProvenanceHeaderV16Account, layout_discriminator) == 98);

// ════════════════════════════════════════════════════════════════════════════
// PortfolioLegV16Account — percolator/src/v16.rs:11410
// ════════════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PortfolioLegV16Account {
    pub active: u8,
    pub asset_index: V16PodU32,
    pub market_id: V16PodU64,
    pub side: u8,
    pub basis_pos_q: V16PodI128,
    pub a_basis: V16PodU128,
    pub k_snap: V16PodI128,
    pub f_snap: V16PodI128,
    pub epoch_snap: V16PodU64,
    pub loss_weight: V16PodU128,
    pub b_snap: V16PodU128,
    pub b_rem: V16PodU128,
    pub b_epoch_snap: V16PodU64,
    pub b_stale: u8,
    pub stale: u8,
}

const _: () = assert!(size_of::<PortfolioLegV16Account>() == EXPECTED_PORTFOLIO_LEG_SIZE);
const _: () = assert!(align_of::<PortfolioLegV16Account>() == 1);
const _: () = assert!(offset_of!(PortfolioLegV16Account, active) == 0);
const _: () = assert!(offset_of!(PortfolioLegV16Account, asset_index) == 1);
const _: () = assert!(offset_of!(PortfolioLegV16Account, market_id) == 5);
const _: () = assert!(offset_of!(PortfolioLegV16Account, side) == 13);
const _: () = assert!(offset_of!(PortfolioLegV16Account, basis_pos_q) == 14);
const _: () = assert!(offset_of!(PortfolioLegV16Account, epoch_snap) == 78);
const _: () = assert!(offset_of!(PortfolioLegV16Account, b_stale) == 142);
const _: () = assert!(offset_of!(PortfolioLegV16Account, stale) == 143);

// ════════════════════════════════════════════════════════════════════════════
// HealthCertV16Account — percolator/src/v16.rs:11478
// ════════════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct HealthCertV16Account {
    pub certified_equity: V16PodI128,
    pub certified_initial_req: V16PodU128,
    pub certified_maintenance_req: V16PodU128,
    pub certified_liq_deficit: V16PodU128,
    pub certified_worst_case_loss: V16PodU128,
    pub cert_oracle_epoch: V16PodU64,
    pub cert_funding_epoch: V16PodU64,
    pub cert_risk_epoch: V16PodU64,
    pub cert_asset_set_epoch: V16PodU64,
    pub active_bitmap_at_cert: [V16PodU64; V16_ACTIVE_BITMAP_WORDS],
    pub valid: u8,
}

const _: () = assert!(size_of::<HealthCertV16Account>() == EXPECTED_HEALTH_CERT_SIZE);
const _: () = assert!(align_of::<HealthCertV16Account>() == 1);

// ════════════════════════════════════════════════════════════════════════════
// CloseProgressLedgerV16Account — percolator/src/v16.rs:11530
// ════════════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct CloseProgressLedgerV16Account {
    pub active: u8,
    pub finalized: u8,
    pub canceled: u8,
    pub close_id: V16PodU64,
    pub asset_index: V16PodU32,
    pub market_id: V16PodU64,
    pub domain_side: u8,
    pub gross_loss_at_close_start: V16PodU128,
    pub drift_reference_slot: V16PodU64,
    pub max_close_slot: V16PodU64,
    pub support_consumed: V16PodU128,
    pub junior_face_burned: V16PodU128,
    pub insurance_spent: V16PodU128,
    pub b_loss_booked: V16PodU128,
    pub explicit_loss_assigned: V16PodU128,
    pub quantity_adl_applied_q: V16PodU128,
    pub drift_consumed: V16PodU128,
    pub residual_remaining: V16PodU128,
}

const _: () = assert!(size_of::<CloseProgressLedgerV16Account>() == EXPECTED_CLOSE_PROGRESS_SIZE);
const _: () = assert!(align_of::<CloseProgressLedgerV16Account>() == 1);
const _: () = assert!(offset_of!(CloseProgressLedgerV16Account, active) == 0);
const _: () = assert!(offset_of!(CloseProgressLedgerV16Account, asset_index) == 11);

// ════════════════════════════════════════════════════════════════════════════
// ResolvedPayoutReceiptV16Account — percolator/src/v16.rs:11646
// ════════════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ResolvedPayoutReceiptV16Account {
    pub prior_bound_contribution_num: V16PodU128,
    pub live_released_face_at_receipt: V16PodU128,
    pub terminal_positive_claim_face: V16PodU128,
    pub paid_effective: V16PodU128,
    pub present: u8,
    pub finalized: u8,
}

const _: () =
    assert!(size_of::<ResolvedPayoutReceiptV16Account>() == EXPECTED_RESOLVED_PAYOUT_RECEIPT_SIZE);
const _: () = assert!(align_of::<ResolvedPayoutReceiptV16Account>() == 1);
const _: () = assert!(offset_of!(ResolvedPayoutReceiptV16Account, present) == 64);
const _: () = assert!(offset_of!(ResolvedPayoutReceiptV16Account, finalized) == 65);

// ════════════════════════════════════════════════════════════════════════════
// PortfolioAccountV16Account — percolator/src/v16.rs:11755 (fixed head only)
// ════════════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PortfolioAccountV16Account {
    pub provenance_header: ProvenanceHeaderV16Account,
    pub owner: [u8; 32],
    pub capital: V16PodU128,
    pub pnl: V16PodI128,
    pub reserved_pnl: V16PodU128,
    pub fee_credits: V16PodI128,
    pub cancel_deposit_escrow: V16PodU128,
    pub last_fee_slot: V16PodU64,
    pub active_bitmap: [V16PodU64; V16_ACTIVE_BITMAP_WORDS],
    pub legs: [PortfolioLegV16Account; V16_MAX_PORTFOLIO_ASSETS_N],
    pub health_cert: HealthCertV16Account,
    pub stale_state: u8,
    pub b_stale_state: u8,
    pub rebalance_lock: u8,
    pub liquidation_lock: u8,
    pub close_progress: CloseProgressLedgerV16Account,
    pub resolved_payout_receipt: ResolvedPayoutReceiptV16Account,
}

const _: () = assert!(size_of::<PortfolioAccountV16Account>() == EXPECTED_PORTFOLIO_ACCOUNT_SIZE);
const _: () = assert!(align_of::<PortfolioAccountV16Account>() == 1);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, provenance_header) == 0);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, owner) == 100);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, capital) == 132);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, last_fee_slot) == 212);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, active_bitmap) == 220);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, legs) == 228);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, health_cert) == 2532);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, stale_state) == 2653);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, liquidation_lock) == 2656);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, close_progress) == 2657);
const _: () = assert!(offset_of!(PortfolioAccountV16Account, resolved_payout_receipt) == 2841);

// ════════════════════════════════════════════════════════════════════════════
// DECODE — mirrors percolator-prog `portfolio_wire`
// ════════════════════════════════════════════════════════════════════════════

/// Why a portfolio account failed to decode. Each variant maps to an
/// NFT-program error so the on-chain handler can reject precisely.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortfolioDecodeError {
    /// Account data shorter than `HEADER_LEN + EXPECTED_PORTFOLIO_ACCOUNT_SIZE`.
    TooShort,
    /// Wrapper header MAGIC mismatch — not a percolator-v16 account.
    BadMagic,
    /// Wrapper header VERSION mismatch.
    BadVersion,
    /// Account kind is not `KIND_PORTFOLIO`.
    BadKind,
    /// `bytemuck` cast failed (should be impossible after the length check).
    Cast,
    /// `provenance_header.layout_discriminator != V16_LAYOUT_DISCRIMINATOR`.
    BadLayoutDiscriminator,
    /// `provenance_header.version != V16_ACCOUNT_VERSION`.
    BadAccountVersion,
    /// Engine invariant `owner == provenance_header.owner` violated.
    OwnerMismatch,
}

/// Decode a wrapper-owned portfolio account's raw data into the vendored fixed
/// head. Validates the wrapper header, the v16 provenance guard, and the
/// engine's `owner == provenance_header.owner` invariant. The variable
/// source-domain tail (if any) is ignored — the NFT never reads it.
///
/// SECURITY: this is the only place the NFT interprets portfolio bytes. The
/// body offset is the constant [`HEADER_LEN`] and field access is by name; no
/// offset is ever hand-computed (closes the v12 layout-offset bug class).
pub fn decode_portfolio(data: &[u8]) -> Result<&PortfolioAccountV16Account, PortfolioDecodeError> {
    if data.len() < HEADER_LEN + EXPECTED_PORTFOLIO_ACCOUNT_SIZE {
        return Err(PortfolioDecodeError::TooShort);
    }
    let magic = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]);
    if magic != MAGIC {
        return Err(PortfolioDecodeError::BadMagic);
    }
    let version = u16::from_le_bytes([data[8], data[9]]);
    if version != VERSION {
        return Err(PortfolioDecodeError::BadVersion);
    }
    if data[10] != KIND_PORTFOLIO {
        return Err(PortfolioDecodeError::BadKind);
    }
    let body = &data[HEADER_LEN..HEADER_LEN + EXPECTED_PORTFOLIO_ACCOUNT_SIZE];
    let account: &PortfolioAccountV16Account =
        bytemuck::try_from_bytes(body).map_err(|_| PortfolioDecodeError::Cast)?;
    if account.provenance_header.layout_discriminator.get() != V16_LAYOUT_DISCRIMINATOR {
        return Err(PortfolioDecodeError::BadLayoutDiscriminator);
    }
    if account.provenance_header.version.get() != V16_ACCOUNT_VERSION {
        return Err(PortfolioDecodeError::BadAccountVersion);
    }
    if account.owner != account.provenance_header.owner {
        return Err(PortfolioDecodeError::OwnerMismatch);
    }
    Ok(account)
}

impl PortfolioAccountV16Account {
    /// Authoritative owner. (`owner == provenance_header.owner` is enforced by
    /// [`decode_portfolio`] and by the engine's `validate`.)
    pub fn owner(&self) -> [u8; 32] {
        self.owner
    }

    /// Find the leg slot whose leg is active and trades `asset_index`. Mirrors
    /// the engine's `active_leg_slot_for_asset` (`v16.rs:2170`). `asset_index`
    /// is the asset identifier — NOT the array slot — so this scans the array.
    /// Returns `None` if no active leg trades that asset.
    pub fn active_leg_slot_for_asset(&self, asset_index: u32) -> Option<usize> {
        self.legs
            .iter()
            .position(|leg| leg.active != 0 && leg.asset_index.get() == asset_index)
    }

    /// True if a close is in progress for `asset_index` (transfer must reject).
    pub fn close_in_progress_for_asset(&self, asset_index: u32) -> bool {
        self.close_progress.active != 0 && self.close_progress.asset_index.get() == asset_index
    }

    /// True if the portfolio has a terminal payout receipt present.
    pub fn has_resolved_receipt(&self) -> bool {
        self.resolved_payout_receipt.present != 0
    }

    /// True if any portfolio-level lock/stale gate blocks free transfer.
    pub fn portfolio_locked_or_stale(&self) -> bool {
        self.liquidation_lock != 0 || self.stale_state != 0 || self.b_stale_state != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_account() -> PortfolioAccountV16Account {
        bytemuck::Zeroable::zeroed()
    }

    /// Build a minimal valid wrapper-framed portfolio buffer for an owner with a
    /// single active leg. Mirrors the wrapper's `[header][POD][tail]` framing.
    fn framed(owner: [u8; 32], asset_index: u32, market_id: u64) -> Vec<u8> {
        let mut acct = empty_account();
        acct.provenance_header.owner = owner;
        acct.provenance_header.version = V16PodU16::new(V16_ACCOUNT_VERSION);
        acct.provenance_header.layout_discriminator = V16PodU16::new(V16_LAYOUT_DISCRIMINATOR);
        acct.owner = owner;
        acct.legs[3].active = 1;
        acct.legs[3].asset_index = V16PodU32::new(asset_index);
        acct.legs[3].market_id = V16PodU64::new(market_id);

        let mut buf = vec![0u8; HEADER_LEN + EXPECTED_PORTFOLIO_ACCOUNT_SIZE + 64];
        buf[0..8].copy_from_slice(&MAGIC.to_le_bytes());
        buf[8..10].copy_from_slice(&VERSION.to_le_bytes());
        buf[10] = KIND_PORTFOLIO;
        let body = bytemuck::bytes_of(&acct);
        buf[HEADER_LEN..HEADER_LEN + body.len()].copy_from_slice(body);
        buf
    }

    #[test]
    fn sizes_match_engine_ground_truth() {
        assert_eq!(size_of::<PortfolioAccountV16Account>(), 2907);
        assert_eq!(size_of::<ProvenanceHeaderV16Account>(), 100);
        assert_eq!(size_of::<PortfolioLegV16Account>(), 144);
        assert_eq!(size_of::<CloseProgressLedgerV16Account>(), 184);
        assert_eq!(size_of::<HealthCertV16Account>(), 121);
        assert_eq!(size_of::<ResolvedPayoutReceiptV16Account>(), 66);
    }

    #[test]
    fn decode_happy_path() {
        let owner = [7u8; 32];
        let buf = framed(owner, 9, 42);
        let acct = decode_portfolio(&buf).expect("decode");
        assert_eq!(acct.owner(), owner);
        let slot = acct.active_leg_slot_for_asset(9).expect("leg");
        assert_eq!(slot, 3);
        assert_eq!(acct.legs[slot].market_id.get(), 42);
        assert!(acct.active_leg_slot_for_asset(10).is_none());
    }

    #[test]
    fn decode_rejects_bad_header() {
        let owner = [1u8; 32];
        let mut buf = framed(owner, 1, 1);
        buf[0] ^= 0xFF;
        assert_eq!(decode_portfolio(&buf), Err(PortfolioDecodeError::BadMagic));

        let mut buf = framed(owner, 1, 1);
        buf[10] = KIND_PORTFOLIO + 1;
        assert_eq!(decode_portfolio(&buf), Err(PortfolioDecodeError::BadKind));

        let short = vec![0u8; HEADER_LEN + 10];
        assert_eq!(decode_portfolio(&short), Err(PortfolioDecodeError::TooShort));
    }

    #[test]
    fn decode_rejects_owner_mismatch() {
        let mut acct = empty_account();
        acct.provenance_header.owner = [1u8; 32];
        acct.provenance_header.version = V16PodU16::new(V16_ACCOUNT_VERSION);
        acct.provenance_header.layout_discriminator = V16PodU16::new(V16_LAYOUT_DISCRIMINATOR);
        acct.owner = [2u8; 32]; // diverges from provenance owner
        let mut buf = vec![0u8; HEADER_LEN + EXPECTED_PORTFOLIO_ACCOUNT_SIZE];
        buf[0..8].copy_from_slice(&MAGIC.to_le_bytes());
        buf[8..10].copy_from_slice(&VERSION.to_le_bytes());
        buf[10] = KIND_PORTFOLIO;
        buf[HEADER_LEN..].copy_from_slice(bytemuck::bytes_of(&acct));
        assert_eq!(
            decode_portfolio(&buf),
            Err(PortfolioDecodeError::OwnerMismatch)
        );
    }

    #[test]
    fn decode_rejects_bad_provenance() {
        let owner = [3u8; 32];
        let mut acct = empty_account();
        acct.provenance_header.owner = owner;
        acct.owner = owner;
        acct.provenance_header.version = V16PodU16::new(V16_ACCOUNT_VERSION);
        acct.provenance_header.layout_discriminator = V16PodU16::new(99); // wrong
        let mut buf = vec![0u8; HEADER_LEN + EXPECTED_PORTFOLIO_ACCOUNT_SIZE];
        buf[0..8].copy_from_slice(&MAGIC.to_le_bytes());
        buf[8..10].copy_from_slice(&VERSION.to_le_bytes());
        buf[10] = KIND_PORTFOLIO;
        buf[HEADER_LEN..].copy_from_slice(bytemuck::bytes_of(&acct));
        assert_eq!(
            decode_portfolio(&buf),
            Err(PortfolioDecodeError::BadLayoutDiscriminator)
        );
    }

    #[test]
    fn close_and_resolve_gates() {
        let mut acct = empty_account();
        acct.legs[0].active = 1;
        acct.legs[0].asset_index = V16PodU32::new(5);
        assert!(!acct.close_in_progress_for_asset(5));
        acct.close_progress.active = 1;
        acct.close_progress.asset_index = V16PodU32::new(5);
        assert!(acct.close_in_progress_for_asset(5));
        assert!(!acct.close_in_progress_for_asset(6));

        assert!(!acct.has_resolved_receipt());
        acct.resolved_payout_receipt.present = 1;
        assert!(acct.has_resolved_receipt());

        assert!(!acct.portfolio_locked_or_stale());
        acct.liquidation_lock = 1;
        assert!(acct.portfolio_locked_or_stale());
    }
}
