export function DbGen() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Code Generation (db-gen)</h1>
                <p className="text-xl text-gray-500">
                    Build-time generation from layered framework + app model/config/permission sources.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What db-gen owns</h2>
                <p>
                    db-gen is the framework code generator for model APIs, enums, guards, permissions, localized
                    helpers, datatable skeletons, and related runtime glue. It should remain the single source for
                    generated code shape rather than hand-maintained copies in app crates.
                </p>

                <h2>Input sources</h2>
                <ul>
                    <li>
                        Framework model sources embedded by the framework build
                    </li>
                    <li>
                        App model sources from <code>app/models/*.rs</code>
                    </li>
                    <li>
                        Permissions from <code>app/permissions.toml</code>
                    </li>
                    <li>
                        Configs from <code>app/configs.toml</code>
                    </li>
                </ul>
                <p>
                    Duplicate model or enum names across layers are a generation error. There is no override mode.
                </p>

                <h2>Build flow</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// scaffold/template/generated/build.rs
let schema = db_gen::load_with_framework(app_models_dir)?;
let permissions = db_gen::load_permissions("app/permissions.toml")?;
let (cfgs, _) = db_gen::config::load("app/configs.toml")?;

db_gen::generate_enums(&schema, &models_out)?;
db_gen::generate_models(&schema, &cfgs, &models_out)?;
db_gen::generate_auth(&cfgs, &schema, &guards_out)?;
db_gen::generate_permissions(&permissions, &out_dir.join("permissions.rs"))?;
db_gen::generate_localized(&cfgs.languages, &cfgs, &schema, out_dir)?;`}</code>
                </pre>

                <h2>Generated surface</h2>
                <ul>
                    <li>
                        Models: <code>Xxx</code>, <code>XxxQuery</code>, <code>XxxInsert</code>,{' '}
                        <code>XxxUpdate</code>, <code>XxxView</code>, collections, relation helpers
                    </li>
                    <li>
                        Guards and permission enums generated from config/permission SSOT
                    </li>
                    <li>
                        Localized types and framework/platform TS export metadata
                    </li>
                    <li>
                        Datatable contracts/adapters/hooks on the model side; app runtime hooks stay app-owned
                    </li>
                </ul>

                <h2>What stays manual</h2>
                <ul>
                    <li>
                        app-facing helper items and <code>XxxView</code> / <code>XxxWithRelations</code> methods in <code>app/models/*.rs</code>
                    </li>
                    <li>
                        app datatable runtime hooks and route registration
                    </li>
                    <li>
                        workflows, handlers, and other business logic
                    </li>
                </ul>

                <h2>Generator design rules</h2>
                <ul>
                    <li>Template-driven file structure, not giant file-sized string dumps</li>
                    <li>Typed-first API generation</li>
                    <li>Model-defined PK type respected everywhere</li>
                    <li>Framework and app inputs treated as layered SSOT, not separate manual systems</li>
                </ul>

                <h2>Commands</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`cargo build -p generated
cargo check -p generated
make gen-types`}</code>
                </pre>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/schema">Model Source Definition</a> for the Rust model-source input surface.
                    </li>
                    <li>
                        <a href="#/model-api">Model API Overview</a> for the generated output surface.
                    </li>
                    <li>
                        <a href="#/permissions">Permissions &amp; AuthZ</a> for permission generation semantics.
                    </li>
                </ul>
            </div>
        </div>
    )
}
