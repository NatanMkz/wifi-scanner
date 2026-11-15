//! Transforma dados crus de beacon (`BssBruto`) em um `Ap` interpretado.
//! Código puro: não usa Windows, nem `unsafe`, nem terminal.

mod ie;
mod rsn;

use crate::domain::{Ap, BssBruto};

fn canal_banda(khz: u32) -> (u32, &'static str) {
    let mhz = khz / 1000;
    match mhz {
        2412..=2472 => ((mhz - 2407) / 5, "2,4"),
        2484 => (14, "2,4"),
        5000..=5895 => ((mhz - 5000) / 5, "5"),
        5955..=7115 => ((mhz - 5950) / 5, "6"),
        _ => (0, "?"),
    }
}

pub fn analisar(b: &BssBruto) -> Ap {
    let ssid = if b.ssid.is_empty() {
        "(oculta)".to_string()
    } else {
        String::from_utf8_lossy(&b.ssid).into_owned()
    };
    let bssid = b.bssid.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(":");
    let (canal, banda) = canal_banda(b.freq_khz);

    let mut rsn = None;
    let (mut wpa, mut wps) = (false, false);
    let (mut k, mut v, mut r) = (false, false, false);
    let (mut ht, mut vht, mut he) = (false, false, false);
    let mut pais = String::from("-");

    for (id, c) in ie::ies(&b.ies) {
        match id {
            7 if c.len() >= 2 => pais = String::from_utf8_lossy(&c[..2]).into_owned(),
            45 => ht = true,
            48 => rsn = rsn::parse_rsn(c),
            54 => r = true, // Mobility Domain -> 802.11r
            70 => k = true, // RM Enabled Capabilities -> 802.11k
            127 if c.len() >= 3 => v = c[2] & 0x08 != 0, // BSS Transition -> 802.11v
            191 => vht = true,
            221 if c.len() >= 4 && c[..3] == [0x00, 0x50, 0xF2] => match c[3] {
                1 => wpa = true,
                4 => wps = true,
                _ => {}
            },
            255 if c.first() == Some(&35) => he = true, // HE Capabilities -> Wi-Fi 6
            _ => {}
        }
    }

    let privacy = b.capability & 0x0010 != 0;
    let pmf = rsn.as_ref().map_or(false, |r| r.mfpc || r.mfpr);
    let (seguranca, risco) = rsn::classificar(rsn.as_ref(), wpa, privacy);

    let flag = |ok: bool, c: char| if ok { c } else { '-' };
    let roaming = format!("{}{}{}", flag(k, 'k'), flag(v, 'v'), flag(r, 'r'));
    let padrao = if he { "ax" } else if vht { "ac" } else if ht { "n" } else { "a/g" };

    Ap { ssid, bssid, rssi: b.rssi, canal, banda, seguranca, risco, pmf, wps, roaming, padrao, pais }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Risco;

    #[test]
    fn canais() {
        assert_eq!(canal_banda(2_412_000), (1, "2,4"));
        assert_eq!(canal_banda(2_437_000), (6, "2,4"));
        assert_eq!(canal_banda(5_180_000), (36, "5"));
        assert_eq!(canal_banda(5_745_000), (149, "5"));
        assert_eq!(canal_banda(5_955_000), (1, "6"));
        assert_eq!(canal_banda(100), (0, "?"));
    }

    #[test]
    fn rede_oculta_wpa3() {
        let ies = vec![
            48, 20, 1, 0, 0x00, 0x0F, 0xAC, 4, 1, 0, 0x00, 0x0F, 0xAC, 4, 1, 0, 0x00, 0x0F, 0xAC, 8,
            0xC0, 0x00,
        ];
        let b = BssBruto {
            ssid: vec![],
            bssid: [0xaa, 0xbb, 0xcc, 0x00, 0x11, 0x22],
            rssi: -50,
            freq_khz: 5_180_000,
            capability: 0x0011,
            ies,
        };
        let ap = analisar(&b);
        assert_eq!(ap.ssid, "(oculta)");
        assert_eq!(ap.bssid, "aa:bb:cc:00:11:22");
        assert_eq!(ap.canal, 36);
        assert_eq!(ap.seguranca, "WPA3");
        assert_eq!(ap.risco, Risco::Bom);
        assert!(ap.pmf);
        assert!(!ap.wps);
    }

    #[test]
    fn rede_aberta() {
        let b = BssBruto {
            ssid: b"Cafe".to_vec(),
            bssid: [0; 6],
            rssi: -70,
            freq_khz: 2_462_000,
            capability: 0x0001,
            ies: vec![],
        };
        let ap = analisar(&b);
        assert_eq!(ap.seguranca, "ABERTA");
        assert_eq!(ap.risco, Risco::Ruim);
        assert_eq!(ap.canal, 11);
    }
}
