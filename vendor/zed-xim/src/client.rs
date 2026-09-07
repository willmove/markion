mod attribute_builder;

pub use self::attribute_builder::AttributeBuilder;
use crate::AHashMap;
use xim_parser::{
    Attr, Attribute, AttributeName, CaretDirection, CaretStyle, CommitData, Extension, Feedback,
    ForwardEventFlag, PreeditDrawStatus, Request,
};

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum ClientError {
    ReadProtocol(xim_parser::ReadError),
    XimError(xim_parser::ErrorCode, String),
    CompoundTextDecode {
        operation: &'static str,
        reason: &'static str,
    },
    UnsupportedTransport,
    InvalidReply,
    NoXimServer,
    #[cfg(feature = "std")]
    Other(alloc::boxed::Box<dyn std::error::Error + Send + Sync>),
}

impl From<xim_parser::ReadError> for ClientError {
    fn from(e: xim_parser::ReadError) -> Self {
        Self::ReadProtocol(e)
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::ReadProtocol(e) => write!(f, "Can't read xim message: {}", e),
            ClientError::XimError(code, detail) => {
                write!(f, "Server send error code: {:?}, detail: {}", code, detail)
            }
            ClientError::CompoundTextDecode { operation, reason } => {
                write!(
                    f,
                    "Can't decode XIM {} compound text: {}",
                    operation, reason
                )
            }
            ClientError::UnsupportedTransport => write!(f, "Server Transport is not supported"),
            ClientError::InvalidReply => write!(f, "Invalid reply from server"),
            ClientError::NoXimServer => write!(f, "Can't connect xim server"),
            #[cfg(feature = "std")]
            ClientError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ClientError {}

fn decode_compound_text(bytes: &[u8], operation: &'static str) -> Result<String, ClientError> {
    xim_ctext::compound_text_to_utf8(bytes).map_err(|error| {
        let reason = match error {
            xim_ctext::DecodeError::InvalidEncoding => "invalid encoding",
            xim_ctext::DecodeError::UnsupportedEncoding => "unsupported encoding",
            xim_ctext::DecodeError::Utf8Error(_) => "invalid UTF-8",
        };
        ClientError::CompoundTextDecode { operation, reason }
    })
}

pub fn handle_request<C: ClientCore>(
    client: &mut C,
    handler: &mut impl ClientHandler<C>,
    req: Request,
) -> Result<(), ClientError> {
    // Request payloads can contain active preedit or committed user text.
    // Record only the protocol operation name at every log level.
    log::debug!("<-: {}", req.name());

    match req {
        Request::ConnectReply {
            server_major_protocol_version: _,
            server_minor_protocol_version: _,
        } => handler.handle_connect(client),
        Request::OpenReply {
            input_method_id,
            im_attrs,
            ic_attrs,
        } => {
            log::debug!("im_attrs: {:#?}", im_attrs);
            log::debug!("ic_attrs: {:#?}", ic_attrs);
            client.set_attrs(im_attrs, ic_attrs);
            // Require for uim
            client.send_req(Request::EncodingNegotiation {
                encodings: vec!["COMPOUND_TEXT".into()],
                encoding_infos: vec![],
                input_method_id,
            })
        }
        Request::EncodingNegotiationReply {
            input_method_id,
            index: _,
            category: _,
        } => handler.handle_open(client, input_method_id),
        Request::QueryExtensionReply {
            input_method_id: _,
            extensions,
        } => handler.handle_query_extension(client, &extensions),
        Request::GetImValuesReply {
            input_method_id,
            im_attributes,
        } => handler.handle_get_im_values(
            client,
            input_method_id,
            im_attributes
                .into_iter()
                .filter_map(|attr| {
                    client
                        .im_attributes()
                        .iter()
                        .find(|(_, v)| **v == attr.id)
                        .map(|(n, _)| (*n, attr.value))
                })
                .collect(),
        ),
        Request::SetIcValuesReply {
            input_method_id,
            input_context_id,
        } => handler.handle_set_ic_values(client, input_method_id, input_context_id),
        Request::CreateIcReply {
            input_method_id,
            input_context_id,
        } => handler.handle_create_ic(client, input_method_id, input_context_id),
        Request::DestroyIcReply {
            input_method_id,
            input_context_id,
        } => handler.handle_destroy_ic(client, input_method_id, input_context_id),
        Request::SetEventMask {
            input_method_id,
            input_context_id,
            forward_event_mask,
            synchronous_event_mask,
        } => handler.handle_set_event_mask(
            client,
            input_method_id,
            input_context_id,
            forward_event_mask,
            synchronous_event_mask,
        ),
        Request::CloseReply { input_method_id } => handler.handle_close(client, input_method_id),
        Request::DisconnectReply {} => {
            handler.handle_disconnect();
            Ok(())
        }
        Request::ResetIcReply {
            input_method_id,
            input_context_id,
            preedit_string,
        } => {
            let preedit_string = decode_compound_text(&preedit_string, "reset")?;
            handler.handle_reset_ic(client, input_method_id, input_context_id, &preedit_string)
        }
        Request::Error { code, detail, .. } => Err(ClientError::XimError(code, detail)),
        Request::ForwardEvent {
            xev,
            input_method_id,
            input_context_id,
            flag,
            ..
        } => {
            handler.handle_forward_event(
                client,
                input_method_id,
                input_context_id,
                flag,
                client.deserialize_event(&xev),
            )?;

            if flag.contains(ForwardEventFlag::SYNCHRONOUS) {
                client.send_req(Request::SyncReply {
                    input_method_id,
                    input_context_id,
                })?;
            }

            Ok(())
        }
        Request::Commit {
            input_method_id,
            input_context_id,
            data,
        } => match data {
            CommitData::Keysym { keysym: _, .. } => {
                log::warn!("Keysym commit is not supported");
                Ok(())
            }
            CommitData::Chars {
                commited,
                syncronous,
            } => {
                let committed = decode_compound_text(&commited, "commit")?;
                handler.handle_commit(client, input_method_id, input_context_id, &committed)?;

                if syncronous {
                    client.send_req(Request::SyncReply {
                        input_method_id,
                        input_context_id,
                    })?;
                }

                Ok(())
            }
            CommitData::Both { .. } => {
                log::warn!("Both commit data is not supported");
                Ok(())
            }
        },
        Request::Sync {
            input_method_id,
            input_context_id,
        } => client.send_req(Request::SyncReply {
            input_method_id,
            input_context_id,
        }),
        Request::SyncReply { .. } => {
            // Nothing to do
            Ok(())
        }
        Request::PreeditStart {
            input_method_id,
            input_context_id,
        } => handler.handle_preedit_start(client, input_method_id, input_context_id),
        Request::PreeditDone {
            input_method_id,
            input_context_id,
        } => handler.handle_preedit_done(client, input_method_id, input_context_id),
        Request::PreeditDraw {
            input_method_id,
            input_context_id,
            caret,
            chg_first,
            chg_length,
            preedit_string,
            status,
            feedbacks,
        } => {
            let preedit_string = decode_compound_text(&preedit_string, "preedit")?;
            handler.handle_preedit_draw(
                client,
                input_method_id,
                input_context_id,
                caret,
                chg_first,
                chg_length,
                status,
                &preedit_string,
                feedbacks,
            )
        }
        Request::PreeditCaret {
            input_method_id,
            input_context_id,
            mut position,
            direction,
            style,
        } => {
            // Handle the request.
            handler.handle_preedit_caret(
                client,
                input_method_id,
                input_context_id,
                &mut position,
                direction,
                style,
            )?;

            // Send the reply.
            client.send_req(Request::PreeditCaretReply {
                input_method_id,
                input_context_id,
                position,
            })
        }
        _ => {
            log::warn!("Unknown request {:?}", req);
            Ok(())
        }
    }
}

pub trait ClientCore {
    type XEvent;

    fn set_attrs(&mut self, ic_attrs: Vec<Attr>, im_attrs: Vec<Attr>);
    fn ic_attributes(&self) -> &AHashMap<AttributeName, u16>;
    fn im_attributes(&self) -> &AHashMap<AttributeName, u16>;
    fn serialize_event(&self, xev: &Self::XEvent) -> xim_parser::XEvent;
    fn deserialize_event(&self, xev: &xim_parser::XEvent) -> Self::XEvent;
    fn send_req(&mut self, req: Request) -> Result<(), ClientError>;
}

pub trait Client {
    type XEvent;

    fn build_ic_attributes(&self) -> AttributeBuilder<'_>;
    fn build_im_attributes(&self) -> AttributeBuilder<'_>;

    fn disconnect(&mut self) -> Result<(), ClientError>;
    fn open(&mut self, locale: &str) -> Result<(), ClientError>;
    fn close(&mut self, input_method_id: u16) -> Result<(), ClientError>;
    fn quert_extension(
        &mut self,
        input_method_id: u16,
        extensions: &[&str],
    ) -> Result<(), ClientError>;
    fn get_im_values(
        &mut self,
        input_method_id: u16,
        names: &[AttributeName],
    ) -> Result<(), ClientError>;
    fn set_ic_values(
        &mut self,
        input_method_id: u16,
        input_context_id: u16,
        ic_attributes: Vec<Attribute>,
    ) -> Result<(), ClientError>;
    fn create_ic(
        &mut self,
        input_method_id: u16,
        ic_attributes: Vec<Attribute>,
    ) -> Result<(), ClientError>;
    fn destroy_ic(
        &mut self,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError>;
    fn forward_event(
        &mut self,
        input_method_id: u16,
        input_context_id: u16,
        flag: ForwardEventFlag,
        xev: &Self::XEvent,
    ) -> Result<(), ClientError>;
    fn set_focus(&mut self, input_method_id: u16, input_context_id: u16)
        -> Result<(), ClientError>;
    fn unset_focus(
        &mut self,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError>;
    fn reset_ic(&mut self, input_method_id: u16, input_context_id: u16) -> Result<(), ClientError>;
}

impl<C> Client for C
where
    C: ClientCore,
{
    type XEvent = C::XEvent;

    fn build_ic_attributes(&self) -> AttributeBuilder<'_> {
        AttributeBuilder::new(self.ic_attributes())
    }

    fn build_im_attributes(&self) -> AttributeBuilder<'_> {
        AttributeBuilder::new(self.im_attributes())
    }

    fn open(&mut self, locale: &str) -> Result<(), ClientError> {
        self.send_req(Request::Open {
            locale: locale.into(),
        })
    }

    fn quert_extension(
        &mut self,
        input_method_id: u16,
        extensions: &[&str],
    ) -> Result<(), ClientError> {
        self.send_req(Request::QueryExtension {
            input_method_id,
            extensions: extensions.iter().map(|&e| e.into()).collect(),
        })
    }

    fn get_im_values(
        &mut self,
        input_method_id: u16,
        names: &[AttributeName],
    ) -> Result<(), ClientError> {
        self.send_req(Request::GetImValues {
            input_method_id,
            im_attributes: names
                .iter()
                .filter_map(|name| self.im_attributes().get(name).copied())
                .collect(),
        })
    }

    fn set_ic_values(
        &mut self,
        input_method_id: u16,
        input_context_id: u16,
        ic_attributes: Vec<Attribute>,
    ) -> Result<(), ClientError> {
        self.send_req(Request::SetIcValues {
            input_method_id,
            input_context_id,
            ic_attributes,
        })
    }

    fn create_ic(
        &mut self,
        input_method_id: u16,
        ic_attributes: Vec<Attribute>,
    ) -> Result<(), ClientError> {
        self.send_req(Request::CreateIc {
            input_method_id,
            ic_attributes,
        })
    }

    fn forward_event(
        &mut self,
        input_method_id: u16,
        input_context_id: u16,
        flag: ForwardEventFlag,
        xev: &Self::XEvent,
    ) -> Result<(), ClientError> {
        let ev = self.serialize_event(xev);
        self.send_req(Request::ForwardEvent {
            input_method_id,
            input_context_id,
            flag,
            serial_number: ev.sequence,
            xev: ev,
        })
    }

    fn disconnect(&mut self) -> Result<(), ClientError> {
        self.send_req(Request::Disconnect {})
    }

    fn close(&mut self, input_method_id: u16) -> Result<(), ClientError> {
        self.send_req(Request::Close { input_method_id })
    }

    fn destroy_ic(
        &mut self,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError> {
        self.send_req(Request::DestroyIc {
            input_method_id,
            input_context_id,
        })
    }

    fn set_focus(
        &mut self,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError> {
        self.send_req(Request::SetIcFocus {
            input_method_id,
            input_context_id,
        })
    }
    fn unset_focus(
        &mut self,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError> {
        self.send_req(Request::UnsetIcFocus {
            input_method_id,
            input_context_id,
        })
    }
    fn reset_ic(&mut self, input_method_id: u16, input_context_id: u16) -> Result<(), ClientError> {
        self.send_req(Request::ResetIc {
            input_method_id,
            input_context_id,
        })
    }
}

#[allow(unused_variables)]
pub trait ClientHandler<C: Client> {
    fn handle_connect(&mut self, client: &mut C) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_disconnect(&mut self) {}
    fn handle_open(&mut self, client: &mut C, input_method_id: u16) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_close(&mut self, client: &mut C, input_method_id: u16) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_query_extension(
        &mut self,
        client: &mut C,
        extensions: &[Extension],
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_get_im_values(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        attributes: AHashMap<AttributeName, Vec<u8>>,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_set_ic_values(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_create_ic(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_destroy_ic(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_commit(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
        text: &str,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_forward_event(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
        flag: ForwardEventFlag,
        xev: C::XEvent,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_set_event_mask(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
        forward_event_mask: u32,
        synchronous_event_mask: u32,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_preedit_start(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_preedit_draw(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
        caret: i32,
        chg_first: i32,
        chg_len: i32,
        status: PreeditDrawStatus,
        preedit_string: &str,
        feedbacks: Vec<Feedback>,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_preedit_caret(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
        position: &mut i32,
        direction: CaretDirection,
        style: CaretStyle,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_preedit_done(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError> {
        Ok(())
    }
    fn handle_reset_ic(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        input_context_id: u16,
        preedit_text: &str,
    ) -> Result<(), ClientError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    const GB2312: &[u8] = &[
        0x1b, 0x24, 0x28, 0x41, 0x3a, 0x5c, 0x38, 0x5f, 0x50, 0x4b, 0x48, 0x4f, 0x4a, 0x36, 0x44,
        0x63,
    ];
    const KSC5601: &[u8] = &[
        0x1b, 0x24, 0x28, 0x43, 0x33, 0x4d, 0x43, 0x56, 0x30, 0x6d, 0x3e, 0x5f,
    ];
    const MIXED_CN_ASCII: &[u8] = &[
        b'2', b'0', b'2', b'6', 0x1b, 0x24, 0x28, 0x41, 0x44, 0x6a, 0x1b, 0x28, 0x42, b'0', b'7',
        0x1b, 0x24, 0x28, 0x41, 0x54, 0x42, 0x1b, 0x28, 0x42, b'1', b'6', 0x1b, 0x24, 0x28, 0x41,
        0x48, 0x55,
    ];
    const INVALID_94N_DESIGNATOR: &[u8] = &[0x1b, 0x24, 0x28, 0x7f, 0x21, 0x21];

    #[derive(Default)]
    struct TestClient {
        ic_attributes: AHashMap<AttributeName, u16>,
        im_attributes: AHashMap<AttributeName, u16>,
        sent: Vec<Request>,
    }

    impl ClientCore for TestClient {
        type XEvent = ();

        fn set_attrs(&mut self, ic_attrs: Vec<Attr>, im_attrs: Vec<Attr>) {
            self.ic_attributes = ic_attrs
                .into_iter()
                .map(|attr| (attr.name, attr.id))
                .collect();
            self.im_attributes = im_attrs
                .into_iter()
                .map(|attr| (attr.name, attr.id))
                .collect();
        }

        fn ic_attributes(&self) -> &AHashMap<AttributeName, u16> {
            &self.ic_attributes
        }

        fn im_attributes(&self) -> &AHashMap<AttributeName, u16> {
            &self.im_attributes
        }

        fn serialize_event(&self, _xev: &Self::XEvent) -> xim_parser::XEvent {
            unreachable!("text request tests do not serialize X events")
        }

        fn deserialize_event(&self, _xev: &xim_parser::XEvent) -> Self::XEvent {
            unreachable!("text request tests do not deserialize X events")
        }

        fn send_req(&mut self, req: Request) -> Result<(), ClientError> {
            self.sent.push(req);
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingHandler {
        commits: Vec<String>,
        preedits: Vec<String>,
        resets: Vec<String>,
    }

    impl ClientHandler<TestClient> for RecordingHandler {
        fn handle_commit(
            &mut self,
            _client: &mut TestClient,
            _input_method_id: u16,
            _input_context_id: u16,
            text: &str,
        ) -> Result<(), ClientError> {
            self.commits.push(text.into());
            Ok(())
        }

        fn handle_preedit_draw(
            &mut self,
            _client: &mut TestClient,
            _input_method_id: u16,
            _input_context_id: u16,
            _caret: i32,
            _chg_first: i32,
            _chg_len: i32,
            _status: PreeditDrawStatus,
            preedit_string: &str,
            _feedbacks: Vec<Feedback>,
        ) -> Result<(), ClientError> {
            self.preedits.push(preedit_string.into());
            Ok(())
        }

        fn handle_reset_ic(
            &mut self,
            _client: &mut TestClient,
            _input_method_id: u16,
            _input_context_id: u16,
            preedit_text: &str,
        ) -> Result<(), ClientError> {
            self.resets.push(preedit_text.into());
            Ok(())
        }
    }

    fn commit(payload: &[u8]) -> Request {
        Request::Commit {
            input_method_id: 1,
            input_context_id: 2,
            data: CommitData::Chars {
                commited: payload.into(),
                syncronous: false,
            },
        }
    }

    fn preedit(payload: &[u8]) -> Request {
        Request::PreeditDraw {
            input_method_id: 1,
            input_context_id: 2,
            caret: 0,
            chg_first: 0,
            chg_length: 0,
            status: PreeditDrawStatus::empty(),
            preedit_string: payload.into(),
            feedbacks: Vec::new(),
        }
    }

    fn reset(payload: &[u8]) -> Request {
        Request::ResetIcReply {
            input_method_id: 1,
            input_context_id: 2,
            preedit_string: payload.into(),
        }
    }

    #[test]
    fn chinese_and_korean_compound_text_reaches_every_text_handler() {
        for (payload, expected) in [(GB2312, "很高兴认识你"), (KSC5601, "넌최고야")] {
            let mut client = TestClient::default();
            let mut handler = RecordingHandler::default();

            handle_request(&mut client, &mut handler, preedit(payload)).unwrap();
            handle_request(&mut client, &mut handler, commit(payload)).unwrap();
            handle_request(&mut client, &mut handler, reset(payload)).unwrap();

            assert_eq!(handler.preedits, [expected]);
            assert_eq!(handler.commits, [expected]);
            assert_eq!(handler.resets, [expected]);
        }
    }

    #[test]
    fn mixed_chinese_and_ascii_compound_text_is_exact() {
        let mut client = TestClient::default();
        let mut handler = RecordingHandler::default();

        handle_request(&mut client, &mut handler, preedit(MIXED_CN_ASCII)).unwrap();
        handle_request(&mut client, &mut handler, commit(MIXED_CN_ASCII)).unwrap();
        handle_request(&mut client, &mut handler, reset(MIXED_CN_ASCII)).unwrap();

        assert_eq!(handler.preedits, ["2026年07月16日"]);
        assert_eq!(handler.commits, ["2026年07月16日"]);
        assert_eq!(handler.resets, ["2026年07月16日"]);
    }

    #[test]
    fn invalid_compound_text_returns_an_error_without_unwinding() {
        let outcomes: Vec<_> = [
            preedit(INVALID_94N_DESIGNATOR),
            commit(INVALID_94N_DESIGNATOR),
            reset(INVALID_94N_DESIGNATOR),
        ]
        .iter()
        .cloned()
        .map(|request| {
            let mut client = TestClient::default();
            let mut handler = RecordingHandler::default();
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handle_request(&mut client, &mut handler, request)
            }))
        })
        .collect();

        for (outcome, expected_operation) in outcomes
            .into_iter()
            .zip(["preedit", "commit", "reset"].iter())
        {
            let error = outcome
                .expect("compound-text errors must not unwind")
                .expect_err("invalid compound text must return ClientError");
            match &error {
                ClientError::CompoundTextDecode { operation, reason } => {
                    assert_eq!(operation, expected_operation);
                    assert_eq!(*reason, "invalid encoding");
                }
                other => panic!("expected compound-text decode error, got {:?}", other),
            }
            let display = error.to_string();
            assert!(!display.contains("127"));
            assert!(!display.contains("21"));
        }
    }
}
