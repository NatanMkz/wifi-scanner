pub fn ies(dados: &[u8]) -> impl Iterator<Item = (u8, &[u8])> + '_ {
    let mut i = 0;
    std::iter::from_fn(move || {
        if i + 2 > dados.len() {
            return None;
        }
        let id = dados[i];
        let len = dados[i + 1] as usize;
        if i + 2 + len > dados.len() {
            return None; // elemento truncado
        }
        let corpo = &dados[i + 2..i + 2 + len];
        i += 2 + len;
        Some((id, corpo))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_dois_elementos() {
        let bytes = [0x00, 0x02, b'h', b'i', 0x07, 0x01, 0x42];
        let v: Vec<_> = ies(&bytes).collect();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], (0, &b"hi"[..]));
        assert_eq!(v[1], (7, &[0x42][..]));
    }

    #[test]
    fn ignora_elemento_truncado() {
        let bytes = [0x00, 0x09, 0x01]; // diz ter 9 bytes mas só tem 1
        assert_eq!(ies(&bytes).count(), 0);
    }

    #[test]
    fn entrada_vazia() {
        assert_eq!(ies(&[]).count(), 0);
    }
}
