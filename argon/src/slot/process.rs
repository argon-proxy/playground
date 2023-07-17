pub struct SlotProcessResult {
    pub forward: Vec<tun::TunPacket>,
    pub exit: Vec<tun::TunPacket>,
}
