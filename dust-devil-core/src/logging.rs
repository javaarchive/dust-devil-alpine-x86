use core::fmt;
use std::{io, net::SocketAddr};

use crate::{
    socks5::{AuthMethod, SocksRequest, SocksRequestAddress},
    users::{UserRole, UsersLoadingError},
};

pub struct LogEvent {
    pub timestamp: i64,
    pub data: LogEventType,
}

impl LogEvent {
    pub fn new(timestamp: i64, data: LogEventType) -> Self {
        LogEvent { timestamp, data }
    }
}

pub enum LogEventType {
    NewSocks5Socket(SocketAddr),
    FailedBindSocks5Socket(SocketAddr, io::Error),
    FailedBindAnySocketAborting,
    RemovedSocks5Socket(SocketAddr),
    NewSandstormSocket(SocketAddr),
    FailedBindSandstormSocket(SocketAddr, io::Error),
    RemovedSandstormSocket(SocketAddr),
    LoadingUsersFromFile(String),
    UsersLoadedFromFile(String, Result<u64, UsersLoadingError>),
    StartingUpWithSingleDefaultUser(String),
    SavingUsersToFile(String),
    UsersSavedToFile(String, Result<u64, io::Error>),
    UserRegisteredByArgs(String, UserRole),
    UserReplacedByArgs(String, UserRole),
    NewClientConnectionAccepted(u64, SocketAddr),
    ClientConnectionAcceptFailed(Option<SocketAddr>, io::Error),
    ClientRequestedUnsupportedVersion(u64, u8),
    ClientRequestedUnsupportedCommand(u64, u8),
    ClientRequestedUnsupportedAtyp(u64, u8),
    ClientSelectedAuthMethod(u64, AuthMethod),
    ClientRequestedUnsupportedUserpassVersion(u64, u8),
    ClientAuthenticatedWithUserpass(u64, String, bool),
    ClientSocksRequest(u64, SocksRequest),
    ClientDnsLookup(u64, String),
    ClientAttemptingConnect(u64, SocketAddr),
    ClientConnectionAttemptBindFailed(u64, io::Error),
    ClientConnectionAttemptConnectFailed(u64, io::Error),
    ClientFailedToConnectToDestination(u64),
    ClientConnectedToDestination(u64, SocketAddr),
    ClientBytesSent(u64, u64),
    ClientBytesReceived(u64, u64),
    ClientSourceShutdown(u64),
    ClientDestinationShutdown(u64),
    ClientConnectionFinished(u64, u64, u64, Result<(), io::Error>),
    NewSandstormConnectionAccepted(u64, SocketAddr),
    SandstormConnectionAcceptFailed(Option<SocketAddr>, io::Error),
    SandstormRequestedUnsupportedVersion(u64, u8),
    SandstormAuthenticatedAs(u64, String, bool),
    NewSocksSocketRequestedByManager(u64, SocketAddr),
    RemoveSocksSocketRequestedByManager(u64, SocketAddr),
    NewSandstormSocketRequestedByManager(u64, SocketAddr),
    RemoveSandstormSocketRequestedByManager(u64, SocketAddr),
    UserRegisteredByManager(u64, String, UserRole),
    UserUpdatedByManager(u64, String, UserRole, bool),
    UserDeletedByManager(u64, String, UserRole),
    AuthMethodToggledByManager(u64, AuthMethod, bool),
    BufferSizeChangedByManager(u64, u32),
    SandstormRequestedShutdown(u64),
    SandstormConnectionFinished(u64, Result<(), io::Error>),
    ShutdownSignalReceived,
}

impl fmt::Display for LogEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NewSocks5Socket(socket_address) => write!(f, "Listening for socks5 client connections at {socket_address}"),
            Self::FailedBindSocks5Socket(socket_address, io_error) => write!(f, "Failed to set up socks5 socket at {socket_address}: {io_error}"),
            Self::FailedBindAnySocketAborting => write!(f, "Failed to bind any socks5 socket! Aborting"),
            Self::RemovedSocks5Socket(socket_address) => write!(f, "Will no longer listen for socks5 client connections at {socket_address}"),
            Self::NewSandstormSocket(socket_address) => write!(f, "Listening for Sandstorm connections at {socket_address}"),
            Self::FailedBindSandstormSocket(socket_address, io_error) => write!(f, "Failed to set up Sandstorm socket at {socket_address}: {io_error}"),
            Self::RemovedSandstormSocket(socket_address) => write!(f, "Will no longer listen for Sandstorm connections at {socket_address}"),
            Self::LoadingUsersFromFile(filename) => write!(f, "Loading users from file {filename}"),
            Self::UsersLoadedFromFile(filename, Ok(user_count)) => write!(f, "Loaded {user_count} users from file {filename}"),
            Self::UsersLoadedFromFile(filename, Err(load_users_error)) => write!(f, "Error while loading users from file {filename}: {load_users_error}"),
            Self::StartingUpWithSingleDefaultUser(userpass) => write!(f, "Starting up with single default user {userpass}"),
            Self::SavingUsersToFile(filename) => write!(f, "Saving users to file {filename}"),
            Self::UsersSavedToFile(filename, Ok(amount)) => write!(f, "Successfully saved {amount} users to file {filename}"),
            Self::UsersSavedToFile(filename, Err(io_error)) => write!(f, "Failed to save users to file {filename}: {io_error}"),
            Self::UserRegisteredByArgs(username, role) => write!(f, "Registered new {role} user {username} specified via argument"),
            Self::UserReplacedByArgs(username, role) => write!(f, "Replaced user loaded from file {username} with new {role} user specified via argument"),
            Self::NewClientConnectionAccepted(client_id, socket_address) => write!(f, "New client connection from {socket_address} assigned ID {client_id}"),
            Self::ClientConnectionAcceptFailed(Some(socket_address), io_error) => write!(f, "Failed to accept incoming socks connection from socket {socket_address}: {io_error}"),
            Self::ClientConnectionAcceptFailed(None, io_error) => write!(f, "Failed to accept incoming socks connection from unknown socket: {io_error}"),
            Self::ClientRequestedUnsupportedVersion(client_id, version) => write!(f, "Client {client_id} requested unsupported socks version: {version}"),
            Self::ClientRequestedUnsupportedCommand(client_id, command) => write!(f, "Client {client_id} requested unsupported socks command: {command}"),
            Self::ClientRequestedUnsupportedAtyp(client_id, atyp) => write!(f, "Client {client_id} requested unsupported socks ATYP: {atyp}"),
            Self::ClientSelectedAuthMethod(client_id, AuthMethod::NoAcceptableMethod) => write!(f, "Client {client_id} no acceptable authentication method found"),
            Self::ClientSelectedAuthMethod(client_id, auth_method) => write!(f, "Client {client_id} will use auth method {auth_method}"),
            Self::ClientRequestedUnsupportedUserpassVersion(client_id, version) => write!(f, "Client {client_id} requested unsupported userpass version: {version}"),
            Self::ClientAuthenticatedWithUserpass(client_id, username, true) => write!(f, "Client {client_id} successfully authenticated as {username}"),
            Self::ClientAuthenticatedWithUserpass(client_id, username, false) => write!(f, "Client {client_id} unsuccessfully authenticated as {username}"),
            Self::ClientSocksRequest(client_id, request) => {
                write!(f, "Client {client_id} requested to connect to ")?;
                match &request.destination {
                    SocksRequestAddress::IPv4(ipv4) => write!(f, "IPv4 {ipv4}:{}", request.port),
                    SocksRequestAddress::IPv6(ipv6) => write!(f, "IPv6 [{ipv6}]:{}", request.port),
                    SocksRequestAddress::Domainname(domainname) => write!(f, "domainname {domainname}:{}", request.port),
                }
            },
            Self::ClientDnsLookup(client_id, domainname) => write!(f, "Client {client_id} performing DNS lookup for {domainname}"),
            Self::ClientAttemptingConnect(client_id, socket_address) => write!(f, "Client {client_id} attempting to connect to destination at {socket_address}"),
            Self::ClientConnectionAttemptBindFailed(client_id, io_error) => write!(f, "Client {client_id} failed to bind local socket: {io_error}"),
            Self::ClientConnectionAttemptConnectFailed(client_id, io_error) => write!(f, "Client {client_id} failed to connect to destination: {io_error}"),
            Self::ClientFailedToConnectToDestination(client_id) => write!(f, "Client {client_id} failed to connect to destination, sending error response"),
            Self::ClientConnectedToDestination(client_id, socket_address) => write!(f, "Client {client_id} successfully established connection to destination at {socket_address}"),
            Self::ClientBytesSent(client_id, count) => write!(f, "Client {client_id} sent {count} bytes"),
            Self::ClientBytesReceived(client_id, count) => write!(f, "Client {client_id} received {count} bytes"),
            Self::ClientSourceShutdown(client_id) => write!(f, "Client {client_id} source socket shutdown"),
            Self::ClientDestinationShutdown(client_id) => write!(f, "Client {client_id} destination socket shutdown"),
            Self::ClientConnectionFinished(client_id, total_bytes_sent, total_bytes_received,Ok(())) => write!(f, "Client {client_id} finished after {total_bytes_sent} bytes sent and {total_bytes_received} bytes received"),
            Self::ClientConnectionFinished(client_id, total_bytes_sent, total_bytes_received, Err(io_error)) => write!(f, "Client {client_id} closed with IO error after {total_bytes_sent} bytes sent and {total_bytes_received} bytes received: {io_error}"),
            Self::NewSandstormConnectionAccepted(manager_id, socket_address) => write!(f, "New management connection from {socket_address} assigned ID {manager_id}"),
            Self::SandstormConnectionAcceptFailed(Some(socket_address), io_error) => write!(f, "Failed to accept incoming management connection from socket {socket_address}: {io_error}"),
            Self::SandstormConnectionAcceptFailed(None, io_error) => write!(f, "Failed to accept incoming management connection from unknown socket: {io_error}"),
            Self::SandstormRequestedUnsupportedVersion(manager_id, version) => write!(f, "Manager {manager_id} requested unsupported sandstorm version: {version}"),
            Self::SandstormAuthenticatedAs(manager_id, username, true) => write!(f, "Manager {manager_id} successfully authenticated as {username}"),
            Self::SandstormAuthenticatedAs(manager_id, username, false) => write!(f, "Manager {manager_id} unsuccessfully authenticated as {username}"),
            Self::NewSocksSocketRequestedByManager(manager_id, socket_address) => write!(f, "Manager {manager_id} requested opening a new socks5 socket at {socket_address}"),
            Self::RemoveSocksSocketRequestedByManager(manager_id, socket_address) => write!(f, "Manager {manager_id} requested closing socks5 socket at {socket_address}"),
            Self::NewSandstormSocketRequestedByManager(manager_id, socket_address) => write!(f, "Manager {manager_id} requested opening a new sandstorm socket at {socket_address}"),
            Self::RemoveSandstormSocketRequestedByManager(manager_id, socket_address) => write!(f, "Manager {manager_id} requested closing sandstorm socket at {socket_address}"),
            Self::UserRegisteredByManager(manager_id, username, role) => write!(f, "Manager {manager_id} registered new {role} user {username}"),
            Self::UserUpdatedByManager(manager_id, username, role, false) => write!(f, "Manager {manager_id} updated role of user {username} to {role}"),
            Self::UserUpdatedByManager(manager_id, username, role, true) => write!(f, "Manager {manager_id} updated user {username} with role {role} and new password"),
            Self::UserDeletedByManager(manager_id, username, role) => write!(f, "Manager {manager_id} deleted {role} user {username}"),
            Self::AuthMethodToggledByManager(manager_id, auth_method, false) => write!(f, "Manager {manager_id} disabled authentication method {auth_method}"),
            Self::AuthMethodToggledByManager(manager_id, auth_method, true) => write!(f, "Manager {manager_id} enabled authentication method {auth_method}"),
            Self::BufferSizeChangedByManager(manager_id, buffer_size) => write!(f, "Manager {manager_id} set client buffer size to {buffer_size}"),
            Self::SandstormRequestedShutdown(manager_id) => write!(f, "Manager {manager_id} requested the server shuts down"),
            Self::SandstormConnectionFinished(manager_id, Ok(())) => write!(f, "Manager {manager_id} finished"),
            Self::SandstormConnectionFinished(manager_id, Err(io_error)) => write!(f, "Manager {manager_id} closed with IO error: {io_error}"),
            Self::ShutdownSignalReceived => write!(f, "Shutdown signal received"),
        }
    }
}
