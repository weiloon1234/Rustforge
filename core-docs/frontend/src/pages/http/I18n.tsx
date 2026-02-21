export function I18n() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Internationalization (i18n)
                </h1>
                <p className="text-xl text-gray-500">
                    Support multiple languages using automatic locale detection.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Overview</h2>
                <p>
                    The framework provides built-in i18n support. User locale is
                    automatically detected from the <code>Accept-Language</code> HTTP
                    header.
                </p>
                <p>
                    Translation catalogs are <strong>project-level assets</strong>. In starter
                    layout, keep them in <code>i18n/</code> at project root.
                </p>

                <div className="bg-blue-50 border-l-4 border-blue-400 p-4 my-6">
                    <h4 className="text-blue-900 font-bold mb-2">Automatic Detection</h4>
                    <p className="text-sm text-blue-700">
                        Every request automatically extracts the user's preferred language
                        from the <code>Accept-Language</code> header. No configuration needed!
                    </p>
                </div>

                <div className="bg-green-50 border-l-4 border-green-400 p-4 my-6">
                    <h4 className="text-green-900 font-bold mb-2">
                        💡 Better DX: English as Key
                    </h4>
                    <p className="text-sm text-green-700">
                        Use <strong>English text directly as the translation key</strong>.
                        This means:
                    </p>
                    <ul className="text-sm text-green-700 mt-2 space-y-1 list-disc pl-5">
                        <li>
                            ✅ No need for <code>en.json</code> - English is the fallback
                        </li>
                        <li>✅ Only non-English languages need translation files</li>
                        <li>✅ Less boilerplate, faster development</li>
                    </ul>
                </div>

                <h3 className="mt-6">✅ Correct: English Text as Key</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_i18n::t;
use core_web::error::AppError;
use core_web::response::ApiResponse;

// ✅ English text is the key - no en.json needed!
Err(AppError::BadRequest(t("Username is already taken")))
Ok(ApiResponse::success(data, &t("Profile updated successfully")))`}</code>
                </pre>

                <h2 className="mt-10">Translation Files</h2>
                <p>
                    Only create translation files for <strong>non-English</strong> languages
                    in <code>i18n/</code>:
                </p>

                <h3 className="mt-6">zh.json - Chinese translations</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`{
  "Username is already taken": "用户名已被占用",
  "Invalid email address": "无效的电子邮件地址",
  "Profile updated successfully": "个人资料更新成功"
}`}</code>
                </pre>

                <h2 className="mt-10">Testing Different Languages</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`# English (default)
curl http://localhost:3000/api/users/123
# Response: "Profile updated successfully"

# Chinese
curl -H "Accept-Language: zh-CN" http://localhost:3000/api/users/123
# Response: "个人资料更新成功"`}</code>
                </pre>

                <div className="bg-gray-50 border-l-4 border-gray-400 p-4 mt-8">
                    <h4 className="font-bold text-gray-900 mb-2">Summary</h4>
                    <ul className="text-sm text-gray-700 space-y-1">
                        <li>
                            ✅ Locale auto-detected from <code>Accept-Language</code> header
                        </li>
                        <li>
                            ✅ Use <code>t("key")</code> instead of hardcoded strings
                        </li>
                        <li>
                            ✅ Translation files: <code>{'i18n/{lang}.json'}</code>
                        </li>
                        <li>
                            ✅ Test with <code>-H "Accept-Language: zh-CN"</code>
                        </li>
                    </ul>
                </div>

                <h2 className="mt-10">Database Content Translation</h2>
                <p>
                    For translating database content (like Article titles), use the <strong>Localized Fields</strong> feature in Active Record.
                    <br />
                    <a href="#/active-record" className="text-orange-600 hover:text-orange-800 font-medium">
                        View Active Record Docs →
                    </a>
                </p>
            </div>
        </div>
    )
}
