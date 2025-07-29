import { Program } from '@coral-xyz/anchor'
import * as anchor from '@coral-xyz/anchor'
import { PublicKey } from '@solana/web3.js'
import { Voteing } from '../target/types/voteing'
import { BankrunProvider, startAnchor } from 'anchor-bankrun'

const IDL = require('../target/idl/voteing.json')

const voteingAddress = new PublicKey('7qndvv9MS9WWNRZctz3gVVYaaDhv3VsT4QhgAAsaA573')

describe('voteing_demo', () => {
  let context
  let provider
  anchor.setProvider(anchor.AnchorProvider.env())
  let voteingProgram = anchor.workspace.Voteing as Program<Voteing>

  beforeAll(async () => {
    // context = await startAnchor('.', [{ name: 'voteing_demo', programId: voteingAddress }], [])
    // provider = new BankrunProvider(context)
    // voteingProgram = new Program<Voteing>(IDL, provider)

    // 初始化 poll 账户
    await voteingProgram.methods.initializePoll(new anchor.BN(1), 'test', new anchor.BN(0), new anchor.BN(115460)).rpc()

    // 初始化候选者账户
    await voteingProgram.methods.initializeCandidate('Smooth', new anchor.BN(1)).rpc()
    await voteingProgram.methods.initializeCandidate('Drity', new anchor.BN(1)).rpc()
  })

  it('Initialize', async () => {
    const [pollAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, 'le', 8)],
      voteingAddress,
    )

    const poll = await voteingProgram.account.poll.fetch(pollAddress)
    console.log(poll)

    expect(poll.pollId.toNumber()).toEqual(1)
    expect(poll.description).toEqual('test')
    expect(poll.pollStart.toNumber()).toBeLessThan(poll.pollEnd.toNumber())
  })

  it('initialize candidate', async () => {
    const [SmoothAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, 'le', 8), Buffer.from('Smooth')],
      voteingAddress,
    )
    const SmoothCandidate = await voteingProgram.account.candidate.fetch(SmoothAddress)
    console.log(SmoothCandidate)
    expect(SmoothCandidate.candidateVotes.toNumber()).toEqual(0)
    expect(SmoothCandidate.candidateName).toEqual('Smooth')

    const [DrityAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, 'le', 8), Buffer.from('Drity')],
      voteingAddress,
    )
    const DrityCandidate = await voteingProgram.account.candidate.fetch(DrityAddress)
    console.log(DrityCandidate)
    expect(DrityCandidate.candidateVotes.toNumber()).toEqual(0)
  })

  it('vote', async () => {
    // 投票
    await voteingProgram.methods.vote('Smooth', new anchor.BN(1)).rpc()

    const [SmoothAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, 'le', 8), Buffer.from('Smooth')],
      voteingAddress,
    )
    const SmoothCandidate = await voteingProgram.account.candidate.fetch(SmoothAddress)
    console.log(SmoothCandidate)
    expect(SmoothCandidate.candidateVotes.toNumber()).toEqual(1) // 正确调用 .toNumber()
    expect(SmoothCandidate.candidateName).toEqual('Smooth')
  })
})
