use bytes::Bytes;
use zeromq::ZmqMessage;

pub(in crate::zeromq) fn frames_to_zmq_message(frames: Vec<Vec<u8>>) -> ZmqMessage {
    let mut frames = frames.into_iter();
    let mut message = ZmqMessage::from(frames.next().unwrap_or_default());
    for frame in frames {
        message.push_back(Bytes::from(frame));
    }
    message
}

pub(in crate::zeromq) fn zmq_message_to_frames(message: ZmqMessage) -> Vec<Vec<u8>> {
    message
        .into_vec()
        .into_iter()
        .map(|frame| frame.to_vec())
        .collect()
}
