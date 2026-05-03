use strum::{Display, EnumIter, EnumString};

#[derive(Debug, EnumIter, Display, EnumString)]
pub enum Nucleotide {
    A,
    T,
    C,
    G,
    U,
}

impl Nucleotide {
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Nucleotide::A => (0, 200, 0),    // green
            Nucleotide::T => (200, 0, 0),    // red
            Nucleotide::C => (0, 100, 255),  // blue
            Nucleotide::G => (220, 220, 0),  // yellow
            Nucleotide::U => (153, 51, 255), // purple
        }
    }
}
