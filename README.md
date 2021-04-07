# PySprint-CLI

The command line tool for PySprint to evaluate interferograms immediately on record.

### Usage

First, set up an `eval.yaml` file where you will work. This defines the behaviour of the program. The full key and value list will be included soon. Here is an example:

```yml
load_options:
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
  - heatmap
  - windows: 200 # number of windows
  - fwhm: 0.05 # PHz

before_evaluate:
  - "print('this is a point where you can')"
  - "print('interact with the program')"

evaluate:
  - reference_frequency: 2.355 # PHz
  - order: 3 # up to TOD

after_evaluate:
  - "print('and also here, after evaluate..')"
```

To start watching a directory, run:

```shell
pysprint-cli watch your/path/here
```

To run an evaluation on an already existing filebase, run:

```shell
pysprint-cli audit your/path/here
```

Optionally generated files can be saved with the `--persist` (or `-p` for short) flag.

### TODO!

- TOP PRIORITY: refactor parser.rs, it is really messy

- method options [partially ok]
- detach [ok]
- automock imports
- termcolor --> color by severity [partially ok]
- implement audit [working on it]

- implement method switch [partially ok]
- custom build steps
- logging to a common result file [partially ok]
- sort files by mod 3
