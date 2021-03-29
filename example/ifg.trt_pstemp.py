ifg = ps.FFTMethod.parse_raw(
    "./example/ifg.trt",
    skiprows=8,
    decimal=",",
    delimiter=";",
    meta_len=6
)

SKIP_IF = ("ref", "sam", "reference", "sample")

for entry in SKIP_IF:
    if entry in ifg.meta:
        import sys
        print("szar")
        sys.exit(1)

ifg.chdomain() ifg.slice(start=2)
print(ifg)

