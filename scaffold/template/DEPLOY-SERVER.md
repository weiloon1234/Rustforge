# Deploy Server Cheatsheet

Step-by-step guide for bootstrapping a fresh Ubuntu 24.04 server for {{PROJECT_NAME}}.

> Ubuntu 24.04 defaults to SSH via the `ubuntu` user (root SSH is disabled).
> The install script rejects root-owned project directories, so all paths use the `ubuntu` user.

---

## 0. Pre-requisite: Create `ubuntu` User (if logged in as root)

Some cloud providers (e.g. DigitalOcean, Vultr) default to a `root` login with no `ubuntu` user. The install script rejects root-owned directories, so you must create the `ubuntu` user first.

```bash
# Run these as root:
adduser ubuntu --disabled-password --gecos ""
usermod -aG sudo ubuntu

# Allow passwordless sudo (required for install script and supervisor)
echo "ubuntu ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/ubuntu
chmod 0440 /etc/sudoers.d/ubuntu

# Copy SSH authorized_keys so you can SSH as ubuntu
mkdir -p /home/ubuntu/.ssh
cp /root/.ssh/authorized_keys /home/ubuntu/.ssh/authorized_keys
chown -R ubuntu:ubuntu /home/ubuntu/.ssh
chmod 700 /home/ubuntu/.ssh
chmod 600 /home/ubuntu/.ssh/authorized_keys

# Test: open a new terminal and SSH as ubuntu before proceeding
# ssh ubuntu@your-server-ip
```

Once confirmed, all remaining steps should be run as `ubuntu` (not root).

---

## 1. Prerequisites

- Fresh Ubuntu 24.04 server
- DNS A record pointing your domain to the server IP
- SSH access as `ubuntu` (see [step 0](#0-pre-requisite-create-ubuntu-user-if-logged-in-as-root) if only root is available)
- GitHub Actions secrets configured (see [GitHub Setup](#github-setup) below)
- At least one release already uploaded to R2 (run `make deploy` from source repo first)

## 2. Install Base Tools

```bash
sudo apt-get update -y
sudo apt-get install -y curl unzip

# Install AWS CLI v2 (not available via apt on Ubuntu 24.04)
curl -fsSL "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o /tmp/awscliv2.zip
unzip -q /tmp/awscliv2.zip -d /tmp
sudo /tmp/aws/install
rm -rf /tmp/awscliv2.zip /tmp/aws
```

## 3. Create Project Directory

```bash
sudo mkdir -p /opt/{{PROJECT_NAME}}
sudo chown ubuntu:ubuntu /opt/{{PROJECT_NAME}}
```

## 4. Create .env File

Copy `.env.example` to `.env` and configure the S3/R2 credentials (needed for initial release download):

```bash
# If you have .env.example from a previous extraction or copy it manually:
cp /opt/{{PROJECT_NAME}}/.env.example /opt/{{PROJECT_NAME}}/.env

# Or create minimal .env with R2 credentials for initial bootstrap:
cat > /opt/{{PROJECT_NAME}}/.env << 'EOF'
S3_ENDPOINT=https://<account-id>.r2.cloudflarestorage.com
S3_BUCKET={{BUCKET_NAME}}
S3_ACCESS_KEY=<your-r2-access-key>
S3_SECRET_KEY=<your-r2-secret-key>
EOF
```

## 5. Download First Release from R2

```bash
export AWS_ACCESS_KEY_ID=<your-r2-access-key>
export AWS_SECRET_ACCESS_KEY=<your-r2-secret-key>
export AWS_DEFAULT_REGION=auto
R2_ENDPOINT="https://<account-id>.r2.cloudflarestorage.com"
R2_BUCKET="{{BUCKET_NAME}}"

# Fetch latest version
aws s3 cp "s3://${R2_BUCKET}/deploy/VERSION" /tmp/deploy-version --endpoint-url "${R2_ENDPOINT}"
VERSION=$(cat /tmp/deploy-version)
echo "Latest version: ${VERSION}"

# Download and verify
aws s3 cp "s3://${R2_BUCKET}/deploy/${VERSION}/release.zip" /tmp/release.zip --endpoint-url "${R2_ENDPOINT}"
aws s3 cp "s3://${R2_BUCKET}/deploy/${VERSION}/SHA256SUMS" /tmp/SHA256SUMS --endpoint-url "${R2_ENDPOINT}"
(cd /tmp && sha256sum -c SHA256SUMS)

# Extract to project directory
unzip -q /tmp/release.zip -d /opt/{{PROJECT_NAME}}
chmod +x /opt/{{PROJECT_NAME}}/target/release/* /opt/{{PROJECT_NAME}}/bin/* /opt/{{PROJECT_NAME}}/console /opt/{{PROJECT_NAME}}/scripts/*.sh 2>/dev/null || true
rm -f /tmp/release.zip /tmp/SHA256SUMS /tmp/deploy-version
```

The install script requires `VERSION` and `bin/api-server` to exist in the project directory before it will run.

## 6. Run Install Script

```bash
sudo /opt/{{PROJECT_NAME}}/scripts/install.sh
```

The script will prompt for:

| Prompt | Default | Notes |
|--------|---------|-------|
| Project directory | `/opt/{{PROJECT_NAME}}` | |
| APP_NAME | `{{PROJECT_NAME}}` | |
| Project slug | `{{PROJECT_NAME}}` | Used for nginx/supervisor file names |
| Domain | `example.com` | Your actual domain |
| APP_ENV | `production` | |
| APP_DEBUG | `no` | |
| SERVER_PORT | `3000` | |
| REALTIME_PORT | `3010` | |
| Setup PostgreSQL | `yes` | Creates DB user/password automatically |
| DATABASE_URL | auto-generated | Only if PostgreSQL setup is yes |
| REDIS_URL | `redis://127.0.0.1:6379/0` | |
| Enable HTTPS | `yes` | Uses Let's Encrypt + certbot |
| Let's Encrypt email | `admin@<domain>` | |
| Enable Supervisor | `yes` | |
| Manage websocket-server | `yes` | |
| Manage worker | `yes` | |
| Enable deploy-poll | `yes` | R2-based auto-deploy poller |

The script installs PostgreSQL, Redis, nginx, supervisor, AWS CLI v2, and certbot as needed. It's idempotent — safe to re-run.

**Save the PostgreSQL credentials** printed during first run.

## 7. Verify

```bash
# Check all processes are running
sudo supervisorctl status

# Test nginx config
sudo nginx -t

# Check the app responds
curl -s http://localhost:3000/health

# Check via domain (after DNS propagates)
curl -s https://yourdomain.com/health
```

## 8. Symlink for Convenience

```bash
ln -sfn /opt/{{PROJECT_NAME}} ~/{{PROJECT_NAME}}
```

Now you can `cd ~/{{PROJECT_NAME}}` instead of `/opt/{{PROJECT_NAME}}`.

---

## GitHub Setup

### Secrets Required

Configure these in **GitHub → Source repo → Settings → Secrets and variables → Actions**:

| Secret | Value | Notes |
|--------|-------|-------|
| `R2_ACCESS_KEY_ID` | R2 API token access key | Same as app's `S3_ACCESS_KEY` |
| `R2_SECRET_ACCESS_KEY` | R2 API token secret key | Same as app's `S3_SECRET_KEY` |
| `R2_ENDPOINT` | `https://<account-id>.r2.cloudflarestorage.com` | Same as app's `S3_ENDPOINT` |
| `R2_BUCKET` | `{{PROJECT_NAME}}` | Same as app's `S3_BUCKET` |
| `VITE_APP_NAME` | App name for frontend | |
| `VITE_S3_URL` | Public R2 URL for frontend assets | |

### R2 Object Layout

Artifacts are stored under the `deploy/` prefix in the same R2 bucket used for app file uploads:

```
{{BUCKET_NAME}}/
├── deploy/
│   ├── VERSION                    ← current version pointer
│   ├── v2026.03.22.120000/
│   │   ├── release.zip
│   │   └── SHA256SUMS
│   └── ...                        ← 3 versions retained
├── uploads/...                    ← app file uploads (untouched)
```

---

## Future Deploys

Deploys are automatic. The workflow:

1. Tag a version in the source repo: `make deploy` (creates `v{TIMESTAMP}` tag)
2. GitHub Actions builds `release.zip` and uploads to R2 under `deploy/{version}/`
3. `deploy-poll.sh` (running via Supervisor) polls R2 every 5 minutes
4. On VERSION mismatch: downloads zip → verifies SHA256 → extracts via rsync → runs migrations → restarts services
5. GitHub Actions cleans up old versions (retains 3 most recent)

No SSH required for routine deploys.

### Manual Deploy (if needed)

```bash
cd /opt/{{PROJECT_NAME}}
export AWS_ACCESS_KEY_ID=$(grep S3_ACCESS_KEY .env | cut -d= -f2)
export AWS_SECRET_ACCESS_KEY=$(grep S3_SECRET_KEY .env | cut -d= -f2)
export AWS_DEFAULT_REGION=auto
R2_ENDPOINT=$(grep S3_ENDPOINT .env | cut -d= -f2)
R2_BUCKET=$(grep S3_BUCKET .env | cut -d= -f2)

# Fetch version and artifacts
aws s3 cp "s3://${R2_BUCKET}/deploy/VERSION" /tmp/deploy-version --endpoint-url "${R2_ENDPOINT}"
VERSION=$(cat /tmp/deploy-version)
aws s3 cp "s3://${R2_BUCKET}/deploy/${VERSION}/release.zip" /tmp/release.zip --endpoint-url "${R2_ENDPOINT}"
aws s3 cp "s3://${R2_BUCKET}/deploy/${VERSION}/SHA256SUMS" /tmp/SHA256SUMS --endpoint-url "${R2_ENDPOINT}"
(cd /tmp && sha256sum -c SHA256SUMS)

# Extract and deploy
staging=$(mktemp -d)
unzip -q /tmp/release.zip -d "$staging"
rsync -a --delete --exclude='.env' --exclude='logs/' --exclude='.git/' "$staging/" /opt/{{PROJECT_NAME}}/
rm -rf "$staging" /tmp/release.zip /tmp/SHA256SUMS /tmp/deploy-version
chmod +x /opt/{{PROJECT_NAME}}/target/release/* /opt/{{PROJECT_NAME}}/bin/* /opt/{{PROJECT_NAME}}/console /opt/{{PROJECT_NAME}}/scripts/*.sh 2>/dev/null || true

# Migrate and restart
./console migrate run
sudo supervisorctl restart {{PROJECT_NAME}}-api {{PROJECT_NAME}}-ws {{PROJECT_NAME}}-worker
```

### Rollback

Update the `deploy/VERSION` file in R2 to point to an older retained version:

```bash
# From any machine with awscli + R2 credentials configured
echo "v2026.03.21.100000" > /tmp/rollback-version
aws s3 cp /tmp/rollback-version "s3://{{PROJECT_NAME}}/deploy/VERSION" --endpoint-url "$R2_ENDPOINT"
# deploy-poll will detect the VERSION change and redeploy within 5 minutes
```

---

## Helpful Commands

```bash
# Supervisor
sudo supervisorctl status                    # all process statuses
sudo supervisorctl restart {{PROJECT_NAME}}-api          # restart API server
sudo supervisorctl restart {{PROJECT_NAME}}-ws           # restart websocket server
sudo supervisorctl restart {{PROJECT_NAME}}-worker       # restart background worker
sudo supervisorctl restart {{PROJECT_NAME}}-deploy-poll  # restart deploy poller
sudo supervisorctl tail -f {{PROJECT_NAME}}-api          # follow API stdout

# Logs
tail -f /var/log/{{PROJECT_NAME}}-api.log               # API stdout
tail -f /var/log/{{PROJECT_NAME}}-api.err.log           # API stderr
tail -f /var/log/{{PROJECT_NAME}}-ws.log                # Websocket stdout
tail -f /var/log/{{PROJECT_NAME}}-worker.log            # Worker stdout
tail -f /var/log/{{PROJECT_NAME}}-deploy-poll.log       # Deploy poller log

# Nginx
sudo nginx -t                               # test config
sudo systemctl reload nginx                 # reload after config change
cat /etc/nginx/sites-available/{{PROJECT_NAME}}.conf    # view site config

# App
cd /opt/{{PROJECT_NAME}} && ./console migrate run       # run migrations manually
cat /opt/{{PROJECT_NAME}}/VERSION                       # current deployed version
cat /opt/{{PROJECT_NAME}}/.env                          # app environment config

# SSL
sudo certbot renew --dry-run                # test certificate renewal
```
