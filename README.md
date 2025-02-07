# KBCom, Unified Communication Backend

## Overview

This backend system, written in Rust, is designed to serve as a unified communication gateway for various messaging platforms. It will eventually supports multiple chat protocols (such as Matrix, Slack, Rocket Chat, etc.) as well as traditional email (mail) functionality. The backend exposes a common interface for operations like authentication, channel (or mailbox) management, message retrieval, and message posting. Communication between the backend and frontends is handled entirely via JSON events over IPC (for example, using Unix domain sockets), allowing for a decoupled and extensible architecture.

## Objectives

- **Multi-Protocol Support:**
  Integrate various chat protocols and email services into a single backend, allowing users to interact with different communication platforms through a unified interface.

- **Unified API:**
  Provide a common API for:
  - User authentication
  - Listing channels or mailboxes
  - Retrieving messages and emails
  - Posting messages and sending emails
  - Managing channels (e.g., joining, leaving) and mail operations

- **Event-Driven JSON Communication:**
  - **Outbound Events:** The backend streams JSON-formatted events (e.g., new messages, channel updates, mail notifications) to frontends.
  - **Inbound Commands:** Frontends send JSON commands (e.g., `post_message`, `send_email`, `leave_channel`) to the backend, allowing real-time control and updates.

- **Persistent, Long-Lived Connections:**
  The backend is designed to run continuously, maintaining persistent connections to various messaging services. This avoids the overhead of re‑authentication or re‑initialization when switching contexts.

- **Modular and Extensible:**
  With a modular Rust architecture, the backend can be easily extended to support additional protocols and features as needed.

## Done

- Started a dummy backend implementation for testing things.

## Todo

- **Unified Protocol Handling:**
  Support for multiple chat protocols and email services within a single backend.

- **Real-Time Event Streaming:**
  Continuous streaming of JSON events that reflect the state of channels, mailboxes, and messages.

- **Two-Way IPC Communication:**
  Frontends and other clients communicate with the backend via a JSON-based IPC protocol (e.g., over Unix domain sockets), sending commands and receiving events.

- **Extensible Architecture:**
  A modular design that allows easy integration of new protocols and features without disrupting existing functionality.

## Running Instructions

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)
- Cargo (included with Rust)

### Building

Clone the repository and build the project in release mode:

```bash
git clone <repository-url>
cd <repository-directory>
cargo build --release

### Running

```bash
./target/release/kbcom --live
```
