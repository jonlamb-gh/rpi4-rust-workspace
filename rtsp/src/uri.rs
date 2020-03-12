// TODO - need a proper URI

use heapless::consts::U256;
use heapless::String;

pub type UriCapacity = U256;

pub type Uri = String<UriCapacity>;
