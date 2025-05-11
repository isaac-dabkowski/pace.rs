mod izaw;
mod nxs;
mod jxs;
mod xxs;

pub use izaw::IzawArray;
pub use nxs::NxsArray;
pub use jxs::JxsArray;
pub use xxs::XxsArray;

pub struct Arrays <'a> {
    pub nxs: &'a NxsArray,
    pub jxs: &'a JxsArray,
    pub xxs: &'a XxsArray,
}