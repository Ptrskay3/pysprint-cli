use std::io::Write;
use tempfile::{Builder, NamedTempFile};

pub fn write_tempfile(name: &str, persist: bool, content: &str) -> std::io::Result<()> {
    let mut tempfile = Builder::new().tempfile_in("./example")?;
    if persist {
        let mut _file = tempfile.persist(format!("./example/{}_pstemp.py", name))?;
        writeln!(
            _file,
            "{}", content
        )?;
    } else {
        writeln!(
            tempfile,
            "{}", content
        )?;
    }

    Ok(())
}
