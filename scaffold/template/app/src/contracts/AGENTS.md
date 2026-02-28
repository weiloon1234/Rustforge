# Contracts

Request/response DTOs that define the API surface. Lives in `contracts/api/v1/{portal}/` (versioned), `contracts/datatable/{portal}/`, and `contracts/types/`.

## Input Structs — `#[rustforge_contract]`

Auto-injects `Debug, Clone, Deserialize, Validate, JsonSchema`. Use `#[rf(...)]` for validation rules.

```rust
use core_web::contracts::rustforge_contract;

#[rustforge_contract]
pub struct CreateArticleInput {
    #[rf(length(min = 3, max = 255))]
    #[rf(alpha_dash)]
    pub slug: String,

    #[rf(length(min = 1, max = 1000))]
    pub title: String,

    #[serde(default)]
    #[rf(email)]
    pub email: Option<String>,

    #[rf(nested)]
    pub metadata: MetadataInput,
}
```

## `#[rf(...)]` Rules

| Rule | Usage |
|------|-------|
| `length(min, max)` | `#[rf(length(min = 3, max = 64))]` |
| `range(min, max)` | `#[rf(range(min = 1, max = 100))]` |
| `email` | `#[rf(email)]` |
| `url` | `#[rf(url)]` |
| `alpha_dash` | letters, digits, `_`, `-` |
| `one_of(...)` | `#[rf(one_of("a", "b", "c"))]` |
| `none_of(...)` | `#[rf(none_of("x", "y"))]` |
| `regex(pattern)` | `#[rf(regex(pattern = r"^\d{4}$"))]` |
| `contains(pattern)` | `#[rf(contains(pattern = "@"))]` |
| `does_not_contain(pattern)` | `#[rf(does_not_contain(pattern = "banned"))]` |
| `must_match(other)` | `#[rf(must_match(other = "password_confirmation"))]` |
| `nested` | validate nested struct recursively |
| `date(format)` | `#[rf(date(format = "%Y-%m-%d"))]` |
| `phonenumber(field)` | `#[rf(phonenumber(field = "country_iso2"))]` |
| `async_unique(...)` | `#[rf(async_unique(table = "user", column = "email"))]` |
| `async_exists(...)` | `#[rf(async_exists(table = "role", column = "id"))]` |
| `async_not_exists(...)` | `#[rf(async_not_exists(table = "banned", column = "email"))]` |
| `openapi(...)` | `#[rf(openapi(description = "...", example = "..."))]` |

### Async unique with modifiers

```rust
#[rf(async_unique(
    table = "admin", column = "username",
    ignore(column = "id", field = "__target_id"),
    where_null(column = "deleted_at")
))]
```

### Update contracts with target ID for ignore

```rust
#[rustforge_contract]
pub struct UpdateArticleInput {
    #[serde(skip, default)]
    __target_id: i64,

    #[serde(default)]
    #[rf(length(min = 3, max = 255))]
    #[rf(async_unique(table = "article", column = "slug", ignore(column = "id", field = "__target_id")))]
    pub slug: Option<String>,
}

impl UpdateArticleInput {
    pub fn with_target_id(mut self, id: i64) -> Self {
        self.__target_id = id;
        self
    }
}
```

## Output Structs — manual derives (no macro)

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ArticleOutput {
    pub id: i64,
    pub title: String,
    #[schemars(with = "String")]
    pub created_at: time::OffsetDateTime,
}

impl From<generated::models::ArticleView> for ArticleOutput {
    fn from(v: generated::models::ArticleView) -> Self {
        Self { id: v.id, title: v.title, created_at: v.created_at }
    }
}
```

Use `#[schemars(with = "String")]` for types that don't implement `JsonSchema` (e.g. `time::OffsetDateTime`).

## Reusable String-Wrapper Types

For validation rules shared across contracts, define in `contracts/types/`:

```rust
use core_web::contracts::rustforge_string_rule_type;

rustforge_string_rule_type! {
    pub struct EmailAddress {
        #[rf(email)]
        #[rf(openapi(description = "Valid email", example = "user@example.com"))]
    }
}
```

Use as field type with `#[rf(nested)]`:
```rust
#[rf(nested)]
pub email: EmailAddress,
```

## TypeScript Type Generation

Contract structs are auto-exported to TypeScript via `ts-rs`. Add `#[derive(TS)]` alongside existing derives.

### Input structs (with `#[rustforge_contract]`)

```rust
use ts_rs::TS;

#[rustforge_contract]
#[derive(TS)]
#[ts(export, export_to = "admin/types/")]
pub struct CreateArticleInput {
    #[rf(length(min = 1, max = 255))]
    pub title: String,

    #[ts(type = "ArticleStatus")]           // generated enum — override type
    pub status: ArticleStatus,

    #[ts(type = "string")]                  // newtype wrapper — flatten to string
    #[rf(nested)]
    pub slug: SlugString,

    #[serde(default)]
    pub tags: Vec<String>,                  // ts-rs handles Vec<String> natively
}
```

### Output structs

```rust
#[derive(Debug, Clone, Serialize, JsonSchema, TS)]
#[ts(export, export_to = "admin/types/")]
pub struct ArticleOutput {
    pub id: i64,
    pub title: String,
    #[ts(type = "string")]                  // OffsetDateTime → string
    #[schemars(with = "String")]
    pub created_at: time::OffsetDateTime,
}
```

### Registering in `export-types.rs`

After adding `#[derive(TS)]` to your structs, register them in `app/src/bin/export-types.rs`:

```rust
// Add a new TsFile block:
{
    use app::contracts::api::v1::admin::article::*;
    files.push(TsFile {
        rel_path: "admin/types/article.ts",
        definitions: vec![
            CreateArticleInput::export_to_string().expect("CreateArticleInput"),
            ArticleOutput::export_to_string().expect("ArticleOutput"),
        ],
        enums: BTreeSet::new(),
    });
}
```

Then update the barrel `frontend/src/admin/types/index.ts` to re-export and run `make gen-types`.

### Conventions

- Only **serde-visible** fields are exported (fields with `#[serde(skip)]` are excluded)
- Use `#[ts(type = "TypeName")]` for types that don't derive `TS` (generated enums, framework types, newtypes)
- Use `#[ts(type = "string")]` for `time::OffsetDateTime` and string newtypes
- `Option<T>` becomes `T | null` automatically
- `Vec<T>` becomes `T[]` automatically
- `#[serde(default)]` fields become optional in TypeScript (with `serde-compat` feature)
