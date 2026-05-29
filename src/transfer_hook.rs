//! Token-2022 TransferHook implementation.
//!
//! A.3 FAIL-CLOSE: The real ownership-transfer CPI into the wrapper
//! (`TransferPortfolioOwnership`, tag B-3) is wired in A.4. Until then,
//! `process_execute` unconditionally rejects so no NFT transfer can complete
//! without the ownership-reassignment CPI. This prevents owner-desync between
//! the NFT token and the portfolio account.

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::NftError;

// ═══════════════════════════════════════════════════════════════
// SPL TransferHook interface constants
// ═══════════════════════════════════════════════════════════════

/// Discriminator for the TransferHook `Execute` instruction.
/// SHA256("spl-transfer-hook-interface:execute")[:8]
pub const EXECUTE_DISCRIMINATOR: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];

/// PDA seed for the ExtraAccountMetaList account.
pub const EXTRA_METAS_SEED: &[u8] = b"extra-account-metas";

/// Derive the ExtraAccountMetaList PDA for a given mint.
pub fn extra_account_metas_pda(mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[EXTRA_METAS_SEED, mint.as_ref()], program_id)
}

// ═══════════════════════════════════════════════════════════════
// Execute — called by Token-2022 on every NFT transfer
// ═══════════════════════════════════════════════════════════════

/// Process the TransferHook Execute instruction.
///
/// A.3 FAIL-CLOSE: rejects all transfers until A.4 wires the
/// TransferPortfolioOwnership CPI. Any NFT transfer will fail with
/// `TransferBlocked` so that the portfolio account's owner field stays in
/// sync with the NFT holder.
pub fn process_execute(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _amount: u64,
) -> ProgramResult {
    msg!("transfer-hook: ownership transfer not yet wired (A.4) — transfers disabled");
    Err(ProgramError::from(NftError::TransferBlocked))
}
