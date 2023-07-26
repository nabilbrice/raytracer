use std::cmp::PartialEq;
use std::ops;
use std::ops::{Deref, DerefMut};
use std::fmt::{self, Formatter, Display};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3(pub [f64;3]);

impl Deref for Vec3 {
    type Target = [f64;3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vec3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self[0], self[1], self[2])
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Vec3) -> bool {
        self.0 == other.0
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, _rhs: Vec3) -> Vec3 {
        Vec3([self[0] + _rhs[0],
             self[1] + _rhs[1],
             self[2] + _rhs[2]])
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3([-self[0], -self[1], -self[2]])
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, _rhs: Vec3) -> Vec3 {
        Vec3([self[0] - _rhs[0],
             self[1] - _rhs[1],
             self[2] - _rhs[2]])
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, _rhs: f64) -> Vec3 {
        Vec3([self[0] * _rhs,
             self[1] * _rhs,
             self[2] * _rhs])
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, _rhs: Vec3) -> Vec3 {
        _rhs.mul(self)
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, _rhs: f64) -> Vec3 {
        Vec3([self[0] / _rhs,
             self[1] / _rhs,
             self[2] / _rhs])
    }
}

impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        *self = Self([self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2]])
    }
}

impl Vec3 {
    pub fn norm(&self) -> f64 {
        (self[0] * self[0] + self[1] * self[1] + self[2] * self[2]).sqrt()
    }

    pub fn dotprod(&self, other: &Vec3) -> f64 {
        self[0] * other[0] + self[1] * other[1] + self[2] * other[2]
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3([self[1] * other[2] - self[2] * other[1],
            self[2] * other[0] - self[0] * other[2],
            self[0] * other[1] - self[1] * other[0]])
    }

    // Changes the input Vec3 to be a normalized Vec3
    pub fn normalize(self) -> Vec3 {
        self / self.norm()
    }

}

pub fn lerp_vec3(p: Vec3, q: Vec3, t: f64) -> Vec3 {
    ((1.0 - t) * p) + (t * q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_test() {
        let u = Vec3([3.0,4.0,0.0]);
        assert_eq!(u.norm(), 5.0)
    }

    #[test]
    fn dotprod_test() {
        let u = Vec3([1.0, 0.0, 0.0]);
        let v = Vec3([0.5, 0.0, 0.0]);
        assert_eq!(u.dotprod(&v), 0.5)
    }

    #[test]
    fn cross_test() {
        let u = Vec3([1.0, 0.0, 0.0]);
        let v = Vec3([0.0, 1.0, 0.0]);
        assert_eq!(u.cross(&v), Vec3([0.0, 0.0, 1.0]))
    }
}