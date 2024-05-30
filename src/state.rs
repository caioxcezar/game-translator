#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    Stopped,
    Started,
    Paused,
}
