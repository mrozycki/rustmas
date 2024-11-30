use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

pub fn wrap_plugin<P, Q, R>(
    output_path: P,
    executable_path: Q,
    manifest_path: R,
) -> std::io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    R: AsRef<Path>,
{
    fn inner(
        output_path: &Path,
        executable_path: &Path,
        manifest_path: &Path,
    ) -> std::io::Result<()> {
        let mut archive = tar::Builder::new(Vec::new());
        archive.append_path_with_name(manifest_path, "manifest.json")?;
        archive.append_path_with_name(executable_path, "plugin.wasm")?;
        let archive_data = archive.into_inner()?;

        BufWriter::new(File::create(output_path)?).write_all(&archive_data)
    }
    inner(
        output_path.as_ref(),
        executable_path.as_ref(),
        manifest_path.as_ref(),
    )
}
