use ariadne::Color;

pub struct PaletteColorGenerator {
    palette: Vec<Color>,
    index: usize,
}

impl PaletteColorGenerator {
    pub fn new(palette: Vec<Color>) -> Self {
        Self {
            palette,
            index: 0,
        }
    }

    pub fn next(&mut self) -> Color {
        if self.palette.len() == 0 {
            return Color::Primary
        }

        let color = self.palette[self.index % self.palette.len()];
        self.index += 1;
        if self.index >= self.palette.len() {
            self.index = 0;
        }

        color
    }
}
