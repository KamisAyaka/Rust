// Here we export some useful types and functions for interacting with the Anchor program.
import { address } from 'gill'
import { SolanaClusterId } from '@wallet-ui/react'
import { VOTEING_DEMO_PROGRAM_ADDRESS } from './client/js'
import VoteingDemoIDL from '../target/idl/voteing.json'

// Re-export the generated IDL and type
export { VoteingDemoIDL }

// This is a helper function to get the program ID for the VoteingDemo program depending on the cluster.
export function getVoteingDemoProgramId(cluster: SolanaClusterId) {
  switch (cluster) {
    case 'solana:devnet':
    case 'solana:testnet':
      // This is the program ID for the VoteingDemo program on devnet and testnet.
      return address('6z68wfurCMYkZG51s1Et9BJEd9nJGUusjHXNt4dGbNNF')
    case 'solana:mainnet':
    default:
      return VOTEING_DEMO_PROGRAM_ADDRESS
  }
}

export * from './client/js'
