use bincode;
use std::io::Cursor;
use std::sync::Arc;

use errors::*;

use super::super::backends::frame::Command;
use super::super::DoubleFrame;
use super::mesh::*;

pub const MAGIC: [u8; 8] = [
    'V' as u8, 'M' as u8, 'S' as u8, 'H' as u8, ' ' as u8, 0, 0, 1,
];

#[derive(Clone)]
pub struct MeshLoader {
    frames: Arc<DoubleFrame>,
}

impl MeshLoader {
    pub(crate) fn new(frames: Arc<DoubleFrame>) -> Self {
        MeshLoader { frames: frames }
    }
}

impl ::res::registry::Register for MeshLoader {
    type Handle = MeshHandle;
    type Intermediate = (MeshParams, Option<MeshData>);
    type Value = MeshParams;

    fn load(&self, handle: Self::Handle, bytes: &[u8]) -> Result<Self::Intermediate> {
        if &bytes[0..8] != &MAGIC[..] {
            bail!("[MeshLoader] MAGIC number not match.");
        }

        let mut file = Cursor::new(&bytes[8..]);
        let params: MeshParams = bincode::deserialize_from(&mut file)?;
        let data = bincode::deserialize_from(&mut file)?;

        info!(
            "[MeshLoader] loads {:?}. (Verts: {}, Indxes: {})",
            handle, params.num_verts, params.num_idxes
        );

        Ok((params, Some(data)))
    }

    fn attach(&self, handle: Self::Handle, item: Self::Intermediate) -> Result<Self::Value> {
        item.0.validate(item.1.as_ref())?;

        let mut frame = self.frames.front();
        let task = Command::CreateMesh(handle, item.0.clone(), item.1);
        frame.cmds.push(task);

        Ok(item.0)
    }

    fn detach(&self, handle: Self::Handle, _: Self::Value) {
        let cmd = Command::DeleteMesh(handle);
        self.frames.front().cmds.push(cmd);
    }
}
