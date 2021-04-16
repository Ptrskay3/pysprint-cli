use crate::parser::{EvaluateOptions, IntermediateHooks};
use lazy_static::lazy_static;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use tempfile::Builder;
use tera::{Context, Tera};

const IMPORT_HEADERS: &str = r#"import numpy as np
import pysprint as ps
import matplotlib.pyplot as plt

"#;

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

{% if not no_comment_check %}
SKIP_IF = ("ref", "sam", "reference", "sample", "noeval")

for entry in SKIP_IF:
    try:
        if entry in ifg.meta['comment']:
            import sys
            sys.exit(f"file skipped due to user comment contains `{entry}`.")
    except KeyError:
        pass
{% endif %}
{% if input_unit != "nm" %}
{% if chdomain %}
ifg.chrange("{{ input_unit }}", "nm")
{% else %}
ifg.chrange("{{ input_unit }}", "phz")
{% endif %}
{% endif %}
{% if chdomain -%} ifg.chdomain() {%- endif %}

{% if slice_start and slice_stop -%} ifg.slice({{ slice_start }}, {{ slice_stop }}){%- endif %}
{%- if slice_start and not slice_stop -%} ifg.slice(start={{ slice_start }}){%- endif %}
{%- if not slice_start and slice_stop -%} ifg.slice(stop={{ slice_stop }}){%- endif %}

x_before_transform = np.copy(ifg.x)
y_before_transform = np.copy(ifg.y_norm)

{%if plot %}
ifg.plot()
plt.show(block=True)
{% endif %}


{% for cmd in before_evaluate_triggers %}
{{ cmd }}
{% endfor %}

{% if methodname == "FFTMethod" %}
import warnings
warnings.simplefilter("ignore")
ifg.autorun({{ reference_frequency }}, {{ order }}, show_graph=False, enable_printing={% if is_audit %}False{% else %}True{% endif %})
{% elif methodname == "CosFitMethod" %}
{% if not is_audit %}
import sys
sys.exit("CosFit is not supported in watch mode")
{% endif %}
ifg.GD_lookup({{reference_frequency}}, silent=True)
ifg._optimizer({{reference_frequency}}, {{ order }}, initial_region_ratio=0.05, extend_by=0.05, show_endpoint=False, nofigure=True)
{% elif methodname == "WFTMethod" %}
ifg.cover(
    {% if windows %}{{ windows }}{% else %}300{% endif %},
    {% if fwhm and not std %}fwhm={{ fwhm }},{% endif %}
    {% if std and not fwhm %}std={{ std }},{% endif %}
    {%if not std and not fwhm %}fwhm=0.05{% endif %}
)

ifg.{%- if is_audit -%}_{%- endif -%}calculate({{ reference_frequency }}, {{ order }}, silent={%- if is_audit -%}True{%- else -%}False{%- endif -%}, parallel={% if parallel %}True{% else %}False{% endif %}, fastmath=False)
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

# if you are working with the generated file, the part below can be safely commented out

fragment = ps.utils._prepare_json_fragment(ifg, "{{ filename_raw }}", x_before_transform, y_before_transform, verbosity={{ verbosity }})
ps.utils._write_or_update_json_fragment("{{ workdir }}/{{ result_file }}", fragment, "{{ filename_raw }}")

{% for cmd in after_evaluate_triggers %}
{{ cmd }}
{% endfor %}"#,
        );
        let _ = tera.add_raw_template(
            "spp.py_t",
            r#"
ifg_files = [
    {% for file in ifg_files %}
    r"{{ file }}",
    {% endfor %}
    ]
sam_files = [
    {% for file in sam_files %}
    r"{{ file }}",
    {% endfor %}
]

ref_files = [
    {% for file in ref_files %}
    r"{{ file }}",
    {% endfor %}
]

myspp = ps.SPPMethod(ifg_files, sam_files, ref_files, {% if skiprows %} skiprows={{ skiprows }}, {%- else %}skiprows=0, {% endif %}
{% if decimal %}decimal="{{ decimal }}", {%- else %} ".", {% endif %}
{% if delimiter %}delimiter="{{ delimiter }}", {%- else %} ",", {% endif %}
{% if meta_len %}meta_len={{ meta_len }} {%- else %} meta_len=0 {% endif %}, 
{% if eager %}callback=ps.eager_executor(reference_point={{ reference_frequency }}, order={{ order }}, logfile="spp.log", verbosity=1){% endif %})

{% for cmd in before_evaluate_triggers %}
{{ cmd }}
{% endfor %}

{% if detach %}
for ifg in myspp:
    {% if chdomain -%}ifg.chdomain(){% endif %}
    ifg.open_SPP_panel(header="comment")
{% endif %}

myspp.calculate({{ reference_frequency }}, {{ order }}, show_graph=False)

{% for cmd in after_evaluate_triggers %}
{{ cmd }}
{% endfor %}
    "#,
        );
        tera
    };
}

pub fn write_tempfile_with_imports(name: &str, content: &str, path: &str) -> std::io::Result<()> {
    // we also write the import headers to the generated file
    let mut accumulator = IMPORT_HEADERS.to_owned();
    accumulator.push_str(content);

    let tempfile = Builder::new().tempfile_in(path)?;

    let mut _file = tempfile.persist(format!("{}/{}_ps.py", path, name))?;
    writeln!(_file, "{}", accumulator)?;

    Ok(())
}

pub fn render_spp_template(
    ifg_files: &[PathBuf],
    ref_files: &[PathBuf],
    sam_files: &[PathBuf],
    path: &str,
    evaluate_options: &EvaluateOptions,
    intermediate_hooks: &IntermediateHooks,
    result_file: &str,
    verbosity: u8,
    is_audit: bool,
) -> Result<std::string::String, tera::Error> {
    let mut context = Context::new();

    for (key, entry) in &evaluate_options.number_options {
        context.insert(key, &entry);
    }

    for (key, entry) in &evaluate_options.text_options {
        context.insert(key, &entry);
    }

    for (key, entry) in &evaluate_options.bool_options {
        context.insert(key, &entry);
    }

    let ifgs = ifg_files
        .iter()
        .filter_map(|p| p.to_str())
        .collect::<Vec<&str>>();

    let refs = ref_files
        .iter()
        .filter_map(|p| p.to_str())
        .collect::<Vec<&str>>();

    let sams = sam_files
        .iter()
        .filter_map(|p| p.to_str())
        .collect::<Vec<&str>>();

    // Specials
    context.insert("sam_files", &sams);
    context.insert("ref_files", &refs);
    context.insert("ifg_files", &ifgs);
    context.insert("verbosity", &verbosity);
    context.insert("result_file", result_file);
    context.insert("workdir", &path);
    context.insert("is_audit", &is_audit);

    // other
    context.insert(
        "before_evaluate_triggers",
        &intermediate_hooks.before_evaluate_triggers,
    );
    context.insert(
        "after_evaluate_triggers",
        &intermediate_hooks.after_evaluate_triggers,
    );

    // render as String
    TEMPLATES.render("spp.py_t", &context)
}

pub fn render_generic_template(
    file: &str,
    path: &str,
    evaluate_options: &EvaluateOptions,
    intermediate_hooks: &IntermediateHooks,
    result_file: &str,
    verbosity: u8,
    is_audit: bool,
    sam_arm: Option<&PathBuf>,
    ref_arm: Option<&PathBuf>,
) -> Result<std::string::String, tera::Error> {
    let mut context = Context::new();

    if let Some(arm) = sam_arm {
        let f2 = arm.as_path().file_name().unwrap().to_str().unwrap_or("");
        context.insert("filename2", &format!("{}/{}", path, f2));
    }

    if let Some(arm) = ref_arm {
        let f3 = arm.as_path().file_name().unwrap().to_str().unwrap_or("");
        context.insert("filename3", &format!("{}/{}", path, f3));
    }

    for (key, entry) in &evaluate_options.number_options {
        context.insert(key, &entry);
    }

    for (key, entry) in &evaluate_options.text_options {
        context.insert(key, &entry);
    }

    for (key, entry) in &evaluate_options.bool_options {
        context.insert(key, &entry);
    }
    // Specials
    context.insert("verbosity", &verbosity);
    context.insert("result_file", result_file);
    context.insert("filename_raw", &file);
    context.insert("workdir", &path);
    context.insert("is_audit", &is_audit);

    // FIXME: this is redundant
    context.insert("filename", &format!("{}/{}", path, file));

    // other
    context.insert(
        "before_evaluate_triggers",
        &intermediate_hooks.before_evaluate_triggers,
    );
    context.insert(
        "after_evaluate_triggers",
        &intermediate_hooks.after_evaluate_triggers,
    );

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
#      - "txt"
#  - exclude_patterns:
#      - "*_randomfile.trt"
#      - "*_to_skip.trt"
#  - skip:
#      - "filename.trt"
#      - "file_to_skip.trt"
  - skiprows: 8 # lines
  - decimal: ","
  - delimiter: ";"
  - meta_len: 6 # lines
  # - mod: 1 # | 3 | -1
  # note that this is only available when using audit, and it only
  # has effect when the method is "cff", "mm" or "spp", the other methods
  # auto-skip files..
  # - no_comment_check: true
  # By default an evaluation will be skipped if the user comment contains one of
  # `noeval`, `sam`, `sample`, `ref` or `reference`.
  # This option turns that metadata checking off.

preprocess:
  - chdomain: true
#  - input_unit: "nm"
#  - slice_start: 2 # PHz
#  - slice_stop: 4 # PHz

method:
  - wft # | fft | mm | cff | spp

method_details:
  # globally available options
  # - only_phase # TODO field
  # - plot

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
  # no specific options availabe

  # options for -- CosFitMethod --
  # no specific options availabe
  # only available in audit

  # options for -- SPPMethod --
  # - eager
  # - detach
  # only available in audit

# before_evaluate:
  # - "print('before_evaluate')"
  # - "print('you have access to the `ifg` variable')"
  # - "print(f'and this is it now: {ifg}')"

evaluate:
  - reference_frequency: 2.355 # PHz
  - order: 3 # up to TOD

# after_evaluate:
  # - "print('and after evaluate too..')"

"#
        .as_bytes(),
    )?;
    Ok(())
}

pub fn maybe_write_default_yaml(path: &str) {
    println!(
        "[INFO] There is no config file detected in the target path.
       Type 'y' or 'yes' if you want to generate a default one, or anything else to quit."
    );

    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("failed to read from stdin");

    match input_text.to_lowercase().trim() {
        "yes" | "y" => {
            let _r = write_default_yaml(path);
            println!("[INFO] Created `eval.yaml` config file.");
        }
        _ => {}
    };
}
