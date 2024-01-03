use std::{
    io::{self, ErrorKind},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    logging::{LogEvent, LogEventType},
    socks5::{AuthMethod, SocksRequest, SocksRequestAddress},
    users::{UserRole, UsersLoadingError},
};

trait ByteWrite {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error>;
}

trait ByteRead: Sized {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error>;
}

impl ByteWrite for () {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, _: &mut W) -> Result<(), io::Error> {
        Ok(())
    }
}

impl ByteRead for () {
    async fn read<R: AsyncRead + Unpin + ?Sized>(_: &mut R) -> Result<Self, io::Error> {
        Ok(())
    }
}

impl ByteWrite for bool {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u8(*self as u8).await
    }
}

impl ByteRead for bool {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        Ok(reader.read_u8().await? != 0)
    }
}

impl ByteWrite for u8 {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u8(*self).await
    }
}

impl ByteRead for u8 {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        reader.read_u8().await
    }
}

impl ByteWrite for u16 {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u16(*self).await
    }
}

impl ByteRead for u16 {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        reader.read_u16().await
    }
}

impl ByteWrite for u32 {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u32(*self).await
    }
}

impl ByteRead for u32 {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        reader.read_u32().await
    }
}

impl ByteWrite for u64 {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u64(*self).await
    }
}

impl ByteRead for u64 {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        reader.read_u64().await
    }
}

impl ByteWrite for char {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u32(*self as u32).await
    }
}

impl ByteRead for char {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let c = reader.read_u32().await?;
        char::from_u32(c).ok_or(ErrorKind::InvalidData.into())
    }
}

impl<T0: ByteWrite, T1: ByteWrite> ByteWrite for (T0, T1) {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.0.write(writer).await?;
        self.1.write(writer).await
    }
}

impl<T0: ByteWrite, T1: ByteWrite, T2: ByteWrite> ByteWrite for (T0, T1, T2) {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.0.write(writer).await?;
        self.1.write(writer).await?;
        self.2.write(writer).await
    }
}

impl<T0: ByteWrite, T1: ByteWrite, T2: ByteWrite, T3: ByteWrite> ByteWrite for (T0, T1, T2, T3) {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.0.write(writer).await?;
        self.1.write(writer).await?;
        self.2.write(writer).await?;
        self.3.write(writer).await
    }
}

impl<T0: ByteWrite, T1: ByteWrite, T2: ByteWrite, T3: ByteWrite, T4: ByteWrite> ByteWrite for (T0, T1, T2, T3, T4) {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.0.write(writer).await?;
        self.1.write(writer).await?;
        self.2.write(writer).await?;
        self.3.write(writer).await?;
        self.4.write(writer).await
    }
}

impl ByteWrite for Ipv4Addr {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(&self.octets()).await
    }
}

impl ByteRead for Ipv4Addr {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let mut octets = [0u8; 4];
        reader.read_exact(&mut octets).await?;
        Ok(octets.into())
    }
}

impl ByteWrite for Ipv6Addr {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(&self.octets()).await
    }
}

impl ByteRead for Ipv6Addr {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let mut octets = [0u8; 16];
        reader.read_exact(&mut octets).await?;

        Ok(octets.into())
    }
}

impl ByteWrite for SocketAddrV4 {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.ip().write(writer).await?;
        writer.write_u16(self.port()).await
    }
}

impl ByteRead for SocketAddrV4 {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let mut octets = [0u8; 4];
        reader.read_exact(&mut octets).await?;
        let port = reader.read_u16().await?;

        Ok(SocketAddrV4::new(octets.into(), port))
    }
}

impl ByteWrite for SocketAddrV6 {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.ip().write(writer).await?;
        writer.write_u16(self.port()).await?;
        writer.write_u32(self.flowinfo()).await?;
        writer.write_u32(self.scope_id()).await
    }
}

impl ByteRead for SocketAddrV6 {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let mut octets = [0u8; 16];
        reader.read_exact(&mut octets).await?;
        let port = reader.read_u16().await?;
        let flowinfo = reader.read_u32().await?;
        let scope_id = reader.read_u32().await?;

        Ok(SocketAddrV6::new(octets.into(), port, flowinfo, scope_id))
    }
}

impl ByteWrite for SocketAddr {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        match self {
            SocketAddr::V4(v4) => {
                writer.write_u8(4).await?;
                v4.write(writer).await
            }
            SocketAddr::V6(v6) => {
                writer.write_u8(6).await?;
                v6.write(writer).await
            }
        }
    }
}

impl ByteRead for SocketAddr {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let addr_type = reader.read_u8().await?;
        match addr_type {
            4 => Ok(SocketAddr::V4(SocketAddrV4::read(reader).await?)),
            6 => Ok(SocketAddr::V6(SocketAddrV6::read(reader).await?)),
            _ => Err(io::ErrorKind::InvalidData.into()),
        }
    }
}

impl<T: ByteWrite> ByteWrite for Option<T> {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        match self {
            Some(value) => {
                writer.write_u8(1).await?;
                value.write(writer).await
            }
            None => writer.write_u8(0).await,
        }
    }
}

impl<T: ByteRead> ByteRead for Option<T> {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let has_value = reader.read_u8().await?;
        match has_value {
            0 => Ok(None),
            _ => Ok(Some(T::read(reader).await?)),
        }
    }
}

impl<T: ByteWrite, E: ByteWrite> ByteWrite for Result<T, E> {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        match self {
            Ok(v) => {
                writer.write_u8(1).await?;
                v.write(writer).await
            }
            Err(e) => {
                writer.write_u8(0).await?;
                e.write(writer).await
            }
        }
    }
}

impl<T: ByteRead, E: ByteRead> ByteRead for Result<T, E> {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        match reader.read_u8().await? {
            0 => Ok(Err(E::read(reader).await?)),
            _ => Ok(Ok(T::read(reader).await?)),
        }
    }
}

impl ByteWrite for io::Error {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        let kind_id = match self.kind() {
            ErrorKind::NotFound => 1,
            ErrorKind::PermissionDenied => 2,
            ErrorKind::ConnectionRefused => 3,
            ErrorKind::ConnectionReset => 4,
            ErrorKind::ConnectionAborted => 5,
            ErrorKind::NotConnected => 6,
            ErrorKind::AddrInUse => 7,
            ErrorKind::AddrNotAvailable => 8,
            ErrorKind::BrokenPipe => 9,
            ErrorKind::AlreadyExists => 10,
            ErrorKind::WouldBlock => 11,
            ErrorKind::InvalidInput => 12,
            ErrorKind::InvalidData => 13,
            ErrorKind::TimedOut => 14,
            ErrorKind::WriteZero => 15,
            ErrorKind::Interrupted => 16,
            ErrorKind::Unsupported => 17,
            ErrorKind::UnexpectedEof => 18,
            ErrorKind::OutOfMemory => 19,
            ErrorKind::Other => 20,
            _ => 0,
        };

        writer.write_u8(kind_id).await
    }
}

impl ByteRead for io::Error {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let kind_id = reader.read_u8().await?;

        let error_kind = match kind_id {
            1 => ErrorKind::NotFound,
            2 => ErrorKind::PermissionDenied,
            3 => ErrorKind::ConnectionRefused,
            4 => ErrorKind::ConnectionReset,
            5 => ErrorKind::ConnectionAborted,
            6 => ErrorKind::NotConnected,
            7 => ErrorKind::AddrInUse,
            8 => ErrorKind::AddrNotAvailable,
            9 => ErrorKind::BrokenPipe,
            10 => ErrorKind::AlreadyExists,
            11 => ErrorKind::WouldBlock,
            12 => ErrorKind::InvalidInput,
            13 => ErrorKind::InvalidData,
            14 => ErrorKind::TimedOut,
            15 => ErrorKind::WriteZero,
            16 => ErrorKind::Interrupted,
            17 => ErrorKind::Unsupported,
            18 => ErrorKind::UnexpectedEof,
            19 => ErrorKind::OutOfMemory,
            _ => ErrorKind::Other,
        };

        Ok(error_kind.into())
    }
}

impl ByteWrite for &str {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        let bytes = self.as_bytes();
        let len = bytes.len();
        if len > u16::MAX as usize {
            return Err(ErrorKind::InvalidData.into());
        }

        let len = len as u16;
        writer.write_u16(len).await?;
        writer.write_all(bytes).await
    }
}

impl ByteWrite for String {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.as_str().write(writer).await
    }
}

impl ByteRead for String {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let len = reader.read_u16().await? as usize;

        let mut s = String::with_capacity(len);
        unsafe {
            // SAFETY: The elements of `v` are initialized by `read_exact`, and then we ensure they are valid UTF-8.
            let v = s.as_mut_vec();
            v.set_len(len);
            reader.read_exact(&mut v[0..len]).await?;
            if std::str::from_utf8(v).is_err() {
                return Err(ErrorKind::InvalidData.into());
            }
        }

        Ok(s)
    }
}

struct SmallWriteString<'a>(&'a str);

impl<'a> ByteWrite for SmallWriteString<'a> {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        let bytes = self.0.as_bytes();
        let len = bytes.len();
        if len > u8::MAX as usize {
            return Err(ErrorKind::InvalidData.into());
        }

        let len = len as u8;
        writer.write_u8(len).await?;
        writer.write_all(bytes).await
    }
}

struct SmallReadString(String);

impl ByteRead for SmallReadString {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let len = reader.read_u8().await? as usize;

        let mut s = String::with_capacity(len);
        unsafe {
            // SAFETY: The elements of `v` are initialized by `read_exact`, and then we ensure they are valid UTF-8.
            let v = s.as_mut_vec();
            v.set_len(len);
            reader.read_exact(&mut v[0..len]).await?;
            if std::str::from_utf8(v).is_err() {
                return Err(ErrorKind::InvalidData.into());
            }
        }

        Ok(SmallReadString(s))
    }
}

impl ByteWrite for UsersLoadingError {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        match self {
            UsersLoadingError::IO(io_error) => (1u8, io_error).write(writer).await,
            UsersLoadingError::InvalidUtf8 { line_number, byte_at } => (2u8, line_number, byte_at).write(writer).await,
            UsersLoadingError::LineTooLong { line_number, byte_at } => (3u8, line_number, byte_at).write(writer).await,
            UsersLoadingError::ExpectedRoleCharGotEOF(line_number, char_at) => (4u8, line_number, char_at).write(writer).await,
            UsersLoadingError::InvalidRoleChar(line_number, char_at, char) => (5u8, line_number, char_at, *char).write(writer).await,
            UsersLoadingError::ExpectedColonGotEOF(line_number, char_at) => (6u8, line_number, char_at).write(writer).await,
            UsersLoadingError::EmptyUsername(line_number, char_at) => (7u8, line_number, char_at).write(writer).await,
            UsersLoadingError::UsernameTooLong(line_number, char_at) => (8u8, line_number, char_at).write(writer).await,
            UsersLoadingError::EmptyPassword(line_number, char_at) => (9u8, line_number, char_at).write(writer).await,
            UsersLoadingError::PasswordTooLong(line_number, char_at) => (10u8, line_number, char_at).write(writer).await,
            UsersLoadingError::NoUsers => 11u8.write(writer).await,
        }
    }
}

impl ByteRead for UsersLoadingError {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let t = reader.read_u8().await?;

        match t {
            1 => Ok(UsersLoadingError::IO(io::Error::read(reader).await?)),
            2 => Ok(UsersLoadingError::InvalidUtf8 {
                line_number: reader.read_u32().await?,
                byte_at: reader.read_u64().await?,
            }),
            3 => Ok(UsersLoadingError::LineTooLong {
                line_number: reader.read_u32().await?,
                byte_at: reader.read_u64().await?,
            }),
            4 => Ok(UsersLoadingError::ExpectedRoleCharGotEOF(
                reader.read_u32().await?,
                reader.read_u32().await?,
            )),
            5 => Ok(UsersLoadingError::InvalidRoleChar(
                reader.read_u32().await?,
                reader.read_u32().await?,
                char::read(reader).await?,
            )),
            6 => Ok(UsersLoadingError::ExpectedColonGotEOF(
                reader.read_u32().await?,
                reader.read_u32().await?,
            )),
            7 => Ok(UsersLoadingError::EmptyUsername(reader.read_u32().await?, reader.read_u32().await?)),
            8 => Ok(UsersLoadingError::UsernameTooLong(
                reader.read_u32().await?,
                reader.read_u32().await?,
            )),
            9 => Ok(UsersLoadingError::EmptyPassword(reader.read_u32().await?, reader.read_u32().await?)),
            10 => Ok(UsersLoadingError::PasswordTooLong(
                reader.read_u32().await?,
                reader.read_u32().await?,
            )),
            11 => Ok(UsersLoadingError::NoUsers),
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }
}

impl ByteWrite for AuthMethod {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u8(*self as u8).await
    }
}

impl ByteRead for AuthMethod {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let value = reader.read_u8().await?;
        AuthMethod::from_u8(value).ok_or(ErrorKind::InvalidData.into())
    }
}

impl ByteWrite for UserRole {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer
            .write_u8(match self {
                UserRole::Admin => 1,
                UserRole::Regular => 2,
            })
            .await
    }
}

impl ByteRead for UserRole {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        match reader.read_u8().await? {
            1 => Ok(UserRole::Admin),
            2 => Ok(UserRole::Regular),
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }
}

impl<T: ByteWrite> ByteWrite for &T {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        (*self).write(writer).await
    }
}

impl ByteWrite for SocksRequestAddress {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        match self {
            Self::IPv4(v4) => (4u8, v4).write(writer).await,
            Self::IPv6(v6) => (6u8, v6).write(writer).await,
            Self::Domainname(domainname) => (200u8, SmallWriteString(domainname)).write(writer).await,
        }
    }
}

impl ByteRead for SocksRequestAddress {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        match reader.read_u8().await? {
            4 => Ok(SocksRequestAddress::IPv4(Ipv4Addr::read(reader).await?)),
            6 => Ok(SocksRequestAddress::IPv6(Ipv6Addr::read(reader).await?)),
            200 => Ok(SocksRequestAddress::Domainname(SmallReadString::read(reader).await?.0)),
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }
}

impl ByteWrite for SocksRequest {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        (&self.destination, self.port).write(writer).await
    }
}

impl ByteRead for SocksRequest {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        Ok(SocksRequest::new(
            SocksRequestAddress::read(reader).await?,
            reader.read_u16().await?,
        ))
    }
}

impl ByteWrite for LogEventType {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        match self {
            Self::NewListeningSocket(socket_address) => (1u8, socket_address).write(writer).await,
            Self::FailedBindListeningSocket(socket_address, io_error) => (2u8, socket_address, io_error).write(writer).await,
            Self::FailedBindAnySocketAborting => 3u8.write(writer).await,
            Self::RemovedListeningSocket(socket_address) => (4u8, socket_address).write(writer).await,
            Self::LoadingUsersFromFile(filename) => (5u8, filename).write(writer).await,
            Self::UsersLoadedFromFile(filename, result) => (6u8, filename, result).write(writer).await,
            Self::StartingUpWithSingleDefaultUser => 7u8.write(writer).await,
            Self::SavingUsersToFile(filename) => (8u8, filename).write(writer).await,
            Self::UsersSavedToFile(filename, result) => (9u8, filename, result).write(writer).await,
            Self::UserRegistered(username, role) => (10u8, SmallWriteString(username), role).write(writer).await,
            Self::UserReplacedByArgs(username, role) => (11u8, SmallWriteString(username), role).write(writer).await,
            Self::UserUpdated(username, role, password_changed) => {
                (12u8, SmallWriteString(username), role, password_changed).write(writer).await
            }
            Self::UserDeleted(username, role) => (13u8, SmallWriteString(username), role).write(writer).await,
            Self::NewClientConnectionAccepted(client_id, socket_address) => (14u8, client_id, socket_address).write(writer).await,
            Self::ClientConnectionAcceptFailed(maybe_socket_address, io_error) => {
                (15u8, maybe_socket_address, io_error).write(writer).await
            }
            Self::ClientRequestedUnsupportedVersion(client_id, ver) => (16u8, client_id, ver).write(writer).await,
            Self::ClientRequestedUnsupportedCommand(client_id, cmd) => (17u8, client_id, cmd).write(writer).await,
            Self::ClientRequestedUnsupportedAtyp(client_id, atyp) => (18u8, client_id, atyp).write(writer).await,
            Self::ClientSelectedAuthMethod(client_id, auth_method) => (19u8, client_id, auth_method).write(writer).await,
            Self::ClientRequestedUnsupportedUserpassVersion(client_id, ver) => (20u8, client_id, ver).write(writer).await,
            Self::ClientAuthenticatedWithUserpass(client_id, username, success) => {
                (21u8, client_id, SmallWriteString(username), success).write(writer).await
            }
            Self::ClientSocksRequest(client_id, request) => (22u8, client_id, request).write(writer).await,
            Self::ClientDnsLookup(client_id, domainname) => (23u8, client_id, SmallWriteString(domainname)).write(writer).await,
            Self::ClientAttemptingConnect(client_id, socket_address) => (24u8, client_id, socket_address).write(writer).await,
            Self::ClientConnectionAttemptBindFailed(client_id, io_error) => (25u8, client_id, io_error).write(writer).await,
            Self::ClientConnectionAttemptConnectFailed(client_id, io_error) => (26u8, client_id, io_error).write(writer).await,
            Self::ClientFailedToConnectToDestination(client_id) => (27u8, client_id).write(writer).await,
            Self::ClientConnectedToDestination(client_id, socket_address) => (28u8, client_id, socket_address).write(writer).await,
            Self::ClientBytesSent(client_id, count) => (29u8, client_id, count).write(writer).await,
            Self::ClientBytesReceived(client_id, count) => (30u8, client_id, count).write(writer).await,
            Self::ClientSourceShutdown(client_id) => (31u8, client_id).write(writer).await,
            Self::ClientDestinationShutdown(client_id) => (32u8, client_id).write(writer).await,
            Self::ClientConnectionFinished(client_id, sent, received, result) => {
                (33u8, client_id, sent, received, result).write(writer).await
            }
        }
    }
}

impl ByteRead for LogEventType {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        let t = reader.read_u8().await?;

        match t {
            1u8 => Ok(Self::NewListeningSocket(SocketAddr::read(reader).await?)),
            2u8 => Ok(Self::FailedBindListeningSocket(
                SocketAddr::read(reader).await?,
                io::Error::read(reader).await?,
            )),
            3u8 => Ok(Self::FailedBindAnySocketAborting),
            4u8 => Ok(Self::RemovedListeningSocket(SocketAddr::read(reader).await?)),
            5u8 => Ok(Self::LoadingUsersFromFile(String::read(reader).await?)),
            6u8 => Ok(Self::UsersLoadedFromFile(
                String::read(reader).await?,
                <Result<u64, UsersLoadingError> as ByteRead>::read(reader).await?,
            )),
            7u8 => Ok(Self::StartingUpWithSingleDefaultUser),
            8u8 => Ok(Self::SavingUsersToFile(String::read(reader).await?)),
            9u8 => Ok(Self::UsersSavedToFile(
                String::read(reader).await?,
                <Result<u64, io::Error> as ByteRead>::read(reader).await?,
            )),
            10u8 => Ok(Self::UserRegistered(
                SmallReadString::read(reader).await?.0,
                UserRole::read(reader).await?,
            )),
            11u8 => Ok(Self::UserReplacedByArgs(
                SmallReadString::read(reader).await?.0,
                UserRole::read(reader).await?,
            )),
            12u8 => Ok(Self::UserUpdated(
                SmallReadString::read(reader).await?.0,
                UserRole::read(reader).await?,
                bool::read(reader).await?,
            )),
            13u8 => Ok(Self::UserDeleted(
                SmallReadString::read(reader).await?.0,
                UserRole::read(reader).await?,
            )),
            14u8 => Ok(Self::NewClientConnectionAccepted(
                reader.read_u64().await?,
                SocketAddr::read(reader).await?,
            )),
            15u8 => Ok(Self::ClientConnectionAcceptFailed(
                <Option<SocketAddr> as ByteRead>::read(reader).await?,
                io::Error::read(reader).await?,
            )),
            16u8 => Ok(Self::ClientRequestedUnsupportedVersion(
                reader.read_u64().await?,
                reader.read_u8().await?,
            )),
            17u8 => Ok(Self::ClientRequestedUnsupportedCommand(
                reader.read_u64().await?,
                reader.read_u8().await?,
            )),
            18u8 => Ok(Self::ClientRequestedUnsupportedAtyp(
                reader.read_u64().await?,
                reader.read_u8().await?,
            )),
            19u8 => Ok(Self::ClientSelectedAuthMethod(
                reader.read_u64().await?,
                AuthMethod::read(reader).await?,
            )),
            20u8 => Ok(Self::ClientRequestedUnsupportedUserpassVersion(
                reader.read_u64().await?,
                reader.read_u8().await?,
            )),
            21u8 => Ok(Self::ClientAuthenticatedWithUserpass(
                reader.read_u64().await?,
                String::read(reader).await?,
                bool::read(reader).await?,
            )),
            22u8 => Ok(Self::ClientSocksRequest(
                reader.read_u64().await?,
                SocksRequest::read(reader).await?,
            )),
            23u8 => Ok(Self::ClientDnsLookup(
                reader.read_u64().await?,
                SmallReadString::read(reader).await?.0,
            )),
            24u8 => Ok(Self::ClientAttemptingConnect(
                reader.read_u64().await?,
                SocketAddr::read(reader).await?,
            )),
            25u8 => Ok(Self::ClientConnectionAttemptBindFailed(
                reader.read_u64().await?,
                io::Error::read(reader).await?,
            )),
            26u8 => Ok(Self::ClientConnectionAttemptConnectFailed(
                reader.read_u64().await?,
                io::Error::read(reader).await?,
            )),
            27u8 => Ok(Self::ClientFailedToConnectToDestination(reader.read_u64().await?)),
            28u8 => Ok(Self::ClientConnectedToDestination(
                reader.read_u64().await?,
                SocketAddr::read(reader).await?,
            )),
            29u8 => Ok(Self::ClientBytesSent(reader.read_u64().await?, reader.read_u64().await?)),
            30u8 => Ok(Self::ClientBytesReceived(reader.read_u64().await?, reader.read_u64().await?)),
            31u8 => Ok(Self::ClientSourceShutdown(reader.read_u64().await?)),
            32u8 => Ok(Self::ClientDestinationShutdown(reader.read_u64().await?)),
            33u8 => Ok(Self::ClientConnectionFinished(
                reader.read_u64().await?,
                reader.read_u64().await?,
                reader.read_u64().await?,
                <Result<(), io::Error> as ByteRead>::read(reader).await?,
            )),
            _ => Err(ErrorKind::InvalidData.into()),
        }
    }
}

impl ByteWrite for LogEvent {
    async fn write<W: AsyncWrite + Unpin + ?Sized>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_i64(self.timestamp).await?;
        self.data.write(writer).await
    }
}

impl ByteRead for LogEvent {
    async fn read<R: AsyncRead + Unpin + ?Sized>(reader: &mut R) -> Result<Self, io::Error> {
        Ok(LogEvent::new(reader.read_i64().await?, LogEventType::read(reader).await?))
    }
}
