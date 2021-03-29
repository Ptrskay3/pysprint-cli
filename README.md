# PySprint-CLI

The command line tool for PySprint to evaluate interferograms immediately on record.

### Usage

First, set up an `eval.yaml` file where you will work. This defines the behaviour of the program. The full key and value list will be included soon. Here is an example:

```yml
load_options:
  - skiprows: 8
  - decimal: ","
  - delimiter: ";"
  - meta_len: 6

preprocess:
  - chdomain: true
  - slice_start: 2

method:
  - fft

before_evaluate:
  - "print(ifg)"

evaluate:
  - reference_frequency: 2.355
  - order: 3

after_evaluate:
  - "print('we are done with this..')"
```

To start watching a directory, run:

```shell
pysprint-cli --path your/path/here
```

Optionally generated files can be saved with the `--persist` flag.

### TODO!

- implement method switch
- custom build steps
- logging to a common result file
- sort files by mod 3
