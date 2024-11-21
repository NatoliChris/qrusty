#[derive(Clone, Debug)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug)]
pub struct BoundingBox {
    pub top_left: Coord,
    pub top_right: Coord,
    pub bottom_left: Coord,
    pub bottom_right: Coord,
}

impl BoundingBox {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            top_left: Coord { x, y },
            top_right: Coord { x: x + width, y },
            bottom_left: Coord { x, y: y + height },
            bottom_right: Coord {
                x: x + width,
                y: y + height,
            },
        }
    }

    pub fn new_from_coords(x: i32, y: i32, x_2: i32, y_2: i32) -> Self {
        Self {
            top_left: Coord { x, y },
            top_right: Coord { x: x_2, y },
            bottom_left: Coord { x, y: y_2 },
            bottom_right: Coord { x: x_2, y: y_2 },
        }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.top_right.x <= other.bottom_left.x
            || self.bottom_left.x >= other.top_right.x
            || self.top_right.y >= other.bottom_left.y
            || self.bottom_left.y <= other.top_right.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ////////////////////////////////////////
    // Non intersecting boxes
    // ////////////////////////////////////////

    #[test]
    fn left_right_no_intersect() {
        let box_one = BoundingBox::new(0, 0, 5, 5);
        let box_two = BoundingBox::new(10, 0, 10, 10);

        assert!(!box_one.intersects(&box_two), "Boxes do not intersect");
        assert!(!box_two.intersects(&box_one), "Boxes do not intersect");
    }

    #[test]
    fn top_bottom_no_intersect() {
        let box_one = BoundingBox::new(0, 0, 5, 5);
        let box_two = BoundingBox::new(0, 6, 5, 5);

        assert!(!box_one.intersects(&box_two), "Boxes do not intersect");
        assert!(!box_two.intersects(&box_one), "Boxes do not intersect");
    }

    #[test]
    fn test_neg_coords() {
        let box_one = BoundingBox::new(0, 0, 5, 5);
        let box_two = BoundingBox::new(-6, -6, 5, 5);

        assert!(!box_one.intersects(&box_two), "Boxes do not intersect");
        assert!(!box_two.intersects(&box_one), "Boxes do not intersect");
    }

    #[test]
    fn test_right_next_to_each_other() {
        let box_one = BoundingBox::new(0, 0, 5, 5);
        let box_two = BoundingBox::new(5, 0, 5, 5);
        let box_three = BoundingBox::new(0, 5, 5, 5);

        assert!(!box_one.intersects(&box_two), "Boxes do not intersect");
        assert!(!box_two.intersects(&box_one), "Boxes do not intersect");
        assert!(!box_one.intersects(&box_three), "Boxes do not intersect");
        assert!(!box_two.intersects(&box_three), "Boxes do not intersect");
    }
}
