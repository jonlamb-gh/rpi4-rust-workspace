use crate::{frame, PlotString, Point2D};
use heapless::consts::U10 as LabelStorageCapacity;
use heapless::Vec;

#[derive(Debug, Clone, Default)]
pub struct Label {
    pub(crate) string: PlotString,
    pub(crate) pos: Point2D<frame::Window>,
}

#[derive(Debug, Clone, Default)]
pub struct LabelStorage {
    pub(crate) labels: Vec<Label, LabelStorageCapacity>,
    pub(crate) value_label: PlotString,
}

impl LabelStorage {
    pub fn new() -> Self {
        LabelStorage::default()
    }
}
