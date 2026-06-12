use std::{
    str::FromStr,
    thread,
    time::{Duration, Instant},
};

use zeromq::{
    DealerSocket, ReqSocket, Socket, SocketOptions, SocketRecv, SocketSend, SubSocket, ZmqMessage,
    util::PeerIdentity,
};

use rum_host::HostSession as Session;

use super::*;
use crate::{
    KernelConnectionFile, KernelMessageHeader, KernelMetadata, KernelReplyStatus, KernelRequest,
    KernelRequestEnvelope, KernelResponse, decode_response_message,
};

mod fake {
    include!("../../zeromq_fake_tests.rs");
}

mod support {
    include!("../../zeromq_native_test_support.rs");
}

use self::support::{
    loopback_connection, signed_loopback_connection, test_runtime, wait_for_next_socket_event,
    wait_for_socket_event,
};

mod native;
