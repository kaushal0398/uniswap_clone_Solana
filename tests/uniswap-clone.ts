import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniswapClone } from "../target/types/uniswap_clone";
import { expect } from "chai";

describe("uniswap-clone", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.UniswapClone as Program<UniswapClone>;

  const pool = anchor.web3.Keypair.generate();
  const user = anchor.web3.Keypair.generate();

  it("Initializes a pool", async () => {
    const connection = provider.connection;
    await connection.confirmTransaction(
      await connection.requestAirdrop(user.publicKey, 2e9), 
      "confirmed"
    );

    const initialTokenA = new anchor.BN(1000);
    const initialTokenB = new anchor.BN(500);

    await program.methods
      .initializePool(initialTokenA, initialTokenB)
      .accounts({
        pool: pool.publicKey,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([pool, user])
      .rpc();

    const poolAccount = await program.account.pool.fetch(pool.publicKey);

    expect(poolAccount.tokenAReserve.toNumber()).to.equal(1000);
    expect(poolAccount.tokenBReserve.toNumber()).to.equal(500);
    expect(poolAccount.totalLpSupply.toNumber()).to.equal(0); 
  });

  it("Adds liquidity to the pool", async () => {
    const tokenAAmount = new anchor.BN(200);
    const tokenBAmount = new anchor.BN(100);

    await program.methods
      .addLiquidity(tokenAAmount, tokenBAmount)
      .accounts({
        pool: pool.publicKey,
        user: user.publicKey,
      })
      .signers([user])
      .rpc();

    const poolAccount = await program.account.pool.fetch(pool.publicKey);

    expect(poolAccount.tokenAReserve.toNumber()).to.equal(1200); 
    expect(poolAccount.tokenBReserve.toNumber()).to.equal(600);  
    expect(poolAccount.totalLpSupply.toNumber()).to.be.greaterThan(0); 
  });

  it("Swaps tokens in the pool", async () => {
    const swapAmountIn = new anchor.BN(50);
    const minAmountOut = new anchor.BN(20); 

    await program.methods
      .swap(swapAmountIn, minAmountOut, true) 
      .accounts({
        pool: pool.publicKey,
        user: user.publicKey,
      })
      .signers([user])
      .rpc();

    const poolAccount = await program.account.pool.fetch(pool.publicKey);

    // Assertions
    expect(poolAccount.tokenAReserve.toNumber()).to.be.greaterThan(1200); 
    expect(poolAccount.tokenBReserve.toNumber()).to.be.lessThan(600);    
  });

  it("Removes liquidity from the pool", async () => {
    const lpTokensToBurn = new anchor.BN(50); 

    await program.methods
      .removeLiquidity(lpTokensToBurn)
      .accounts({
        pool: pool.publicKey,
        user: user.publicKey,
      })
      .signers([user])
      .rpc();

    const poolAccount = await program.account.pool.fetch(pool.publicKey);

    expect(poolAccount.totalLpSupply.toNumber()).to.be.lessThan(100); 
    expect(poolAccount.tokenAReserve.toNumber()).to.be.lessThan(1200);
    expect(poolAccount.tokenBReserve.toNumber()).to.be.lessThan(600);  
  });
});

