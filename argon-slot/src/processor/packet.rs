pub enum SlotPacket<E, D> {
    Event(E),
    Data(D),
    Forward(tun::TunPacket),
}
