
pub enum EscrowInstruction{
    /// Starts the trade by creating a escrow account owned by program and storing the amount of tokens that are getting exchanged
    /// accounts required :
    /// 0 - [signer] Account which is starting the escrow trade
    /// 1 - [writer] Token Account which will be sending Y tokens to escrow
    /// 2 - [] Account of user that will be receiving the Y tokens
    /// 3 - [writer] escrow account which will hold all info and tokens
    /// 4 - [rent] rent sysvar
    /// 5 - [] token program
    Initialize{
        amount : u64
    }
}

impl EscrowInstruction {
   pub fn unpack(input:&[u8]) -> Result<self,ProgramError> {
        let (tag,rest) = input.split_first().ok_or(InvalidInstruction);
        Ok(match tag {
            0 => self::Initialize{
                amount : self::unpack_amount(rest)?,
            },
            _ => return Err(InvalidData)
        })
   }

   pub fn unpack_amount(input:&[u8]) -> Result<u64,ProgramError> {
        let amount = input.get(..8).and_then(|slice| slice.try_into().ok()).map(u64::from_le_bytes).ok_or(InvalidInstruction)?;
        Ok(amount)
   } 
}
