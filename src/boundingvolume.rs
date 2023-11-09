use core::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::geometry;
use crate::intervals::{cover, get_larger, intersection, Interval};
use crate::ray::Ray;
use crate::vector::Vec3;
use crate::Hittable;

struct BoundingBox {
    dims: [Interval; 3],
    boxed: Option<&'static Hittable>,
}

impl Deref for BoundingBox {
    type Target = Option<&'static Hittable>;

    fn deref(&self) -> &Self::Target {
        &self.boxed
    }
}

impl DerefMut for BoundingBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.boxed
    }
}

impl BoundingBox {
    fn empty() -> BoundingBox {
        BoundingBox {
            dims: [Interval::new(0.0, 0.0); 3],
            boxed: None,
        }
    }

    fn with_dims(dims: [Interval; 3]) -> BoundingBox {
        BoundingBox { dims, boxed: None }
    }

    fn check_intersection(&self, ray: &Ray) -> bool {
        let mut times = [Interval::new(0.0, 0.0); 3];
        for i in 0..=2 {
            let start = (self.dims[i].start - ray.orig[i]) / ray.dir[i];
            let end = (self.dims[i].end - ray.orig[i]) / ray.dir[i];
            times[i] = Interval::new(start, end);
        }

        let Some(xy) = intersection(&times[0], &times[1]) else {
            return false;
        };
        return intersection(&xy, &times[2]).is_some();
    }

    // for use in the node split by longest axis
    fn longest_axis(&self) -> usize {
        let x_size = self.dims[0].size();
        let y_size = self.dims[1].size();
        let z_size = self.dims[2].size();

        if x_size > y_size && x_size > z_size {
            return 0;
        }
        if y_size > z_size {
            return 1;
        }
        2
    }

    fn midpoint(&self) -> Vec3 {
        Vec3([
            self.dims[0].midpoint(),
            self.dims[1].midpoint(),
            self.dims[2].midpoint(),
        ])
    }

    // this composition consumes the self and creates a new one
    fn compose_with(self, other: &BoundingBox) -> BoundingBox {
        make_cover_of(&self, other)
    }
}

fn make_cover_of(bbox1: &BoundingBox, bbox2: &BoundingBox) -> BoundingBox {
    BoundingBox {
        dims: [
            cover(&bbox1.dims[0], &bbox2.dims[0]),
            cover(&bbox1.dims[1], &bbox2.dims[1]),
            cover(&bbox1.dims[2], &bbox2.dims[2]),
        ],
        boxed: None,
    }
}

trait BoundingBoxes {
    fn sort_on_index(&mut self, idx: usize);

    fn make_all_covering(&self) -> BoundingBox;
}

impl BoundingBoxes for &mut [BoundingBox] {
    fn sort_on_index(&mut self, idx: usize) {
        self.sort_unstable_by(|b1, b2| {
            b1.dims[idx]
                .size()
                .partial_cmp(&b2.dims[idx].size())
                .unwrap()
        });
    }

    fn make_all_covering(&self) -> BoundingBox {
        self.iter()
            .fold(BoundingBox::empty(), |acc, bbox| acc.compose_with(&bbox))
    }
}

impl BoundingBoxes for [BoundingBox] {
    fn sort_on_index(&mut self, idx: usize) {
        self.sort_unstable_by(|b1, b2| {
            b1.dims[idx]
                .size()
                .partial_cmp(&b2.dims[idx].size())
                .unwrap()
        });
    }

    fn make_all_covering(&self) -> BoundingBox {
        self.iter()
            .fold(BoundingBox::empty(), |acc, bbox| acc.compose_with(&bbox))
    }
}

fn split_on_covering(boxes: &mut [BoundingBox]) -> (&mut [BoundingBox], &mut [BoundingBox]) {
    let halfway: usize = boxes.len() / 2;
    let covering = boxes.make_all_covering();
    boxes.sort_on_index(covering.longest_axis());

    let (left_half, right_half) = boxes.split_at_mut(halfway);
    (left_half, right_half)
}

struct CoveringTree {
    cover: BoundingBox,
    left: Option<Box<CoveringTree>>,
    right: Option<Box<CoveringTree>>,
}

fn make_coveringtree(boxes: &mut [BoundingBox]) -> Box<CoveringTree> {
    let covering = boxes.make_all_covering();
    let mut coveringtree = CoveringTree {
        cover: covering,
        left: None,
        right: None,
    };

    let (left_half, right_half) = split_on_covering(boxes);
    if left_half.len() > 1 {
        coveringtree.left = Some(make_coveringtree(left_half));
    };
    if right_half.len() > 1 {
        coveringtree.right = Some(make_coveringtree(right_half));
    }

    Box::new(coveringtree)
}

trait Cover {
    fn make_covering(&self) -> BoundingBox;
}

trait EachCover {
    fn make_each_covering(&self) -> Box<[BoundingBox]>;
}

impl Cover for geometry::Sphere {
    fn make_covering(&self) -> BoundingBox {
        let midpoint = self.centre;
        let mut cover = BoundingBox::empty();
        for i in 0..2 {
            cover.dims[i] =
                Interval::new(self.centre[i] - self.radius, self.centre[i] + self.radius);
        }
        cover
    }
}

impl EachCover for [geometry::Sphere] {
    fn make_each_covering(&self) -> Box<[BoundingBox]> {
        self.iter().map(|sphere| sphere.make_covering()).collect()
    }
}

mod tests {
    use super::*;
    use crate::vector::Vec3;
    use crate::Hittable;

    #[test]
    fn test_bbox_intersection() {
        let bbox = BoundingBox::with_dims([Interval::new(0.0, 1.0); 3]);
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
        let bbox1 = BoundingBox::with_dims([Interval::new(0.0, 1.0); 3]);
        let bbox2 = BoundingBox::with_dims([Interval::new(1.0, 2.0); 3]);
        let bbox3 = make_cover_of(&bbox1, &bbox2);

        assert_eq!(bbox3.dims[0].start, 0.0);
        assert_eq!(bbox3.dims[0].end, 2.0);
        assert_eq!(bbox3.dims[1].start, 0.0);
        assert_eq!(bbox3.dims[1].end, 2.0);
        assert_eq!(bbox3.dims[2].start, 0.0);
        assert_eq!(bbox3.dims[2].end, 2.0);
    }

    #[test]
    fn test_longest_axis() {
        let bbox1 = BoundingBox::with_dims([Interval::new(0.0, 1.0); 3]);
        assert_eq!(bbox1.longest_axis(), 2);

        let bbox2 = BoundingBox::with_dims([
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
        let bbox = BoundingBox::with_dims([
            Interval::new(0.0, 1.0),
            Interval::new(0.0, 2.0),
            Interval::new(0.0, 3.0),
        ]);
        assert_eq!(bbox.midpoint(), Vec3([0.5, 1.0, 1.5]));
    }

    #[test]
    fn test_sorting() {
        let bbox1 = BoundingBox::with_dims([
            Interval::new(0.0, 1.0),
            Interval::new(0.0, 2.0),
            Interval::new(0.0, 3.0),
        ]); // has midpoint Vec3([0.5,1.0,1.5])
        let bbox2 = BoundingBox::with_dims([
            Interval::new(-1.0, 1.0),
            Interval::new(-1.0, 1.0),
            Interval::new(-1.0, 1.0),
        ]); // has midpoint Vec3([0.0,0.0,0.0])
        let bbox3 = BoundingBox::with_dims([
            Interval::new(-2.0, 5.0),
            Interval::new(-2.0, 2.0),
            Interval::new(-1.0, 3.0),
        ]); // has midpoint Vec3([1.5,0.0,1.0])

        let mut list = [bbox1, bbox2, bbox3];
        list.sort_on_index(0);
        assert_eq!(list[0].midpoint(), Vec3([0.5, 1.0, 1.5]));
        list.sort_on_index(2);
        assert_eq!(list[1].midpoint(), Vec3([0.5, 1.0, 1.5]));
    }

    #[test]
    fn test_compose_with() {
        let bbox1 = BoundingBox::empty();
        let bbox2 = BoundingBox::with_dims([Interval::new(-1.0, 1.0); 3]);

        let cover = bbox1.compose_with(&bbox2);
        assert_eq!(cover.midpoint(), Vec3([0.0; 3]));
    }

    #[test]
    fn test_all_covering() {
        let bbox1 = BoundingBox::with_dims([Interval::new(0.0, 1.0); 3]);
        let bbox2 = BoundingBox::with_dims([Interval::new(-1.0, 1.0); 3]);
        let bbox3 = BoundingBox::with_dims([
            Interval::new(0.0, 2.0),
            Interval::new(0.0, 1.0),
            Interval::new(-1.0, 1.0),
        ]);

        let mut list = [bbox1, bbox2, bbox3];
        let total_cover = list.make_all_covering();
        assert_eq!(total_cover.longest_axis(), 0);

        list.sort_on_index(total_cover.longest_axis());
        assert_eq!(list[0].midpoint(), Vec3([0.5; 3]));
    }

    #[test]
    fn test_splitting() {
        let bbox1 = BoundingBox::with_dims([
            Interval::new(0.0, 1.0),
            Interval::new(0.0, 2.0),
            Interval::new(-1.0, 2.0),
        ]);
        let bbox2 = BoundingBox::with_dims([
            Interval::new(-2.0, 0.0),
            Interval::new(-3.0, 0.0),
            Interval::new(-2.0, 0.0),
        ]);
        let bbox3 = BoundingBox::with_dims([
            Interval::new(-2.0, 1.0),
            Interval::new(0.0, 1.0),
            Interval::new(3.0, 4.0),
        ]);

        let bbox1_midpoint = bbox1.midpoint();
        let bbox2_midpoint = bbox2.midpoint();
        let bbox3_midpoint = bbox3.midpoint();

        let mut list = [bbox1, bbox2, bbox3];
        let total_cover = list.make_all_covering(); // (-2.0,1.0), (-3.0,2.0), (-2.0,4.0)
        list.sort_on_index(total_cover.longest_axis());
        assert_eq!(list[0].midpoint(), bbox3_midpoint);

        let (left_half, right_half) = split_on_covering(&mut list);
        assert_eq!(right_half[0].midpoint(), bbox2_midpoint);
        let right_cover = right_half.make_all_covering();
        assert_eq!(right_cover.longest_axis(), 1);
        right_half.sort_on_index(right_cover.longest_axis());
        assert_eq!(right_half[0].midpoint(), bbox1_midpoint);
    }

    #[test]
    fn test_coveringtree() {
        let bbox1 = BoundingBox::with_dims([
            Interval::new(0.0, 1.0),
            Interval::new(0.0, 2.0),
            Interval::new(-1.0, 2.0),
        ]);
        let bbox2 = BoundingBox::with_dims([
            Interval::new(-2.0, 0.0),
            Interval::new(-3.0, 0.0),
            Interval::new(-2.0, 0.0),
        ]);
        let bbox3 = BoundingBox::with_dims([
            Interval::new(-2.0, 1.0),
            Interval::new(0.0, 1.0),
            Interval::new(3.0, 4.0),
        ]);

        let b3cover = make_cover_of(&bbox3, &BoundingBox::empty());
        let b1b2cover = make_cover_of(&bbox1, &bbox2);

        let mut list = [bbox1, bbox2, bbox3];

        let treebase = make_coveringtree(&mut list);
        assert!(treebase.right.is_some());

        assert_eq!(
            treebase.right.unwrap().cover.midpoint(),
            b1b2cover.midpoint()
        );
    }
}
