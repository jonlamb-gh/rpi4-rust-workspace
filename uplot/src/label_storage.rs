use crate::string::PlotString;
use heapless::consts::U10 as LabelStorageCapacity;
use heapless::Vec;

#[derive(Debug, Clone, Default)]
pub struct Label {
    pub(crate) string: PlotString,
    pub(crate) x_to: i32,
    pub(crate) y_from: i32,
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
