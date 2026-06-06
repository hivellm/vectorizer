# Spec: Docker deployment documentation

## ADDED Requirements

### Requirement: Docker README + compose sample advertise `/data`

The root `README.md` Docker section and the sample
`docker-compose.yml` MUST document `/data` as the single canonical
volume mount point and reference the `VECTORIZER_DATA_DIR` env var
for non-Docker deployments.

`docs/deployment/docker.md` (or an equivalent file under `docs/`)
MUST contain:

- A mount-points table listing `/data` (persistent state) and the
  config file path.
- A migration section for operators currently mounting
  `/.local/share/vectorizer` as a second volume, with explicit
  commands to copy state from the old location into the `/data`
  volume.

#### Scenario: Reader follows compose sample and survives recreate

Given a new operator copy-pastes the sample `docker-compose.yml`
And runs `docker compose up -d`
And creates a collection
And runs `docker compose up -d --force-recreate`
When the operator queries `GET /collections`
Then the collection persists across the recreate
