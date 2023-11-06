use core::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::geometry;
use crate::intervals::{cover, get_larger, intersection, Interval};
use crate::ray::Ray;
use crate::vector::Vec3;

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
    fn empty() -> BoundingBox {
        BoundingBox([Interval::new(0.0, 0.0); 3])
    }

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

        let Some(xy) = intersection(&times_axis0, &times_axis1) else {
            return false;
        };
        return intersection(&xy, &times_axis2).is_some();
    }

    // for use in the node split by longest axis
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

    fn midpoint(&self) -> Vec3 {
        Vec3([self[0].midpoint(), self[1].midpoint(), self[2].midpoint()])
    }

    // this composition consumes the self and creates a new one
    fn compose_with(self, other: &BoundingBox) -> BoundingBox {
        make_cover_of(&self, other)
    }
}

pub fn make_cover_of(bbox1: &BoundingBox, bbox2: &BoundingBox) -> BoundingBox {
    BoundingBox([
        cover(&bbox1[0], &bbox2[0]),
        cover(&bbox1[1], &bbox2[1]),
        cover(&bbox1[2], &bbox2[2]),
    ])
}

pub struct BoundingBoxList<const N: usize>([BoundingBox; N]);

impl<const N: usize> Deref for BoundingBoxList<N> {
    type Target = [BoundingBox; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for BoundingBoxList<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> BoundingBoxList<N> {
    pub fn sort_on_index(&mut self, idx: usize) {
        self.sort_unstable_by(|b1, b2| {
            b1[idx].midpoint().partial_cmp(&b2[idx].midpoint()).unwrap()
        });
    }

    pub fn make_all_containing(&self) -> BoundingBox {
        self.iter()
            .fold(BoundingBox::empty(), |acc, bbox| acc.compose_with(&bbox))
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
    fn test_make_containers() {
        let bbox1 = BoundingBox([Interval::new(0.0, 1.0); 3]);
        let bbox2 = BoundingBox([Interval::new(1.0, 2.0); 3]);
        let bbox3 = make_cover_of(&bbox1, &bbox2);

        assert_eq!(bbox3[0].start, 0.0);
        assert_eq!(bbox3[0].end, 2.0);
        assert_eq!(bbox3[1].start, 0.0);
        assert_eq!(bbox3[1].end, 2.0);
        assert_eq!(bbox3[2].start, 0.0);
        assert_eq!(bbox3[2].end, 2.0);
    }

    #[test]
    fn test_longest_axis() {
        let bbox1 = BoundingBox([Interval::new(0.0, 1.0); 3]);
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

    #[test]
    fn test_midpoint() {
        let bbox = BoundingBox([
            Interval::new(0.0, 1.0),
            Interval::new(0.0, 2.0),
            Interval::new(0.0, 3.0),
        ]);
        assert_eq!(bbox.midpoint(), Vec3([0.5, 1.0, 1.5]));
    }

    #[test]
    fn test_sorting() {
        let bbox1 = BoundingBox([
            Interval::new(0.0, 1.0),
            Interval::new(0.0, 2.0),
            Interval::new(0.0, 3.0),
        ]); // has midpoint Vec3([0.5,1.0,1.5])
        let bbox2 = BoundingBox([
            Interval::new(-1.0, 1.0),
            Interval::new(-1.0, 1.0),
            Interval::new(-1.0, 1.0),
        ]); // has midpoint Vec3([0.0,0.0,0.0])
        let bbox3 = BoundingBox([
            Interval::new(-2.0, 5.0),
            Interval::new(-2.0, 2.0),
            Interval::new(-1.0, 3.0),
        ]); // has midpoint Vec3([1.5,0.0,1.0])

        let mut list: BoundingBoxList<3> = BoundingBoxList([bbox1, bbox2, bbox3]);
        list.sort_on_index(0);
        assert_eq!(list[1].midpoint(), Vec3([0.5, 1.0, 1.5]));
        list.sort_on_index(2);
        assert_eq!(list[2].midpoint(), Vec3([0.5, 1.0, 1.5]));
    }

    #[test]
    fn test_compose_with() {
        let bbox1 = BoundingBox::empty();
        let bbox2 = BoundingBox([Interval::new(-1.0, 1.0); 3]);

        let cover = bbox1.compose_with(&bbox2);
        assert_eq!(cover.midpoint(), Vec3([0.0; 3]));
    }

    #[test]
    fn test_all_containing() {
        let bbox1 = BoundingBox([Interval::new(0.0, 1.0); 3]);
        let bbox2 = BoundingBox([Interval::new(-1.0, 1.0); 3]);
        let bbox3 = BoundingBox([
            Interval::new(0.0, 2.0),
            Interval::new(0.0, 1.0),
            Interval::new(-1.0, 1.0),
        ]);

        let mut list: BoundingBoxList<3> = BoundingBoxList([bbox1, bbox2, bbox3]);
        let total_cover = list.make_all_containing();
        assert_eq!(total_cover.longest_axis(), 0);

        list.sort_on_index(total_cover.longest_axis());
        assert_eq!(list[1].midpoint(), Vec3([0.5; 3]));
    }
}
