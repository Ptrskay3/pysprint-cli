use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Write;
use tempfile::Builder;
use tera::{Context, Tera};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template(
            "boilerplate.py_t",
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
ifg.calculate({{ reference_frequency }}, {{ order }}, parallel=True, fastmath=False)
{% else %}
print("not implemented yet")
{% endif %}

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
    context.insert("methodname", "FFTMethod");
    context.insert("filename", &format!("{}/{}", path, file));
    context.insert("detach", &false);

    // other
    context.insert("before_evaluate_triggers", &before_evaluate_triggers);
    context.insert("after_evaluate_triggers", &after_evaluate_triggers);

    // render as String
    TEMPLATES.render("boilerplate.py_t", &context)
}
