use crate::ray::Ray;
use crate::geometry;

struct Interval {
    minimum: f64,
    maximum: f64,
}

impl Interval {
    fn size(&self) -> f64 {
        self.maximum - self.minimum
    }
}

fn intersection(in1: &Interval, in2: &Interval) -> Option<Interval> {
    let minimum = if in1.minimum > in2.minimum { in1.minimum } else { in2.minimum };
    let maximum = if in1.maximum < in2.maximum { in1.maximum } else { in2.maximum };

    if minimum > maximum {
        return None;
    }
    Some(Interval{minimum, maximum})
}

fn union(in1: &Interval, in2: &Interval) -> Interval {
    let minimum = if in1.minimum < in2.minimum { in1.minimum } else { in2.minimum };
    let maximum = if in1.maximum > in2.maximum { in1.maximum } else { in2.maximum };
    Interval{minimum, maximum}
}

struct BoundingBox(Interval, Interval, Interval);

impl BoundingBox {
    fn check_intersection(&self, ray: &Ray) -> bool {
        let times0 = Interval{minimum: (self.0.minimum - ray.orig.0) / ray.dir.0, maximum: (self.0.maximum - ray.orig.0) / ray.dir.0};
        let times1 = Interval{minimum: (self.1.minimum - ray.orig.1) / ray.dir.1, maximum: (self.1.maximum - ray.orig.1) / ray.dir.1};
        let times2 = Interval{minimum: (self.2.minimum - ray.orig.2) / ray.dir.2, maximum: (self.2.maximum - ray.orig.2) / ray.dir.2};

        let xy = intersection(&times0, &times1);
        if xy.is_none() {
            return false;
        }
        intersection(&xy.unwrap(), &times2).is_some()
    }

    fn compose(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox(union(&self.0, &other.0), union(&self.1, &other.1), union(&self.2, &other.2))
    }

    fn longest_axis(&self) -> usize {
        let x_size = self.0.size();
        let y_size = self.1.size();
        let z_size = self.2.size();

        if x_size > y_size && x_size > z_size {
            return 0;
        }
        if y_size > z_size {
            return 1;
        }
        2
    }
}

impl geometry::Sphere {
    fn surround(&self) -> BoundingBox {
        let radius = self.radius;
        let centre = self.centre;
        BoundingBox(Interval{minimum: centre.0 - radius, maximum: centre.0 + radius},
                    Interval{minimum: centre.1 - radius, maximum: centre.1 + radius},
                    Interval{minimum: centre.2 - radius, maximum: centre.2 + radius})
    }
}

mod tests {
    use super::*;
    use crate::vector::Vec3;

    #[test]
    fn test_bbox_intersection() {
        let bbox = BoundingBox(Interval{minimum: 0.0, maximum: 1.0}, Interval{minimum: 0.0, maximum: 1.0}, Interval{minimum: 0.0, maximum: 1.0});
        let ray = Ray{orig: Vec3(1.5, 0.5, 0.5), dir: Vec3(1.0, 0.0, 0.0)};
        assert!(bbox.check_intersection(&ray));

        let miss_ray = Ray{orig: Vec3(1.5,1.5,0.5), dir: Vec3(1.0, 0.0, 0.0)};
        assert!(!bbox.check_intersection(&miss_ray));
    }

    #[test]
    fn test_composition() {
        let bbox1 = BoundingBox(Interval{minimum: 0.0, maximum: 1.0}, Interval{minimum: 0.0, maximum: 1.0}, Interval{minimum: 0.0, maximum: 1.0});
        let bbox2 = BoundingBox(Interval{minimum: 1.0, maximum: 2.0}, Interval{minimum: 1.0, maximum: 2.0}, Interval{minimum: 1.0, maximum: 2.0});
        let bbox3 = bbox1.compose(&bbox2);

        assert_eq!(bbox3.0.minimum, 0.0);
        assert_eq!(bbox3.0.maximum, 2.0);
        assert_eq!(bbox3.1.minimum, 0.0);
        assert_eq!(bbox3.1.maximum, 2.0);
        assert_eq!(bbox3.2.minimum, 0.0);
        assert_eq!(bbox3.2.maximum, 2.0);
    }

    #[test]
    fn test_longest_axis() {
        let bbox1 = BoundingBox(Interval{minimum: 0.0, maximum: 1.0}, Interval{minimum: 0.0, maximum: 1.0}, Interval{minimum: 0.0, maximum: 1.0});
        assert_eq!(bbox1.longest_axis(), 2);

        let bbox2 = BoundingBox(Interval{minimum: 0.0, maximum: 1.0}, Interval{minimum: 0.0, maximum: 2.0}, Interval{minimum: 0.0, maximum: 1.0});
        assert_eq!(bbox2.longest_axis(), 1);
    }

    #[test]
    fn test_surround() {
        let sphere = geometry::Sphere{centre: Vec3(0.0, 0.0, 0.0), radius: 1.0};
        let bbox = sphere.surround();
        assert_eq!(bbox.0.minimum, -1.0);
        assert_eq!(bbox.0.maximum, 1.0);
        assert_eq!(bbox.1.minimum, -1.0);
        assert_eq!(bbox.1.maximum, 1.0);
        assert_eq!(bbox.2.minimum, -1.0);
        assert_eq!(bbox.2.maximum, 1.0);
    }
}