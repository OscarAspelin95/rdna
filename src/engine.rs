use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{Rng, seq::IteratorRandom};
use std::{
    io::{self, Write},
    time::Duration,
};
use strum::IntoEnumIterator;

use crate::types::Nucleotide;

struct Column {
    x: u16,
    y: i16,
    speed: u16,
    trail_len: i16,
    nts: Vec<Nucleotide>,
    head_nt: Nucleotide,
}

impl Column {
    fn new(x: u16, height: u16) -> Self {
        let mut rng = rand::thread_rng();
        let trail_len = (height / 2).max(4) as i16;

        Self {
            x,
            y: rng.gen_range(-trail_len..0),
            speed: rng.gen_range(1..2),
            trail_len,
            nts: (0..height)
                .map(|_| Nucleotide::iter().choose(&mut rng).unwrap())
                .collect(),
            head_nt: Nucleotide::iter().choose(&mut rng).unwrap(),
        }
    }

    fn draw(&self, stdout: &mut impl Write, height: u16) -> io::Result<()> {
        for i in 0..=self.trail_len {
            let row = self.y - i;
            if row < 0 || row >= height as i16 {
                continue;
            }

            let nt: &Nucleotide = if i == 0 {
                &self.head_nt
            } else {
                &self.nts[row as usize]
            };

            let color = if i == 0 {
                Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                }
            } else {
                let (r, g, b) = nt.color();
                let fade = 1.0 - (i as f32 / self.trail_len as f32);
                Color::Rgb {
                    r: (r as f32 * fade) as u8,
                    g: (g as f32 * fade) as u8,
                    b: (b as f32 * fade) as u8,
                }
            };

            execute!(
                stdout,
                cursor::MoveTo(self.x, row as u16),
                SetForegroundColor(color),
                Print(nt)
            )?;
        }

        // Erase character just beyond the trail
        let erase_row = self.y - self.trail_len - 1;
        if erase_row >= 0 && erase_row < height as i16 {
            execute!(stdout, cursor::MoveTo(self.x, erase_row as u16), Print(' '))?;
        }

        Ok(())
    }

    fn update(&mut self, height: u16) {
        let mut rng = rand::thread_rng();
        self.y += self.speed as i16;

        // Randomly glitch a few trail characters each frame
        for nt in self.nts.iter_mut().choose_multiple(&mut rng, 3) {
            *nt = Nucleotide::iter().choose(&mut rng).unwrap();
        }

        if self.y - self.trail_len > height as i16 {
            self.y = rng.gen_range(-self.trail_len..0);
            self.head_nt = Nucleotide::iter().choose(&mut rng).unwrap();
            for nt in &mut self.nts {
                *nt = Nucleotide::iter().choose(&mut rng).unwrap();
            }
        }
    }
}

pub fn setup_terminal(stdout: &mut impl Write) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    Ok(())
}

pub fn cleanup_terminal(stdout: &mut impl Write) -> io::Result<()> {
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn run(stdout: &mut impl Write) -> io::Result<()> {
    let (mut width, mut height) = terminal::size()?;
    let mut columns: Vec<Column> = (0..width)
        .step_by(2)
        .map(|x| Column::new(x, height))
        .collect();

    loop {
        if event::poll(Duration::from_millis(45))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        match terminal::size() {
            Ok((current_width, current_height))
                if current_width != width || current_height != height =>
            {
                width = current_width;
                height = current_height;

                execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

                columns = (0..width)
                    .step_by(2)
                    .map(|x| Column::new(x, height))
                    .collect();
            }
            _ => {}
        };

        for col in &mut columns {
            col.draw(stdout, height)?;
        }

        for col in &mut columns {
            col.update(height);
        }

        stdout.flush()?;
    }

    Ok(())
}
