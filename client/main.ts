import {
    LAMPORTS_PER_SOL,
    sendAndConfirmTransaction,
    PublicKey,
    Connection,
    Keypair,
    Transaction,
    TransactionInstruction,
    SYSVAR_RENT_PUBKEY,
    SystemProgram,
    Struct
} from "@solana/web3.js";
import {TOKEN_PROGRAM_ID,ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createAssociatedTokenAccount, AccountLayout, transfer, mintTo, createAssociatedTokenAccountInstruction, createMint, createInitializeAccountInstruction, createInitializeAccount3Instruction, createMintToInstruction} from "@solana/spl-token";
import fs from 'mz/fs';
import os from 'os';
import path from 'path';
import yaml from 'yaml';
import {struct,u32,u8} from "@solana/buffer-layout";
import {bigInt, publicKey, u64} from "@solana/buffer-layout-utils"
import { serialize, deserialize, deserializeUnchecked } from "borsh";
import BN from "bn.js";

class Payload extends Struct {
    constructor(properties : any) {
      super(properties);
    }
  }
  
// class Escrow extends Struct {
//     sendersTokenReceiverAccount : PublicKey
//     isInitialized : number
//     escrowTokenAccount : PublicKey
//     expectedAmount : bigint
//     constructor(fields: {name: string, counter: number} | undefined = undefined) {
//       if (fields) {
//         this.name = fields.name;
//         this.counter = fields.counter;
//       }
//     }
//   }
  
// Path to local Solana CLI config file.
const CONFIG_FILE_PATH = path.resolve(
    os.homedir(),
    '.config',
    'solana',
    'cli',
    'config.yml',
);
const PROGRAM_KEYPAIR_PATH = path.join(
    path.resolve(__dirname,"../dist/program/"),"escrow-keypair.json"
);

const createAccount = async(connection:Connection) : Promise<Keypair> => {
    const key = Keypair.generate();
    const airdrop = await connection.requestAirdrop(key.publicKey,2*LAMPORTS_PER_SOL);
    await connection.confirmTransaction(airdrop)
    return key;
}

const createKeypairFromFile = async(path:string): Promise<Keypair> => {
    const secret_keypair = await fs.readFile(path,{encoding:"utf8"});
    const secret_key = Uint8Array.from(JSON.parse(secret_keypair));
    const programKeypair = Keypair.fromSecretKey(secret_key);
    return programKeypair;
}
/*pub user_sender : Pubkey,
    pub is_initialized : bool,
    pub escrow_token_account : Pubkey,
    pub senders_token_receiver_account : Pubkey,
    pub expected_amount : u64 */

interface escrowWalletData {
    userSender: PublicKey;
    isInitialized : number;
    escrowTokenAccount : PublicKey;
    sendersTokenReceiverAccount : PublicKey;
    expectedAmount : bigint;
} 

const ESCROW_WALLET_LAYOUT = struct<escrowWalletData>([
    publicKey("userSender"),
    u8("isInitialized"),
    publicKey("escrowTokenAccount"),
    publicKey("sendersTokenReceiverAccount"),
    u64("expectedAmount"),
])


const main = async()=>{
    const localenet = "http://127.0.0.1:8899";
    const connection = new Connection(localenet);    
    const programId = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
    const tx = new Transaction();
    console.log("Pinging ... !");
    const alice = await createAccount(connection);
    const mint = Keypair.generate();
    const newTx = new Transaction();
    let escrow_token_account : Keypair , token_account : Keypair;
    const value = new Payload({
        id:0,
        amount: 5
      });
    
    const schema = new Map([
        [
            Payload,
          {
            kind: "struct",
            fields: [
              ["id" , "u8"],
              ["amount", "u64"],
            ],
          },
        ],
      ]);
    try{
        // console.log(alice.publicKey.toString())
        await createMint(connection,alice,alice.publicKey,alice.publicKey,6,mint,undefined,TOKEN_PROGRAM_ID);
        escrow_token_account = Keypair.generate();
        const create_escrow_token_account_tx = SystemProgram.createAccount({
            space: AccountLayout.span,
            lamports: await connection.getMinimumBalanceForRentExemption(
                AccountLayout.span
            ),
            fromPubkey: alice.publicKey,
            newAccountPubkey: escrow_token_account.publicKey,
            programId: TOKEN_PROGRAM_ID,
          })
        const init_escrow_token_account_tx = createInitializeAccountInstruction(escrow_token_account.publicKey,mint.publicKey,alice.publicKey,TOKEN_PROGRAM_ID);
        token_account = Keypair.generate();
        const create_token_account_tx = SystemProgram.createAccount({
            space: AccountLayout.span,
            lamports: await connection.getMinimumBalanceForRentExemption(
                AccountLayout.span
            ),
            fromPubkey: alice.publicKey,
            newAccountPubkey: token_account.publicKey,
            programId: TOKEN_PROGRAM_ID,
          })
        const init_token_account_tx = createInitializeAccount3Instruction(token_account.publicKey,mint.publicKey,alice.publicKey,TOKEN_PROGRAM_ID);        
        newTx.add(create_escrow_token_account_tx);
        newTx.add(init_escrow_token_account_tx);
        newTx.add(create_token_account_tx);
        newTx.add(init_token_account_tx);
        await sendAndConfirmTransaction(connection,newTx,[alice,escrow_token_account,token_account]); 
    }catch(e){
        console.log(e);
        return;
    }

    //mint_to_token_account
    await mintTo(connection,alice,mint.publicKey,token_account.publicKey,alice,5,undefined,undefined,TOKEN_PROGRAM_ID);
    // transfer token to escrow
    await transfer(connection,alice,token_account.publicKey,escrow_token_account.publicKey,alice,5,undefined,undefined,TOKEN_PROGRAM_ID);
    const escrow_wallet = Keypair.generate();
    const create_escrow_wallet_tx = SystemProgram.createAccount(
    {
        space: 32+1+32+32+8,
        lamports: await connection.getMinimumBalanceForRentExemption(
            32+1+32+32+8
        ),
        fromPubkey: alice.publicKey,
        newAccountPubkey: escrow_wallet.publicKey,
        programId: programId.publicKey,
      })
    const transactionInstruction = new TransactionInstruction({
        keys:[
            {pubkey:alice.publicKey,isSigner:true,isWritable:true},
            {pubkey:token_account.publicKey,isSigner:false,isWritable:false},
            {pubkey:escrow_token_account.publicKey,isSigner:false,isWritable:true},
            {pubkey:escrow_wallet.publicKey,isSigner:false,isWritable:true},
            {pubkey:SYSVAR_RENT_PUBKEY,isSigner:false,isWritable:false},
            {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
        ],
        programId:programId.publicKey,
        data:Buffer.from(serialize(schema, value))
    })
    tx.add(create_escrow_wallet_tx,transactionInstruction);
    await sendAndConfirmTransaction(connection,tx,[alice,escrow_wallet]);
    await new Promise((resolve) => setTimeout(resolve, 1000));

    const escrowAccount = await connection.getAccountInfo(
        escrow_wallet.publicKey
    );

    if (escrowAccount === null || escrowAccount.data.length === 0) {
        console.log("Escrow state account has not been initialized properly");
        process.exit(1);
    }
    console.log("///////// sent ///////////");
    console.log("token account : ",token_account.publicKey.toString());
    console.log("escrow token account : ",escrow_token_account.publicKey.toString());
    console.log("sender account : ",alice.publicKey.toString());
    const data = ESCROW_WALLET_LAYOUT.decode(escrowAccount.data);
    console.log("/////// received //////");
    console.log("token account : ",data.sendersTokenReceiverAccount.toString())
    console.log("escrow token account : ",data.escrowTokenAccount.toString())
    console.log("sender account :" , data.userSender.toString())

    console.log("/////// Bob transactions ///////////");

    const bob = await createAccount(connection);
    const bob_token_account = Keypair.generate();
    const bob_token_account_inst = SystemProgram.createAccount({
        space: AccountLayout.span,
        lamports: await connection.getMinimumBalanceForRentExemption(
            AccountLayout.span
        ),
        fromPubkey: bob.publicKey,
        newAccountPubkey: bob_token_account.publicKey,
        programId: TOKEN_PROGRAM_ID,
    })
    const value2 = new Payload({
        id:1,
        amount: 5
      });
    const [pda,bump] = await PublicKey.findProgramAddress([Buffer.from("token")],programId.publicKey);
    const init_bob_token_account_inst = createInitializeAccountInstruction(bob_token_account.publicKey,mint.publicKey,bob.publicKey,TOKEN_PROGRAM_ID);
    const newTx2 = new Transaction();
    newTx2.add(bob_token_account_inst,init_bob_token_account_inst);
    await sendAndConfirmTransaction(connection,newTx2,[bob,bob_token_account])
    await mintTo(connection,bob,mint.publicKey,bob_token_account.publicKey,alice.publicKey,6,[alice],undefined,TOKEN_PROGRAM_ID);
    const compelete_escrow_tx = new TransactionInstruction({
        keys:[
            {pubkey:bob.publicKey,isSigner:true,isWritable:false},
            {pubkey:bob_token_account.publicKey,isSigner:false,isWritable:true},
            {pubkey:bob_token_account.publicKey,isSigner:false,isWritable:true},
            {pubkey:escrow_wallet.publicKey,isSigner:false,isWritable:true},
            {pubkey:TOKEN_PROGRAM_ID,isSigner:false,isWritable:false},
            {pubkey:pda,isSigner:false,isWritable:false},
            {pubkey:escrow_token_account.publicKey,isSigner:false,isWritable:true},
            {pubkey:token_account.publicKey,isSigner:false,isWritable:true},
            {pubkey:alice.publicKey,isSigner:false,isWritable:true},
        ],
        programId:programId.publicKey,
        data:Buffer.from(serialize(schema, value2))
    });
    const tx2 = new Transaction();
    tx2.add(compelete_escrow_tx);
    await sendAndConfirmTransaction(connection,tx2,[bob]);
    await new Promise((resolve) => setTimeout(resolve, 1000));//wait 
    const alice_token_account_after_exch = await connection.getTokenAccountBalance(token_account.publicKey); // initial 5 - sent 5 + received 5 = final 5 tokens
    const bob_token_account_after_exch = await connection.getTokenAccountBalance(bob_token_account.publicKey); // initial 6 - sent 5 + received 5 = final 6 tokens
    console.log((alice_token_account_after_exch.value))
    console.log((bob_token_account_after_exch.value))
}

main().then(
    ()=>process.exit(),
    err =>{
        console.log(err);
        process.exit(-1);
    }
)