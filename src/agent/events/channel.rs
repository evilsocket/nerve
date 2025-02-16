use tokio::sync::broadcast;

pub type Sender = broadcast::Sender<super::Event>;
pub type Receiver = broadcast::Receiver<super::Event>;

pub fn create_channel() -> (Sender, Receiver) {
    broadcast::channel(0xffff)
}
