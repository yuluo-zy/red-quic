use crate::Digest;
use crate::services::multi_map::MultiMap;

pub type ControlChannelMap = MultiMap<Digest, Digest, ControlChannelHandle>;

pub struct  ControlChannelHandle {

}