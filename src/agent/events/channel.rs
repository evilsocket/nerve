pub type Sender = tokio::sync::mpsc::UnboundedSender<super::Event>;
pub type Receiver = tokio::sync::mpsc::UnboundedReceiver<super::Event>;

pub fn create_channel() -> (Sender, Receiver) {
    tokio::sync::mpsc::unbounded_channel()
}
