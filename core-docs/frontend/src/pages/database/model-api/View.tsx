import { MethodTable } from './MethodTable'

export function ModelApiView() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">`XxxView` &amp; Model Methods</h1>
                <p className="text-xl text-gray-500">
                    Hydrated app-facing read model plus the intended extension point for computed helpers.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    <code>XxxRecord</code> is the stable app-facing model. It already includes hydrated framework
                    features such as localized values, meta bags, attachments, and generated helper methods.
                    App-specific computed values should be added on <code>XxxRecord</code>, not the raw DB row type.
                </p>

                <MethodTable
                    rows={[
                        {
                            method: 'update(db)',
                            returns: 'XxxUpdate',
                            notes: 'Start update scoped to this row primary key.',
                        },
                        {
                            method: 'update_with(&Xxx)',
                            returns: 'XxxUpdate',
                            notes: 'Use existing facade/model context.',
                        },
                        {
                            method: 'to_json()',
                            returns: 'XxxJson',
                            notes: 'Projection that respects hidden/computed/generated settings.',
                        },
                        {
                            method: 'meta_<field>()',
                            returns: 'Option<T> or Result<Option<T>>',
                            notes: 'Typed meta readers for declared schema keys.',
                        },
                        {
                            method: 'foo_explained',
                            returns: 'String or Option<String>',
                            notes: 'Generated explained field for enum-backed app-facing outputs where applicable.',
                        },
                    ]}
                />

                <h2>Use `XxxRecord` as the extension surface</h2>
                <p>
                    Put app-specific helpers in <code>app/models/&lt;model&gt;.rs</code> inside <code>#[rf_record_impl]</code>. This keeps DB row shapes,
                    generated code, and manual app semantics in one source of truth.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rf_record_impl]
impl AdminRecord {
    fn identity(&self) -> String {
        admin_identity(
            Some(self.username.as_str()),
            Some(self.name.as_str()),
            self.email.as_deref(),
            Some(self.id),
        )
    }
}`}</code>
                </pre>

                <h2>What not to extend</h2>
                <ul>
                    <li>
                        <code>XxxRow</code>: raw DB/internal shape, not the stable app model.
                    </li>
                    <li>
                        generated export helpers: output projection, not the primary place for business helpers.
                    </li>
                    <li>
                        Handler-local mapping code for every request: move reusable logic into view methods instead.
                    </li>
                </ul>

                <h2>Starter docs handoff</h2>
                <p>
                    For the starter-side cookbook version of this pattern, see
                    <code> scaffold/template/docs/computed-model-values.md</code>.
                </p>

                <h2>Example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let admin = AdminModel::query().find(db, 1001).await?.unwrap();

let display_name = admin.identity();
let featured = admin.meta_is_featured().unwrap_or(false);
let explained = admin.status_explained.clone();`}</code>
                </pre>
            </div>
        </div>
    )
}
