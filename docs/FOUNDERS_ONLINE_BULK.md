# Founders Online bulk import

Founders Online publishes metadata for the full corpus and a document-content
API. This repository keeps the bulk import separate from the curated manifest:

- metadata: `data/founders_online/metadata/`
- raw API JSON: `data/founders_online/raw/`
- cleaned text: `data/founders_online/clean/`
- chunk corpus: `data/founders_online/chunks/founders_online_corpus.json`
- fetch reports: `data/founders_online/reports/`

The official API documentation asks automated clients to use a reasonable
delay, with a maximum of 10 requests per second. The importer defaults to
2 requests per second and refuses values above 10.

## Commands

Download and normalize the metadata:

```bash
python3 scripts/fetch_founders_online.py metadata
```

Fetch a small smoke-test batch:

```bash
python3 scripts/fetch_founders_online.py fetch --limit 25 --requests-per-second 2
python3 scripts/fetch_founders_online.py chunk --limit 25
```

Fetch all primary dated documents and build the chunk corpus:

```bash
python3 scripts/fetch_founders_online.py all --requests-per-second 2
```

Resume a stopped run:

```bash
python3 scripts/fetch_founders_online.py fetch --requests-per-second 2
```

Existing raw JSON and clean text files are reused unless `--force` is passed.
Use `--offset` and `--limit` to divide the corpus into batches.

## If command-line downloads are challenged

Some networks receive a CloudFront WAF `202` challenge from
`founders.archives.gov` for command-line requests. When that happens, download
this file in a browser:

```text
https://founders.archives.gov/Metadata/founders-online-metadata.json
```

Then pass the local path:

```bash
python3 scripts/fetch_founders_online.py fetch \
  --metadata-file /path/to/founders-online-metadata.json \
  --requests-per-second 2
```

If API document requests are also challenged, copy the `aws-waf-token` cookie
from a browser request into a local file outside the repo:

```bash
printf '%s\n' 'aws-waf-token=...' > /private/tmp/founders_cookie.txt
```

Then include it with the fetch:

```bash
python3 scripts/fetch_founders_online.py fetch \
  --metadata-file /path/to/founders-online-metadata.json \
  --cookie-file /private/tmp/founders_cookie.txt \
  --user-agent 'Mozilla/5.0 ...' \
  --requests-per-second 2
```

The WAF token is a browser-session credential. Keep it out of committed files,
and refresh it from the browser if it expires during a long run.

## Licensing note

Founders Online is an official National Archives/NHPRC and UVA Press project.
The National Archives dataset page describes the metadata release as
non-commercial with attribution. Confirm the current Founders Online terms
before redistributing a bulk copy of metadata or transcriptions.
