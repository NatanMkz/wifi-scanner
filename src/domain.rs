#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Risco {
    Bom,
    Atencao,
    Ruim,
}


#[derive(Clone, Debug)]
pub struct BssBruto {
    pub ssid: Vec<u8>,
    pub bssid: [u8; 6],
    pub rssi: i32,    
    pub freq_khz: u32,  
    pub capability: u16,  
    pub ies: Vec<u8>,   
}

#[derive(Clone, Debug)]
pub struct Ap {
    pub ssid: String,
    pub bssid: String,
    pub rssi: i32,
    pub canal: u32,
    pub banda: &'static str,
    pub seguranca: String,
    pub risco: Risco,
    pub pmf: bool,
    pub wps: bool,
    pub roaming: String,
    pub padrao: &'static str,
    pub pais: String,
}
