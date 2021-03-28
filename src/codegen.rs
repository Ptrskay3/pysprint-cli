use tempfile::{Builder, NamedTempFile};
use std::io::Write;


pub fn write_tempfile(name: &str, persist: bool) -> std::io::Result<()> {

    let mut tempfile = Builder::new().tempfile_in("./example")?;
    if persist {
        let mut _file = tempfile.persist(format!("./example/{}_pstemp.py", name))?;
             writeln!(_file, "import pysprint as ps
ps.print_info()")?;
    } else {
        writeln!(tempfile, "import pysprint as ps
ps.print_info()")?;
    }


    Ok(())
}