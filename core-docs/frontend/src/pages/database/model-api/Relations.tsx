export function ModelApiRelations() {
    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <h1 className="text-4xl font-extrabold text-gray-900">Relations and Joins</h1>
                <p className="text-xl text-gray-500">
                    Typed relation helpers first, explicit raw joins only when needed.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Relation declarations in <code>app/schemas/*.toml</code> generate typed
                    relation helpers and relation-aware query operations.
                </p>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`relations = [
  "author:belongs_to:User:author_id:id",
  "comments:has_many:Comment:article_id:id"
]`}</code>
                </pre>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`let rows = article
    .query()
    .where_has_comments(|q| q.where_is_spam(Op::Eq, false))
    .get()
    .await?;

let with_rels = article.query().get_with_relations().await?;`}</code>
                </pre>

                <p>
                    For complex cross-table cases that exceed typed relation helpers, use{' '}
                    <a href="#/model-api-unsafe">unsafe SQL escape hatch</a> with validated raw
                    helper types.
                </p>
            </div>
        </div>
    )
}
