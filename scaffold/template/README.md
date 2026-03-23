# Rustforge Starter

Full-stack application skeleton: **Rust API** (Axum + SQLx + Redis) + **React frontend** (Vite + Tailwind 4).
Build your product here. Keep framework changes in [Rustforge](https://github.com/weiloon1234/Rustforge.git), keep domain logic in this repo.

## Prerequisites

- **Rust** (stable) + `cargo-watch` (`cargo install cargo-watch`)
- **Node.js** (20+) + npm
- **PostgreSQL** (15+)
- **Redis** (7+)

## Quick Start

```bash
# 1. Environment
cp .env.example .env          # edit DB/Redis credentials

# 2. Code generation
cargo build -p generated

# 3. Database
./console migrate pump        # generate framework migrations
./console migrate run         # apply all migrations

# 4. Frontend
make install-frontend         # npm install

# 5. Run everything
make dev                      # Rust API (:3000) + user portal (:5173) + admin portal (:5174)
```

Open [http://localhost:5173](http://localhost:5173) for the user portal and [http://localhost:5174](http://localhost:5174) for the admin portal during development.

## Repository Layout

```
app/                    Rust application crate (API, websocket, worker, console)
  configs.toml          Languages, auth guards, realtime, CORS
  permissions.toml      Permission catalog
  models/*.rs           Model definitions, enums, helper DTOs, generated View methods
  src/
    internal/api/       Route handlers + state
    internal/workflows/ Business logic
    internal/jobs/      Background jobs
    contracts/          Request/response DTOs
    validation/         Validation rules
    seeds/              Database seeders
frontend/               Multi-portal React + Vite + Tailwind 4
  src/user/             User portal (served at /)
  src/admin/            Admin portal (served at /admin/)
  src/shared/           Shared components & utilities
generated/              Auto-generated code — never edit directly
migrations/             SQL migration files
i18n/                   Translation catalogs (en.json, zh.json, ...)
public/                 Built frontend output (git-ignored)
bin/                    Shell wrappers for running services
scripts/                Server install & update scripts
```

## Development

### Make targets

| Command | What it does |
|---------|-------------|
| `make dev` | Start Rust API + both Vite portals (all-in-one) |
| `make dev-api` | Rust API only with cargo-watch (auto-reload) |
| `make dev-frontend` | Both Vite portals |
| `make dev-user` | Vite user portal only (port 5173) |
| `make dev-admin` | Vite admin portal only (port 5174) |
| `make build-frontend` | Production build all portals into `public/` |
| `make install-frontend` | `npm install` for frontend |
| `make check` | `cargo check --workspace` + frontend `typecheck` + frontend production build (warnings fail) |
| `make gen` | Rebuild generated code |
| `make run-api` | Run API server (release) |
| `make run-ws` | Run WebSocket server |
| `make run-worker` | Run background worker |
| `make deploy` | Tag and trigger CI/CD release to production |

### Frontend architecture

The frontend ships two independent SPA portals, each with its own Vite config, dev server, and CSS theme:

| Portal | URL | Dev port | Vite config | Source |
|--------|-----|----------|-------------|--------|
| User | `/` | 5173 | `vite.config.user.ts` | `frontend/src/user/` |
| Admin | `/admin/` | 5174 | `vite.config.admin.ts` | `frontend/src/admin/` |

Both dev servers proxy `/api` to the Rust API on port 3000.

**Frontend env (Laravel-style)**: put browser-safe keys in project-root `.env` using `VITE_` prefix (example: `VITE_APP_NAME=${APP_NAME}`). Vite loads from root `.env`, and only `VITE_*` is exposed to React via `import.meta.env`.

**Tailwind 4**: No `tailwind.config.js` needed. Each portal customises design tokens via `@theme { }` in its own `app.css`.

**Production build**: `make build-frontend` cleans `public/`, builds admin into `public/admin/`, then user into `public/`. The Rust API serves `public/admin/index.html` as the admin SPA fallback and `public/index.html` as the user SPA fallback.

### Migrations

```bash
./console migrate pump          # generate framework migrations
./console migrate run           # apply pending migrations
./console migrate revert        # revert last migration
./console migrate add my_table  # create new migration file
```

### Seeds

```bash
./console db seed                         # run all seeders
./console db seed --name AdminBootstrap   # run one seeder (suffix optional: AdminBootstrapSeeder also works)
```

## Deployment

Production uses a **build-only deployment** workflow. Source code never reaches the production server — only pre-compiled artifacts.

### How it works

1. Developer runs `make deploy` on their machine
2. A timestamp-based git tag is created and pushed (e.g., `v2026.03.18.153000`)
3. GitHub Actions builds Rust binaries + React frontend on `ubuntu-24.04`
4. Artifacts are packaged into `release.zip` and pushed to a separate deploy repository
5. Production server polls that repo every 5 minutes and auto-deploys

No Rust, Node, or npm is needed on the production server.

### Initial Setup (one-time)

#### 1. Create the deploy repository

Create a private, empty repository for your deploy artifacts (e.g., `YOUR_ORG/YOUR_PROJECT-deploy`).

Update `.github/workflows/deploy.yml` to point to your deploy repository.

#### 2. Create a GitHub PAT

Create a fine-grained Personal Access Token at https://github.com/settings/tokens?type=beta:
- Repository access: Only select your deploy repository
- Permissions -> Repository permissions -> Contents: **Read and write**

#### 3. Add secrets to the source repo

In your source repo Settings -> Secrets and variables -> Actions, add these secrets:

| Secret | Description |
|--------|-------------|
| `DEPLOY_REPO_TOKEN` | The GitHub PAT from step 2 |
| `VITE_APP_NAME` | App display name |
| `VITE_S3_URL` | Public S3/CDN base URL for file access |

Add any other `VITE_*` variables your frontend needs. These are injected as environment variables during the frontend build in GitHub Actions. Vite embeds them into the JS bundle at build time — they are not available at runtime from `.env`. Update the `env:` block in `.github/workflows/deploy.yml` to match.

#### 4. First deploy

```bash
make deploy
```

Monitor the GitHub Actions tab. Once it succeeds, `release.zip`, `VERSION`, and `SHA256SUMS` will appear in your deploy repository.

### Production Server Setup

#### 1. Generate a deploy key

On the production server:

```bash
ssh-keygen -t ed25519 -C "deploy-key" -f ~/.ssh/deploy_key -N ""
cat ~/.ssh/deploy_key.pub
```

Add the public key as a **read-only deploy key** in your deploy repository -> Settings -> Deploy keys.

#### 2. Configure SSH to use the deploy key

```bash
cat >> ~/.ssh/config << 'EOF'
Host github.com-deploy
    HostName github.com
    User git
    IdentityFile ~/.ssh/deploy_key
    IdentitiesOnly yes
EOF
```

#### 3. Clone the deploy repo and extract

```bash
git clone git@github.com-deploy:YOUR_ORG/YOUR_PROJECT-deploy.git /opt/your-project-deploy
mkdir -p /opt/your-project
cd /opt/your-project-deploy
unzip release.zip -d /opt/your-project
```

#### 4. Run the installer

```bash
sudo /opt/your-project/scripts/install.sh
```

This will prompt for configuration (domain, database, ports, etc.) and set up:
- PostgreSQL, Redis, Nginx, Let's Encrypt
- Supervisor processes (api-server, websocket-server, worker, deploy-poll)
- `.env` file with all settings

#### 5. Verify

```bash
supervisorctl status
```

You should see all processes running, including the deploy-poll process.

### Deploying Updates

```bash
make deploy
```

The production server will detect the new version within 5 minutes, pull it, run migrations, and restart services automatically.

### Rollback

To rollback to the previous version, revert the last commit in your deploy repository and push:

```bash
cd /path/to/your-project-deploy
git revert HEAD --no-edit
git push
```

The deploy-poll script detects the version mismatch and re-deploys the reverted release.

### Manual Deploy (legacy)

For servers with the source code and build toolchain installed:

```bash
./scripts/deploy.sh
RUN_MIGRATIONS=false ./scripts/deploy.sh  # skip migrations
```

This script auto-detects whether Rust/Node are available and skips build steps if not.

## Key Concepts

### Code Generation (SSOT)

| Source file | Generates |
|-------------|-----------|
| `app/models/*.rs` | Model structs, enums, repos, query builders, datatable skeletons, generated View methods |
| `app/permissions.toml` | `Permission` enum |
| `app/configs.toml` | Typed `Settings`, auth guards, localization artifacts |

Generated code is output to `OUT_DIR` (inside `target/`) at build time — not to `generated/src/`. The `generated/src/lib.rs` uses `include!()` to pull in the generated modules. Never edit `generated/src/lib.rs` — put model-specific helper items and generated `View` / `WithRelations` methods in `app/models/*.rs`.

### Crate Naming

The scaffold generates project-unique crate names to avoid build cache collisions when multiple Rustforge projects share a cargo target directory:

- `app/Cargo.toml`: package name = your project name, library name = `app`
- `generated/Cargo.toml`: package name = `{project-name}-generated`

Code continues to use `app::` and `generated::` — only the cargo package names are unique.

### i18n

All user-facing strings go through `core_i18n::t()`. Translation files live in `i18n/`. Locale is resolved per-request from `X-Locale` or `Accept-Language` headers.

### Redis

`REDIS_CACHE_PREFIX` auto-derives from `{APP_NAME}_{APP_ENV}`. Leave empty unless you need custom namespacing.

### Dependency Pinning

This starter uses git dependencies to Rustforge `main` branch. For production stability, pin to a specific tag in `Cargo.toml`.

Starter scaffold intentionally does **not** ship a `Cargo.lock` file, so new projects resolve against current framework git references instead of stale pinned commits.

When maintaining this template inside the Rustforge framework repository, generated assets under `scaffold/template/` are cleaned explicitly via framework-root command:

```bash
make scaffold-template-clean
```

### Framework Documentation

```bash
make framework-docs-build
```

Publishes framework docs to `public/framework-documentation/`.

Framework feature/API/cookbook documentation is owned by Rustforge `core-docs`.
Starter-local operational guides stay under `docs/` in this project template. Start with `docs/README.md` for the starter-local index.

Use this split deliberately:

- `public/framework-documentation/`: canonical framework reference
- `docs/README.md`: starter-local docs index
- `docs/`: project/starter-specific guides and playbooks
