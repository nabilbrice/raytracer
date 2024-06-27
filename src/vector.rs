use ndarray::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt::{self, Display, Formatter};
use std::ops::{self, AddAssign};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct Vector {
    pub values: Array<f64, Ix1>,
}

impl std::cmp::PartialEq for Vector {
    fn eq(&self, other: &Vector) -> bool {
        self.values == other.values
    }
}

impl Vector {
    fn new(values: Array<f64, Ix1>) -> Self {
        Self { values }
    }

    #[inline]
    fn dotprod(&self, other: &Vector) -> f64 {
        self.values.dot(&other.values)
    }

    #[inline]
    fn norm(&self) -> f64 {
        self.dotprod(self).sqrt()
    }

    fn normalize(self) -> Self {
        Self::new(self.values / self.norm())
    }

    fn cross3d(&self, other: &Self) -> Self {
        Self::new(array![
            self.values[1] * other.values[2] - self.values[2] * other.values[1],
            self.values[2] * other.values[0] - self.values[0] * other.values[2],
            self.values[0] * other.values[1] - self.values[1] * other.values[0],
        ])
    }
}

impl ops::Add<Vector> for Vector {
    type Output = Vector;

    fn add(mut self, rhs: Vector) -> Vector {
        Vector::new(self.values + rhs.values)
    }
}

impl ops::Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        Vector::new(-self.values)
    }
}

impl ops::Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(mut self, rhs: Vector) -> Vector {
        Vector::new(self.values - rhs.values)
    }
}

impl ops::Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Vector {
        Vector::new(self.values * rhs)
    }
}

impl ops::Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Vector {
        rhs.mul(self)
    }
}

impl ops::Div<f64> for Vector {
    type Output = Vector;

    fn div(self, rhs: f64) -> Vector {
        Vector::new(self.values / rhs)
    }
}

pub fn lerp(p: Vector, q: Vector, t: f64) -> Vector {
    ((1.0 - t) * p) + (t * q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_test() {
        let u = Vector::new(array![3.0, 4.0, 0.0]);
        assert_eq!(u.norm(), 5.0)
    }

    #[test]
    fn dotprod_test() {
        let u = Vector::new(array![1.0, 0.0, 0.0]);
        let v = Vector::new(array![0.5, 0.0, 0.0]);
        assert_eq!(u.dotprod(&v), 0.5)
    }

    #[test]
    fn cross_test() {
        let u = Vector::new(array![1.0, 0.0, 0.0]);
        let v = Vector::new(array![0.0, 1.0, 0.0]);
        assert_eq!(u.cross3d(&v), Vector::new(array![0.0, 0.0, 1.0]))
    }
}
