//! Test-only helpers: a scriptable [`Interface`] implementation, so command
//! logic can be exercised end-to-end without a real BMC.
//!
//! Output capture lives in [`crate::output::capture`]; this module only provides
//! the mock transport and response builders.

use std::sync::{Arc, Mutex};

use ipmi_core::error::Result;
use ipmi_core::interface::Interface;
use ipmi_core::types::{Request, Response, SessionParams};

/// A responder: given a request, produce a response or a transport error.
type Responder = Box<dyn FnMut(&Request) -> Result<Response> + Send>;

/// A scriptable transport. Every request passed to `sendrecv` is recorded, and
/// the configured responder produces the reply.
pub struct MockInterface {
    pub params: SessionParams,
    requests: Arc<Mutex<Vec<Request>>>,
    responder: Responder,
    opened: bool,
}

impl MockInterface {
    /// Build a mock whose requests are answered by `responder`.
    pub fn new(responder: impl FnMut(&Request) -> Result<Response> + Send + 'static) -> Self {
        MockInterface {
            params: SessionParams::default(),
            requests: Arc::new(Mutex::new(Vec::new())),
            responder: Box::new(responder),
            opened: false,
        }
    }

    /// A mock that answers every request with an empty success.
    pub fn always_ok() -> Self {
        Self::new(|_| Ok(ok(Vec::new())))
    }

    /// A snapshot of the recorded requests.
    pub fn recorded(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }
}

impl Interface for MockInterface {
    fn name(&self) -> &str {
        "mock"
    }

    fn open(&mut self) -> Result<()> {
        self.opened = true;
        Ok(())
    }

    fn close(&mut self) {
        self.opened = false;
    }

    fn is_open(&self) -> bool {
        self.opened
    }

    fn sendrecv(&mut self, req: &Request) -> Result<Response> {
        self.requests.lock().unwrap().push(req.clone());
        (self.responder)(req)
    }

    fn session_params(&self) -> &SessionParams {
        &self.params
    }

    fn session_params_mut(&mut self) -> &mut SessionParams {
        &mut self.params
    }
}

/// A successful response carrying `data`.
pub fn ok(data: impl Into<Vec<u8>>) -> Response {
    Response {
        ccode: 0,
        data: data.into(),
        ..Default::default()
    }
}

/// A response carrying a completion code and no data.
pub fn ccode(code: u8) -> Response {
    Response {
        ccode: code,
        data: Vec::new(),
        ..Default::default()
    }
}
