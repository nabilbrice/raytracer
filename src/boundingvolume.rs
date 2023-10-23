use core::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::geometry;
use crate::intervals::{cover, get_larger, intersection, Interval};
use crate::ray::Ray;

pub struct BoundingBox(pub [Interval; 3]);

impl Deref for BoundingBox {
    type Target = [Interval; 3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BoundingBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BoundingBox {
    fn check_intersection(&self, ray: &Ray) -> bool {
        let times_axis0 = Interval {
            start: (self[0].start - ray.orig[0]) / ray.dir[0],
            end: (self[0].end - ray.orig[0]) / ray.dir[0],
        };
        let times_axis1 = Interval {
            start: (self[1].start - ray.orig[1]) / ray.dir[1],
            end: (self[1].end - ray.orig[1]) / ray.dir[1],
        };
        let times_axis2 = Interval {
            start: (self[2].start - ray.orig[2]) / ray.dir[2],
            end: (self[2].end - ray.orig[2]) / ray.dir[2],
        };

        let xy = intersection(&times_axis0, &times_axis1);

        if let Some(times) = xy {
            return intersection(&times, &times_axis2).is_some();
        } else {
            return false;
        };
    }

    fn compose(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox([
            cover(&self[0], &other[0]),
            cover(&self[1], &other[1]),
            cover(&self[2], &other[2]),
        ])
    }

    fn longest_axis(&self) -> usize {
        let x_size = self[0].size();
        let y_size = self[1].size();
        let z_size = self[2].size();

        if x_size > y_size && x_size > z_size {
            return 0;
        }
        if y_size > z_size {
            return 1;
        }
        2
    }
}

mod tests {
    use super::*;
    use crate::vector::Vec3;
    use crate::Hittable;

    #[test]
    fn test_bbox_intersection() {
        let bbox = BoundingBox([
            Interval {
                start: 0.0,
                end: 1.0,
            },
            Interval {
                start: 0.0,
                end: 1.0,
            },
            Interval {
                start: 0.0,
                end: 1.0,
            },
        ]);
        let ray = Ray {
            orig: Vec3([1.5, 0.5, 0.5]),
            dir: Vec3([1.0, 0.0, 0.0]),
        };
        assert!(bbox.check_intersection(&ray));

        let miss_ray = Ray {
            orig: Vec3([1.5, 1.5, 0.5]),
            dir: Vec3([1.0, 0.0, 0.0]),
        };
        assert!(!bbox.check_intersection(&miss_ray));
    }

    #[test]
    fn test_composition() {
        let bbox1 = BoundingBox([
            Interval {
                start: 0.0,
                end: 1.0,
            },
            Interval {
                start: 0.0,
                end: 1.0,
            },
            Interval {
                start: 0.0,
                end: 1.0,
            },
        ]);
        let bbox2 = BoundingBox([
            Interval {
                start: 1.0,
                end: 2.0,
            },
            Interval {
                start: 1.0,
                end: 2.0,
            },
            Interval {
                start: 1.0,
                end: 2.0,
            },
        ]);
        let bbox3 = bbox1.compose(&bbox2);

        assert_eq!(bbox3[0].start, 0.0);
        assert_eq!(bbox3[0].end, 2.0);
        assert_eq!(bbox3[1].start, 0.0);
        assert_eq!(bbox3[1].end, 2.0);
        assert_eq!(bbox3[2].start, 0.0);
        assert_eq!(bbox3[2].end, 2.0);
    }

    #[test]
    fn test_longest_axis() {
        let bbox1 = BoundingBox([
            Interval {
                start: 0.0,
                end: 1.0,
            },
            Interval {
                start: 0.0,
                end: 1.0,
            },
            Interval {
                start: 0.0,
                end: 1.0,
            },
        ]);
        assert_eq!(bbox1.longest_axis(), 2);

        let bbox2 = BoundingBox([
            Interval {
                start: 0.0,
                end: 1.0,
            },
            Interval {
                start: 0.0,
                end: 2.0,
            },
            Interval {
                start: 0.0,
                end: 1.0,
            },
        ]);
        assert_eq!(bbox2.longest_axis(), 1);
    }
}
