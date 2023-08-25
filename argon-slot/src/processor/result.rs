// TODO: repr(C) vec
pub struct SlotProcessorResult {
    pub forward: Vec<tun::TunPacket>,
    pub exit: Vec<tun::TunPacket>,
}
