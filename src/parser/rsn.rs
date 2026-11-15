//! Leitura do IE RSN (ID 48) e classificação da segurança.

use crate::domain::Risco;

#[derive(Debug)]
pub struct Rsn {
    pub akms: Vec<u8>,
    pub mfpc: bool, // PMF suportado
    pub mfpr: bool, // PMF obrigatório
}

pub fn parse_rsn(c: &[u8]) -> Option<Rsn> {
    let mut p = 6; // versão (2) + cifra de grupo (4)
    let n_pair = u16::from_le_bytes([*c.get(p)?, *c.get(p + 1)?]) as usize;
    p += 2 + 4 * n_pair;
    let n_akm = u16::from_le_bytes([*c.get(p)?, *c.get(p + 1)?]) as usize;
    p += 2;

    let mut akms = Vec::new();
    for _ in 0..n_akm {
        let s = c.get(p..p + 4)?;
        if s[..3] == [0x00, 0x0F, 0xAC] {
            akms.push(s[3]);
        }
        p += 4;
    }
    let caps = match (c.get(p), c.get(p + 1)) {
        (Some(a), Some(b)) => u16::from_le_bytes([*a, *b]),
        _ => 0,
    };
    Some(Rsn { akms, mfpr: caps & 0x40 != 0, mfpc: caps & 0x80 != 0 })
}

/// Decide o nome da segurança e o risco a partir do que o beacon anuncia.
pub fn classificar(rsn: Option<&Rsn>, wpa: bool, privacy: bool) -> (String, Risco) {
    if let Some(rsn) = rsn {
        let tem = |ids: &[u8]| rsn.akms.iter().any(|a| ids.contains(a));
        let (sae, psk, eap, owe) = (tem(&[8, 9]), tem(&[2, 4, 6]), tem(&[1, 3, 5, 11, 12]), tem(&[18]));

        let (nome, risco) = if owe {
            ("OWE", Risco::Bom)
        } else if eap {
            ("WPA2-Ent", Risco::Bom)
        } else if sae && psk {
            ("WPA2/WPA3", Risco::Atencao)
        } else if sae {
            ("WPA3", Risco::Bom)
        } else if wpa {
            ("WPA/WPA2", Risco::Atencao)
        } else if psk {
            ("WPA2", Risco::Atencao)
        } else {
            ("RSN", Risco::Atencao)
        };
        (nome.to_string(), risco)
    } else if wpa {
        ("WPA".to_string(), Risco::Ruim)
    } else if privacy {
        ("WEP".to_string(), Risco::Ruim)
    } else {
        ("ABERTA".to_string(), Risco::Ruim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RSN com CCMP e um AKM, mais as capabilities.
    fn rsn_bytes(akm: u8, caps: u16) -> Vec<u8> {
        let mut c = vec![
            1, 0, 0x00, 0x0F, 0xAC, 4, 1, 0, 0x00, 0x0F, 0xAC, 4, 1, 0, 0x00, 0x0F, 0xAC, akm,
        ];
        c.extend_from_slice(&caps.to_le_bytes());
        c
    }

    #[test]
    fn le_psk_sem_pmf() {
        let r = parse_rsn(&rsn_bytes(2, 0)).unwrap();
        assert_eq!(r.akms, vec![2]);
        assert!(!r.mfpc && !r.mfpr);
    }

    #[test]
    fn le_sae_com_pmf_obrigatorio() {
        let r = parse_rsn(&rsn_bytes(8, 0x00C0)).unwrap();
        assert_eq!(r.akms, vec![8]);
        assert!(r.mfpc && r.mfpr);
    }

    #[test]
    fn rsn_truncado_devolve_none() {
        assert!(parse_rsn(&[1, 0, 0x00, 0x0F]).is_none());
    }

    #[test]
    fn classifica_wpa3() {
        let r = parse_rsn(&rsn_bytes(8, 0)).unwrap();
        assert_eq!(classificar(Some(&r), false, true), ("WPA3".to_string(), Risco::Bom));
    }

    #[test]
    fn classifica_wpa2() {
        let r = parse_rsn(&rsn_bytes(2, 0)).unwrap();
        assert_eq!(classificar(Some(&r), false, true), ("WPA2".to_string(), Risco::Atencao));
    }

    #[test]
    fn classifica_sem_rsn() {
        assert_eq!(classificar(None, true, true).1, Risco::Ruim);
        assert_eq!(classificar(None, false, true).0, "WEP");
        assert_eq!(classificar(None, false, false).0, "ABERTA");
    }
}
