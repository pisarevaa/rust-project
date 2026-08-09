/// Имитатор умной розетки: неблокирующий TCP-сервер.

use crate::error::DeviceError;
use crate::protocol::{Command, Response, REQUEST_LEN};
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

/// Пауза между проходами цикла, чтобы опрос не занимал ядро целиком.
const IDLE: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct SocketSimulator {
    listener: TcpListener,
    address: SocketAddr,
    clients: Vec<TcpStream>,
    is_on: bool,
    power: f64,
}

impl SocketSimulator {
    pub fn bind(address: impl ToSocketAddrs, power: f64) -> Result<Self, DeviceError> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        Ok(Self {
            listener,
            address,
            clients: Vec::new(),
            is_on: false,
            power,
        })
    }

    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.address
    }

    #[must_use]
    pub fn is_on(&self) -> bool {
        self.is_on
    }

    #[must_use]
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.poll();
            thread::sleep(IDLE);
        }
    }

    pub fn poll(&mut self) {
        self.accept_clients();
        self.serve_clients();
    }

    fn accept_clients(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if stream.set_nonblocking(true).is_ok() {
                        self.clients.push(stream);
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }

    fn serve_clients(&mut self) {
        let clients = std::mem::take(&mut self.clients);
        let mut alive = Vec::with_capacity(clients.len());
        for client in clients {
            if let Some(client) = self.serve_client(client) {
                alive.push(client);
            }
        }
        self.clients = alive;
    }

    fn serve_client(&mut self, mut client: TcpStream) -> Option<TcpStream> {
        let mut request = [0_u8; REQUEST_LEN];
        match client.read(&mut request) {
            Ok(0) => None,
            Ok(_) => {
                let command = Command::from_byte(request[0]).ok()?;
                let response = self.apply(command);
                client.write_all(&response.to_bytes()).ok()?;
                client.flush().ok()?;
                Some(client)
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => Some(client),
            Err(_) => None,
        }
    }

    fn apply(&mut self, command: Command) -> Response {
        match command {
            Command::TurnOn => self.is_on = true,
            Command::TurnOff => self.is_on = false,
            Command::Status => {}
        }
        Response {
            is_on: self.is_on,
            power: if self.is_on { self.power } else { 0.0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::SmartSocket;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    struct RunningSimulator {
        address: SocketAddr,
        running: Arc<AtomicBool>,
        worker: Option<thread::JoinHandle<()>>,
    }

    impl RunningSimulator {
        fn start(power: f64) -> Self {
            let mut simulator =
                SocketSimulator::bind("127.0.0.1:0", power).expect("адрес свободен");
            let address = simulator.local_addr();
            let running = Arc::new(AtomicBool::new(true));

            let worker = {
                let running = Arc::clone(&running);
                thread::spawn(move || {
                    while running.load(Ordering::Relaxed) {
                        simulator.poll();
                        thread::sleep(Duration::from_millis(1));
                    }
                })
            };

            Self {
                address,
                running,
                worker: Some(worker),
            }
        }
    }

    impl Drop for RunningSimulator {
        fn drop(&mut self) {
            self.running.store(false, Ordering::Relaxed);
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
    }

    #[test]
    fn bind_reports_a_real_address() {
        let simulator = SocketSimulator::bind("127.0.0.1:0", 100.0).expect("адрес свободен");
        assert_ne!(simulator.local_addr().port(), 0);
        assert!(!simulator.is_on());
        assert_eq!(simulator.client_count(), 0);
    }

    #[test]
    fn a_client_can_switch_the_socket_on_and_off() {
        let simulator = RunningSimulator::start(150.0);
        let socket = SmartSocket::connect(simulator.address).expect("имитатор слушает");

        assert!(!socket.is_on().expect("имитатор отвечает"));

        socket.turn_on().expect("имитатор отвечает");
        assert!(socket.is_on().expect("имитатор отвечает"));
        assert!((socket.current_power().expect("имитатор отвечает") - 150.0).abs() < f64::EPSILON);

        socket.turn_off().expect("имитатор отвечает");
        assert!(!socket.is_on().expect("имитатор отвечает"));
        assert!((socket.current_power().expect("имитатор отвечает")).abs() < f64::EPSILON);
    }

    #[test]
    fn state_is_shared_between_several_clients() {
        let simulator = RunningSimulator::start(60.0);
        let first = SmartSocket::connect(simulator.address).expect("имитатор слушает");
        let second = SmartSocket::connect(simulator.address).expect("имитатор слушает");

        first.turn_on().expect("имитатор отвечает");
        assert!(
            second.is_on().expect("имитатор отвечает"),
            "второй клиент должен видеть включение, сделанное первым"
        );

        second.turn_off().expect("имитатор отвечает");
        assert!(!first.is_on().expect("имитатор отвечает"));
    }

    #[test]
    fn many_clients_are_served_concurrently() {
        let simulator = RunningSimulator::start(75.0);
        let sockets: Vec<_> = (0..8)
            .map(|_| SmartSocket::connect(simulator.address).expect("имитатор слушает"))
            .collect();

        for socket in &sockets {
            socket.status().expect("имитатор отвечает каждому клиенту");
        }

        sockets[0].turn_on().expect("имитатор отвечает");
        for socket in &sockets {
            assert!(socket.is_on().expect("имитатор отвечает"));
        }
    }

    #[test]
    fn a_disconnected_client_is_forgotten() {
        let mut simulator = SocketSimulator::bind("127.0.0.1:0", 100.0).expect("адрес свободен");
        let address = simulator.local_addr();

        let client = TcpStream::connect(address).expect("имитатор слушает");
        wait_until(&mut simulator, |s| s.client_count() == 1);

        drop(client);
        wait_until(&mut simulator, |s| s.client_count() == 0);
    }

    #[test]
    fn an_unknown_command_closes_the_connection() {
        let mut simulator = SocketSimulator::bind("127.0.0.1:0", 100.0).expect("адрес свободен");
        let address = simulator.local_addr();

        let mut client = TcpStream::connect(address).expect("имитатор слушает");
        wait_until(&mut simulator, |s| s.client_count() == 1);

        client.write_all(&[200]).expect("байт отправлен");
        wait_until(&mut simulator, |s| s.client_count() == 0);
    }

    fn wait_until(simulator: &mut SocketSimulator, condition: impl Fn(&SocketSimulator) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(1);
        while !condition(simulator) {
            assert!(Instant::now() < deadline, "условие так и не выполнилось");
            simulator.poll();
            thread::sleep(Duration::from_millis(5));
        }
    }
}
