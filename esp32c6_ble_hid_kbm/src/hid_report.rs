use bytemuck::Pod;
use bytemuck::Zeroable;
use bytemuck::bytes_of;
use const_array_concat::concat_arrays;
use usbd_hid::descriptor::KeyboardUsage;
use usbd_hid::descriptor::MediaKey;
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::descriptor::gen_hid_descriptor;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (report_id = 0x01,) = {
            (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
                #[packed_bits = 8]
                #[item_settings(data, variable, absolute)]
                modifiers = input;
            };
            (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0xDD) = {
                #[packed_bits = 224]
                #[item_settings(data, variable, absolute)]
                keys_bitmap = input;
            }
        };
    }
)]
#[derive(Default, Pod, Zeroable)]
pub struct KeyboardReport {
    modifiers: u8,
    keys_bitmap: [u8; 28],
}

impl KeyboardReport {
    pub fn set(&mut self, key: KeyboardUsage, pressed: bool) {
        let key_num = key as usize;
        if key_num > 0xE0 {
            set_bit(&mut self.modifiers, key_num & 0b111, pressed);
            return;
        }

        let array_index = key_num / 8;
        assert!(array_index < self.keys_bitmap.len());
        let bit_index = key_num % 8;
        set_bit(&mut self.keys_bitmap[array_index], bit_index, pressed);
    }
}

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MOUSE) = {
        (report_id = 0x02,) = {
            (collection = PHYSICAL, usage = POINTER) = {
                (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_8) = {
                    #[packed_bits = 8]
                    #[item_settings(data, variable, absolute)]
                    buttons = input;
                };
                (usage_page = GENERIC_DESKTOP,) = {
                    (usage = X,) = {
                        #[item_settings(data, variable, relative)] x = input;
                    };
                    (usage = Y,) = {
                        #[item_settings(data, variable, relative)] y = input;
                    };
                };
            };
        };
    }
)]
#[derive(Default, Pod, Zeroable)]
pub struct MouseReport {
    buttons: u8,
    x: i8,
    y: i8,
}

#[repr(u8)]
pub enum MouseUsage {
    Left = 0,
    Right = 1,
    Middle = 2,
    Button4 = 3,
    Button5 = 4,
    Button6 = 5,
    Button7 = 6,
    Button8 = 7,
}

impl MouseReport {
    pub fn new(x: i8, y: i8) -> Self {
        Self { buttons: 0, x, y }
    }

    pub fn set(&mut self, button: MouseUsage, pressed: bool) {
        set_bit(&mut self.buttons, button as usize, pressed);
    }
}

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (report_id = 0x01,) = {
            (usage_page = LEDS, usage_min = 0x01, usage_max = 0x05) = {
                #[packed_bits = 5]
                #[item_settings(data, variable, absolute)]
                leds = output;
            };
        };
    }
)]
#[derive(Pod, Zeroable)]
pub struct KeyboardOutputReport {
    leds: u8,
}

impl KeyboardOutputReport {
    pub fn new(leds: u8) -> Self {
        Self { leds }
    }

    pub fn leds(&self) -> KeyboardLeds {
        KeyboardLeds {
            num_lock: (self.leds & 0x01) != 0,
            caps_lock: (self.leds & 0x02) != 0,
            scroll_lock: (self.leds & 0x04) != 0,
            compose: (self.leds & 0x08) != 0,
            kana: (self.leds & 0x10) != 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KeyboardLeds {
    pub num_lock: bool,
    pub caps_lock: bool,
    pub scroll_lock: bool,
    pub compose: bool,
    pub kana: bool,
}

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = CONSUMER, usage = CONSUMER_CONTROL) = {
        (report_id = 0x03,) = {
            (usage_page = CONSUMER, usage_min = 0x00, usage_max = 0x514) = {
                #[item_settings(data, array, absolute, not_null)]
                usage_id = input;
            };
        };
    }
)]
#[derive(Default, Pod, Zeroable)]
pub struct MediaKeyboardReport {
    usage_id: u16,
}

impl MediaKeyboardReport {
    pub fn set(&mut self, key: MediaKey) {
        self.usage_id = key as u16;
    }
}

pub const HID_DESCRIPTOR: [u8; 144] = concat_arrays!(
    KeyboardReport::DESC,
    KeyboardOutputReport::DESC,
    MouseReport::DESC,
    MediaKeyboardReport::DESC
)
.into_array();

pub const KEYBOARD_REPORT_SIZE: usize = size_of::<KeyboardReport>();
pub const MOUSE_REPORT_SIZE: usize = size_of::<MouseReport>();
pub const MEDIA_KEYBOARD_REPORT_SIZE: usize = size_of::<MediaKeyboardReport>();

macro_rules! impl_hid_report_serialize {
    ($t:ty, $id:expr) => {
        impl $t {
            pub fn serialize(&self) -> [u8; size_of::<$t>()] {
                let payload = bytes_of(self);
                let mut buf = [0; size_of::<$t>()];
                buf[..].copy_from_slice(payload);
                buf
            }
        }
    };
}

impl_hid_report_serialize!(KeyboardReport, 0x01);
impl_hid_report_serialize!(MouseReport, 0x02);
impl_hid_report_serialize!(MediaKeyboardReport, 0x03);

#[inline(always)]
fn set_bit(byte: &mut u8, index: usize, pressed: bool) {
    let bit_pattern = 1 << index;
    if pressed {
        *byte |= bit_pattern;
    } else {
        *byte &= !bit_pattern;
    }
}
