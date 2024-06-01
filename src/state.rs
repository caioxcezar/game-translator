#[derive(Clone, Default, Debug, PartialEq)]
pub enum State {
    #[default]
    Stopped,
    Started,
    Paused,
}
