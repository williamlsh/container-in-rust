use nix::sys::socket::{recv, send, socketpair, AddressFamily, MsgFlags, SockFlag, SockType};
use std::os::unix::prelude::RawFd;

pub(crate) fn generate_socketpair() -> nix::Result<(RawFd, RawFd)> {
    socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None, //The socket will use the default protocol associated with the socket type.
        SockFlag::SOCK_CLOEXEC,
    )
}

pub(crate) fn send_boolean(fd: RawFd, boolean: bool) -> nix::Result<()> {
    let data: [u8; 1] = [boolean.into()];
    send(fd, &data, MsgFlags::empty())?;
    Ok(())
}

pub(crate) fn recv_boolean(fd: RawFd) -> nix::Result<bool> {
    let mut data: [u8; 1] = [0];
    recv(fd, &mut data, MsgFlags::empty())?;
    Ok(data[0] == 1)
}
