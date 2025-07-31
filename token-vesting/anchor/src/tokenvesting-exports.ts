// Here we export some useful types and functions for interacting with the Anchor program.
import { Account, address, getBase58Decoder, SolanaClient } from 'gill'
import { SolanaClusterId } from '@wallet-ui/react'
import { getProgramAccountsDecoded } from './helpers/get-program-accounts-decoded'
import { EmployeeAccount, VestingAccount, TOKENVESTING_PROGRAM_ADDRESS } from './client/js'
import TokenvestingIDL from '../target/idl/tokenvesting.json'

export type TokenvestingAccount = Account<EmployeeAccount | VestingAccount, string>

// Re-export the generated IDL and type
export { TokenvestingIDL }

// This is a helper function to get the program ID for the Tokenvesting program depending on the cluster.
export function getTokenvestingProgramId(cluster: SolanaClusterId) {
  switch (cluster) {
    case 'solana:devnet':
    case 'solana:testnet':
      // This is the program ID for the Tokenvesting program on devnet and testnet.
      return address('AfJ7jgnc2VQ2tzTrNzVzCrq6VtHi9DhzYFuUUFmh49jF')
    case 'solana:mainnet':
    default:
      return TOKENVESTING_PROGRAM_ADDRESS
  }
}

export { getProgramAccountsDecoded }
export * from './client/js'
