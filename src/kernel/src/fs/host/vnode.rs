use crate::errno::Errno;
use crate::fs::{ComponentName, Vnode, VopVector, DEFAULT_VNODEOPS};
use crate::process::VThread;
use std::sync::Arc;

pub static VNODE_OPS: VopVector = VopVector {
    default: Some(&DEFAULT_VNODEOPS),
    access: Some(access),
    accessx: None,
    lookup: Some(lookup),
};

fn access(_: &Arc<Vnode>, _: Option<&VThread>, _: u32) -> Result<(), Box<dyn Errno>> {
    todo!()
}

fn lookup(_: &Arc<Vnode>, _: &ComponentName) -> Result<Arc<Vnode>, Box<dyn Errno>> {
    todo!()
}