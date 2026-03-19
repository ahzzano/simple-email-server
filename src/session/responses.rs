use tokio::io::AsyncWriteExt;

use crate::session::send;

pub(super) enum Response {
    Ok(PositiveResponse), // 200s
    OkHold,               // 354
    Deny(DenyCode),       // 400s
    Error(ErrorCode),     // 500s
}

pub(super) enum PositiveResponse {
    SystemStatus,           // 211
    HelpMessage,            // 214
    ServiceReady,           // 220
    ServiceClosing(String), // 221
    ActionCompleted,        // 250
    UserNotLocal,           // 251
}

pub(super) enum DenyCode {
    DomainServiceNotAvailable, //421
    MailboxUnavailable,        // 450
    ProcessingError,           // 451
    NoStorage,                 // 452
    UnableToAccommodateParams, // 455
}

pub(super) enum ErrorCode {
    SyntaxError,                    // 500
    SyntaxErrorParams,              // 501
    CommandNotImplemented,          // 502
    BadSequence,                    // 503
    CommandParameterNotImplemented, // 504
    MailboxUnavailable,             // 550
    UserNotLocal,                   // 551
    NoStorage,                      // 552
    MailboxNameNotAllowed,          // 553
    TransactionFailed,              // 554
    ParamsNotRecognized,            // 555
}

pub(super) async fn send_response<W>(writer: &mut W, response: Response)
where
    W: AsyncWriteExt + Unpin,
{
    match response {
        Response::Ok(r) => match r {
            PositiveResponse::SystemStatus => send(writer, "211 OK"),
            PositiveResponse::HelpMessage => send(writer, "214 OK"),
            PositiveResponse::ServiceReady => send(writer, "220 OK"),
            PositiveResponse::ServiceClosing(_) => send(writer, "221 OK"),
            PositiveResponse::ActionCompleted => send(writer, "250 OK"),
            PositiveResponse::UserNotLocal => send(writer, "251 OK"),
        },
        Response::OkHold => send(writer, "354 End data with <CR><LF>.<CR><LF>"),
        _ => send(writer, "220 - Ready"),
    }
    .await;
}
