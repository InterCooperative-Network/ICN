mod blockchain;
mod did;

use blockchain::Block;
use did::DID;
use secp256k1::SecretKey;

fn main() {
    // Generate the Genesis Block
    let genesis_block = Block::new(0, String::from("0"), String::from("Genesis Block"));
    println!("Genesis Block: {:?}", genesis_block);

    // Generate a random DID
    let (new_did, secret_key) = DID::generate_random(String::from("did:icn:001"));
    println!("New DID: {:?}", new_did);
    println!("Secret Key (Keep this secure!): {:?}", secret_key);
}
