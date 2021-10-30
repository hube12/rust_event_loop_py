#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) id: u8,
}

pub fn create_heap_pointer<T>(x: T) -> *mut T {
    Box::into_raw(Box::new(x))
}

#[derive(Debug)]
pub struct Channel<T: std::fmt::Debug> {
    pub(crate) rx: tokio::sync::mpsc::Receiver<T>,
}

impl<T: std::fmt::Debug> Channel<T> {
    #[allow(dead_code)]
    pub(crate) async fn run(&mut self) {
        let threshold = std::time::Duration::from_secs(2);
        let mut iter = 0;
        let mut time = std::time::Instant::now();
        loop {
            if let Ok(msg) = self.rx.try_recv() {
               dbg!("received msg {:?} {:?}", &msg, std::time::SystemTime::now());
            }
            if time.elapsed() > threshold {
                if let Ok(mut stream) = std::net::TcpStream::connect("127.0.0.1:8000") {
                    let _ = std::io::Write::write(&mut stream, format!("{} iter from rust event loop 1.0.1\n", iter).as_bytes());
                }
                time = std::time::Instant::now();
                iter += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) tx: tokio::sync::mpsc::Sender<Data>,
    pub(crate) runtime: RuntimeHandle,
}

#[derive(Debug, Clone)]
pub struct Runtime {
    pub(crate) inner: std::sync::Arc<tokio::runtime::Runtime>,
}

impl Runtime {
    fn new(inner: tokio::runtime::Runtime) -> Self {
        Self {
            inner: std::sync::Arc::new(inner),
        }
    }

    pub(crate) fn handle(&self) -> RuntimeHandle {
        RuntimeHandle {
            inner: std::sync::Arc::downgrade(&self.inner),
        }
    }
}

#[derive(Clone,Debug)]
pub(crate) struct RuntimeHandle {
    inner: std::sync::Weak<tokio::runtime::Runtime>,
}

#[no_mangle]
pub extern "C" fn create_runtime() -> *mut Runtime {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    let rt = builder.worker_threads(4).enable_all().build().expect("builder");
    create_heap_pointer(Runtime::new(rt))
}

#[no_mangle]
pub unsafe extern "C" fn destroy_runtime(runtime: *mut Runtime) {
    if !runtime.is_null() {
        Box::from_raw(runtime);
    };
}

#[no_mangle]
pub extern "C" fn create_client(runtime: *mut Runtime) -> *mut Client {
    let runtime = unsafe { runtime.as_ref() }.expect("Runtime");
    let (tx, rx) = tokio::sync::mpsc::channel(120);
    let task = async move {
        Channel::<Data> { rx }
            .run()
            .await
    };
    runtime.inner.spawn(task);
    create_heap_pointer(Client {
        tx,
        runtime: runtime.handle(),
    })
}

#[no_mangle]
pub unsafe extern "C" fn destroy_channel(channel: *mut crate::Client) {
    if !channel.is_null() {
        Box::from_raw(channel);
    };
}