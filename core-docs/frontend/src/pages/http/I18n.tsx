export function I18n() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Internationalization</h1>
                <p className="text-xl text-gray-500">
                    Locale resolution, translation catalogs, and frontend/runtime locale handoff.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Where the SSOT lives</h2>
                <ul>
                    <li>
                        Supported/default locales: <code>app/configs.toml</code>
                    </li>
                    <li>
                        Translation catalogs: <code>i18n/*.json</code>
                    </li>
                    <li>
                        Request locale resolution: <code>core-i18n</code> middleware
                    </li>
                    <li>
                        Frontend runtime locale state: shared locale runtime/store in the starter frontend
                    </li>
                </ul>

                <h2>Locale resolution order</h2>
                <p>
                    Backend locale resolution is explicit:
                </p>
                <ol>
                    <li><code>X-Locale</code> header</li>
                    <li><code>Accept-Language</code> header</li>
                    <li>configured default locale</li>
                </ol>
                <p>
                    Supported locales are matched against configured values, including language-tag normalization
                    such as <code>zh-CN -&gt; zh</code>.
                </p>

                <h2>Config example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-toml">{`[languages]
default_locale = "en"
supported_locales = ["en", "zh"]
default_timezone = "+08:00"`}</code>
                </pre>

                <h2>Translation catalog ownership</h2>
                <p>
                    Translation catalogs are project-owned assets. Keep keys grouped and maintained in
                    <code> i18n/en.json</code>, <code>i18n/zh.json</code>, and any other supported locale files.
                    Do not treat English as an implicit non-file fallback policy for the starter.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-json">{`{
  "Validation failed": "Validation failed",
  "admin.read": "Read Admins",
  "admin.manage": "Manage Admins"
}`}</code>
                </pre>

                <h2>Backend usage</h2>
                <p>
                    Use <code>core_i18n::t()</code> or <code>t_args()</code> from Rust. Keep user-facing strings in
                    translation catalogs rather than scattering hardcoded response text across handlers/workflows.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_i18n::t;

return Err(AppError::Validation {
    message: t("Validation failed"),
    errors,
});`}</code>
                </pre>

                <h2>Frontend/runtime behavior</h2>
                <p>
                    The starter frontend sends <code>X-Locale</code> on API requests and keeps locale state on the
                    client. Runtime bootstrap can inject available locales and other locale-adjacent platform data.
                </p>
                <ul>
                    <li>Pre-login locale changes are frontend-only.</li>
                    <li>
                        After login, authenticated account locale should override local pre-login choice when the app
                        account model says so.
                    </li>
                    <li>
                        Frontend should use generated/shared locale types instead of handwritten locale unions where possible.
                    </li>
                </ul>

                <h2>Database content vs UI translations</h2>
                <p>
                    Use translation catalogs for UI/system text. Use schema-localized fields for model content such as
                    article titles and summaries.
                </p>
                <ul>
                    <li>
                        UI/system text: <code>i18n/*.json</code>
                    </li>
                    <li>
                        Database content: localized model fields and generated localized types
                    </li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/feature-localized-relations">Localized &amp; Relationships</a> for translated model content.
                    </li>
                    <li>
                        <a href="#/requests">Requests &amp; Validation</a> for request-side locale and bootstrap interactions.
                    </li>
                </ul>
            </div>
        </div>
    )
}
