// 0%: ' '
// 25%: '·'
// 50%: '+'
// 75%: '*'
// 100%: '#'

#[derive(Clone)]
pub struct Char {
    pub pixels: Vec<Vec<u8>>,
}

impl Char {
    pub fn new() -> Self {
        Self {
            pixels: vec![vec![0; 8]; 8],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        self.pixels[y][x] = value;
    }
}
