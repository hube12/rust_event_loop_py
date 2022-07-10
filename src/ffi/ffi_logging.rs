use anyhow::anyhow;
/// Global FFI logging implementation with little to no name clash and fully
/// thread local Use tokio::tracing under the hood to provide dimensional
/// logging through all the crates
use std::borrow::Borrow;
use std::panic::catch_unwind;

use tracing::{
    span::{Attributes, Record},
    Event,
    Id,
    Metadata,
};

use tracing_subscriber::fmt::{
    time::{SystemTime, Uptime, UtcTime},
    MakeWriter,
};

use crate::ffi::{
    create_heap_pointer,
    ffi_bytes_array,
    ffi_string,
    FFIArray,
    FFIError,
    FFINull,
    FFISafeMove,
};

thread_local! {
   pub static LOG_BUFFER: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::new());
}
#[repr(u8)]
pub enum FFILogLevel {
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    FFILogLevelTrace = 0,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    FFILogLevelDebug = 1,
    /// The "info" level.
    ///
    /// Designates useful information.
    FFILogLevelInfo = 2,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    FFILogLevelWarn = 3,
    /// The "error" level.
    ///
    /// Designates very serious errors.
    FFILogLevelError = 4,
}

#[repr(u8)]
pub enum FFILogTimeFormat {
    /// No format of message timestamp
    FFILogTimeFormatNone = 0,
    /// Use RFC 3339 to format
    FFILogTimeFormatRfc3339 = 1,
    /// Human readable format (eg 'Thu 3 14:23:59.001')
    FFILogTimeFormatSystem = 2,
    /// Uptime (since startup)
    FFILogTimeFormatUpTime = 3,
}

#[repr(u8)]
pub enum FFILogFormat {
    /// Text based
    FFILogFormatText = 0,
    /// Json based
    FFILogFormatJson = 1,
    /// Compact
    FFILogFormatCompact = 2,
    /// Pretty format
    FFILogFormatPretty = 3,
}

#[repr(C)]
pub struct FFILoggingConfig {
    /// Logging Level
    pub(crate) level:       FFILogLevel,
    /// Time format
    pub(crate) time_format: FFILogTimeFormat,
    /// Log output format
    pub(crate) log_format:  FFILogFormat,
    /// Show log level
    pub(crate) show_level:  bool,
    /// Show where was the log issued in Rust
    pub(crate) show_trace:  bool,
}

#[repr(C)]
pub struct FFILogger {
    callback: extern "C" fn(FFILogLevel, *mut FFIArray<std::os::raw::c_uchar>),
}

impl FFILogger {
    /// Tested callback assigned directly to the logger
    #[no_mangle]
    pub extern "C" fn create_logger(
        callback: extern "C" fn(FFILogLevel, *mut FFIArray<std::os::raw::c_uchar>),
    ) -> FFIError<Self> {
        let logger = Self { callback };
        match catch_unwind(|| {
            logger.on_message(
                FFILogLevel::FFILogLevelInfo,
                ffi_string("Starting logger : testing callback"),
            );
        }) {
            Ok(_) => FFIError::from_value(logger),
            Err(e) => FFIError::from(anyhow::Error::msg(format!(
                "Panicked when testing callback with error :\n {:?}",
                e
            ))),
        }
    }

    /// Untested callback assigned directly to the logger, will fail later on
    #[no_mangle]
    pub extern "C" fn create_unsafe_logger(
        callback: extern "C" fn(FFILogLevel, *mut FFIArray<std::os::raw::c_uchar>),
    ) -> Self {
        FFILogger { callback }
    }

    fn on_message(&self, level: FFILogLevel, message: FFIArray<std::os::raw::c_uchar>) {
        (self.callback)(level, create_heap_pointer(message));
    }
}

impl FFISafeMove for FFILogger {
}

#[no_mangle]
pub extern "C" fn create_logger_config(
    level: FFILogLevel,
    time_format: FFILogTimeFormat,
    log_format: FFILogFormat,
    show_level: bool,
    show_trace: bool,
) -> FFILoggingConfig {
    FFILoggingConfig {
        level,
        time_format,
        log_format,
        show_level,
        show_trace,
    }
}

#[no_mangle]
pub extern "C" fn configure_logging(
    config: FFILoggingConfig,
    mut handler: *mut FFILogger,
) -> FFIError<FFINull> {
    let handler = match FFILogger::safe_move(&mut handler) {
        Ok(x) => x,
        Err(error) => return error,
    };
    match tracing::subscriber::set_global_default(adapter(config, handler)) {
        Ok(_) => FFIError::from_value(FFINull),
        Err(e) => FFIError::from(anyhow!(e)),
    }
}

struct ThreadLocalBufferWriter;

struct ThreadLocalMakeWriter;

impl<'a> MakeWriter<'a> for ThreadLocalMakeWriter {
    type Writer = ThreadLocalBufferWriter;

    fn make_writer(&self) -> Self::Writer {
        ThreadLocalBufferWriter
    }
}

impl std::io::Write for ThreadLocalBufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        LOG_BUFFER.with(|vec| vec.borrow_mut().extend_from_slice(buf));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn adapter(
    config: FFILoggingConfig,
    handler: FFILogger,
) -> impl tracing::Subscriber + Send + Sync + 'static {
    Adapter {
        handler,
        inner: config.build(),
    }
}

impl FFILoggingConfig {
    fn build(&self) -> Box<dyn tracing::Subscriber + Send + Sync> {
        let level: tracing::Level = self.level.borrow().into();

        let builder = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(level)
            .with_level(self.show_level)
            .with_target(self.show_trace)
            .with_writer(ThreadLocalMakeWriter);
        match self.time_format {
            FFILogTimeFormat::FFILogTimeFormatNone => {
                let builder = builder.without_time();
                match self.log_format {
                    FFILogFormat::FFILogFormatText => Box::new(builder.finish()),
                    FFILogFormat::FFILogFormatJson => Box::new(builder.json().finish()),
                    FFILogFormat::FFILogFormatCompact => Box::new(builder.compact().finish()),
                    FFILogFormat::FFILogFormatPretty => Box::new(builder.pretty().finish()),
                }
            },
            FFILogTimeFormat::FFILogTimeFormatRfc3339 => {
                let builder = builder.with_timer(UtcTime::rfc_3339());
                match self.log_format {
                    FFILogFormat::FFILogFormatText => Box::new(builder.finish()),
                    FFILogFormat::FFILogFormatJson => Box::new(builder.json().finish()),
                    FFILogFormat::FFILogFormatCompact => Box::new(builder.compact().finish()),
                    FFILogFormat::FFILogFormatPretty => Box::new(builder.pretty().finish()),
                }
            },
            FFILogTimeFormat::FFILogTimeFormatSystem => {
                let builder = builder.with_timer(SystemTime::default());
                match self.log_format {
                    FFILogFormat::FFILogFormatText => Box::new(builder.finish()),
                    FFILogFormat::FFILogFormatJson => Box::new(builder.json().finish()),
                    FFILogFormat::FFILogFormatCompact => Box::new(builder.compact().finish()),
                    FFILogFormat::FFILogFormatPretty => Box::new(builder.pretty().finish()),
                }
            },
            FFILogTimeFormat::FFILogTimeFormatUpTime => {
                let builder = builder.with_timer(Uptime::default());
                match self.log_format {
                    FFILogFormat::FFILogFormatText => Box::new(builder.finish()),
                    FFILogFormat::FFILogFormatJson => Box::new(builder.json().finish()),
                    FFILogFormat::FFILogFormatCompact => Box::new(builder.compact().finish()),
                    FFILogFormat::FFILogFormatPretty => Box::new(builder.pretty().finish()),
                }
            },
        }
    }
}

struct Adapter {
    handler: FFILogger,
    inner:   Box<dyn tracing::Subscriber + Send + Sync + 'static>,
}

impl tracing::Subscriber for Adapter {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.inner.enabled(metadata)
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        self.inner.new_span(span)
    }

    fn record(&self, span: &Id, values: &Record<'_>) {
        self.inner.record(span, values)
    }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        self.inner.record_follows_from(span, follows)
    }

    fn event(&self, event: &Event<'_>) {
        self.inner.event(event);
        let message = LOG_BUFFER.with(|vec| ffi_bytes_array(vec.borrow().as_slice()));
        self.handler.on_message((*event.metadata().level()).into(), message);
        LOG_BUFFER.with(|vec| vec.borrow_mut().clear())
    }

    fn enter(&self, span: &Id) {
        self.inner.enter(span)
    }

    fn exit(&self, span: &Id) {
        self.inner.exit(span)
    }

    fn clone_span(&self, span: &Id) -> Id {
        self.inner.clone_span(span)
    }

    fn try_close(&self, span: Id) -> bool {
        self.inner.try_close(span)
    }

    fn current_span(&self) -> tracing_core::span::Current {
        self.inner.current_span()
    }
}

impl From<tracing::Level> for FFILogLevel {
    fn from(level: tracing::Level) -> Self {
        level.borrow().into()
    }
}

impl From<&tracing::Level> for FFILogLevel {
    fn from(level: &tracing::Level) -> Self {
        match *level {
            tracing::Level::DEBUG => FFILogLevel::FFILogLevelDebug,
            tracing::Level::TRACE => FFILogLevel::FFILogLevelTrace,
            tracing::Level::INFO => FFILogLevel::FFILogLevelInfo,
            tracing::Level::WARN => FFILogLevel::FFILogLevelWarn,
            tracing::Level::ERROR => FFILogLevel::FFILogLevelError,
        }
    }
}

impl From<FFILogLevel> for tracing::Level {
    fn from(level: FFILogLevel) -> Self {
        level.borrow().into()
    }
}

impl From<&FFILogLevel> for tracing::Level {
    fn from(level: &FFILogLevel) -> Self {
        match *level {
            FFILogLevel::FFILogLevelDebug => tracing::Level::DEBUG,
            FFILogLevel::FFILogLevelTrace => tracing::Level::TRACE,
            FFILogLevel::FFILogLevelInfo => tracing::Level::INFO,
            FFILogLevel::FFILogLevelWarn => tracing::Level::WARN,
            FFILogLevel::FFILogLevelError => tracing::Level::ERROR,
        }
    }
}
