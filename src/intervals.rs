#[derive(Debug)]
pub struct Interval {
    pub start: f64,
    pub end: f64,
}

impl Interval {
    pub fn size(&self) -> f64 {
        self.end - self.start
    }

    pub fn new(start: f64, end: f64) -> Self {
        Interval { start, end }
    }
}

pub fn intersection(in1: &Interval, in2: &Interval) -> Option<Interval> {
    let start = if in1.start < in2.start {
        in2.start
    } else {
        in2.start
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
        return &in2
    }
    &in1
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
}
