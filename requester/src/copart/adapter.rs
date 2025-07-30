use crate::copart::sink::MsgIn;
use common::kafka::ReceiveHandle;
use tokio::sync::mpsc::UnboundedSender;

pub struct CopartSinkKafkaAdapter {
    pub cmd_sender: UnboundedSender<MsgIn>,
}

impl ReceiveHandle for CopartSinkKafkaAdapter {
    type RxItem = MsgIn;

    async fn on_message(&self, msg: Self::RxItem) {
        self.cmd_sender.send(msg).unwrap();
    }
}
