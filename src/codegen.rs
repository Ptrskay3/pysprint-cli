use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Write;
use tempfile::Builder;
use tera::{Context, Tera};

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
    path: &str,
    text_options: &HashMap<String, String>,
    number_options: &HashMap<String, Box<f64>>,
    bool_options: &HashMap<String, Box<bool>>,
    before_evaluate_triggers: &Vec<String>,
    after_evaluate_triggers: &Vec<String>,
) -> Result<std::string::String, tera::Error> {
    let mut context = Context::new();
    // TODO: read these from the current config
    context.insert("methodname", "WFTMethod");
    context.insert("filename", &format!("{}/{}", path, file));
    context.insert("skiprows", &number_options["skiprows"]);
    context.insert("decimal", &text_options["decimal"]);
    context.insert("delimiter", &text_options["delimiter"]);
    context.insert("meta_len", &number_options["meta_len"]);
    context.insert("chdomain", &bool_options["chdomain"]); // TODO
    context.insert("detach", &false);
    context.insert("before_evaluate_triggers", &before_evaluate_triggers);
    context.insert("after_evaluate_triggers", &after_evaluate_triggers);
    // context.insert("slice_start", &number_options["slice_start"]);
    // context.insert("slice_stop", &number_options["slice_stop"]);

    // evaluate
    context.insert(
        "reference_frequency",
        &number_options["reference_frequency"],
    );
    context.insert("order", &number_options["order"]);
    // render to the tempfile
    TEMPLATES.render("boilerplate.py_t", &context)

    // TODO: change test to filename
    // write_tempfile(file, true, &result.unwrap());
}
