//! Integration test: browser surface commands dispatch through RuntimeApplicationService.
//!
//! Verifies the single-owner invariant: browser surface commands go through
//! `RuntimeApplicationService` which delegates to `BrowserSurfacePort`, not
//! directly to infrastructure.

use std::sync::Mutex;

use tabby_contracts::{BrowserSurfaceBoundsDto, BrowserSurfaceCommandDto};

/// Recorded call captured by the mock `BrowserSurfacePort`.
#[derive(Debug, Clone, PartialEq)]
enum BrowserCall {
    Ensure {
        pane_id: String,
        url: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    SetBounds {
        pane_id: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    SetVisible {
        pane_id: String,
        visible: bool,
    },
    Close {
        pane_id: String,
    },
}

// ---------------------------------------------------------------------------
// Mock implementations of the three ports required by RuntimeApplicationService
// ---------------------------------------------------------------------------

mod mocks {
    use std::sync::{Arc, Mutex};

    use tabby_runtime::PaneRuntime;
    use tabby_settings::UserPreferences;

    // Re-use the parent module's BrowserCall
    use super::BrowserCall;

    // Import port traits and error type from the app crate.
    // `tabby` is the lib name of the `src-tauri` crate (declared in Cargo.toml).
    use tabby_app_lib::application::ports::{
        BrowserSurfacePort, ProjectionPublisherPort, TerminalProcessPort,
    };
    use tabby_app_lib::application::runtime_observation_receiver::RuntimeObservationReceiver;
    use tabby_app_lib::shell::error::ShellError;

    // -- MockBrowserSurfacePort ------------------------------------------------

    #[derive(Debug)]
    pub struct MockBrowserSurfacePort {
        pub calls: Arc<Mutex<Vec<BrowserCall>>>,
    }

    impl MockBrowserSurfacePort {
        pub fn new(calls: Arc<Mutex<Vec<BrowserCall>>>) -> Self {
            Self { calls }
        }
    }

    impl BrowserSurfacePort for MockBrowserSurfacePort {
        fn ensure_surface(
            &self,
            pane_id: &str,
            url: &str,
            x: f64,
            y: f64,
            width: f64,
            height: f64,
        ) -> Result<(), ShellError> {
            if let Ok(mut c) = self.calls.lock() {
                c.push(BrowserCall::Ensure {
                    pane_id: pane_id.to_string(),
                    url: url.to_string(),
                    x,
                    y,
                    width,
                    height,
                });
            }
            Ok(())
        }

        fn set_bounds(
            &self,
            pane_id: &str,
            x: f64,
            y: f64,
            width: f64,
            height: f64,
        ) -> Result<(), ShellError> {
            if let Ok(mut c) = self.calls.lock() {
                c.push(BrowserCall::SetBounds {
                    pane_id: pane_id.to_string(),
                    x,
                    y,
                    width,
                    height,
                });
            }
            Ok(())
        }

        fn set_visible(&self, pane_id: &str, visible: bool) -> Result<(), ShellError> {
            if let Ok(mut c) = self.calls.lock() {
                c.push(BrowserCall::SetVisible {
                    pane_id: pane_id.to_string(),
                    visible,
                });
            }
            Ok(())
        }

        fn close_surface(&self, pane_id: &str) -> Result<(), ShellError> {
            if let Ok(mut c) = self.calls.lock() {
                c.push(BrowserCall::Close {
                    pane_id: pane_id.to_string(),
                });
            }
            Ok(())
        }

        fn navigate(&self, _pane_id: &str, _url: &str) -> Result<(), ShellError> {
            Ok(())
        }
    }

    // -- Stub TerminalProcessPort (unused in browser tests) --------------------

    #[derive(Debug)]
    pub struct StubTerminalProcessPort;

    impl TerminalProcessPort for StubTerminalProcessPort {
        fn spawn(
            &self,
            _pane_id: &str,
            _working_directory: &str,
            _startup_command: Option<&str>,
            _observation_receiver: Arc<dyn RuntimeObservationReceiver>,
        ) -> Result<String, ShellError> {
            Ok(String::from("stub-session"))
        }

        fn kill(&self, _runtime_session_id: &str) -> Result<(), ShellError> {
            Ok(())
        }

        fn resize(
            &self,
            _runtime_session_id: &str,
            _cols: u16,
            _rows: u16,
        ) -> Result<(), ShellError> {
            Ok(())
        }

        fn write_input(&self, _runtime_session_id: &str, _data: &str) -> Result<(), ShellError> {
            Ok(())
        }
    }

    // -- Stub ProjectionPublisherPort (unused in browser tests) ----------------

    #[derive(Debug)]
    pub struct StubProjectionPublisher;

    impl ProjectionPublisherPort for StubProjectionPublisher {
        fn publish_workspace_projection(&self, _workspace: &tabby_workspace::WorkspaceSession) {}
        fn publish_settings_projection(&self, _preferences: &UserPreferences) {}
        fn publish_runtime_status(&self, _runtime: &PaneRuntime) {}
    }
}

// ---------------------------------------------------------------------------
// Helper to build a RuntimeApplicationService with mock browser port
// ---------------------------------------------------------------------------

fn build_service_with_mock() -> (
    tabby_app_lib::application::RuntimeApplicationService,
    std::sync::Arc<Mutex<Vec<BrowserCall>>>,
) {
    let calls = std::sync::Arc::new(Mutex::new(Vec::<BrowserCall>::new()));

    let browser_port = mocks::MockBrowserSurfacePort::new(std::sync::Arc::clone(&calls));
    let terminal_port = mocks::StubTerminalProcessPort;
    let emitter = mocks::StubProjectionPublisher;

    let service = tabby_app_lib::application::RuntimeApplicationService::new(
        Box::new(terminal_port),
        Box::new(browser_port),
        Box::new(emitter),
    );

    (service, calls)
}

fn make_bounds(x: f64, y: f64, w: f64, h: f64) -> BrowserSurfaceBoundsDto {
    BrowserSurfaceBoundsDto {
        x,
        y,
        width: w,
        height: h,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn ensure_command_dispatches_through_runtime_service() {
    let (service, calls) = build_service_with_mock();

    let cmd = BrowserSurfaceCommandDto::Ensure {
        pane_id: "pane-1".into(),
        url: "https://example.com".into(),
        bounds: make_bounds(10.0, 20.0, 800.0, 600.0),
    };

    service
        .dispatch_browser_surface_command(cmd)
        .expect("dispatch should succeed");

    let recorded = calls.lock().expect("lock");
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        recorded[0],
        BrowserCall::Ensure {
            pane_id: "pane-1".into(),
            url: "https://example.com".into(),
            x: 10.0,
            y: 20.0,
            width: 800.0,
            height: 600.0,
        }
    );
}

#[test]
fn set_bounds_command_dispatches_through_runtime_service() {
    let (service, calls) = build_service_with_mock();

    let cmd = BrowserSurfaceCommandDto::SetBounds {
        pane_id: "pane-2".into(),
        bounds: make_bounds(0.0, 0.0, 1024.0, 768.0),
    };

    service
        .dispatch_browser_surface_command(cmd)
        .expect("dispatch should succeed");

    let recorded = calls.lock().expect("lock");
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        recorded[0],
        BrowserCall::SetBounds {
            pane_id: "pane-2".into(),
            x: 0.0,
            y: 0.0,
            width: 1024.0,
            height: 768.0,
        }
    );
}

#[test]
fn set_visible_command_dispatches_through_runtime_service() {
    let (service, calls) = build_service_with_mock();

    let cmd = BrowserSurfaceCommandDto::SetVisible {
        pane_id: "pane-3".into(),
        visible: false,
    };

    service
        .dispatch_browser_surface_command(cmd)
        .expect("dispatch should succeed");

    let recorded = calls.lock().expect("lock");
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        recorded[0],
        BrowserCall::SetVisible {
            pane_id: "pane-3".into(),
            visible: false,
        }
    );
}

#[test]
fn close_command_dispatches_through_runtime_service() {
    let (service, calls) = build_service_with_mock();

    let cmd = BrowserSurfaceCommandDto::Close {
        pane_id: "pane-4".into(),
    };

    service
        .dispatch_browser_surface_command(cmd)
        .expect("dispatch should succeed");

    let recorded = calls.lock().expect("lock");
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        recorded[0],
        BrowserCall::Close {
            pane_id: "pane-4".into(),
        }
    );
}

#[test]
fn multiple_commands_dispatch_sequentially() {
    let (service, calls) = build_service_with_mock();

    service
        .dispatch_browser_surface_command(BrowserSurfaceCommandDto::Ensure {
            pane_id: "p1".into(),
            url: "https://a.com".into(),
            bounds: make_bounds(0.0, 0.0, 100.0, 100.0),
        })
        .expect("ensure should succeed");

    service
        .dispatch_browser_surface_command(BrowserSurfaceCommandDto::SetVisible {
            pane_id: "p1".into(),
            visible: true,
        })
        .expect("set_visible should succeed");

    service
        .dispatch_browser_surface_command(BrowserSurfaceCommandDto::Close {
            pane_id: "p1".into(),
        })
        .expect("close should succeed");

    let recorded = calls.lock().expect("lock");
    assert_eq!(recorded.len(), 3);

    assert!(matches!(recorded[0], BrowserCall::Ensure { .. }));
    assert!(matches!(recorded[1], BrowserCall::SetVisible { .. }));
    assert!(matches!(recorded[2], BrowserCall::Close { .. }));
}
