use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use mlua::{Error, Lua, Result, Table, Value};

#[derive(Clone)]
struct FsContext {
    remote_file: PathBuf,
    remote_dir: PathBuf,
}

fn get_fs_context(lua: &Lua) -> FsContext {
    lua.app_data_ref::<FsContext>()
        .expect("fs context not found in lua state")
        .clone()
}

// Context functions

fn remotefile(lua: &Lua, _: ()) -> Result<String> {
    let ctx = get_fs_context(lua);
    Ok(ctx.remote_file.display().to_string())
}

fn remotedir(lua: &Lua, _: ()) -> Result<String> {
    let ctx = get_fs_context(lua);
    Ok(ctx.remote_dir.display().to_string())
}

fn workingdir(_lua: &Lua, _: ()) -> Result<String> {
    std::env::current_dir()
        .map(|path| path.display().to_string())
        .map_err(|error| Error::runtime(format!("failed to get working directory: {error}")))
}

// Directory functions

fn homedir(_lua: &Lua, _: ()) -> Result<String> {
    dirs::home_dir()
        .ok_or_else(|| Error::runtime("failed to get home directory"))
        .map(|p| p.display().to_string())
}

fn appdir(lua: &Lua, _: ()) -> Result<String> {
    let ctx = get_fs_context(lua);
    // appdir returns the server directory, which is the parent of remotes_dir
    // For now, we'll return the remote_dir's parent
    Ok(ctx
        .remote_dir
        .parent()
        .unwrap_or(&ctx.remote_dir)
        .display()
        .to_string())
}

fn special(_lua: &Lua, _csidl: String) -> Result<String> {
    // CSIDL folders are Windows-specific and not supported
    Err(Error::runtime(
        "special folders are not supported (Windows only)",
    ))
}

// Common functions

fn name(_lua: &Lua, path: String) -> Result<String> {
    let path = Path::new(&path);
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::runtime("failed to get file name"))?;
    Ok(file_stem.to_string())
}

fn fullname(_lua: &Lua, path: String) -> Result<String> {
    let path = Path::new(&path);
    Ok(path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string())
}

fn extension(_lua: &Lua, path: String) -> Result<String> {
    let path = Path::new(&path);
    Ok(path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string())
}

fn exists(_lua: &Lua, path: String) -> Result<bool> {
    Ok(Path::new(&path).exists())
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn copy(_lua: &Lua, (source, destination): (String, String)) -> Result<()> {
    let src = Path::new(&source);
    let dst = Path::new(&destination);

    if src.is_file() {
        fs::copy(&source, &destination)
            .map_err(|error| Error::runtime(format!("failed to copy file: {error}")))?;
    } else if src.is_dir() {
        copy_dir_all(src, dst)
            .map_err(|error| Error::runtime(format!("failed to copy directory: {error}")))?;
    } else {
        return Err(Error::runtime("source path does not exist"));
    }
    Ok(())
}

fn move_path(_lua: &Lua, (source, destination): (String, String)) -> Result<()> {
    fs::rename(&source, &destination).map_err(|error| {
        Error::runtime(format!(
            "failed to move '{source}' to '{destination}': {error}"
        ))
    })?;
    Ok(())
}

fn rename(lua: &Lua, (source, destination): (String, String)) -> Result<()> {
    // rename is the same as move in Rust's fs API
    move_path(lua, (source, destination))
}

fn delete(_lua: &Lua, (path, recursive): (String, Option<bool>)) -> Result<()> {
    let path = Path::new(&path);
    let recursive = recursive.unwrap_or(false);

    if path.is_file() {
        fs::remove_file(path).map_err(|error| {
            Error::runtime(format!(
                "failed to delete file '{}': {error}",
                path.display()
            ))
        })?;
    } else if path.is_dir() {
        if recursive {
            fs::remove_dir_all(path).map_err(|error| {
                Error::runtime(format!(
                    "failed to delete directory '{}': {error}",
                    path.display()
                ))
            })?;
        } else {
            fs::remove_dir(path).map_err(|error| {
                Error::runtime(format!(
                    "failed to delete directory '{}': {error}",
                    path.display()
                ))
            })?;
        }
    } else {
        return Err(Error::runtime(format!(
            "path '{}' does not exist",
            path.display()
        )));
    }

    Ok(())
}

// Path functions

fn parent(_lua: &Lua, path: String) -> Result<String> {
    let path = Path::new(&path);
    let parent = path
        .parent()
        .ok_or_else(|| Error::runtime("path has no parent"))?;
    Ok(parent.display().to_string())
}

fn expand(_lua: &Lua, path: String) -> Result<String> {
    // Expand environment variables and tilde
    let expanded = shellexpand::full(&path)
        .map_err(|error| Error::runtime(format!("failed to expand path: {error}")))?;
    Ok(expanded.to_string())
}

fn path(_lua: &Lua, str: String) -> Result<String> {
    // Normalize path separators for the current platform
    let path = PathBuf::from(str);
    Ok(path.display().to_string())
}

fn combine(_lua: &Lua, (a, b): (String, String)) -> Result<String> {
    let path_a = Path::new(&a);
    let combined = path_a.join(b);
    Ok(combined.display().to_string())
}

fn absolute(lua: &Lua, rel: String) -> Result<String> {
    let ctx = get_fs_context(lua);
    let rel_path = Path::new(&rel);

    if rel_path.is_absolute() {
        return Ok(rel);
    }

    let abs = ctx.remote_dir.join(rel);
    Ok(abs.display().to_string())
}

fn temp(_lua: &Lua, _: ()) -> Result<String> {
    let temp_file = tempfile::Builder::new()
        .prefix("uniremote_")
        .tempfile()
        .map_err(|error| Error::runtime(format!("failed to create temp file: {error}")))?;

    let path = temp_file.path().to_path_buf();
    // Drop the temp file so it doesn't get cleaned up automatically
    std::mem::forget(temp_file);

    Ok(path.display().to_string())
}

// Tree functions

fn roots(_lua: &Lua, _: ()) -> Result<Vec<String>> {
    Ok(vec!["/".to_string()])
}

fn files(_lua: &Lua, (path, hidden): (String, Option<bool>)) -> Result<Vec<String>> {
    let path = Path::new(&path);
    let show_hidden = hidden.unwrap_or(false);

    let mut files = Vec::new();

    let entries = fs::read_dir(path).map_err(|error| {
        Error::runtime(format!(
            "failed to read directory '{}': {error}",
            path.display()
        ))
    })?;

    for entry in entries {
        let entry = entry
            .map_err(|error| Error::runtime(format!("failed to read directory entry: {error}")))?;

        let entry_path = entry.path();

        if entry_path.is_file() {
            let file_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| Error::runtime("invalid file name"))?;

            // Skip hidden files if not requested
            if !show_hidden && file_name.starts_with('.') {
                continue;
            }

            files.push(entry_path.display().to_string());
        }
    }

    Ok(files)
}

fn dirs(_lua: &Lua, (path, hidden): (String, Option<bool>)) -> Result<Vec<String>> {
    let path = Path::new(&path);
    let show_hidden = hidden.unwrap_or(false);

    let mut dirs = Vec::new();

    let entries = fs::read_dir(path).map_err(|error| {
        Error::runtime(format!(
            "failed to read directory '{}': {error}",
            path.display()
        ))
    })?;

    for entry in entries {
        let entry = entry
            .map_err(|error| Error::runtime(format!("failed to read directory entry: {error}")))?;

        let entry_path = entry.path();

        if entry_path.is_dir() {
            let dir_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| Error::runtime("invalid directory name"))?;

            // Skip hidden directories if not requested
            if !show_hidden && dir_name.starts_with('.') {
                continue;
            }

            dirs.push(entry_path.display().to_string());
        }
    }

    Ok(dirs)
}

fn list(_lua: &Lua, (path, hidden): (String, Option<bool>)) -> Result<Vec<String>> {
    let path = Path::new(&path);
    let show_hidden = hidden.unwrap_or(false);

    let mut items = Vec::new();

    let entries = fs::read_dir(path).map_err(|error| {
        Error::runtime(format!(
            "failed to read directory '{}': {error}",
            path.display()
        ))
    })?;

    for entry in entries {
        let entry = entry
            .map_err(|error| Error::runtime(format!("failed to read directory entry: {error}")))?;

        let entry_path = entry.path();
        let item_name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::runtime("invalid entry name"))?;

        // Skip hidden items if not requested
        if !show_hidden && item_name.starts_with('.') {
            continue;
        }

        items.push(entry_path.display().to_string());
    }

    Ok(items)
}

// Create functions

fn createdir(_lua: &Lua, path: String) -> Result<()> {
    let path = Path::new(&path);
    fs::create_dir(path).map_err(|error| {
        Error::runtime(format!(
            "failed to create directory '{}': {error}",
            path.display()
        ))
    })
}

fn createdirs(_lua: &Lua, path: String) -> Result<()> {
    let path = Path::new(&path);
    fs::create_dir_all(path).map_err(|error| {
        Error::runtime(format!(
            "failed to create directories '{}': {error}",
            path.display()
        ))
    })
}

fn createfile(_lua: &Lua, path: String) -> Result<()> {
    let path = Path::new(&path);
    File::create(path).map_err(|error| {
        Error::runtime(format!(
            "failed to create file '{}': {error}",
            path.display()
        ))
    })?;
    Ok(())
}

// Read & Write functions

fn write(_lua: &Lua, (path, content): (String, String)) -> Result<()> {
    let path = Path::new(&path);
    fs::write(path, content).map_err(|error| {
        Error::runtime(format!(
            "failed to write to file '{}': {error}",
            path.display()
        ))
    })
}

fn writelines(lua: &Lua, (path, lines): (String, Value)) -> Result<()> {
    let path = Path::new(&path);

    // Convert Lua table to Vec<String>
    let lines_table: Table = lua.unpack(lines)?;
    let mut lines_vec = Vec::new();

    for i in 1..=lines_table.len()? {
        let line: String = lines_table.get(i)?;
        lines_vec.push(line);
    }

    let content = lines_vec.join("\n");
    fs::write(path, content).map_err(|error| {
        Error::runtime(format!(
            "failed to write lines to file '{}': {error}",
            path.display()
        ))
    })
}

fn append(_lua: &Lua, (path, content): (String, String)) -> Result<()> {
    let path = Path::new(&path);
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(|error| {
            Error::runtime(format!(
                "failed to open file '{}' for appending: {error}",
                path.display()
            ))
        })?;

    file.write_all(content.as_bytes()).map_err(|error| {
        Error::runtime(format!(
            "failed to append to file '{}': {error}",
            path.display()
        ))
    })
}

fn appendlines(lua: &Lua, (path, lines): (String, Value)) -> Result<()> {
    let path = Path::new(&path);

    // Convert Lua table to Vec<String>
    let lines_table: Table = lua.unpack(lines)?;
    let mut lines_vec = Vec::new();

    for i in 1..=lines_table.len()? {
        let line: String = lines_table.get(i)?;
        lines_vec.push(line);
    }

    let content = lines_vec.join("\n") + "\n";

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(|error| {
            Error::runtime(format!(
                "failed to open file '{}' for appending: {error}",
                path.display()
            ))
        })?;

    file.write_all(content.as_bytes()).map_err(|error| {
        Error::runtime(format!(
            "failed to append lines to file '{}': {error}",
            path.display()
        ))
    })
}

fn read(_lua: &Lua, path: String) -> Result<String> {
    let path = Path::new(&path);
    fs::read_to_string(path).map_err(|error| {
        Error::runtime(format!("failed to read file '{}': {error}", path.display()))
    })
}

fn readlines(lua: &Lua, path: String) -> Result<Table> {
    let path = Path::new(&path);
    let content = fs::read_to_string(path).map_err(|error| {
        Error::runtime(format!("failed to read file '{}': {error}", path.display()))
    })?;

    let lines = lua.create_table()?;
    for (i, line) in content.lines().enumerate() {
        lines.set(i + 1, line)?;
    }

    Ok(lines)
}

// Attribute functions

fn isfile(_lua: &Lua, path: String) -> Result<bool> {
    let path = Path::new(&path);
    Ok(path.is_file())
}

fn isdir(_lua: &Lua, path: String) -> Result<bool> {
    let path = Path::new(&path);
    Ok(path.is_dir())
}

fn ishidden(_lua: &Lua, path: String) -> Result<bool> {
    let path = Path::new(&path);

    #[cfg(unix)]
    {
        // On Unix, hidden files start with a dot
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            return Ok(file_name.starts_with('.'));
        }
        Ok(false)
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // On Windows, check the hidden attribute
        let metadata = path.metadata().map_err(|error| {
            Error::runtime(format!(
                "failed to get metadata for '{}': {error}",
                path.display()
            ))
        })?;

        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        Ok(metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(false)
    }
}

fn size(_lua: &Lua, path: String) -> Result<u64> {
    let path = Path::new(&path);
    let metadata = path.metadata().map_err(|error| {
        Error::runtime(format!(
            "failed to get metadata for '{}': {error}",
            path.display()
        ))
    })?;

    if metadata.is_dir() {
        // For directories, calculate total size of all files
        let mut total_size = 0;
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry
                .map_err(|error| Error::runtime(format!("failed to walk directory: {error}")))?;
            if entry.file_type().is_file() {
                total_size += entry
                    .metadata()
                    .map_err(|error| {
                        Error::runtime(format!("failed to get file metadata: {error}"))
                    })?
                    .len();
            }
        }
        Ok(total_size)
    } else {
        Ok(metadata.len())
    }
}

fn created(_lua: &Lua, path: String) -> Result<u64> {
    let path = Path::new(&path);
    let metadata = path.metadata().map_err(|error| {
        Error::runtime(format!(
            "failed to get metadata for '{}': {error}",
            path.display()
        ))
    })?;

    let created = metadata.created().map_err(|error| {
        Error::runtime(format!(
            "failed to get creation time for '{}': {error}",
            path.display()
        ))
    })?;

    let duration = created
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::runtime(format!("invalid creation time: {error}")))?;

    Ok(duration.as_secs())
}

fn modified(_lua: &Lua, path: String) -> Result<u64> {
    let path = Path::new(&path);
    let metadata = path.metadata().map_err(|error| {
        Error::runtime(format!(
            "failed to get metadata for '{}': {error}",
            path.display()
        ))
    })?;

    let modified = metadata.modified().map_err(|error| {
        Error::runtime(format!(
            "failed to get modification time for '{}': {error}",
            path.display()
        ))
    })?;

    let duration = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::runtime(format!("invalid modification time: {error}")))?;

    Ok(duration.as_secs())
}

pub fn load(lua: &Lua, libs: &Table) -> anyhow::Result<()> {
    let module = lua.create_table()?;

    // Context
    module.set("remotefile", lua.create_function(remotefile)?)?;
    module.set("remotedir", lua.create_function(remotedir)?)?;
    module.set("workingdir", lua.create_function(workingdir)?)?;

    // Directories
    module.set("homedir", lua.create_function(homedir)?)?;
    module.set("appdir", lua.create_function(appdir)?)?;
    module.set("special", lua.create_function(special)?)?;

    // Common
    module.set("name", lua.create_function(name)?)?;
    module.set("fullname", lua.create_function(fullname)?)?;
    module.set("extension", lua.create_function(extension)?)?;
    module.set("exists", lua.create_function(exists)?)?;
    module.set("copy", lua.create_function(copy)?)?;
    module.set("move", lua.create_function(move_path)?)?;
    module.set("rename", lua.create_function(rename)?)?;
    module.set("delete", lua.create_function(delete)?)?;

    // Path
    module.set("parent", lua.create_function(parent)?)?;
    module.set("expand", lua.create_function(expand)?)?;
    module.set("path", lua.create_function(path)?)?;
    module.set("combine", lua.create_function(combine)?)?;
    module.set("absolute", lua.create_function(absolute)?)?;
    module.set("temp", lua.create_function(temp)?)?;

    // Tree
    module.set("roots", lua.create_function(roots)?)?;
    module.set("files", lua.create_function(files)?)?;
    module.set("dirs", lua.create_function(dirs)?)?;
    module.set("list", lua.create_function(list)?)?;

    // Create
    module.set("createdir", lua.create_function(createdir)?)?;
    module.set("createdirs", lua.create_function(createdirs)?)?;
    module.set("createfile", lua.create_function(createfile)?)?;

    // Read & Write
    module.set("write", lua.create_function(write)?)?;
    module.set("writelines", lua.create_function(writelines)?)?;
    module.set("append", lua.create_function(append)?)?;
    module.set("appendlines", lua.create_function(appendlines)?)?;
    module.set("read", lua.create_function(read)?)?;
    module.set("readlines", lua.create_function(readlines)?)?;

    // Attributes
    module.set("isfile", lua.create_function(isfile)?)?;
    module.set("isdir", lua.create_function(isdir)?)?;
    module.set("ishidden", lua.create_function(ishidden)?)?;
    module.set("size", lua.create_function(size)?)?;
    module.set("created", lua.create_function(created)?)?;
    module.set("modified", lua.create_function(modified)?)?;

    libs.set("fs", &module)?;
    lua.register_module("fs", module)?;
    Ok(())
}

pub fn set_context(lua: &Lua, remote_file: PathBuf, remote_dir: PathBuf) {
    let context = FsContext {
        remote_file,
        remote_dir,
    };
    lua.set_app_data(context);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_name() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local fs = require("fs")
            result = fs.name("/path/to/file.txt")
        "#,
        )
        .exec()
        .unwrap();

        let result: String = lua.globals().get("result").unwrap();
        assert_eq!(result, "file");
    }

    #[test]
    fn test_fs_fullname() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local fs = require("fs")
            result = fs.fullname("/path/to/file.txt")
        "#,
        )
        .exec()
        .unwrap();

        let result: String = lua.globals().get("result").unwrap();
        assert_eq!(result, "file.txt");
    }

    #[test]
    fn test_fs_extension() {
        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();

        lua.load(
            r#"
            local fs = require("fs")
            result = fs.extension("/path/to/file.txt")
        "#,
        )
        .exec()
        .unwrap();

        let result: String = lua.globals().get("result").unwrap();
        assert_eq!(result, "txt");
    }

    #[test]
    fn test_fs_read_write() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let test_path = test_file.display().to_string();

        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();
        lua.globals().set("test_path", test_path.clone()).unwrap();

        // Test write
        lua.load(
            r#"
            local fs = require("fs")
            fs.write(test_path, "Hello, World!")
        "#,
        )
        .exec()
        .unwrap();

        // Test read
        lua.load(
            r#"
            local fs = require("fs")
            result = fs.read(test_path)
        "#,
        )
        .exec()
        .unwrap();

        let result: String = lua.globals().get("result").unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_fs_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test").unwrap();

        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();
        lua.globals()
            .set("existing_path", test_file.display().to_string())
            .unwrap();
        lua.globals()
            .set(
                "nonexistent_path",
                temp_dir
                    .path()
                    .join("nonexistent.txt")
                    .display()
                    .to_string(),
            )
            .unwrap();

        lua.load(
            r#"
            local fs = require("fs")
            exists_true = fs.exists(existing_path)
            exists_false = fs.exists(nonexistent_path)
        "#,
        )
        .exec()
        .unwrap();

        let exists_true: bool = lua.globals().get("exists_true").unwrap();
        let exists_false: bool = lua.globals().get("exists_false").unwrap();

        assert!(exists_true);
        assert!(!exists_false);
    }

    #[test]
    fn test_fs_isfile_isdir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test").unwrap();

        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();
        lua.globals()
            .set("file_path", test_file.display().to_string())
            .unwrap();
        lua.globals()
            .set("dir_path", temp_dir.path().display().to_string())
            .unwrap();

        lua.load(
            r#"
            local fs = require("fs")
            file_is_file = fs.isfile(file_path)
            file_is_dir = fs.isdir(file_path)
            dir_is_file = fs.isfile(dir_path)
            dir_is_dir = fs.isdir(dir_path)
        "#,
        )
        .exec()
        .unwrap();

        assert!(lua.globals().get::<bool>("file_is_file").unwrap());
        assert!(!lua.globals().get::<bool>("file_is_dir").unwrap());
        assert!(!lua.globals().get::<bool>("dir_is_file").unwrap());
        assert!(lua.globals().get::<bool>("dir_is_dir").unwrap());
    }

    #[test]
    fn test_fs_createdir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let new_dir = temp_dir.path().join("newdir");

        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();
        lua.globals()
            .set("new_dir_path", new_dir.display().to_string())
            .unwrap();

        lua.load(
            r#"
            local fs = require("fs")
            fs.createdir(new_dir_path)
        "#,
        )
        .exec()
        .unwrap();

        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }

    #[test]
    fn test_fs_readlines_writelines() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("lines.txt");

        let lua = Lua::new();
        let libs = lua.create_table().unwrap();

        load(&lua, &libs).unwrap();
        lua.globals().set("libs", libs).unwrap();
        lua.globals()
            .set("test_path", test_file.display().to_string())
            .unwrap();

        // Test writelines
        lua.load(
            r#"
            local fs = require("fs")
            fs.writelines(test_path, {"Line 1", "Line 2", "Line 3"})
        "#,
        )
        .exec()
        .unwrap();

        // Test readlines
        lua.load(
            r#"
            local fs = require("fs")
            lines = fs.readlines(test_path)
        "#,
        )
        .exec()
        .unwrap();

        let lines: Table = lua.globals().get("lines").unwrap();
        let line1: String = lines.get(1).unwrap();
        let line2: String = lines.get(2).unwrap();
        let line3: String = lines.get(3).unwrap();

        assert_eq!(line1, "Line 1");
        assert_eq!(line2, "Line 2");
        assert_eq!(line3, "Line 3");
    }
}
