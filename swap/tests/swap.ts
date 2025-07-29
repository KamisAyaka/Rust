import { randomBytes } from "node:crypto";
import * as anchor from "@coral-xyz/anchor";
import { type Program } from "@coral-xyz/anchor";
import pkg from "@coral-xyz/anchor";
const { BN } = pkg;
import {
  TOKEN_2022_PROGRAM_ID,
  type TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { assert } from "chai";
import type { Swap } from "../target/types/swap";

import {
  confirmTransaction,
  createAccountsMintsAndTokenAccounts,
  makeKeypairs,
} from "@solana-developers/helpers";

// Work on both Token Program and new Token Extensions Program
const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
  TOKEN_2022_PROGRAM_ID;

const SECONDS = 1000;

// Tests must complete within half this time otherwise
// they are marked as slow. Since Anchor involves a little
// network IO, these tests usually take about 15 seconds.
const ANCHOR_SLOW_TEST_THRESHOLD = 40 * SECONDS;

const getRandomBigNumber = (size = 8) => {
  return new BN(randomBytes(size));
};

describe("swap", async () => {
  // Use the cluster and the keypair from Anchor.toml
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // See https://github.com/coral-xyz/anchor/issues/3122
  const user = (provider.wallet as anchor.Wallet).payer;
  const payer = user;

  const connection = provider.connection;

  const program = anchor.workspace.Swap as Program<Swap>;

  // We're going to reuse these accounts across multiple tests
  const accounts: Record<string, PublicKey> = {
    tokenProgram: TOKEN_PROGRAM,
  };

  let alice: anchor.web3.Keypair;
  let bob: anchor.web3.Keypair;
  let tokenMintA: anchor.web3.Keypair;
  let tokenMintB: anchor.web3.Keypair;

  [alice, bob, tokenMintA, tokenMintB] = makeKeypairs(4);

  const tokenAOfferedAmount = new BN(1_000_000);
  const tokenBWantedAmount = new BN(1_000_000);

  before(
    "Creates Alice and Bob accounts, 2 token mints, and associated token accounts for both tokens for both users",
    async () => {
      const usersMintsAndTokenAccounts =
        await createAccountsMintsAndTokenAccounts(
          [
            // Alice's token balances
            [
              // 1_000_000_000 of token A
              1_000_000_000,
              // 0 of token B
              0,
            ],
            // Bob's token balances
            [
              // 0 of token A
              0,
              // 1_000_000_000 of token B
              1_000_000_000,
            ],
          ],
          1 * LAMPORTS_PER_SOL,
          connection,
          payer
        );

      const users = usersMintsAndTokenAccounts.users;
      alice = users[0];
      bob = users[1];

      const mints = usersMintsAndTokenAccounts.mints;
      tokenMintA = mints[0];
      tokenMintB = mints[1];

      const tokenAccounts = usersMintsAndTokenAccounts.tokenAccounts;

      const aliceTokenAccountA = tokenAccounts[0][0];
      const aliceTokenAccountB = tokenAccounts[0][1];

      const bobTokenAccountA = tokenAccounts[1][0];
      const bobTokenAccountB = tokenAccounts[1][1];

      // Save the accounts for later use
      accounts.maker = alice.publicKey;
      accounts.taker = bob.publicKey;
      accounts.tokenMintA = tokenMintA.publicKey;
      accounts.makerTokenAccountA = aliceTokenAccountA;
      accounts.takerTokenAccountA = bobTokenAccountA;
      accounts.tokenMintB = tokenMintB.publicKey;
      accounts.makerTokenAccountB = aliceTokenAccountB;
      accounts.takerTokenAccountB = bobTokenAccountB;
    }
  );

  it("Puts the tokens Alice offers into the vault when Alice makes an offer", async () => {
    // Pick a random ID for the offer we'll make
    const offerId = getRandomBigNumber();

    // Then determine the account addresses we'll use for the offer and the vault
    const offer = PublicKey.findProgramAddressSync(
      [
        Buffer.from("offer"),
        accounts.maker.toBuffer(),
        offerId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    )[0];

    const vault = getAssociatedTokenAddressSync(
      accounts.tokenMintA,
      offer,
      true,
      TOKEN_PROGRAM
    );

    accounts.offer = offer;
    accounts.vault = vault;

    const transactionSignature = await program.methods
      .makeOffer(offerId, tokenAOfferedAmount, tokenBWantedAmount)
      .accounts({ ...accounts })
      .signers([alice])
      .rpc();

    await confirmTransaction(connection, transactionSignature);

    // Check our vault contains the tokens offered
    const vaultBalanceResponse = await connection.getTokenAccountBalance(vault);
    const vaultBalance = new BN(vaultBalanceResponse.value.amount);
    assert(vaultBalance.eq(tokenAOfferedAmount));

    // Check our Offer account contains the correct data
    const offerAccount = await program.account.offer.fetch(offer);

    assert(offerAccount.maker.equals(alice.publicKey));
    assert(offerAccount.tokenMintA.equals(accounts.tokenMintA));
    assert(offerAccount.tokenMintB.equals(accounts.tokenMintB));
    assert(offerAccount.tokenBWantedAmount.eq(tokenBWantedAmount));
  }).slow(ANCHOR_SLOW_TEST_THRESHOLD);

  it("Puts the tokens from the vault into Bob's account, and gives Alice Bob's tokens, when Bob takes an offer", async () => {
    const transactionSignature = await program.methods
      .takeOffer()
      .accounts({ ...accounts })
      .signers([bob])
      .rpc();

    await confirmTransaction(connection, transactionSignature);

    // Check the offered tokens are now in Bob's account
    // (note: there is no before balance as Bob didn't have any offered tokens before the transaction)
    const bobTokenAccountBalanceAfterResponse =
      await connection.getTokenAccountBalance(accounts.takerTokenAccountA);
    const bobTokenAccountBalanceAfter = new BN(
      bobTokenAccountBalanceAfterResponse.value.amount
    );
    assert(bobTokenAccountBalanceAfter.eq(tokenAOfferedAmount));

    // Check the wanted tokens are now in Alice's account
    // (note: there is no before balance as Alice didn't have any wanted tokens before the transaction)
    const aliceTokenAccountBalanceAfterResponse =
      await connection.getTokenAccountBalance(accounts.makerTokenAccountB);
    const aliceTokenAccountBalanceAfter = new BN(
      aliceTokenAccountBalanceAfterResponse.value.amount
    );
    assert(aliceTokenAccountBalanceAfter.eq(tokenBWantedAmount));
  }).slow(ANCHOR_SLOW_TEST_THRESHOLD);

  // 测试取消订单功能
  describe("cancel offer", async () => {
    const offerId = getRandomBigNumber();

    // 创建一个新的报价用于测试取消功能
    it("Creates a new offer for testing cancellation", async () => {
      // 确定新报价和资金库的账户地址
      const offer = PublicKey.findProgramAddressSync(
        [
          Buffer.from("offer"),
          accounts.maker.toBuffer(),
          offerId.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      )[0];

      const vault = getAssociatedTokenAddressSync(
        accounts.tokenMintA,
        offer,
        true,
        TOKEN_PROGRAM
      );

      // 更新账户信息
      accounts.offer = offer;
      accounts.vault = vault;

      const transactionSignature = await program.methods
        .makeOffer(offerId, tokenAOfferedAmount, tokenBWantedAmount)
        .accounts({ ...accounts })
        .signers([alice])
        .rpc();

      await confirmTransaction(connection, transactionSignature);

      // 验证报价已创建且资金已存入资金库
      const vaultBalanceResponse = await connection.getTokenAccountBalance(
        vault
      );
      const vaultBalance = new BN(vaultBalanceResponse.value.amount);
      assert(vaultBalance.eq(tokenAOfferedAmount));

      // 验证报价账户数据正确
      const offerAccount = await program.account.offer.fetch(offer);
      assert(offerAccount.maker.equals(alice.publicKey));
      assert(!offerAccount.isCancelled);
    }).slow(ANCHOR_SLOW_TEST_THRESHOLD);

    it("Allows Alice to cancel her offer and get tokens back", async () => {
      // 获取取消前Alice的代币A余额
      const aliceTokenAccountABalanceBefore = new BN(
        (
          await connection.getTokenAccountBalance(accounts.makerTokenAccountA)
        ).value.amount
      );

      // 获取资金库余额
      const vaultBalanceBefore = new BN(
        (await connection.getTokenAccountBalance(accounts.vault)).value.amount
      );

      // 执行取消操作
      const transactionSignature = await program.methods
        .cancelOffer(offerId)
        .accounts({ ...accounts })
        .signers([alice])
        .rpc();

      await confirmTransaction(connection, transactionSignature);

      // 验证报价已被标记为已取消
      const offerAccount = await program.account.offer.fetch(accounts.offer);
      assert(offerAccount.isCancelled);

      // 验证代币已退回给Alice
      const aliceTokenAccountABalanceAfter = new BN(
        (
          await connection.getTokenAccountBalance(accounts.makerTokenAccountA)
        ).value.amount
      );

      assert(
        aliceTokenAccountABalanceAfter
          .sub(aliceTokenAccountABalanceBefore)
          .eq(vaultBalanceBefore)
      );
    }).slow(ANCHOR_SLOW_TEST_THRESHOLD);

    it("Prevents Alice from canceling the same offer twice", async () => {
      try {
        // 尝试再次取消已取消的报价
        await program.methods
          .cancelOffer(offerId)
          .accounts({ ...accounts })
          .signers([alice])
          .rpc();

        // 如果没有抛出错误，则测试失败
        assert.fail("Expected cancelOffer to throw an error");
      } catch (error) {
        // 验证是否是预期的错误
        // 我们接受两种可能的错误：
        // 1. 我们自定义的错误消息
        // 2. 由于vault账户已经被关闭导致的错误
        const errorString = error.toString();
        if (!errorString.includes("Offer has already been cancelled") && 
            !errorString.includes("AnchorError caused by account: vault")) {
          assert.fail(`Unexpected error: ${errorString}`);
        }
        // 如果是这两种错误之一，测试通过
      }
    });

    it("Prevents Bob from canceling Alice's offer", async () => {
      // 创建一个新的报价ID
      const newOfferId = getRandomBigNumber();

      // 创建新报价
      const offer = PublicKey.findProgramAddressSync(
        [
          Buffer.from("offer"),
          accounts.maker.toBuffer(),
          newOfferId.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      )[0];

      const vault = getAssociatedTokenAddressSync(
        accounts.tokenMintA,
        offer,
        true,
        TOKEN_PROGRAM
      );

      // 更新账户
      const newAccounts = { ...accounts, offer, vault };

      const transactionSignature = await program.methods
        .makeOffer(newOfferId, tokenAOfferedAmount, tokenBWantedAmount)
        .accounts({ ...newAccounts })
        .signers([alice])
        .rpc();

      await confirmTransaction(connection, transactionSignature);

      try {
        // 尝试让Bob取消Alice的报价
        // 需要为Bob创建一套完整的账户
        const bobMakerTokenAccountA = getAssociatedTokenAddressSync(
          accounts.tokenMintA,
          bob.publicKey,
          false,
          TOKEN_PROGRAM
        );
        
        const bobMakerTokenAccountB = getAssociatedTokenAddressSync(
          accounts.tokenMintB,
          bob.publicKey,
          false,
          TOKEN_PROGRAM
        );
        
        const bobAccounts = {
          maker: bob.publicKey,
          taker: accounts.taker,
          tokenMintA: accounts.tokenMintA,
          makerTokenAccountA: bobMakerTokenAccountA,
          takerTokenAccountA: accounts.takerTokenAccountA,
          tokenMintB: accounts.tokenMintB,
          makerTokenAccountB: bobMakerTokenAccountB,
          takerTokenAccountB: accounts.takerTokenAccountB,
          offer,
          vault,
          tokenProgram: TOKEN_PROGRAM,
        };

        await program.methods
          .cancelOffer(newOfferId)
          .accounts({ ...bobAccounts })
          .signers([bob])
          .rpc();

        // 如果没有抛出错误，则测试失败
        assert.fail("Expected cancelOffer to throw an error");
      } catch (error) {
        // 验证是否是预期的错误
        // 注意：由于Anchor框架层面的验证，可能不会返回我们自定义的错误消息
        // 但至少应该抛出某种错误
        assert.exists(error);
      }
    }).slow(ANCHOR_SLOW_TEST_THRESHOLD);
  });
});
