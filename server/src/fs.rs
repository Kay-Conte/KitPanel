use std::{
    fs::File,
    io::{self, ErrorKind, Read, Write},
    path::PathBuf,
};

pub trait Config: Default {
    fn rel_path(rel: PathBuf) -> PathBuf;

    fn full_path() -> PathBuf {
        Self::rel_path(
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_owned(),
        )
    }

    fn bytes(&self) -> Vec<u8>;

    fn from_bytes(bytes: &[u8]) -> Option<Self>;

    fn get() -> std::io::Result<Self> {
        match File::open(Self::full_path()) {
            Ok(mut file) => {
                let mut buf = vec![];

                file.read_to_end(&mut buf)?;

                Self::from_bytes(&buf).ok_or(io::Error::new(
                    ErrorKind::Other,
                    "Failed to parse object from bytes",
                ))
            }

            Err(_) => {
                let config = Self::default();

                config.save()?;

                Ok(config)
            }
        }
    }

    fn save(&self) -> std::io::Result<()> {
        File::create(Self::full_path())?.write_all(&self.bytes())
    }
}
