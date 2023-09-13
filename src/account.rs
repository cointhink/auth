use rand::RngCore;

#[derive(Debug)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub token: String,
}

pub fn get_nice_rand_str() -> String {
    let mut data: [u8; 14] = [0; 14];
    rand::thread_rng().fill_bytes(&mut data);
    bs58::encode(data)
        .with_alphabet(&bs58::Alphabet::BITCOIN)
        .into_string()
}
