use crate::image::ImageIterator;
#[allow(missing_docs)]
mod battery_full;
#[allow(missing_docs)]
mod battery_half;
#[allow(missing_docs)]
mod battery_quarter;
#[allow(missing_docs)]
mod battery_three_quarter;
#[allow(missing_docs)]
mod bluetooth_connected;
#[allow(missing_docs)]
mod bluetooth_disconnected;
#[allow(missing_docs)]
pub enum Icon {
    BatteryFull,
    BatteryHalf,
    BatteryQuarter,
    BatteryThreeQuarter,
    BluetoothConnected,
    BluetoothDisconnected,
}
impl Icon {
    #[allow(missing_docs)]
    pub fn get_image<'a>(&self) -> ImageIterator<'a> {
        match &self {
            Icon::BatteryFull => battery_full::IMAGE_DATA.to_iterator(),
            Icon::BatteryHalf => battery_half::IMAGE_DATA.to_iterator(),
            Icon::BatteryQuarter => battery_quarter::IMAGE_DATA.to_iterator(),
            Icon::BatteryThreeQuarter => battery_three_quarter::IMAGE_DATA.to_iterator(),
            Icon::BluetoothConnected => bluetooth_connected::IMAGE_DATA.to_iterator(),
            Icon::BluetoothDisconnected => bluetooth_disconnected::IMAGE_DATA.to_iterator(),
        }
    }
}
