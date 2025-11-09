use crate::copart::browser::smf::SmfSizesDecoder;
use crate::copart::browser::ResponseSender;
use crate::copart::response::auction::{plain, solace};
use base64::Engine;
use chromiumoxide::cdp::browser_protocol::network::EventWebSocketFrameReceived;
use chromiumoxide::Page;
use common::io::error::GeneralError;
use futures::StreamExt;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

const SMF_FOOTER_SIZE: usize = 40;

pub struct WsHandler {
    page: Arc<Page>,
    _response_sender: ResponseSender,
}

impl WsHandler {
    pub fn new(page: Arc<Page>, _response_sender: ResponseSender) -> Self {
        Self {
            page,
            _response_sender,
        }
    }

    pub async fn handle_blocking(self) {
        let mut events = self
            .page
            .as_ref()
            .event_listener::<EventWebSocketFrameReceived>()
            .await
            .expect("failed to get ws events");

        while let Some(event) = events.next().await {
            if let Err(e) = self.handle_event(event).await {
                error!("ws event handler error: `{e}`");
            }
        }
    }

    pub fn handle(self) -> JoinHandle<()> {
        tokio::spawn(self.handle_blocking())
    }

    pub fn handle_with_cancel(self, cancellation_token: CancellationToken) -> Arc<Notify> {
        let done = Arc::new(Notify::new());
        let join = tokio::spawn(self.handle_blocking());

        tokio::spawn({
            let done = done.clone();
            async move {
                cancellation_token.cancelled().await;
                join.abort();
                info!("ws handler exited");
                done.notify_waiters();
            }
        });

        done
    }

    async fn handle_event(
        &self,
        event: Arc<EventWebSocketFrameReceived>,
    ) -> Result<(), GeneralError> {
        let payload = ws_payload_by_opcode(&event)?;

        // if the decoded payload is valid utf8, then it means the event comes
        // from the copart infra, and it is represented as a plain JSON message.
        // if the decoded payload is not valid utf8, then it means the event comes
        // from the solace infra, and it is represented as an SMF message.
        if is_valid_utf8(&payload) {
            self.handle_plaintext(payload).await?;
        } else {
            self.handle_solace(payload).await?;
        }

        Ok(())
    }

    async fn handle_plaintext(&self, payload: Vec<u8>) -> Result<(), GeneralError> {
        let msg = serde_json::from_slice::<plain::SoldMessage>(&payload)?;
        info!("ws event plain: {:?}", msg);
        Ok(())
    }

    async fn handle_solace(&self, payload: Vec<u8>) -> Result<(), GeneralError> {
        let decoded = decode_smf(payload)?;
        let msg = serde_json::from_slice::<solace::SoldMessage>(&decoded)?;
        info!("ws event solace: {:?}", msg);
        Ok(())
    }
}

fn ws_payload_by_opcode(
    event: impl AsRef<EventWebSocketFrameReceived>,
) -> Result<Vec<u8>, GeneralError> {
    // based on https://chromedevtools.github.io/devtools-protocol/1-3/Network/#type-WebSocketFrame
    match event.as_ref().response.opcode {
        1. => Ok(event.as_ref().response.payload_data.as_bytes().to_vec()),
        _ => Ok(base64::engine::general_purpose::STANDARD
            .decode(&event.as_ref().response.payload_data)?),
    }
}

fn is_valid_utf8(bytes: impl AsRef<[u8]>) -> bool {
    std::str::from_utf8(bytes.as_ref()).is_ok()
}

fn decode_smf(payload: Vec<u8>) -> Result<Vec<u8>, GeneralError> {
    let smf_sizes = SmfSizesDecoder::decode(payload.as_slice())?;
    // msg bytes include the footer size, so we need to subtract it.
    let msg_size = smf_sizes
        .msg_bytes
        .checked_sub(SMF_FOOTER_SIZE)
        .ok_or(GeneralError::Smf(
            "footer size subtraction failed".to_string(),
        ))?;

    let mut cursor = Cursor::new(payload);
    cursor
        .seek(SeekFrom::Start(smf_sizes.header_bytes as u64))
        .map_err(|e| GeneralError::Smf(format!("seek failed: `{e}`")))?;

    let mut buffer = Vec::with_capacity(msg_size);
    buffer.resize(msg_size, 0);
    cursor
        .read_exact(&mut buffer)
        .map_err(|e| GeneralError::Smf(format!("read exact failed: `{e}`")))?;

    let decoded = base64::engine::general_purpose::STANDARD.decode(&buffer)?;
    Ok(decoded)
}
