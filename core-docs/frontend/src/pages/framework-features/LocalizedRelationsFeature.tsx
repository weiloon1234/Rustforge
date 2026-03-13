import { useEffect } from 'react'
import Prism from 'prismjs'

export function LocalizedRelationsFeature() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Localized &amp; Relationships</h1>
                <p className="text-xl text-gray-500">
                    Localized field storage, locale-aware view hydration, and typed relation helpers generated from model source.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Where the SSOT lives</h2>
                <p>
                    Both localized fields and relations are declared in <code>app/models/*.rs</code>. Generation
                    owns the typed insert/update/query/view APIs from those declarations. App code should not rebuild
                    the same relation or locale semantics manually.
                </p>

                <h2>Model source example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub author_id: i64,
    pub status: ArticleStatus,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    pub title: Localized<String>,
    pub summary: Localized<String>,
    #[rf(foreign_key = "author_id")]
    pub author: BelongsTo<User>,
    #[rf(foreign_key = "article_id")]
    pub comments: HasMany<Comment>,
}`}</code>
                </pre>

                <h2>Localized runtime surface</h2>
                <ul>
                    <li>
                        Insert/update helpers: <code>set_&lt;field&gt;_lang(...)</code> and{' '}
                        <code>set_&lt;field&gt;_langs(...)</code>
                    </li>
                    <li>
                        View hydration: locale-specific field value plus full translations payload
                    </li>
                    <li>
                        Shared TS/runtime support: localized types are exported from Rust so the frontend uses the
                        same shape
                    </li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use crate::generated::localized::{Locale, LocalizedText};

let article = article_model
    .insert()
    .set_title_lang(Locale::En, "Rust Performance Guide")
    .set_title_lang(Locale::Zh, "Rust 性能指南")
    .set_summary_langs(LocalizedText {
        en: "Typed-first DX".to_string(),
        zh: "类型优先 DX".to_string(),
    })
    .save()
    .await?;

println!("title = {:?}", article.title);
println!("title translations = {:?}", article.title_translations);`}</code>
                </pre>

                <h2>Locale-aware view behavior</h2>
                <p>
                    View hydration reads the current locale context and still keeps the full translation bag. That
                    means the app-facing model can expose a direct localized value for common use and retain the full
                    multi-locale data for editors or admin UI.
                </p>

                <h2>Relation runtime surface</h2>
                <ul>
                    <li>
                        Query predicates such as <code>where_has_*</code> and <code>where_doesnt_have_*</code>
                    </li>
                    <li>
                        Batch loaders and relation-aware retrieval like <code>get_with_relations()</code>
                    </li>
                    <li>
                        Count helpers such as <code>with_counts()</code> for has-many relations
                    </li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_db::common::sql::Op;

let rows = article_model
    .query()
    .where_has_comments(|q| q.where_status(Op::Eq, CommentStatus::Published))
    .get_with_relations()
    .await?;

for item in rows {
    println!("article id = {}", item.row.id);
    println!("author = {:?}", item.author);
    println!("comments count = {}", item.comments.len());
}`}</code>
                </pre>

                <h2>Typed-first rules</h2>
                <ul>
                    <li>Declare localized fields and relations in model source, not as ad-hoc workflow conventions.</li>
                    <li>Use generated relation helpers before reaching for manual SQL joins.</li>
                    <li>
                        Keep localized values in Rust/TS shared types instead of duplicating translation bag shapes.
                    </li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li>
                        <a href="#/model-api-relations">Relations &amp; Joins</a> for lower-level relation query API details.
                    </li>
                    <li>
                        <a href="#/i18n">Internationalization</a> for locale resolution and transport behavior.
                    </li>
                    <li>
                        <a href="#/cookbook/build-crud-admin-resource">Build a CRUD Admin Resource</a> for a starter recipe.
                    </li>
                </ul>
            </div>
        </div>
    )
}
