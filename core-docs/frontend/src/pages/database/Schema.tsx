export function Schema() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Schema Definition</h1>
                <p className="text-xl text-gray-500">
                    TOML in <code>app/schemas</code> is the source of truth for generated models.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Single Source of Truth</h2>
                <p>
                    Define models in <code>app/schemas/*.toml</code>. Generated output is written
                    to <code>generated/src</code>. App-level logic should consume generated APIs,
                    not redefine schema elsewhere.
                </p>

                <h2>Example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-toml">{`[ArticleStatus]
kind = "string"
values = ["draft", "published"]

[model.article]
table = "articles"
pk = "id"
pk_type = "i64"
fields = [
  "id:i64",
  "title:string",
  "status:ArticleStatus",
  "created_at:datetime",
  "updated_at:datetime"
]

multilang = ["title"]
attachment = ["cover:image"]
attachments = ["gallery:image"]
relations = ["category:belongs_to:article_category:category_id:id"]`}</code>
                </pre>

                <h2>Meta Typed Shapes</h2>
                <p>
                    Custom meta shapes can live in <code>generated/src/extensions.rs</code>.
                </p>
            </div>
        </div>
    )
}
