//! COSMIC KVM UI Application
//!
//! Native COSMIC application for managing KVM sharing connections

use cosmic::app::{Core, Settings, Task};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, container, text};
use cosmic::{executor, Application, Element};
use std::process;

fn main() -> cosmic::iced::Result {
    let settings = Settings::default()
        .size_limits(cosmic::iced::Limits::NONE.min_width(400.0).min_height(300.0));

    cosmic::app::run::<CosmicKvmApp>(settings, ())
}

#[derive(Debug, Clone)]
enum ConnectionState {
    Disconnected,
    #[allow(dead_code)]
    Connecting,
    Connected { server: String },
    ServerRunning,
}

struct CosmicKvmApp {
    core: Core,
    connection_state: ConnectionState,
    server_address: String,
    daemon_process: Option<process::Child>,
}

#[derive(Debug, Clone)]
enum Message {
    StartServer,
    StopServer,
    Connect,
    Disconnect,
    ServerAddressChanged(String),
}

impl Application for CosmicKvmApp {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.system76.CosmicKvm";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = CosmicKvmApp {
            core,
            connection_state: ConnectionState::Disconnected,
            server_address: String::from(""),
            daemon_process: None,
        };

        (app, Task::none())
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        vec![text("COSMIC KVM").size(14).into()]
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let title = container(
            text("COSMIC KVM Manager").size(32)
        )
        .width(Length::Fill)
        .align_x(Alignment::Center);

        let subtitle = container(
            text("Share keyboard and mouse across computers").size(14)
        )
        .width(Length::Fill)
        .align_x(Alignment::Center);

        // Status section
        let status_text = match &self.connection_state {
            ConnectionState::Disconnected => "Not connected",
            ConnectionState::Connecting => "Connecting...",
            ConnectionState::Connected { server } => {
                return self.view_connected(server);
            }
            ConnectionState::ServerRunning => {
                return self.view_server_running();
            }
        };

        let status = container(
            container(text(status_text).size(16))
                .width(Length::Fill)
                .align_x(Alignment::Center)
        )
        .padding(20)
        .width(Length::Fill);

        // Server mode section
        let server_section = container(
            cosmic::widget::column()
                .push(text("Server Mode").size(18))
                .push(text("Share this computer's keyboard and mouse").size(12))
                .push(
                    button::standard("Start Server")
                        .on_press(Message::StartServer)
                )
                .spacing(cosmic::theme::spacing().space_xs)
                .padding(15)
        )
        .padding(10)
        .width(Length::Fill);

        // Client mode section
        let client_input = cosmic::widget::text_input("192.168.1.100:24900", &self.server_address)
            .on_input(Message::ServerAddressChanged);

        let connect_button = button::standard("Connect to Server")
            .on_press(Message::Connect);

        let client_section = container(
            cosmic::widget::column()
                .push(text("Client Mode").size(18))
                .push(text("Control this computer from another").size(12))
                .push(text("Server Address:").size(12))
                .push(client_input)
                .push(connect_button)
                .spacing(cosmic::theme::spacing().space_xs)
                .padding(15)
        )
        .padding(10)
        .width(Length::Fill);

        // Help text
        let help_text = container(
            container(
                text("Note: Server mode requires permission to read /dev/input devices.\nClient mode requires permission to access /dev/uinput.")
                    .size(10)
            )
            .width(Length::Fill)
            .align_x(Alignment::Center)
        )
        .padding(10)
        .width(Length::Fill);

        let content = cosmic::widget::column()
            .push(title)
            .push(subtitle)
            .push(status)
            .push(server_section)
            .push(client_section)
            .push(help_text)
            .padding(20)
            .spacing(cosmic::theme::spacing().space_m)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::StartServer => {
                // Try to find cosmic-kvm-daemon in PATH or cargo target
                let daemon_cmd = if let Ok(output) = process::Command::new("which")
                    .arg("cosmic-kvm-daemon")
                    .output()
                {
                    if output.status.success() {
                        "cosmic-kvm-daemon".to_string()
                    } else {
                        // Try cargo target directory
                        "target/debug/cosmic-kvm-daemon".to_string()
                    }
                } else {
                    "cosmic-kvm-daemon".to_string()
                };

                let args = vec!["--server".to_string(), "--debug".to_string()];
                match process::Command::new(&daemon_cmd).args(&args).spawn() {
                    Ok(child) => {
                        self.daemon_process = Some(child);
                        self.connection_state = ConnectionState::ServerRunning;
                    }
                    Err(e) => {
                        eprintln!("Failed to launch daemon: {}", e);
                        eprintln!("Tried to run: {} {:?}", daemon_cmd, args);
                        self.connection_state = ConnectionState::Disconnected;
                    }
                }
                Task::none()
            }
            Message::StopServer => {
                if let Some(mut child) = self.daemon_process.take() {
                    let _ = child.kill();
                }
                self.connection_state = ConnectionState::Disconnected;
                Task::none()
            }
            Message::Connect => {
                if !self.server_address.is_empty() {
                    // Try to find cosmic-kvm-daemon in PATH or cargo target
                    let daemon_cmd = if let Ok(output) = process::Command::new("which")
                        .arg("cosmic-kvm-daemon")
                        .output()
                    {
                        if output.status.success() {
                            "cosmic-kvm-daemon".to_string()
                        } else {
                            // Try cargo target directory
                            "target/debug/cosmic-kvm-daemon".to_string()
                        }
                    } else {
                        "cosmic-kvm-daemon".to_string()
                    };

                    let server = self.server_address.clone();
                    let args = vec![
                        "--client".to_string(),
                        "--connect".to_string(),
                        server.clone(),
                        "--debug".to_string(),
                    ];

                    match process::Command::new(&daemon_cmd).args(&args).spawn() {
                        Ok(child) => {
                            self.daemon_process = Some(child);
                            self.connection_state = ConnectionState::Connected { server };
                        }
                        Err(e) => {
                            eprintln!("Failed to launch daemon: {}", e);
                            eprintln!("Tried to run: {} {:?}", daemon_cmd, args);
                            self.connection_state = ConnectionState::Disconnected;
                        }
                    }
                }
                Task::none()
            }
            Message::Disconnect => {
                if let Some(mut child) = self.daemon_process.take() {
                    let _ = child.kill();
                }
                self.connection_state = ConnectionState::Disconnected;
                Task::none()
            }
            Message::ServerAddressChanged(value) => {
                self.server_address = value;
                Task::none()
            }
        }
    }
}

impl CosmicKvmApp {
    fn view_server_running(&self) -> Element<'_, Message> {
        let title = container(
            text("COSMIC KVM Manager").size(32)
        )
        .width(Length::Fill)
        .align_x(Alignment::Center);

        let status = container(
            cosmic::widget::column()
                .push(container(text("Server Running").size(24))
                    .width(Length::Fill)
                    .align_x(Alignment::Center))
                .push(container(text("Your keyboard and mouse are being shared").size(14))
                    .width(Length::Fill)
                    .align_x(Alignment::Center))
                .push(container(text("Clients can connect to this computer on port 24900").size(12))
                    .width(Length::Fill)
                    .align_x(Alignment::Center))
                .padding(30)
                .spacing(cosmic::theme::spacing().space_s)
                .align_x(Alignment::Center)
        )
        .padding(20)
        .width(Length::Fill);

        let stop_button = button::destructive("Stop Server")
            .on_press(Message::StopServer);

        let content = cosmic::widget::column()
            .push(title)
            .push(status)
            .push(stop_button)
            .padding(20)
            .spacing(cosmic::theme::spacing().space_m)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }

    fn view_connected(&self, server: &str) -> Element<'_, Message> {
        let title = container(
            text("COSMIC KVM Manager").size(32)
        )
        .width(Length::Fill)
        .align_x(Alignment::Center);

        let status = container(
            cosmic::widget::column()
                .push(container(text("Connected").size(24))
                    .width(Length::Fill)
                    .align_x(Alignment::Center))
                .push(container(text(format!("Receiving input from: {}", server)).size(14))
                    .width(Length::Fill)
                    .align_x(Alignment::Center))
                .push(container(text("Your keyboard and mouse are being controlled remotely").size(12))
                    .width(Length::Fill)
                    .align_x(Alignment::Center))
                .padding(30)
                .spacing(cosmic::theme::spacing().space_s)
                .align_x(Alignment::Center)
        )
        .padding(20)
        .width(Length::Fill);

        let disconnect_button = button::destructive("Disconnect")
            .on_press(Message::Disconnect);

        let content = cosmic::widget::column()
            .push(title)
            .push(status)
            .push(disconnect_button)
            .padding(20)
            .spacing(cosmic::theme::spacing().space_m)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}
