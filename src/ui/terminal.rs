//! Controle do terminal: tela alternativa e leitura de teclas.

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io::stdout, time::Duration};

pub enum Tecla {
    Atualizar,
    Sair,
}

/// Garante que o terminal volta ao normal quando o programa termina,
/// mesmo em caso de pânico (equivalente ao IDisposable do C#).
pub struct TelaAlternativa;

impl TelaAlternativa {
    pub fn nova() -> Self {
        execute!(stdout(), EnterAlternateScreen).unwrap();
        Self
    }
}

impl Drop for TelaAlternativa {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

pub fn limpar() {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
}

pub fn esperar_tecla() -> Tecla {
    terminal::enable_raw_mode().unwrap();

    // Descarta teclas pressionadas enquanto o scan rodava
    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
        let _ = event::read();
    }

    let resultado = loop {
        if let Ok(Event::Key(k)) = event::read() {
            // No Windows o crossterm manda evento de pressionar e de soltar
            if k.kind != KeyEventKind::Press {
                continue;
            }
            match k.code {
                KeyCode::Char('r') | KeyCode::Char('R') => break Tecla::Atualizar,
                KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc => break Tecla::Sair,
                _ => {}
            }
        }
    };

    terminal::disable_raw_mode().unwrap();
    resultado
}
