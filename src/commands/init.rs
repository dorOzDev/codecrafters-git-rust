use std::fs;
use std::io;
use std::path::Path;
use crate::constants::*;

pub fn run() -> io::Result<()> {
    init_git_dir(std::env::current_dir()?.as_path())?;
    println!("Initialized git directory");
    Ok(())
}

pub fn init_git_dir(root: &Path) -> io::Result<()> {
    let git_dir = root.join(GIT_DIR);
    let objects_dir = root.join(GIT_OBJECTS_DIR);
    let refs_dir = root.join(GIT_REF_DIR);
    let head_path = root.join(GIT_HEAD_PATH);

    fs::create_dir_all(&git_dir)?;
    fs::create_dir_all(&objects_dir)?;
    fs::create_dir_all(&refs_dir)?;
    fs::write(&head_path, "ref: refs/heads/main\n")?;

    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::fileapi::SetFileAttributesW;
        use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

        let wide: Vec<u16> = OsStr::new(git_dir.as_os_str())
            .encode_wide()
            .chain(once(0))
            .collect();

        unsafe {
            if SetFileAttributesW(wide.as_ptr(), FILE_ATTRIBUTE_HIDDEN) == 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    Ok(())
}