# Built-In Countries Seed Sources

Seed snapshot generated on **2026-02-20** from:

1. Country metadata (ISO2/ISO3/name/capital/region/calling codes/timezones):
   - https://raw.githubusercontent.com/mledoze/countries/master/countries.json
2. ISO 4217 minor units (currency decimal precision):
   - https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml

Normalization used:

- `calling_code` = `calling_root + suffix` when exactly one suffix exists.
- `calling_code` = `calling_root` when multiple suffixes exist (for shared plans like `+1`).
- Full suffix list is preserved in `calling_suffixes`.
- App-level `status` is not from upstream: seeding defaults to `enabled` for `MY`, and `disabled` for all other ISO2 codes.
