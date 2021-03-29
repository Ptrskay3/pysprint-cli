import pysprint as ps

ifg = ps.FFTMethod.parse_raw(
    "a", "b",  "c", 
    skiprows=8,
    decimal=",",
    delimiter=";",
    meta_len=6
)

SKIP_IF = ("ref", "sam", "reference", "sample")

for entry in SKIP_IF:
    if entry in ifg.meta:
        import sys
        sys.exit(1)
