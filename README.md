# 📁 file_watcher

**file_watcher** is a cross-platform, command-line file synchronization tool written in Rust. It enables users to monitor local directories and synchronize changes with popular cloud storage providers such as AWS S3, Google Drive, and Dropbox.

## 🚀 Features

* Real-time monitoring of specified directories for file changes.
* Synchronization with multiple cloud storage providers.
* Configurable conflict resolution strategies.
* Secure storage of authentication credentials.
* Extensible architecture for adding new cloud providers.

## 🧱 Project Structure

The project follows a modular architecture to enhance maintainability and scalability:

```

file_watcher/
├── Cargo.toml
├── src/
│   ├── main.rs            # Entry point for the CLI application
│   ├── lib.rs             # Shared library code
│   ├── commands/          # CLI subcommands
│   │   ├── mod.rs
│   │   ├── sync.rs
│   │   ├── config.rs
│   │   └── auth.rs
│   ├── provider/             # Cloud provider integrations
│   │   ├── mod.rs
│   │   ├── aws_s3.rs
│   │   ├── google_drive.rs
│   │   └── dropbox.rs
│   ├── sync/              # Core synchronization logic
│   │   ├── mod.rs
│   │   ├── watcher.rs
│   │   └── comparer.rs
│   └── utils/             # Utility functions
│       ├── mod.rs
│       ├── logger.rs
│       └── config.rs
├── tests/                 # Integration and unit tests
│   ├── integration_test.rs
│   └── unit_test.rs
├── examples/              # Example usage scenarios
│   └── basic_sync.rs
└── README.md
```



## 🏗️ Architecture Decisions

* **Language Choice**: Rust is chosen for its performance, safety, and strong ecosystem support for building CLI applications.

* **Asynchronous Operations**: The `tokio` crate is utilized to handle asynchronous tasks, ensuring non-blocking I/O operations.

* **CLI Parsing**: The `clap` crate is used for parsing command-line arguments and subcommands, providing a user-friendly interface.

* **File Watching**: The `notify` crate monitors file system events to detect changes in real-time.

* **Cloud Integrations**: Each cloud provider is implemented as a separate module, adhering to a common trait to facilitate extensibility.

* **Configuration Management**: Configurations are handled using the `serde` and `toml` crates, allowing users to define settings in a `config.toml` file.

## 📦 Dependencies

Key dependencies include:([The Rust Programming Language Forum][1])

* [`tokio`](https://crates.io/crates/tokio): Asynchronous runtime for Rust.
* [`clap`](https://crates.io/crates/clap): Command-line argument parser.
* [`notify`](https://crates.io/crates/notify): File system event watcher.
* [`serde`](https://crates.io/crates/serde) and [`toml`](https://crates.io/crates/toml): Serialization and deserialization of configuration files.
* Cloud-specific SDKs or HTTP clients for interacting with cloud storage APIs.

## 🧪 Testing

Testing is an integral part of the development process:

* **Unit Tests**: Located in the `tests/unit_test.rs` file, covering individual modules and functions.

* **Integration Tests**: Located in the `tests/integration_test.rs` file, testing the interaction between different components.

* **Example Scenarios**: The `examples/basic_sync.rs` file demonstrates typical usage patterns.

## 📄 Configuration

Users can configure the tool using a `config.toml` file, specifying:

* Directories to monitor.
* Cloud storage provider credentials.
* Synchronization preferences and conflict resolution strategies.

## 🛠️ Building and Running

To build the application:

```

cargo build --release
```



To run the application:([Medium][2])

```

./target/release/file_watcher [SUBCOMMAND] [OPTIONS]
```



For example, to synchronize a directory:

```

./target/release/file_watcher sync --config config.toml
```



## 📌 Roadmap

* Implement additional cloud storage providers.
* Add support for bidirectional synchronization.
* Enhance conflict resolution strategies.
* Develop a graphical user interface (GUI) for non-CLI users.

## 🤝 Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## 📄 License

This project is licensed under the MIT License.

---

