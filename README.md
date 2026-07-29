# win-containers-registry

Registry of windows service builds. CI packs a windows service into a zip and uploads it here
under a container name and a tag; windows machines download it back by the same name and tag.

Storage is content-addressable: the sha256 of the uploaded content becomes the file name, and a tag
is just a pointer to a hash. Uploading the same build twice occupies one file.

## Storage layout

```text
{containers_path}/
  mt4-bridge/
    container-info.yaml                 <- all metadata of the container (no database)
    b1ac8873ed85....zip                 <- the blobs, named by sha256 of their content
    1da238d07a0b....zip
  mt5-bridge/
    container-info.yaml
    ...
```

`container-info.yaml`:

```yaml
container: mt4-bridge
tags:
  0.1.0:
    hash: b1ac8873ed85297ea546f9439d736c95b56934f237f8afb3b71d7a1c427a6802
    size: 120
    uploaded_at: 2026-07-26T11:54:38.807570+00:00
    uploaded_by: 10.0.0.5
  latest:
    hash: 1da238d07a0b58b35e6555ba90cbcaf1ef53570cbb7285a04a68af154d944ea2
    size: 120
    uploaded_at: 2026-07-26T11:56:16.557232+00:00
    uploaded_by: 10.0.0.5
```

Everything that touches one container runs under that container's own `tokio::Mutex`, so writing the
zip and updating `container-info.yaml` is a single atomic step, and two parallel uploads of the same
container can never lose each other's tag. Different containers do not block each other.

## Settings

`~/.win-containers-registry`:

```yaml
ContainersPath: ~/containers-storage
ApiKey: <api-key-value>
```

`ApiKey` goes into the `X-API-Key` header and is required by two groups of endpoints:

- **writes** — upload / delete;
- **browsing** — the containers list and the tags of a container. Enumerating what the registry
  holds is not something an anonymous caller gets.

`download` and `hash` stay open: a target machine pulls a build it already knows the name of without
carrying a key.

`ApiKey` is optional — when it is not in the settings file, everything is open (same convention as
my-files-storage).

Settings are read once at startup — editing the file needs a restart.

## Run

This is a standalone infrastructure service: no service-sdk, no Postgres, no MyNoSql, no Service Bus.
It is `my-http-server` + `my-logger` + the storage folder, and it logs to the console.

```bash
cargo run          # HTTP on 0.0.0.0:8000
```

## API

Swagger UI: `http://<host>:8000/swagger/index.html`

A container tag is addressed as a single `{container_name}:{tag}` string — `mt4-bridge:0.1.0`.
The split is on the **last** `:`, so everything after the final colon is the tag.

| Method | Route | Auth | Description |
|---|---|---|---|
| POST | `/api/containers/v1/upload/{container}` | `X-API-Key` | Body is the zip itself. Hash is calculated, blob is stored, tag points at it. |
| GET | `/api/containers/v1/download/{container}` | — | Resolves tag into a hash, returns the zip as `{container_name}-{tag}.zip` |
| GET | `/api/containers/v1/hash/{container}` | — | Hash of one tag, without downloading the zip |
| GET | `/api/containers/v1/list` | `X-API-Key` | All containers with their tag counts |
| GET | `/api/containers/v1/tags/{containerName}` | `X-API-Key` | Every tag of a container (name only, no tag) with the hash it points at, plus size / uploaded_at / uploaded_by |
| DELETE | `/api/containers/v1/tag/{container}` | `X-API-Key` | Removes the tag, and the blob if no other tag references it |
| GET | `/api/system/v1/ping` | — | Liveness probe |

Container names and tags accept `a-z A-Z 0-9 . _ -` only, up to 128 chars, and must not start with
`.` or `-` — they become file-system paths. A `{container}` without a colon is a 400: there is no
default tag.

### Upload from CI

```bash
curl -sf -X POST "https://<host>/api/containers/v1/upload/mt4-bridge:0.1.0" \
     -H "X-API-Key: $WIN_CONTAINERS_API_KEY" \
     --data-binary @build/output.zip
```

```json
{
  "container": "mt4-bridge",
  "tag": "0.1.0",
  "hash": "b1ac8873ed85297ea546f9439d736c95b56934f237f8afb3b71d7a1c427a6802",
  "size": 120,
  "replaced_hash": null,
  "orphan_deleted": false
}
```

Re-uploading an existing tag overwrites it. `replaced_hash` reports which hash the tag pointed at
before, and `orphan_deleted` says whether that blob was removed (it is kept while any other tag
still references it).

### Check what a tag points at

```bash
curl -s "https://<host>/api/containers/v1/hash/mt4-bridge:0.1.0"
```

```json
{
  "container": "mt4-bridge",
  "tag": "0.1.0",
  "hash": "b1ac8873ed85297ea546f9439d736c95b56934f237f8afb3b71d7a1c427a6802",
  "size": 120,
  "uploaded_at": "2026-07-26T11:54:38.807570+00:00",
  "uploaded_by": "10.0.0.5"
}
```

`uploaded_by` is the ip the upload came from — with a single shared api key there is no key name to
record.

### List every tag of a container and what it points at

```bash
curl -s "https://<host>/api/containers/v1/tags/mt4-bridge" \
     -H "X-API-Key: $WIN_CONTAINERS_API_KEY"
```

```json
[
  {
    "tag": "0.1.0",
    "hash": "b1ac8873ed85297ea546f9439d736c95b56934f237f8afb3b71d7a1c427a6802",
    "size": 120,
    "uploaded_at": "2026-07-26T11:54:38.807570+00:00",
    "uploaded_by": "10.0.0.5"
  },
  {
    "tag": "latest",
    "hash": "1da238d07a0b58b35e6555ba90cbcaf1ef53570cbb7285a04a68af154d944ea2",
    "size": 120,
    "uploaded_at": "2026-07-26T11:56:16.557232+00:00",
    "uploaded_by": "10.0.0.5"
  }
]
```

The path here is the container **name only** — no `:tag`. Tags come back sorted by name (the yaml
keeps them in a `BTreeMap`). Two tags pointing at the same hash means one blob on disk.

### Download on the target machine

```powershell
Invoke-WebRequest "https://<host>/api/containers/v1/download/mt4-bridge:0.1.0" -OutFile mt4-bridge.zip
```

## Behaviour notes

- **Tags are mutable.** The same tag can be uploaded again with different content; the last upload
  wins. Uploading byte-identical content again is a no-op on disk.
- **Orphan blobs are deleted immediately** when the last tag referencing them goes away, so
  rebuilding a moving tag such as `latest` does not grow the folder.
- **Yaml is written before the orphan is deleted**, and both zip and yaml are written through a
  `.tmp` file plus rename — a crash can leave an unreferenced blob, never a tag pointing at a
  missing file.
- **Deleting the last tag keeps the (now empty) container folder**, so the container still shows up
  in `/list` with `tags_amount: 0`.
