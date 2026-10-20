## Deployment (devnet)

- Vault program: `Gx5it45U9NS2eswoy1J3ahEr4B8FYm6xAFDmBiKozkQA`
- Upgrade authority: `63fGMft5oDwd1vYQJViuzFLfe979Ji4id5b1xpbbdxCL`
- Registration program: `TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM`

## Proof of CPI

- Withdraw tx:
  https://explorer.solana.com/tx/6puKXocwK1PPLbEMuzYJbr12h5R22j9LUeSUNCfcrhqxRhjsNFTdCxu6eV3nivkqhRTobfDBdUAnr1Pgyp53Mfq?cluster=devnet
  Inner instruction shows a CPI to `TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM::initialize`.

- ApplicationAccount PDA: `5VkruHwgBM7agqX4dQvcfHBocKJeT8J8gbY9ZKoPEg12`
  Decoded contents:
  - github: "georgetsolos"
  - user: 63fGMft5oDwd1vYQJViuzFLfe979Ji4id5b1xpbbdxCL

## Changes

- programs/pre-req-vault/src/lib.rs
  - declare_program!(registration) loads the CPI bindings from idls/registration.json.
  - withdraw(amount, github_username) transfers SOL out of the vault and then
    performs a CPI into registration::initialize with the given username.
  - application_account is validated as a PDA with seeds ["prereqs", user]
    under the registration program ID (seeds::program = registration::ID).
  - application_program is checked with require_keys_eq! against registration::ID.
- tests/pre-req-vault.ts
  - Runs initialize → deposit → withdraw("georgetsolos").
  - Asserts the ApplicationAccount PDA exists after withdraw.
