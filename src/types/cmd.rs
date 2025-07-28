use std::fmt;
use std::fmt::Display;

pub enum Command {
    MU,
    MD,
    S,
    DO,
    DC,
    CP(u8),
    CU(u8),
    CD(u8),
    IU(u8),
    ID(u8),
    CI(u8),
    R,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::MU => write!(f, "MU"),
            Command::MD => write!(f, "MD"),
            Command::S => write!(f, "S"),
            Command::DO => write!(f, "DO"),
            Command::DC => write!(f, "DC"),
            Command::R => write!(f, "R"),
            Command::CP(v) => write!(f, "CP{v}"),
            Command::CU(v) => write!(f, "CU{v}"),
            Command::CD(v) => write!(f, "CD{v}"),
            Command::IU(v) => write!(f, "IU{v}"),
            Command::ID(v) => write!(f, "ID{v}"),
            Command::CI(v) => write!(f, "CI{v}"),
        }
    }
}
