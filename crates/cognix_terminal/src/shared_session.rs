use crate::model::Point;

impl From<Point> for cognix_session::common::Point {
    fn from(val: Point) -> Self {
        cognix_session::common::Point {
            row: val.row,
            col: val.col,
        }
    }
}

impl From<cognix_session::common::Point> for Point {
    fn from(value: cognix_session::common::Point) -> Self {
        Self {
            row: value.row,
            col: value.col,
        }
    }
}
