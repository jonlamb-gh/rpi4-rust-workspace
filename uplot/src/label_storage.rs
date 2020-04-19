use crate::string::PlotString;
use heapless::consts::U10 as LabelStorageCapacity;
use heapless::Vec;

#[derive(Debug, Clone, Default)]
pub struct Label {
    pub(crate) string: PlotString,
    pub(crate) x_to: i32,
    pub(crate) y_from: i32,
}

pub type LabelStorage = Vec<Label, LabelStorageCapacity>;
