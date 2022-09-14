use anyhow::Result;
use libc::TIOCSTI;
use log::debug;
use nix::{sched::CloneFlags, sys::stat::Mode};
use syscallz::{Action, Cmp, Comparator, Context, Syscall};

const EPERM: u16 = 1;

pub fn set_syscalls() -> Result<()> {
    debug!("Refusing / Filtering unwanted syscalls");

    // Unconditionnal syscall deny
    let syscalls_refused = [
        Syscall::keyctl,
        Syscall::add_key,
        Syscall::request_key,
        Syscall::mbind,
        Syscall::migrate_pages,
        Syscall::move_pages,
        Syscall::set_mempolicy,
        Syscall::userfaultfd,
        Syscall::perf_event_open,
    ];

    let s_isuid: u64 = Mode::S_ISUID.bits().into();
    let s_isgid: u64 = Mode::S_ISGID.bits().into();
    let clone_new_user: u64 = CloneFlags::CLONE_NEWUSER.bits() as u64;

    // Conditional syscall deny
    let syscalls_refuse_if_comp = [
        (Syscall::chmod, 1, s_isuid),
        (Syscall::chmod, 1, s_isgid),
        (Syscall::fchmod, 1, s_isuid),
        (Syscall::fchmod, 1, s_isgid),
        (Syscall::fchmodat, 2, s_isuid),
        (Syscall::fchmodat, 2, s_isgid),
        (Syscall::unshare, 0, clone_new_user),
        (Syscall::clone, 0, clone_new_user),
        (Syscall::ioctl, 1, TIOCSTI),
    ];

    // Initialize seccomp profile with all syscalls allowed by default
    let mut ctx = Context::init_with_action(Action::Allow)?;
    ctx.load()?;
    for (sc, ind, biteq) in syscalls_refuse_if_comp.iter() {
        refuse_if_comp(&mut ctx, *ind, sc, *biteq)?;
    }
    for sc in syscalls_refused.iter() {
        refuse_syscall(&mut ctx, sc)?;
    }
    Ok(())
}

fn refuse_syscall(ctx: &mut Context, sc: &Syscall) -> Result<()> {
    Ok(ctx.set_action_for_syscall(Action::Errno(EPERM), *sc)?)
}

fn refuse_if_comp(ctx: &mut Context, ind: u32, sc: &Syscall, biteq: u64) -> Result<()> {
    Ok(ctx.set_rule_for_syscall(
        Action::Errno(EPERM),
        *sc,
        &[Comparator::new(ind, Cmp::MaskedEq, biteq, Some(biteq))],
    )?)
}
