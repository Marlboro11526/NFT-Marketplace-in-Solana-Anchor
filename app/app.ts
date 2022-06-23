import * as anchor from '@project-serum/anchor';
import { Program, Wallet } from '@project-serum/anchor';

import { NftMarketplace } from '../target/types/nft_marketplace';

import {
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  MINT_SIZE,
} from '@solana/spl-token';

// Our NFT details
const nftMetadataUri =
  'https://raw.githubusercontent.com/JohnBashombe/NFT-Marketplace-in-Solana---Anchor/main/nft-example.json';
const nftTitle = 'Artist NFT';

// Anchor config
const provider = anchor.AnchorProvider.env();
const wallet = provider.wallet as Wallet;
anchor.setProvider(provider);

const program = anchor.workspace.NftMarketplace as Program<NftMarketplace>;
const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
);

const main = async () => {
  // create the NFT token and the associated token account that will hold it

  const mintKeypair: anchor.web3.Keypair = anchor.web3.Keypair.generate();

  const nftTokenAccount = await getAssociatedTokenAddress(
    mintKeypair.publicKey,
    wallet.publicKey
  );

  const requiredLamports: number =
    await program.provider.connection.getMinimumBalanceForRentExemption(
      MINT_SIZE
    );

  await program.provider.sendAndConfirm(
    new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: mintKeypair.publicKey,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
        lamports: requiredLamports,
      }),
      createInitializeMintInstruction(
        mintKeypair.publicKey,
        0,
        wallet.publicKey,
        wallet.publicKey
      ),
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        nftTokenAccount,
        wallet.publicKey,
        mintKeypair.publicKey
      )
    ),
    [mintKeypair]
  );

  // Config the NFT token's associated metadata & mint it to the recipient
  const metadataAddress = (
    await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    )
  )[0];

  const masterEditionAddress = (
    await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
        Buffer.from('edition'),
      ],
      TOKEN_METADATA_PROGRAM_ID
    )
  )[0];

  await program.methods
    .mintNft(mintKeypair.publicKey, nftMetadataUri, nftTitle)
    .accounts({
      masterEdition: masterEditionAddress,
      metadata: metadataAddress,
      mint: mintKeypair.publicKey,
      mintAuthority: wallet.publicKey,
      payer: wallet.publicKey,
      tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      tokenAccount: nftTokenAccount,
    })
    .rpc();
};

main().then(
  () => process.exit(),
  (err) => {
    console.log(err);
    process.exit(-1);
  }
);
