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

impl PartialEq for BoundingBox {
    fn eq(&self, rhs: &BoundingBox) -> bool {
        // currently, only the size of the BoundingBox is compared
        self.dims == rhs.dims
    }
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

    fn dims_copy(&self) -> BoundingBox {
        BoundingBox::with_dims(self.dims.clone())
    }

    fn check_intersection(&self, ray: &Ray) -> bool {
        // the times are generated from the bbox.dims and ray.orig, ray.dir
        // which is difficult to zip [(interval, orig, dir)]
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
        let sizes: [f64; 3] = self.dims.map(|interval| interval.size());

        if sizes[0] > sizes[1] && sizes[0] > sizes[2] {
            return 0;
        }
        if sizes[1] > sizes[2] {
            return 1;
        }
        2
    }

    fn midpoint(&self) -> Vec3 {
        Vec3(self.dims.map(|interval| interval.midpoint()))
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
        let dims: [Interval; 3] = self
            .centre
            .map(|centre| Interval::new(centre - self.radius, centre + self.radius));
        BoundingBox::with_dims(dims)
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

        let mut list = [bbox1.dims_copy(), bbox2.dims_copy(), bbox3.dims_copy()];
        list.sort_on_index(0);
        assert!(list[0] == bbox1);
        list.sort_on_index(2);
        assert!(list[1] == bbox1);
    }

    #[test]
    fn test_compose_with() {
        let bbox1 = BoundingBox::empty();
        let bbox2 = BoundingBox::with_dims([Interval::new(-1.0, 1.0); 3]);

        let cover = bbox1.compose_with(&bbox2);
        assert!(cover == bbox2);
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

        let mut list = [bbox1.dims_copy(), bbox2, bbox3];
        let total_cover = list.make_all_covering();
        assert_eq!(total_cover.longest_axis(), 0);

        list.sort_on_index(total_cover.longest_axis());
        assert!(list[0] == bbox1);
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

        let mut list = [bbox1.dims_copy(), bbox2.dims_copy(), bbox3.dims_copy()];
        let total_cover = list.make_all_covering(); // (-2.0,1.0), (-3.0,2.0), (-2.0,4.0)
        list.sort_on_index(total_cover.longest_axis());
        assert!(list[0] == bbox3);

        let (left_half, right_half) = split_on_covering(&mut list);
        assert!(right_half[0] == bbox2);
        let right_cover = right_half.make_all_covering();
        assert_eq!(right_cover.longest_axis(), 1);
        right_half.sort_on_index(right_cover.longest_axis());
        assert!(right_half[0] == bbox1);
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

        assert!(treebase.right.unwrap().cover == b1b2cover);
    }
}
