use lazy_static::lazy_static;
use std::collections::HashMap;
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
{% if skiprows %} skiprows={{ skiprows }}, {%- else %} skiprows=0, {% endif %}
{% if decimal %} decimal="{{ decimal }}", {%- else %} ".", {% endif %}
{% if delimiter %} delimiter="{{ delimiter }}", {%- else %} ",", {% endif %}
{% if meta_len %} meta_len={{ meta_len }} {%- else %} meta_len=0 {% endif %}
)

SKIP_IF = ("ref", "sam", "reference", "sample", "noeval")

for entry in SKIP_IF:
    try:
        if entry in ifg.meta['comment']:
            import sys
            sys.exit("file skipped due to user comment")
    except KeyError:
        pass

{% if chdomain -%} ifg.chdomain() {%- endif %}

{% if slice_start and slice_stop -%} ifg.slice({{ slice_start }}, {{ slice_stop }}){%- endif %}
{%- if slice_start and not slice_stop -%} ifg.slice(start={{ slice_start }}){%- endif %}
{%- if not slice_start and slice_stop -%} ifg.slice(stop={{ slice_stop }}){%- endif %}

x_before_transform = np.copy(ifg.x)
y_before_transform = np.copy(ifg.y_norm)

{%if detach %}
ifg.plot()
plt.show(block=True)
{% endif %}


{% for cmd in before_evaluate_triggers %}
{{ cmd }}
{% endfor %}

{% if methodname == "FFTMethod" %}
ifg.autorun({{ reference_frequency }}, {{ order }}, show_graph=False, enable_printing=False)
{% elif methodname == "WFTMethod" %}
ifg.cover(
    {% if windows %}{{ windows }}{% else %}300{% endif %},
    {% if fwhm and not std %}fwhm={{ fwhm }},{% endif %}
    {% if std and not fwhm %}std={{ std }},{% endif %}
    {%if not std and not fwhm %}fwhm=0.05{% endif %}
)

ifg.calculate({{ reference_frequency }}, {{ order }}, parallel={% if parallel %}True{% else %}False{% endif %}, fastmath=False)
{% elif methodname == "MinMaxMethod" %}
ifg.init_edit_session(
    {% if min and max %}
    side="both"
    {% elif min %}
    side="min"
    {% elif max %}
    side="max"
    {% else %}
    side="both"
    {% endif %}
)
plt.show(block=True)
ifg.calculate({{ reference_frequency }}, {{ order }}, scan=True,
    {% if min and max %}
    onesided=False
    {% elif min %}
    onesided=True
    {% elif max %}
    onesided=True
    {% else %}
    onesided=False
{% endif %})
{% else %}
print("{{ methodname }} is not yet implemented..")
{% endif %}

{% if heatmap and methodname == "WFTMethod" %}
ifg.heatmap()
plt.show(block=True)
{% endif %}

fragment = ps.utils._prepare_json_fragment(ifg, "{{ filename_raw }}", x_before_transform, y_before_transform, verbosity={{ verbosity }})
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
    verbosity: u8,
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
    context.insert("verbosity", &verbosity);
    context.insert("result_file", result_file);
    context.insert("filename_raw", &file);
    context.insert("workdir", &path);
    context.insert("filename", &format!("{}/{}", path, file));

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
  - extensions:
      - "trt"
      - "txt"
  - exclude_patterns:
      - "*_randomfile.trt"
      - "*_to_skip.trt"
  - skip:
      - "filename.trt"
      - "file_to_skip.trt"
  - skiprows: 8 # lines
  - decimal: ","
  - delimiter: ";"
  - meta_len: 6 # lines

preprocess:
  - input_unit: "nm"
  - chdomain: true
  - slice_start: 2 # PHz
  - slice_stop: 4 # PHz

method:
  - wft

method_details:
  # globally available options
  # - auto
  # - only_phase
  # - detach

  # options for -- MinMaxMethod --
  # - min
  # - max
  # - both

  # options for -- WFTMethod --
  # - heatmap
  # - windows: 200
  # - fwhm: 0.05 # PHz
  # - std: 0.05 # PHz
  # - parallel

  # options for -- FFTMethod --
  # there is no option currently available

  # options for -- CosFitMethod --
  # not implemented yet

  # options for -- SPPMethod --
  # not implemented yet

before_evaluate:
  - "print('before_evaluate')"

evaluate:
  - reference_frequency: 2.355 # PHz
  - order: 3 # up to TOD

after_evaluate:
  - "print('and after..')"

"#
        .as_bytes(),
    );
    Ok(())
}

pub fn maybe_write_default_yaml(path: &str) {
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
                let _r = write_default_yaml(path);
                println!("[INFO] Created `eval.yaml` config file.");
                break;
            }
            _ => {
                break;
            }
        };
    }
}
