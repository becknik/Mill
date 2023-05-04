pub mod game {

    pub mod logic;
    //pub state; TODO
    pub mod state;
    pub enum PlayerColor {
        White = 0b10,
        Black = 0b01,
    }
}
