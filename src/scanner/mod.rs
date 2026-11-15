use crate::domain::BssBruto;

pub trait Scanner {
    fn escanear(&self) -> Result<Vec<BssBruto>, String>;
}

#[cfg(windows)]
pub mod wlan;

pub mod simulado;
