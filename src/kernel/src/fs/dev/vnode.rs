use super::dirent::Dirent;
use crate::errno::{Errno, EIO, ENOTDIR, EOPNOTSUPP};
use crate::fs::{
    check_access, ComponentName, DevFs, NameiFlags, NameiOp, Vnode, VnodeType, VopVector,
    DEFAULT_VNODEOPS,
};
use crate::process::VThread;
use crate::ucred::Ucred;
use std::num::NonZeroI32;
use std::sync::Arc;
use thiserror::Error;

pub static VNODE_OPS: VopVector = VopVector {
    default: Some(&DEFAULT_VNODEOPS),
    access: Some(access),
    accessx: None,
    lookup: Some(lookup),
};

fn access(vn: &Arc<Vnode>, _: &VThread, cred: &Ucred, access: u32) -> Result<(), Box<dyn Errno>> {
    // Get dirent.
    let mut dirent = vn.data().clone().downcast::<Dirent>().unwrap();
    let is_dir = match vn.ty() {
        VnodeType::Directory(_) => {
            if let Some(v) = dirent.dir() {
                // Is it possible the parent will be gone here?
                dirent = v.upgrade().unwrap();
            }

            true
        }
        _ => false,
    };

    // Get file permissions as atomic.
    let (uid, gid, mode) = {
        let uid = dirent.uid();
        let gid = dirent.gid();
        let mode = dirent.mode();

        (*uid, *gid, *mode)
    };

    // Check access.
    let err = match check_access(cred, uid, gid, mode.into(), access, is_dir) {
        Ok(_) => return Ok(()),
        Err(e) => e,
    };

    // TODO: Check if file is a controlling terminal.
    return Err(Box::new(err));
}

fn lookup(vn: &Arc<Vnode>, cn: &ComponentName) -> Result<Arc<Vnode>, Box<dyn Errno>> {
    // Populate devices.
    let fs = vn
        .fs()
        .data()
        .and_then(|v| v.downcast_ref::<DevFs>())
        .unwrap();

    fs.populate();

    // Check if last component.
    let op = cn.op;
    let flags = cn.flags;

    if flags.intersects(NameiFlags::ISLASTCN) && op == NameiOp::Rename {
        return Err(Box::new(LookupError::RenameWithLastComponent));
    }

    // Check if directory.
    match vn.ty() {
        VnodeType::Directory(root) => {
            if flags.intersects(NameiFlags::ISDOTDOT) && *root {
                return Err(Box::new(LookupError::DotdotOnRoot));
            }
        }
        _ => return Err(Box::new(LookupError::NotDirectory)),
    }

    // TODO: Implement the remaining lookup.
    todo!()
}

/// Represents an error when [`lookup()`] is failed.
#[derive(Debug, Error)]
enum LookupError {
    #[error("rename with last component is not supported")]
    RenameWithLastComponent,

    #[error("file is not a directory")]
    NotDirectory,

    #[error("cannot resolve '..' on the root directory")]
    DotdotOnRoot,
}

impl Errno for LookupError {
    fn errno(&self) -> NonZeroI32 {
        match self {
            Self::RenameWithLastComponent => EOPNOTSUPP,
            Self::NotDirectory => ENOTDIR,
            Self::DotdotOnRoot => EIO,
        }
    }
}
