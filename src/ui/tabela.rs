use crate::domain::{Ap, Risco};
use comfy_table::{presets::UTF8_BORDERS_ONLY, Cell, Color, ContentArrangement, Table};

fn cor_risco(r: Risco) -> Color {
    match r {
        Risco::Bom => Color::Green,
        Risco::Atencao => Color::Yellow,
        Risco::Ruim => Color::Red,
    }
}

fn barra(rssi: i32) -> String {
    let pct = ((rssi + 100) * 2).clamp(0, 100) as usize;
    let cheios = (pct * 8 + 50) / 100;
    format!("{}{}", "█".repeat(cheios), "░".repeat(8 - cheios))
}

fn cor_sinal(rssi: i32) -> Color {
    if rssi >= -60 {
        Color::Green
    } else if rssi >= -75 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn sim_ou_traco(ok: bool, cor: Color) -> Cell {
    if ok { Cell::new("sim").fg(cor) } else { Cell::new("-").fg(Color::DarkGrey) }
}

pub fn exibir(aps: &[Ap]) {
    // Agrupa por SSID e, dentro do grupo, do sinal mais forte para o mais fraco
    let mut ordenados: Vec<&Ap> = aps.iter().collect();
    ordenados.sort_by(|a, b| a.ssid.cmp(&b.ssid).then(b.rssi.cmp(&a.rssi)));

    let mut t = Table::new();
    t.load_preset(UTF8_BORDERS_ONLY).set_content_arrangement(ContentArrangement::Dynamic);
    t.set_header(vec![
        "SSID", "BSSID", "Sinal", "Canal", "Banda", "Seg.", "PMF", "WPS", "k/v/r", "Pad.", "País",
    ]);

    let mut anterior: &str = "";
    for ap in ordenados {
        // Mostra o SSID só na primeira linha do grupo
        let nome = if ap.ssid != anterior { ap.ssid.as_str() } else { "" };
        anterior = ap.ssid.as_str();

        t.add_row(vec![
            Cell::new(nome).fg(Color::Cyan),
            Cell::new(&ap.bssid).fg(Color::DarkGrey),
            Cell::new(format!("{} {:>4}", barra(ap.rssi), ap.rssi)).fg(cor_sinal(ap.rssi)),
            Cell::new(ap.canal),
            Cell::new(ap.banda),
            Cell::new(&ap.seguranca).fg(cor_risco(ap.risco)),
            sim_ou_traco(ap.pmf, Color::Green),
            sim_ou_traco(ap.wps, Color::Yellow),
            Cell::new(&ap.roaming),
            Cell::new(ap.padrao),
            Cell::new(&ap.pais),
        ]);
    }

    println!("{t}");
    println!(
        "Sinal em dBm (acima de -60 é bom). PMF = proteção de quadros de gerenciamento. \
         WPS \"sim\" = WPS anunciado no beacon. k/v/r = roaming 802.11k/v/r."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn barra_tem_sempre_8_caracteres() {
        for rssi in [-120, -100, -85, -60, -50, 0] {
            assert_eq!(barra(rssi).chars().count(), 8);
        }
        assert_eq!(barra(-50), "████████");
        assert_eq!(barra(-100), "░░░░░░░░");
    }
}
