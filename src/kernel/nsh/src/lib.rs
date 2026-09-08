#![allow(non_local_definitions)]
use pyo3::prelude::*;
use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::path::PathBuf;

#[pyclass]
pub struct NativeSharedWorkspace {
    mmap: MmapMut,
    #[allow(dead_code)]
    path: PathBuf,
}

#[pymethods]
impl NativeSharedWorkspace {
    #[new]
    pub fn new(path_str: &str, size_bytes: usize) -> PyResult<Self> {
        let path = PathBuf::from(path_str);
        
        // Ensure path exists in /dev/shm or similar
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;

        // Allocate space if needed (simulating truncate)
        file.set_len(size_bytes as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(NativeSharedWorkspace { mmap, path })
    }

    pub fn write_float_at(&mut self, offset: usize, value: f32) -> PyResult<()> {
        let bytes = value.to_ne_bytes();
        if offset + 4 > self.mmap.len() {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>("Offset out of bounds"));
        }
        self.mmap[offset..offset + 4].copy_from_slice(&bytes);
        Ok(())
    }

    pub fn read_float_at(&self, offset: usize) -> PyResult<f32> {
        if offset + 4 > self.mmap.len() {
            return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>("Offset out of bounds"));
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.mmap[offset..offset + 4]);
        Ok(f32::from_ne_bytes(bytes))
    }

    pub fn flush(&self) -> PyResult<()> {
        self.mmap.flush()?;
        Ok(())
    }

    #[getter]
    pub fn size(&self) -> usize {
        self.mmap.len()
    }
}

#[pymodule]
fn omnimind_nsh(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<NativeSharedWorkspace>()?;
    Ok(())
}
