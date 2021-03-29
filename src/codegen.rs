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

    // numerics
    for (key, entry) in number_options {
        context.insert(key, &entry);
    }

    // Specials
    context.insert("methodname", "WFTMethod");
    context.insert("filename", &format!("{}/{}", path, file));
    context.insert("detach", &false);

    // textual
    context.insert("decimal", &text_options["decimal"]);
    context.insert("delimiter", &text_options["delimiter"]);

    // boolean
    context.insert("chdomain", &bool_options["chdomain"]);
    
    // other
    context.insert("before_evaluate_triggers", &before_evaluate_triggers);
    context.insert("after_evaluate_triggers", &after_evaluate_triggers);

    // render to the tempfile
    TEMPLATES.render("boilerplate.py_t", &context)

    // TODO: change test to filename
    // write_tempfile(file, true, &result.unwrap());
}
