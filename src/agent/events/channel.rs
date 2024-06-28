pub(crate) type Sender = tokio::sync::mpsc::UnboundedSender<super::Event>;
pub(crate) type Receiver = tokio::sync::mpsc::UnboundedReceiver<super::Event>;

pub(crate) fn create_channel() -> (Sender, Receiver) {
    tokio::sync::mpsc::unbounded_channel()
}
