//! Scanner falso, com dados fixos. Serve para testar a interface em qualquer
//! sistema e para desenvolver sem depender do hardware (`--demo`).

use super::Scanner;
use crate::domain::BssBruto;

pub struct SimuladoScanner;

fn tlv(id: u8, corpo: &[u8]) -> Vec<u8> {
    let mut v = vec![id, corpo.len() as u8];
    v.extend_from_slice(corpo);
    v
}

/// IE RSN (ID 48) com cifra CCMP e um único AKM.
fn rsn(akm: u8, caps: u16) -> Vec<u8> {
    let mut c = vec![
        1, 0, // versão
        0x00, 0x0F, 0xAC, 4, // cifra de grupo: CCMP
        1, 0, 0x00, 0x0F, 0xAC, 4, // cifra pairwise: CCMP
        1, 0, 0x00, 0x0F, 0xAC, akm, // AKM
    ];
    c.extend_from_slice(&caps.to_le_bytes());
    tlv(48, &c)
}

fn wps() -> Vec<u8> {
    tlv(221, &[0x00, 0x50, 0xF2, 4, 0x10, 0x4A])
}

impl Scanner for SimuladoScanner {
    fn escanear(&self) -> Result<Vec<BssBruto>, String> {
        Ok(vec![
            BssBruto {
                ssid: b"CasaWiFi".to_vec(),
                bssid: [0x98, 0x2f, 0xf8, 0x57, 0x66, 0x18],
                rssi: -55,
                freq_khz: 2_437_000,
                capability: 0x0011,
                ies: [rsn(2, 0), wps(), tlv(45, &[0; 4])].concat(),
            },
            BssBruto {
                ssid: b"CasaWiFi".to_vec(),
                bssid: [0x98, 0x2f, 0xf8, 0x57, 0x66, 0x1c],
                rssi: -70,
                freq_khz: 5_745_000,
                capability: 0x0011,
                ies: [rsn(2, 0), tlv(191, &[0; 4]), tlv(255, &[35, 0])].concat(),
            },
            BssBruto {
                ssid: b"Vizinho_WPA3".to_vec(),
                bssid: [0xe8, 0x65, 0xd4, 0x36, 0xb2, 0x80],
                rssi: -80,
                freq_khz: 5_180_000,
                capability: 0x0011,
                ies: [rsn(8, 0x00C0), tlv(7, b"BR "), tlv(255, &[35, 0])].concat(),
            },
            BssBruto {
                ssid: b"CafeAberto".to_vec(),
                bssid: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
                rssi: -65,
                freq_khz: 2_462_000,
                capability: 0x0001,
                ies: vec![],
            },
            BssBruto {
                ssid: vec![],
                bssid: [0xb8, 0x8e, 0x82, 0x86, 0x28, 0x45],
                rssi: -86,
                freq_khz: 2_412_000,
                capability: 0x0011,
                ies: [rsn(2, 0), wps(), tlv(70, &[0; 5])].concat(),
            },
        ])
    }
}
