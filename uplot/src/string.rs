use heapless::consts::U256;
use heapless::String;

pub type PlotStringCapacity = U256;
pub type PlotString = String<PlotStringCapacity>;
