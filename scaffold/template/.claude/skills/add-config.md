---
name: add-config
description: Add a custom configuration section to settings.toml
---

# Add a Configuration Section

Follow these steps to add a new configuration section to the application.

## Step 1: Add the TOML section

Edit `app/settings.toml` and add the new configuration section:

```toml
[my_feature]
enabled = true
max_retries = 3
api_url = "https://example.com"
timeout_seconds = 30
```

Conventions:
- Section names use snake_case.
- Provide sensible defaults for all values.
- Use descriptive key names.
- Group related settings under one section.

## Step 2: Create the config struct

Create `app/src/internal/config/{name}.rs`:

```rust
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct MyFeatureConfig {
    pub enabled: bool,
    pub max_retries: u32,
    pub api_url: String,
    pub timeout_seconds: u64,
}

impl Default for MyFeatureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            api_url: String::new(),
            timeout_seconds: 30,
        }
    }
}
```

Conventions:
- Derive `Debug, Clone, serde::Deserialize`.
- Use `#[serde(default)]` so missing keys fall back to the `Default` impl.
- Implement `Default` with the same values as in `settings.toml`.
- Use standard Rust types: `bool`, `u32`, `u64`, `String`, `Option<T>`.

## Step 3: Register the module

Add `pub mod {name};` to `app/src/internal/config/mod.rs`.

## Step 4: Load the config in AppApiState

Update the `AppApiState::new()` function (or equivalent initialization) to load the section:

```rust
use crate::internal::config::{name}::MyFeatureConfig;

// In AppApiState struct:
pub my_feature_config: MyFeatureConfig,

// In AppApiState::new():
my_feature_config: ctx.settings.section::<MyFeatureConfig>("my_feature").unwrap_or_default(),
```

The `section::<T>("name")` method deserializes the TOML section into the config struct. `unwrap_or_default()` ensures the app starts even if the section is missing.

## Step 5: Add the field to AppApiState

Ensure the struct has the new field:

```rust
pub struct AppApiState {
    // ... existing fields
    pub my_feature_config: MyFeatureConfig,
}
```

## Step 6: Use the config

Access the config from handlers, workflows, or jobs via the state:

```rust
// In a workflow:
if state.my_feature_config.enabled {
    let timeout = Duration::from_secs(state.my_feature_config.timeout_seconds);
    // Use the config
}
```

## Step 7: Environment variable overrides

Settings can be overridden via environment variables. The convention is uppercase with underscores:

```bash
MY_FEATURE_ENABLED=false
MY_FEATURE_MAX_RETRIES=5
MY_FEATURE_API_URL=https://production.example.com
```

Check how the settings loader maps env vars to TOML keys in the project -- typically it uses a prefix and separator convention.

## Step 8: Verify

```bash
cargo check
```

Common issues:
- Type mismatch between TOML value and struct field (e.g., string in TOML but `u32` in Rust).
- Missing `Default` impl when using `#[serde(default)]`.
- Forgetting to add the field to `AppApiState` struct.
