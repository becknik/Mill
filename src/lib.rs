/// The purpose of this module is to share contents which are important for the games coordination (player handling, game phase enforcement) and the play field storing the state of the game and abstractions around it.
pub mod game {
    use std::fmt::Display;

    use self::state::representation::types::FieldState;

    pub mod logic;
    mod state;

    #[derive(Debug, Clone, Copy)]
    pub enum PlayerColor {
        White = 0b10,
        Black = 0b01,
    }

    impl Display for PlayerColor {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                PlayerColor::White => f.write_str("●"),
                PlayerColor::Black => f.write_str("X"),
            }
        }
    }

    impl Into<FieldState> for PlayerColor {
        fn into(self) -> FieldState {
            match self {
                PlayerColor::White => FieldState::White,
                PlayerColor::Black => FieldState::Black,
            }
        }
    }
}
