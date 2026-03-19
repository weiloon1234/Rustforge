# Deploy Server Cheatsheet

Step-by-step guide for bootstrapping a fresh Ubuntu 24.04 server for your app.

> Replace `<your-app>` with your project slug (e.g. `myapp`) and `<your-github-user>` with your GitHub username throughout this guide.

> Ubuntu 24.04 defaults to SSH via the `ubuntu` user (root SSH is disabled).
> The install script rejects root-owned project directories, so all paths use the `ubuntu` user.

---

## 1. Prerequisites

- Fresh Ubuntu 24.04 server
- DNS A record pointing your domain to the server IP
- SSH access as `ubuntu`

## 2. Install Base Tools

```bash
sudo apt-get update -y
sudo apt-get install -y git curl unzip
```

## 3. Create Deploy Key

```bash
ssh-keygen -t ed25519 -C "<your-app>-deploy" -f /home/ubuntu/.ssh/<your-app>_deploy -N ""
cat /home/ubuntu/.ssh/<your-app>_deploy.pub
```

Copy the public key output.

## 4. Add Key to GitHub

1. Go to **GitHub → `<your-github-user>/<your-app>-deploy` → Settings → Deploy keys**
2. Click **Add deploy key**
3. Paste the public key, title it `<your-app>-server`, leave **Allow write access** unchecked
4. Save

## 5. Configure SSH Host Alias

```bash
cat >> /home/ubuntu/.ssh/config << 'EOF'
Host github-<your-app>
    HostName github.com
    User git
    IdentityFile /home/ubuntu/.ssh/<your-app>_deploy
    IdentitiesOnly yes
EOF
chmod 600 /home/ubuntu/.ssh/config
```

## 6. Test GitHub Auth

```bash
ssh -T git@github-<your-app>
```

You should see: `Hi <your-github-user>/<your-app>-deploy! You've successfully authenticated...`

## 7. Create Directories

```bash
sudo mkdir -p /opt/<your-app> /opt/<your-app>-deploy
sudo chown ubuntu:ubuntu /opt/<your-app> /opt/<your-app>-deploy
```

## 8. Clone Deploy Repo

```bash
git clone git@github-<your-app>:<your-github-user>/<your-app>-deploy.git /opt/<your-app>-deploy
```

## 9. Extract First Release

```bash
cd /opt/<your-app>-deploy
sha256sum -c SHA256SUMS
unzip -q release.zip -d /opt/<your-app>
chmod +x /opt/<your-app>/target/release/* /opt/<your-app>/bin/* /opt/<your-app>/console /opt/<your-app>/scripts/*.sh 2>/dev/null || true
```

The install script requires `VERSION` and `bin/api-server` to exist in the project directory before it will run.

## 10. Run Install Script

```bash
sudo /opt/<your-app>/scripts/install.sh
```

The script will prompt for:

| Prompt | Default | Notes |
|--------|---------|-------|
| Project directory | `/opt/<your-app>` | |
| APP_NAME | `<your-app>` | |
| Project slug | `<your-app>` | Used for nginx/supervisor file names |
| Domain | `example.com` | Your actual domain |
| Deploy repo directory | `/opt/<your-app>-deploy` | |
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
| Enable deploy-poll | `yes` | Auto-deploy poller |

The script installs PostgreSQL, Redis, nginx, supervisor, and certbot as needed. It's idempotent — safe to re-run.

**Save the PostgreSQL credentials** printed during first run.

## 11. Verify

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

## 12. Symlink for Convenience

```bash
ln -sfn /opt/<your-app> ~/<your-app>
```

Now you can `cd ~/<your-app>` instead of `/opt/<your-app>`.

---

## Future Deploys

Deploys are automatic. The workflow:

1. Tag a version in the source repo: `git tag v1.2.3 && git push --tags`
2. GitHub Actions builds `release.zip` and pushes to `<your-github-user>/<your-app>-deploy`
3. `deploy-poll.sh` (running via Supervisor) checks every 5 minutes
4. On VERSION mismatch: verifies SHA256 → extracts via rsync → runs migrations → restarts services

No SSH required for routine deploys.

### Manual Deploy (if needed)

```bash
cd /opt/<your-app>-deploy && git pull
cd /opt/<your-app>
sha256sum -c /opt/<your-app>-deploy/SHA256SUMS
# Extract
staging=$(mktemp -d)
unzip -q /opt/<your-app>-deploy/release.zip -d "$staging"
rsync -a --delete --exclude='.env' --exclude='logs/' --exclude='.git/' "$staging/" /opt/<your-app>/
rm -rf "$staging"
chmod +x /opt/<your-app>/target/release/* /opt/<your-app>/bin/* /opt/<your-app>/console /opt/<your-app>/scripts/*.sh 2>/dev/null || true
# Migrate and restart
./console migrate run
sudo supervisorctl restart <your-app>-api <your-app>-ws <your-app>-worker
```

### Rollback

Revert the commit in the deploy repo and push — deploy-poll picks it up automatically:

```bash
cd /opt/<your-app>-deploy
git revert HEAD
git push
# deploy-poll will detect the VERSION change and redeploy within 5 minutes
```

---

## Helpful Commands

```bash
# Supervisor
sudo supervisorctl status                           # all process statuses
sudo supervisorctl restart <your-app>-api           # restart API server
sudo supervisorctl restart <your-app>-ws            # restart websocket server
sudo supervisorctl restart <your-app>-worker        # restart background worker
sudo supervisorctl restart <your-app>-deploy-poll   # restart deploy poller
sudo supervisorctl tail -f <your-app>-api           # follow API stdout

# Logs
tail -f /var/log/<your-app>-api.log                 # API stdout
tail -f /var/log/<your-app>-api.err.log             # API stderr
tail -f /var/log/<your-app>-ws.log                  # Websocket stdout
tail -f /var/log/<your-app>-worker.log              # Worker stdout
tail -f /var/log/<your-app>-deploy-poll.log         # Deploy poller log

# Nginx
sudo nginx -t                                       # test config
sudo systemctl reload nginx                         # reload after config change
cat /etc/nginx/sites-available/<your-app>.conf      # view site config

# App
cd /opt/<your-app> && ./console migrate run         # run migrations manually
cat /opt/<your-app>/VERSION                         # current deployed version
cat /opt/<your-app>/.env                            # app environment config

# SSL
sudo certbot renew --dry-run                        # test certificate renewal
```
