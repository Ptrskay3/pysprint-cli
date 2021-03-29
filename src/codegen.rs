use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Write;
use tempfile::{Builder, NamedTempFile};
use tera::{Context, Tera};
use std::path::Path;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("src/templates/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}


pub fn write_tempfile(name: &str, persist: bool, content: &str) -> std::io::Result<()> {
    let mut tempfile = Builder::new().tempfile_in("./example")?;
    if persist {
        let mut _file = tempfile.persist(format!("./example/{}_pstemp.py", name))?;
        writeln!(_file, "{}", content)?;
    } else {
        writeln!(tempfile, "{}", content)?;
    }

    Ok(())
}

pub fn render_template(
    file: &str,
    text_options: &HashMap<String, String>,
    number_options: &HashMap<String, Box<f64>>,
) -> Result<std::string::String, tera::Error> {
    let mut context = Context::new();
    // TODO: read these from the current config
    context.insert("methodname", "FFTMethod");
    context.insert("filename", &format!("{}/{}", "./example", file));
    context.insert("skiprows", &number_options["skiprows"]);
    context.insert("decimal", &text_options["decimal"]);
    context.insert("delimiter", &text_options["delimiter"]);
    context.insert("meta_len", &number_options["meta_len"]);
    context.insert("chdomain", &true);
    // context.insert("slice_start", &2.0);
    // context.insert("slice_stop", &4.0);
    // render to the tempfile
    let result = TEMPLATES.render("boilerplate.py_t", &context);

    // TODO: change test to filename
    // write_tempfile(file, true, &result.unwrap());
    result
}
