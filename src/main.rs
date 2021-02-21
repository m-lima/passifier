// /// Possible backends for the secret store
// pub enum Backend {
//     /// File based secret store
//     File(std::path::PathBuf),
// }

// impl Backend {
//     fn load(&self) -> Result<Vec<u8>> {
//         match self {
//             Self::File(path) => {
//                 use std::io::Read;

//                 let mut file = std::fs::File::open(path).map_err(Error::IO)?;
//                 let mut buffer = Vec::new();
//                 file.read_to_end(&mut buffer).map_err(Error::IO)?;
//                 Ok(buffer)
//             }
//         }
//     }

//     fn save(&self, buffer: &[u8]) -> Result<()> {
//         match self {
//             Self::File(path) => {
//                 use std::io::Write;

//                 std::fs::File::create(path)
//                     .map_err(Error::IO)?
//                     .write_all(buffer)
//                     .map_err(Error::IO)
//                     .map(|_| ())
//             }
//         }
//     }
// }

fn main() {
    store::Store::load(&[], "").unwrap();
    println!("Hello!");
}
