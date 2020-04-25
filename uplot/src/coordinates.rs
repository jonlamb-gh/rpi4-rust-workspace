use embedded_graphics::geometry::Point;

//struct World;
//struct Window;
//
//pub trait CoordinateFrame {}
//impl CoordinateFrame for World {}
//impl CoordinateFrame for Window {}

pub trait CoordinateFrame: private::SealedCoordinateFrame {}

pub mod frame {
    use super::CoordinateFrame;

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct World;
    impl CoordinateFrame for World {}

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    pub struct Window;
    impl CoordinateFrame for Window {}
}

mod private {
    use super::frame::{Window, World};
    pub trait SealedCoordinateFrame {}
    impl SealedCoordinateFrame for World {}
    impl SealedCoordinateFrame for Window {}
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Position<F: CoordinateFrame> {
    pub p: i32,
    _marker: core::marker::PhantomData<F>,
}

impl<F: CoordinateFrame> Position<F> {
    pub fn new(p: i32) -> Self {
        Position {
            p,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<F: CoordinateFrame> From<i32> for Position<F> {
    fn from(p: i32) -> Position<F> {
        Position::new(p)
    }
}

impl<F: CoordinateFrame> Into<i32> for Position<F> {
    fn into(self) -> i32 {
        self.p
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Point2D<F: CoordinateFrame> {
    pub p: Point,
    _marker: core::marker::PhantomData<F>,
}

impl<F: CoordinateFrame> Point2D<F> {
    pub fn new(p: Point) -> Self {
        Point2D {
            p,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<F: CoordinateFrame> From<Point> for Point2D<F> {
    fn from(p: Point) -> Point2D<F> {
        Point2D::new(p)
    }
}

impl<F: CoordinateFrame> Into<Point> for Point2D<F> {
    fn into(self) -> Point {
        self.p
    }
}

impl<F: CoordinateFrame> From<(Position<F>, Position<F>)> for Point2D<F> {
    fn from(p: (Position<F>, Position<F>)) -> Point2D<F> {
        Point2D::new(Point::new(p.0.p, p.1.p))
    }
}

// TODO - shouldn't need these...
//impl<F: CoordinateFrame> From<(Position<frame::Window>,
// Position<frame::World>)> for Point2D<F> {    fn from(p:
// (Position<frame::Window>, Position<frame::World>)) -> Point2D<F> {
//        Point2D::new(Point::new(p.0.p, p.1.p))
//    }
//}
//
//impl<F: CoordinateFrame> From<(Position<frame::World>,
// Position<frame::Window>)> for Point2D<F> {    fn from(p:
// (Position<frame::World>, Position<frame::Window>)) -> Point2D<F> {
//        Point2D::new(Point::new(p.0.p, p.1.p))
//    }
//}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Range1D<F: CoordinateFrame> {
    pub r: (i32, i32),
    _marker: core::marker::PhantomData<F>,
}

impl<F: CoordinateFrame> Range1D<F> {
    pub fn new(r: (i32, i32)) -> Self {
        Range1D {
            r,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<F: CoordinateFrame> From<(i32, i32)> for Range1D<F> {
    fn from(r: (i32, i32)) -> Range1D<F> {
        Range1D::new(r)
    }
}

impl<F: CoordinateFrame> Into<(i32, i32)> for Range1D<F> {
    fn into(self) -> (i32, i32) {
        self.r
    }
}

impl Range1D<frame::World> {
    pub fn scale(
        &self,
        p: Position<frame::World>,
        range: &Range1D<frame::Window>,
    ) -> Position<frame::Window> {
        let from = (self.r.0 as f32, self.r.1 as f32);
        let to = (range.r.0 as f32, range.r.1 as f32);
        let sp = map_range(p.p as f32, from, to);
        Position::new(sp as i32)
    }
}

fn map_range(s: f32, from_range: (f32, f32), to_range: (f32, f32)) -> f32 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}
