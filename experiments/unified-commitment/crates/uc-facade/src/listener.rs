use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use uc_protocol::{Registry, handle_connection};

/// The sole bind operation is IPv4 loopback with an OS-assigned port.
pub struct LoopbackListener(TcpListener);
impl LoopbackListener {
    pub fn bind() -> io::Result<Self> {
        TcpListener::bind("127.0.0.1:0").map(Self)
    }
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }
}

struct BufferedStream {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}
impl Read for BufferedStream {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.reader.read(bytes)
    }
}
impl Write for BufferedStream {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writer.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// One thread per accepted socket, sharing the existing Registry. The stop flag
/// stops acceptance; callers close their clients before joining this function.
/// All accepted connection threads are joined, including after an accept failure.
/// A malformed/disconnected client is isolated to its own connection.
pub fn serve(listener: LoopbackListener, registry: Registry, stop: &AtomicBool) -> io::Result<()> {
    listener.0.set_nonblocking(true)?;
    let mut workers = Vec::new();
    let result = (|| {
        while !stop.load(Ordering::Acquire) {
            match listener.0.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(false)?;
                    let reader = BufReader::new(stream.try_clone()?);
                    let registry = registry.clone();
                    workers.push(
                        thread::Builder::new()
                            .name("uc-facade-connection".into())
                            .spawn(move || {
                                let mut stream = BufferedStream {
                                    reader,
                                    writer: BufWriter::new(stream),
                                };
                                handle_connection(&mut stream, &registry)
                            })?,
                    );
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5))
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    })();
    let mut panicked = false;
    for worker in workers {
        panicked |= worker.join().is_err();
    }
    result?;
    if panicked {
        return Err(io::Error::other("connection thread panicked"));
    }
    Ok(())
}
