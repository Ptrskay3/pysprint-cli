use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use tempfile::Builder;
use tera::{Context, Tera};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        let _ = tera.add_raw_template(
            "pstemplate.py_t",
            r#"ifg = ps.{{ methodname }}.parse_raw(
    "{{ filename }}",
{%- if filename2 %} "{{ filename2 }}", {% endif %}
{%- if filename3 %} "{{ filename3 }}", {% endif %}
{% if skiprows %} skiprows={{ skiprows }}, {%- else %} 0, {% endif %}
{% if decimal %} decimal="{{ decimal }}", {%- else %} ".", {% endif %}
{% if delimiter %} delimiter="{{ delimiter }}", {%- else %} ",", {% endif %}
{% if meta_len %} meta_len={{ meta_len }} {%- else %} "0" {% endif %}
)

SKIP_IF = ("ref", "sam", "reference", "sample", "noeval")

for entry in SKIP_IF:
    if entry in ifg.meta['comment']:
        import sys
        sys.exit("file skipped due to user comment")

{% if chdomain -%} ifg.chdomain() {%- endif %}

{% if slice_start and slice_stop -%} ifg.slice({{ slice_start }}, {{ slice_stop }}){%- endif %}
{%- if slice_start and not slice_stop -%} ifg.slice(start={{ slice_start }}){%- endif %}
{%- if not slice_start and slice_stop -%} ifg.slice(stop={{ slice_stop }}){%- endif %}

x_before_transform = np.copy(ifg.x)
y_before_transform = np.copy(ifg.y_norm)

{%if detach %}
with ps.interactive("TkAgg"):
    ifg.plot()
    ifg.show()
{% endif %}


{% for cmd in before_evaluate_triggers %}
{{ cmd }}
{% endfor %}

{% if methodname == "FFTMethod" %}
ifg.autorun({{ reference_frequency }}, {{ order }}, show_graph=False, enable_printing=False)
{% elif methodname == "WFTMethod" %}
ifg.cover(200, fwhm=0.05)
ifg.calculate({{ reference_frequency }}, {{ order }}, parallel=False, fastmath=False)
{% else %}
print("not implemented yet")
{% endif %}

fragment = ps.utils._prepare_json_fragment(ifg, "{{ filename_raw }}", x_before_transform, y_before_transform)
ps.utils._write_or_update_json_fragment("{{ workdir }}/{{ result_file }}", fragment, "{{ filename_raw }}")

{% for cmd in after_evaluate_triggers %}
{{ cmd }}
{% endfor %}"#,
        );
        tera
    };
}

pub fn write_tempfile(name: &str, content: &str, path: &str) -> std::io::Result<()> {
    let tempfile = Builder::new().tempfile_in(path)?;

    let mut _file = tempfile.persist(format!("{}/{}_pstemp.py", path, name))?;
    writeln!(_file, "{}", content)?;

    Ok(())
}

pub fn render_template(
    file: &str,
    path: &str,
    text_options: &HashMap<String, String>,
    number_options: &HashMap<String, Box<f64>>,
    bool_options: &HashMap<String, Box<bool>>,
    before_evaluate_triggers: &[String],
    after_evaluate_triggers: &[String],
    result_file: &str,
) -> Result<std::string::String, tera::Error> {
    let mut context = Context::new();

    for (key, entry) in number_options {
        context.insert(key, &entry);
    }

    for (key, entry) in text_options {
        context.insert(key, &entry);
    }

    for (key, entry) in bool_options {
        context.insert(key, &entry);
    }

    // Specials
    context.insert("result_file", result_file);
    context.insert("filename_raw", &file);
    context.insert("workdir", &path);
    context.insert("filename", &format!("{}/{}", path, file));
    context.insert("detach", &false);

    // other
    context.insert("before_evaluate_triggers", &before_evaluate_triggers);
    context.insert("after_evaluate_triggers", &after_evaluate_triggers);

    // render as String
    TEMPLATES.render("pstemplate.py_t", &context)
}

fn write_default_yaml(path: &str) -> std::io::Result<()> {
    let cfg_path = PathBuf::from(path).join("eval.yaml");
    std::fs::write(
        cfg_path,
        r#"load_options:
  - skiprows: 8
  - decimal: ","
  - delimiter: ";"
  - meta_len: 6

preprocess:
  - input_unit: "nm"
  - chdomain: true
  - slice_start: 2
  - slice_stop: 4

method:
  - fft

before_evaluate:
  - "print('you can interact with the program through this hook')"

evaluate:
  - reference_frequency: 2.355
  - order: 3

after_evaluate:
  - "print('and also here at this point')"
"#
        .as_bytes(),
    );
    Ok(())
}

pub fn default_yaml_if_needed(path: &str) {
    println!(
        "[INFO] No `eval.yaml` file was detected in the target path. 
       If you named it something different, use the `-c` option.
       Type 'y' or 'yes' if you want to generate a default one, or anything else to quit."
    );
    loop {
        let mut input_text = String::new();
        io::stdin()
            .read_line(&mut input_text)
            .expect("failed to read from stdin");

        match input_text.to_lowercase().trim() {
            "yes" | "y" => {
                let r = write_default_yaml(path);
                println!("[INFO] Created `eval.yaml` config file.");
                break;
            }
            _ => {
                break;
            }
        };
    }
}
