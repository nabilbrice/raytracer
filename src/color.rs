use std::ops;
use serde::{Serialize, Deserialize};

const NUMBER_OF_BINS:usize = 3;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub bin: [f64; NUMBER_OF_BINS],
}

impl Color {
    pub fn new(bin: [f64; NUMBER_OF_BINS]) -> Color {
        Color{bin}
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;

    fn add(mut self, _rhs: Color) -> Color {
        for index in 0..NUMBER_OF_BINS {
           self.bin[index] += _rhs.bin[index]; 
        };
        self
    }
}

impl ops::Mul<Color> for Color {
    type Output = Color;

    fn mul(mut self, _rhs: Color) -> Color {
        for index in 0..NUMBER_OF_BINS {
            self.bin[index] *= _rhs.bin[index];
        } 
        self
    }
}

impl ops::Mul<f64> for Color {
    type Output = Color;

    fn mul(mut self, _rhs: f64) -> Color {
        for index in 0..NUMBER_OF_BINS {
            self.bin[index] *= _rhs;
        } 
        self
    }
}

impl ops::Mul<Color> for f64 {
    type Output = Color;

    fn mul(self, _rhs: Color) -> Color {
        _rhs.mul(self)
    }
}