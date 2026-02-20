import { useEffect } from 'react'
import Prism from 'prismjs'

export function FrameworkFeatures() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Framework Features</h1>
                <p className="text-xl text-gray-500">
                    Dedicated framework-level capabilities for generated models.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    These are first-class model features provided by the framework itself, not
                    app-specific conventions:
                </p>
                <ul>
                    <li>
                        <strong>Meta</strong> for flexible JSONB key-value data with typed read
                        helpers.
                    </li>
                    <li>
                        <strong>Attachments</strong> for single/multi file references with typed
                        input and hydrated URLs.
                    </li>
                    <li>
                        <strong>Localized fields + Relationships</strong> for multilingual content
                        and relation-aware query patterns.
                    </li>
                    <li>
                        <strong>Realtime / WebSocket</strong> for native WS subscriptions,
                        guard-reused auth, channel policies, and presence.
                    </li>
                </ul>

                <h2>Feature Menu</h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 not-prose">
                    <a
                        href="#/feature-meta"
                        className="block rounded-lg border border-gray-200 bg-white p-4 hover:bg-gray-50"
                    >
                        <h3 className="m-0 text-base font-semibold text-gray-900">Meta</h3>
                        <p className="mt-2 text-sm text-gray-600">
                            Schema keys, generated APIs, and typed + fallback access patterns.
                        </p>
                    </a>
                    <a
                        href="#/feature-attachments"
                        className="block rounded-lg border border-gray-200 bg-white p-4 hover:bg-gray-50"
                    >
                        <h3 className="m-0 text-base font-semibold text-gray-900">Attachments</h3>
                        <p className="mt-2 text-sm text-gray-600">
                            Attachment type registry, insert/update APIs, and hydrated URL usage.
                        </p>
                    </a>
                    <a
                        href="#/feature-localized-relations"
                        className="block rounded-lg border border-gray-200 bg-white p-4 hover:bg-gray-50"
                    >
                        <h3 className="m-0 text-base font-semibold text-gray-900">
                            Localized + Relationships
                        </h3>
                        <p className="mt-2 text-sm text-gray-600">
                            Multilingual field APIs, relation loaders, and relation query helpers.
                        </p>
                    </a>
                    <a
                        href="#/feature-realtime"
                        className="block rounded-lg border border-gray-200 bg-white p-4 hover:bg-gray-50"
                    >
                        <h3 className="m-0 text-base font-semibold text-gray-900">
                            Realtime / WebSocket
                        </h3>
                        <p className="mt-2 text-sm text-gray-600">
                            Native websocket protocol, guard reuse auth, Redis pubsub fan-out,
                            and room presence.
                        </p>
                    </a>
                </div>

                <h2 className="mt-10">Single-Source-of-Truth Schema Example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`[model.article]
table = "articles"
pk = "id"

fields = [
  "id:i64",
  "author_id:i64",
  "status:ArticleStatus",
  "created_at:datetime",
  "updated_at:datetime"
]

multilang = ["title", "summary"]
meta = [
  "seo_title:string",
  "is_featured:bool",
  "priority:i32",
  "extra:ExtraMeta",
  "published_at:datetime"
]
attachment = ["cover:image"]
attachments = ["gallery:image"]
relations = [
  "author:belongs_to:User:author_id:id",
  "comments:has_many:Comment:article_id:id"
]`}</code>
                </pre>

                <h2 className="mt-10">Generated API Mapping</h2>
                <div className="overflow-x-auto">
                    <table className="min-w-full text-sm border-collapse border border-gray-200">
                        <thead className="bg-gray-100">
                            <tr>
                                <th className="border p-2 text-left">Schema Key</th>
                                <th className="border p-2 text-left">Generated Types / Methods</th>
                                <th className="border p-2 text-left">Primary Page</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td className="border p-2 font-mono">meta</td>
                                <td className="border p-2 font-mono">
                                    set_meta_*, meta_*, meta_*_as&lt;T&gt;
                                </td>
                                <td className="border p-2">
                                    <a href="#/feature-meta">Meta</a>
                                </td>
                            </tr>
                            <tr>
                                <td className="border p-2 font-mono">attachment / attachments</td>
                                <td className="border p-2 font-mono">
                                    set_attachment_*, add_attachment_*, clear_attachment_*, delete_attachment_*
                                </td>
                                <td className="border p-2">
                                    <a href="#/feature-attachments">Attachments</a>
                                </td>
                            </tr>
                            <tr>
                                <td className="border p-2 font-mono">multilang / relations</td>
                                <td className="border p-2 font-mono">
                                    set_*_lang, set_*_langs, where_has_*, get_with_relations
                                </td>
                                <td className="border p-2">
                                    <a href="#/feature-localized-relations">Localized + Relationships</a>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    )
}
