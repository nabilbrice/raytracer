use core::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

// the macro import from the interval module
use crate::interval;

use crate::geometry;
use crate::intervals::{cover, get_larger, intersection, Interval};
use crate::ray::Ray;
use crate::vector::Vec3;
use crate::Hittable;

// the BoundingBox struct is capable of storing any Type that implements intersect()
// the question is: should BoundingBox be the object allocated to the heap or is it a helper?
// these are meant to be operated on sequentially
// but they are also meant to be in the CoveringTree structure
pub struct BoundingBox {
    dims: [Interval; 3],
    boxed: Option<Hittable>,
}

impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox {
            dims: [interval!(0.0, 0.0); 3],
            boxed: None,
        }
    }
}

impl PartialEq for BoundingBox {
    fn eq(&self, rhs: &BoundingBox) -> bool {
        // currently, only the size of the BoundingBox is compared
        self.dims == rhs.dims
    }
}

impl Deref for BoundingBox {
    type Target = Option<Hittable>;

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
            dims: [interval!(0.0, 0.0); 3],
            boxed: None,
        }
    }

    fn with_dims(dims: [Interval; 3]) -> BoundingBox {
        BoundingBox { dims, boxed: None }
    }

    fn dims_copy(&self) -> BoundingBox {
        BoundingBox::with_dims(self.dims.clone())
    }

    // the function should return true if there exists some time parameter
    // for which (ray.orig + t * ray.dir) is in the BoundingBox
    pub fn check_intersection(&self, ray: &Ray) -> bool {
        // the times are generated from the bbox.dims and ray.orig, ray.dir
        // which is difficult to zip [(interval, orig, dir)]
        let mut times = [interval!(0.0, 0.0); 3];
        for i in 0..=2 {
            let divisor: f64;
            if ray.dir[i] == 0.0 {
                divisor = 1.0e-4;
            } else {
                divisor = ray.dir[i];
            }
            let start = (self.dims[i].start - ray.orig[i]) / divisor;
            let end = (self.dims[i].end - ray.orig[i]) / divisor;
            // need to reverse the times ordering in case of
            // negative ray.dir[i]:
            times[i] = interval!(start, end);
            if times[i].size() < 0.0 {
                times[i] = interval!(end, start);
            }
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
    // implementations for sort_on_index can be given a fn
    // currently a size()
    // alternatively, can the usual partial_cmp be used
    // and a specific fn that gets dims[idx].size() be passed?
    fn sort_on_index(&mut self, idx: usize);

    fn make_all_covering(&self) -> BoundingBox;
}

impl BoundingBoxes for &mut [BoundingBox] {
    fn sort_on_index(&mut self, idx: usize) {
        self.sort_unstable_by(|b1, b2| b1.dims[idx].size_partial_cmp(&b2.dims[idx]).unwrap());
    }

    fn make_all_covering(&self) -> BoundingBox {
        self.iter()
            .fold(BoundingBox::empty(), |acc, bbox| acc.compose_with(&bbox))
    }
}

impl BoundingBoxes for [BoundingBox] {
    fn sort_on_index(&mut self, idx: usize) {
        self.sort_unstable_by(|b1, b2| b1.dims[idx].size_partial_cmp(&b2.dims[idx]).unwrap());
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

// this struct should actually only contain the pointer to the BoundingBox
// as once it is constructed, it does not need to mutate it
pub struct CoveringTree {
    pub cover: BoundingBox,
    pub left: Option<Box<CoveringTree>>,
    pub right: Option<Box<CoveringTree>>,
}

impl CoveringTree {
    fn make_from(boxes: &[BoundingBox]) -> CoveringTree {
        CoveringTree {
            cover: boxes.make_all_covering(),
            left: None,
            right: None,
        }
    }
}

// there is a problem in the allocation...
// Box is still dropped at the end of the function...
pub fn make_coveringtree(boxes: &mut [BoundingBox]) -> Box<CoveringTree> {
    if boxes.len() > 1 {
        let mut tree = CoveringTree::make_from(boxes);
        let (left_half, right_half) = split_on_covering(boxes);
        tree.left = Some(make_coveringtree(left_half));
        tree.right = Some(make_coveringtree(right_half));

        Box::new(tree)
    } else {
        Box::new(CoveringTree {
            // need to use the take to grab the first bbox
            cover: std::mem::take(boxes.first_mut().unwrap()),
            left: None,
            right: None,
        })
    }
}

/* a traversal method on the CoveringTree is needed
which tests for intersection and then on its children if true
until no more children to test, whereupon it tests on the BoundingBox boxed Hittable
*/
pub fn tree_filter<'a>(
    root: &'a Box<CoveringTree>,
    subscene: &mut Vec<(&'a Hittable, Option<f64>)>,
    ray: &Ray,
) {
    if root.cover.check_intersection(ray) {
        if let Some(hittable) = &root.cover.boxed {
            let possible_param = hittable.shape.intersect(ray);
            subscene.push((&hittable, possible_param));
        }
        if let Some(left_root) = &root.left {
            tree_filter(left_root, subscene, ray);
        }
        if let Some(right_root) = &root.right {
            tree_filter(right_root, subscene, ray);
        }
    }
}

pub trait Cover {
    // need to move the Hittable into the BoundingBox
    fn make_covering(self) -> BoundingBox;
}

trait EachCover {
    fn make_each_covering(self) -> Vec<BoundingBox>;
}

impl Cover for Hittable {
    fn make_covering(self) -> BoundingBox {
        match &self.shape {
            geometry::Shape::Sphere(sphere) => {
                let dims: [Interval; 3] = sphere
                    .centre
                    .map(|centre| interval!(centre - sphere.radius, centre + sphere.radius));
                BoundingBox {
                    dims,
                    boxed: Some(self),
                }
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

mod tests {
    use super::*;
    use crate::cmp_intersection;
    use crate::color::Color;
    use crate::geometry::Sphere;
    use crate::materials::Material;
    use crate::scenegen;
    use crate::vector::Vec3;
    use crate::Hittable;
    use crate::Shape;

    #[test]
    fn test_bbox_intersection() {
        let bbox = BoundingBox::with_dims([Interval::new(0.0, 1.0); 3]);
        let ray = Ray {
            orig: Vec3([1.5, 0.5, 0.5]),
            dir: Vec3([1.0, 0.0, 0.0]),
        };
        assert!(bbox.check_intersection(&ray));
        let reverseray = Ray {
            orig: Vec3([1.5, 0.5, 0.5]),
            dir: Vec3([-1.0, 0.0, 0.0]),
        };
        assert!(bbox.check_intersection(&reverseray));

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
            interval!(0.0, 1.0),
            interval!(0.0, 2.0),
            interval!(0.0, 3.0),
        ]); // has midpoint Vec3([0.5,1.0,1.5])
        let bbox2 = BoundingBox::with_dims([
            interval!(-1.0, 1.0),
            interval!(-1.0, 1.0),
            interval!(-1.0, 1.0),
        ]); // has midpoint Vec3([0.0,0.0,0.0])
        let bbox3 = BoundingBox::with_dims([
            interval!(-2.0, 5.0),
            interval!(-2.0, 2.0),
            interval!(-1.0, 3.0),
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
            interval!(0.0, 2.0),
            interval!(0.0, 1.0),
            interval!(1.0, 1.0),
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
            interval!(0.0, 1.0),
            interval!(0.0, 2.0),
            interval!(-1.0, 2.0),
        ]);
        let bbox2 = BoundingBox::with_dims([
            interval!(-2.0, 0.0),
            interval!(-3.0, 0.0),
            interval!(-2.0, 0.0),
        ]);
        let bbox3 = BoundingBox::with_dims([
            interval!(-2.0, 1.0),
            interval!(0.0, 1.0),
            interval!(3.0, 4.0),
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
            interval!(0.0, 1.0),
            interval!(0.0, 2.0),
            interval!(-1.0, 2.0),
        ]);
        let bbox2 = BoundingBox::with_dims([
            interval!(-2.0, 0.0),
            interval!(-3.0, 0.0),
            interval!(-2.0, 0.0),
        ]);
        let bbox3 = BoundingBox::with_dims([
            interval!(-2.0, 1.0),
            interval!(0.0, 1.0),
            interval!(3.0, 4.0),
        ]);

        let b3cover = make_cover_of(&bbox3, &BoundingBox::empty());
        let b1b2cover = make_cover_of(&bbox1, &bbox2);

        let mut list = [bbox1, bbox2, bbox3];

        let treebase = make_coveringtree(&mut list);
        assert!(treebase.right.is_some());

        assert!(treebase.right.unwrap().cover == b1b2cover);
    }

    #[test]
    fn test_coveringtree_intersect() {
        let mut covers = Vec::<BoundingBox>::new();
        let sphere1 = Sphere::new(Vec3([0.0, 0.0, 0.0]), 5.0);
        let material1 = Material::Diffuse {
            albedo: Color::new(1.0, 1.0, 1.0),
        };
        let sphere2 = Sphere::new(Vec3([0.0, 0.0, 2.0]), 1.0);
        let material2 = Material::Diffuse {
            albedo: Color::new(1.0, 1.0, 1.0),
        };
        let sphere3 = Sphere::new(Vec3([0.0, 2.0, 0.0]), 1.0);
        let material3 = Material::Diffuse {
            albedo: Color::new(1.0, 1.0, 1.0),
        };
        let hittable1 = Hittable {
            shape: Shape::Sphere(sphere1),
            material: material1,
        };
        covers.push(hittable1.make_covering());
        let hittable2 = Hittable {
            shape: Shape::Sphere(sphere2),
            material: material2,
        };
        covers.push(hittable2.make_covering());
        let hittable3 = Hittable {
            shape: Shape::Sphere(sphere3),
            material: material3,
        };
        covers.push(hittable3.make_covering());

        let tree = {
            let mut boxes = covers.into_boxed_slice();
            make_coveringtree(&mut boxes)
        };

        let mut subscene = Vec::<(&Hittable, Option<f64>)>::new();
        let ray = Ray {
            orig: Vec3([-1.5, -0.5, -0.5]),
            dir: Vec3([1.0, 0.0, 0.0]),
        };
        tree_filter(&tree, &mut subscene, &ray);

        if let Some((hittable, Some(param))) =
            subscene.iter().min_by(|x, y| cmp_intersection(x.1, y.1))
        {
            assert!(param.is_finite(), "gone into");
        }

        assert!(tree.cover.check_intersection(&ray), "intersection failed!");
        assert!(!subscene.is_empty(), "subscene should contain hittable1");
    }

    #[test]
    fn test_debug_scene() {
        let tree = scenegen::debug_scene();
        let mut subscene = Vec::<(&Hittable, Option<f64>)>::new();
        let outray = Ray {
            orig: Vec3([10.0, 0.0, 0.0]),
            dir: Vec3([1.0, 0.0, 0.0]),
        };
        tree_filter(&tree, &mut subscene, &outray);
    }
}
