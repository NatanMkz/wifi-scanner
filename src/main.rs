//! Scanner Wi-Fi: lê beacons, interpreta os Information Elements e mostra
//! uma tabela com sinal, segurança e recursos de cada ponto de acesso.
//!
//! Uso: `cargo run`            -> usa o Wi-Fi real (Windows)
//!      `cargo run -- --demo`  -> usa dados simulados (qualquer SO)

mod domain;
mod parser;
mod scanner;
mod ui;

use domain::Ap;
use scanner::Scanner;
use ui::terminal::{self, Tecla, TelaAlternativa};

fn criar_scanner(demo: bool) -> Box<dyn Scanner> {
    #[cfg(windows)]
    {
        if !demo {
            return Box::new(scanner::wlan::WlanScanner);
        }
    }
    let _ = demo;
    Box::new(scanner::simulado::SimuladoScanner)
}

fn main() {
    let demo = std::env::args().any(|a| a == "--demo");
    let scanner = criar_scanner(demo);

    // Vive até o fim do main; o Drop restaura o terminal.
    let _tela = TelaAlternativa::nova();

    loop {
        terminal::limpar();
        println!("Escaneando (aguarde alguns segundos)...\n");

        match scanner.escanear() {
            Ok(brutos) if brutos.is_empty() => println!(
                "Nenhuma rede encontrada. Veja se o Wi-Fi está ligado e se a localização está ativa."
            ),
            Ok(brutos) => {
                let aps: Vec<Ap> = brutos.iter().map(parser::analisar).collect();
                terminal::limpar();
                ui::tabela::exibir(&aps);
            }
            Err(e) => println!("Erro: {e}"),
        }

        println!("\n[R] atualizar   [C] sair");

        if let Tecla::Sair = terminal::esperar_tecla() {
            break;
        }
    }
}
