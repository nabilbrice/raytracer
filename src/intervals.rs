#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub start: f64,
    pub end: f64,
}

impl PartialEq for Interval {
    fn eq(&self, rhs: &Interval) -> bool {
        self.start == rhs.start && self.end == rhs.end
    }
}

impl Interval {
    pub fn size(&self) -> f64 {
        self.end - self.start
    }

    pub fn midpoint(&self) -> f64 {
        0.5 * (self.start + self.end)
    }

    pub fn new(start: f64, end: f64) -> Self {
        Interval { start, end }
    }

    pub fn size_partial_cmp(&self, other: &Interval) -> Option<std::cmp::Ordering> {
        self.size().partial_cmp(&other.size())
    }
}

pub fn intersection(in1: &Interval, in2: &Interval) -> Option<Interval> {
    let start = if in1.start < in2.start {
        in2.start
    } else {
        in1.start
    };
    let end = if in1.end < in2.end { in1.end } else { in2.end };

    if start >= end {
        return None;
    }

    Some(Interval::new(start, end))
}

pub fn cover(in1: &Interval, in2: &Interval) -> Interval {
    let start = if in1.start < in2.start {
        in1.start
    } else {
        in2.start
    };
    let end = if in1.end < in2.end { in2.end } else { in1.end };

    Interval::new(start, end)
}

pub fn get_larger<'input>(in1: &'input Interval, in2: &'input Interval) -> &'input Interval {
    if in1.size() < in2.size() {
        return &in2;
    }
    &in1
}

#[macro_export]
macro_rules! interval {
    ( $start:expr, $end:expr ) => {
        Interval::new($start, $end)
    };
}

mod tests {
    use super::*;

    #[test]
    fn test_interval() {
        assert_eq!(Interval::new(1.0, 2.0).size(), 1.0);
    }

    #[test]
    fn test_intersection() {
        let in1 = Interval::new(0.0, 1.0);
        let in2 = Interval::new(1.0, 2.0);

        assert!(intersection(&in1, &in2).is_none());

        let in3 = Interval::new(1.0, 0.0);

        assert!(intersection(&in1, &in3).is_none());

        assert!(intersection(&in2, &in3).is_none());
    }

    #[test]
    fn test_cover() {
        let in1 = Interval::new(-1.0, 1.0);
        let in2 = Interval::new(2.0, 3.0);

        assert_eq!(cover(&in1, &in2).start, in1.start);
        assert_eq!(cover(&in1, &in2).end, in2.end);
    }

    #[test]
    fn test_larger() {
        let in1 = Interval::new(-1.0, 1.0);
        let in2 = Interval::new(2.0, 3.0);

        assert_eq!(get_larger(&in1, &in2).start, -1.0);
        assert_eq!(get_larger(&in1, &in2).end, 1.0);
    }

    #[test]
    fn test_macro() {
        let macro_generated = interval!(0.0, 1.0);
        let funct_generated = Interval::new(0.0, 1.0);

        assert_eq!(macro_generated, funct_generated);
    }
}
