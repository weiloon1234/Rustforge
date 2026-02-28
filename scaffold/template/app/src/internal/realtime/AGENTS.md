# Realtime

WebSocket channel policies and authorizers. Channels are configured in `app/configs.toml`:

```toml
[realtime.channels.notifications]
enabled = true
guard = "admin"
presence_enabled = true
```

## Channel Policy

Implement subscribe/publish authorization logic here for channels that need custom access control.
