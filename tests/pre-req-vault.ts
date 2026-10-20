import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { PreReqVault } from "../target/types/pre_req_vault";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

const REGISTRATION_PROGRAM_ID = new PublicKey(
  "TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM"
);
const GITHUB_USERNAME = "georgetsolos";

describe("pre-req-vault", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.preReqVault as Program<PreReqVault>;

  it("withdraw triggers registration CPI", async () => {
    const user = provider.wallet.publicKey;

    const [vaultState] = PublicKey.findProgramAddressSync(
      [Buffer.from("state"), user.toBuffer()],
      program.programId
    );
    const [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vaultState.toBuffer()],
      program.programId
    );
    const [applicationAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("prereqs"), user.toBuffer()],
      REGISTRATION_PROGRAM_ID
    );

    const stateExists = await provider.connection.getAccountInfo(vaultState);
    if (!stateExists) {
      const sig = await program.methods
        .initialize()
        .accountsPartial({ user, vaultState, vault, systemProgram: SystemProgram.programId })
        .rpc();
      console.log("Initialize tx:", sig);
    }

    const depositSig = await program.methods
      .deposit(new anchor.BN(1_000_000_000))
      .accountsPartial({ user, vaultState, vault, systemProgram: SystemProgram.programId })
      .rpc();
    console.log("Deposit tx:", depositSig);

    const withdrawSig = await program.methods
      .withdraw(new anchor.BN(500_000_000), GITHUB_USERNAME)
      .accountsPartial({
        user,
        vaultState,
        vault,
        applicationAccount,
        applicationProgram: REGISTRATION_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("Withdraw tx (contains CPI):", withdrawSig);

    const info = await provider.connection.getAccountInfo(applicationAccount);
    assert.isNotNull(info, "ApplicationAccount was not created by the CPI");
  });
});
