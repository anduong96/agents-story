use crate::stream::protocol::StreamEvent;

#[derive(Debug)]
pub enum ReaderMessage {
    Event {
        session_id: String,
        project: String,
        event: StreamEvent,
    },
    SessionEnded {
        session_id: String,
    },
    ReaderError {
        session_id: String,
        #[allow(dead_code)]
        error: String,
    },
}
