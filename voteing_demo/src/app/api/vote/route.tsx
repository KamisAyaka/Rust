import { ActionGetResponse, ActionPostRequest, ACTIONS_CORS_HEADERS, createPostResponse } from '@solana/actions'
import { Connection, PublicKey, Transaction } from '@solana/web3.js'
import { Voteing } from '@/../anchor/target/types/voteing'
import { Program } from '@coral-xyz/anchor'
import { BN } from '@coral-xyz/anchor'
const IDL = require('../../../../anchor/target/idl/voteing.json')

export const OPTIONS = GET

export async function GET(request: Request) {
  const actionMetadata: ActionGetResponse = {
    icon: 'https://media.istockphoto.com/id/534129810/photo/textured-rainbow-painted-background.jpg?s=1024x1024&w=is&k=20&c=VQR3_x8kJcP3qBzrWeUj7ZwSt2G2QqAnBggDtR0Pix4=',
    title: 'Vote for your favorite picture',
    description: 'Vote between 2 pictures',
    label: 'Vote',
    links: {
      actions: [
        {
          label: 'Vote for Smooth',
          href: '/api/vote?candidate=Smooth',
          type: 'post',
        },
        {
          label: 'Vote for Drity',
          href: '/api/vote?candidate=Drity',
          type: 'post',
        },
      ],
    },
  }
  return Response.json(actionMetadata, { headers: ACTIONS_CORS_HEADERS })
}

export async function POST(request: Request) {
  const url = new URL(request.url)
  const candidate = url.searchParams.get('candidate')

  if (candidate != 'Smooth' && candidate != 'Drity') {
    return Response.json({ error: 'Invalid candidate' }, { status: 400, headers: ACTIONS_CORS_HEADERS })
  }

  const connection = new Connection('http://127.0.0.1:8899', 'confirmed')
  const program: Program<Voteing> = new Program(IDL, { connection })

  const body: ActionPostRequest = await request.json()
  let voter

  try {
    voter = new PublicKey(body.account)
  } catch (error) {
    return Response.json({ error: 'Invalid account' }, { status: 400, headers: ACTIONS_CORS_HEADERS })
  }

  const instruction = await program.methods.vote(candidate, new BN(1)).accounts({ authority: voter }).instruction()

  const blockhash = await connection.getLatestBlockhash()

  const transaction = new Transaction({
    feePayer: voter,
    blockhash: blockhash.blockhash,
    lastValidBlockHeight: blockhash.lastValidBlockHeight,
  }).add(instruction)

  const response = await createPostResponse({
    fields: {
      transaction: transaction,
      type: 'transaction',
    },
  })
  return Response.json(response, { headers: ACTIONS_CORS_HEADERS })
}
