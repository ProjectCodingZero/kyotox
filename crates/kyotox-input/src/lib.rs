use bitflags::bitflags;
pub enum Key {

    Char(char),
    Enter, Backspace, Tab, BackTab, Esc,
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,
    Insert, Delete,
    F(u8),
}
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Modifiers: u8{
        const SHIFT             = 1 << 0;
        const CTRL              = 1 << 1;
        const ALT               = 1 << 2;
    }
}
pub fn encode(_key: Key, _mods: Modifiers) -> Vec<u8>{
    Vec::new()
}
pub fn encode_to(_buf: &mut Vec<u8>, _key: Key, _mods: Modifiers){}