use tempfile::{Builder, NamedTempFile};
use std::io::Write;


pub fn write_tempfile(name: &str, persist: bool) -> std::io::Result<()> {

    let tempfile = Builder::new().tempfile_in("./example")?;

    match persist {
        true => {
            let mut _file = tempfile.persist(format!("./example/{}_pstemp.py", name))?;
                writeln!(_file,
        "import pysprint as ps
ps.print_info()")?;
        },
        _ => {
            let mut _file = tempfile;
                writeln!(_file,
        "import pysprint as ps
ps.print_info()")?;
        }
    };
    

    Ok(())
}