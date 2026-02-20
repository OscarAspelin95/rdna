use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::{
    io::{self, Write, stdout},
    time::Duration,
};

const NUCLEOTIDES: [char; 5] = ['A', 'T', 'C', 'G', 'U'];

fn nucleotide_color(ch: char) -> (u8, u8, u8) {
    match ch {
        'A' => (0, 200, 0),    // green
        'T' => (200, 0, 0),    // red
        'C' => (0, 100, 255),  // blue
        'G' => (220, 220, 0),  // yellow
        'U' => (153, 51, 255), // purple
        _ => (0, 200, 0),
    }
}

struct Column {
    x: u16,
    y: i16,
    speed: u16,
    trail_len: i16,
    chars: Vec<char>,
}

impl Column {
    fn new(x: u16, height: u16) -> Self {
        let mut rng = rand::thread_rng();
        let trail_len = (height / 3).max(4) as i16;
        Self {
            x,
            y: rng.gen_range(-trail_len..0),
            speed: rng.gen_range(1..2),
            trail_len,
            chars: (0..height)
                .map(|_| NUCLEOTIDES[rng.gen_range(0..NUCLEOTIDES.len())])
                .collect(),
        }
    }

    fn draw(&self, stdout: &mut impl Write, height: u16) -> io::Result<()> {
        for i in 0..=self.trail_len {
            let row = self.y - i;
            if row < 0 || row >= height as i16 {
                continue;
            }
            let ch = self.chars[row as usize];
            let color = match i {
                // Head - bright white
                0 => Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                _ => {
                    // Body/tail - nucleotide color, dimming toward the tail
                    let (r, g, b) = nucleotide_color(ch);
                    let fade = 1.0 - (i as f32 / self.trail_len as f32);
                    Color::Rgb {
                        r: (r as f32 * fade) as u8,
                        g: (g as f32 * fade) as u8,
                        b: (b as f32 * fade) as u8,
                    }
                }
            };
            execute!(
                stdout,
                cursor::MoveTo(self.x, row as u16),
                SetForegroundColor(color),
                Print(ch)
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
        if self.y - self.trail_len > height as i16 {
            self.y = rng.gen_range(-self.trail_len..0);
            // Regenerate characters for variety
            for ch in &mut self.chars {
                *ch = NUCLEOTIDES[rng.gen_range(0..4)];
            }
        }
    }
}

fn setup_terminal(stdout: &mut impl Write) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    Ok(())
}

fn cleanup_terminal(stdout: &mut impl Write) -> io::Result<()> {
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn run(stdout: &mut impl Write) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    let mut columns: Vec<Column> = (0..width)
        .step_by(2)
        .map(|x| Column::new(x, height))
        .collect();

    loop {
        if event::poll(Duration::from_millis(60))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        for col in &mut columns {
            col.draw(stdout, height)?;
            col.update(height);
        }

        stdout.flush()?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    setup_terminal(&mut stdout)?;
    let result = run(&mut stdout);
    cleanup_terminal(&mut stdout)?;
    result
}
