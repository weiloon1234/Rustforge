import { useEffect } from 'react'
import Prism from 'prismjs'

export function LocalizedRelationsFeature() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Feature: Localized + Relationships
                </h1>
                <p className="text-xl text-gray-500">
                    Multilingual fields and relation-aware query/model helpers.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Schema</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-toml">{`[model.article]
fields = [
  "id:i64",
  "author_id:i64",
  "status:ArticleStatus",
  "created_at:datetime",
  "updated_at:datetime"
]

# localized fields
multilang = ["title", "summary"]

# relations
relations = [
  "author:belongs_to:User:author_id:id",
  "comments:has_many:Comment:article_id:id"
]`}</code>
                </pre>

                <h2>Localized Field APIs</h2>
                <ul>
                    <li>
                        On insert/update: <code>set_&lt;field&gt;_lang(Locale, value)</code> and{' '}
                        <code>set_&lt;field&gt;_langs(MultiLang)</code>.
                    </li>
                    <li>
                        On view: <code>title: Option&lt;String&gt;</code> and{' '}
                        <code>title_translations: Option&lt;MultiLang&gt;</code> (same pattern for
                        other localized fields).
                    </li>
                    <li>
                        View hydration reads locale from framework i18n context and also keeps full
                        translations.
                    </li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use crate::generated::localized::{Locale, MultiLang};

let article = article_model
    .insert()
    .set_title_lang(Locale::En, "Rust Performance Guide")
    .set_title_lang(Locale::Zh, "Rust 性能指南")
    .set_summary_langs(MultiLang {
        en: "Typed-first DX".to_string(),
        zh: "类型优先 DX".to_string(),
    })
    .save()
    .await?;

// Hydrated from current request locale + full translations
println!("title = {:?}", article.title);
println!("title translations = {:?}", article.title_translations);`}</code>
                </pre>

                <h2>Relationship APIs</h2>
                <h3>Model Loaders (batch/eager map style)</h3>
                <ul>
                    <li>
                        <code>load_&lt;has_many_relation&gt;(&amp;[XxxView]) -&gt; HashMap&lt;id,
                            Vec&lt;TargetRow&gt;&gt;</code>
                    </li>
                    <li>
                        <code>load_&lt;belongs_to_relation&gt;(&amp;[XxxView]) -&gt; HashMap&lt;id,
                            Option&lt;TargetRow&gt;&gt;</code>
                    </li>
                </ul>

                <h3>Query Relation Filters</h3>
                <ul>
                    <li>
                        <code>where_has_&lt;relation&gt;(scope)</code>
                    </li>
                    <li>
                        <code>where_doesnt_have_&lt;relation&gt;(scope)</code>
                    </li>
                    <li>
                        <code>or_where_has_&lt;relation&gt;(scope)</code>
                    </li>
                </ul>

                <h3>Relation-Aware Retrieval</h3>
                <ul>
                    <li>
                        <code>get_with_relations()</code> returning{' '}
                        <code>Vec&lt;XxxWithRelations&gt;</code>
                    </li>
                    <li>
                        <code>paginate_with_relations(page, per_page)</code> returning{' '}
                        <code>Page&lt;XxxWithRelations&gt;</code>
                    </li>
                    <li>
                        <code>with_counts(&amp;[XxxRel])</code> returning relation count maps for
                        has-many relations
                    </li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
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
}

let (articles, counts) = article_model
    .query()
    .with_counts(&[ArticleRel::Comments])
    .await?;

let comment_counts = counts.get("comments");`}</code>
                </pre>

                <h2>Typed-First Notes</h2>
                <p>
                    Localized and relation methods are generated from schema names, so you keep
                    compile-time safety and avoid stringly-typed relation building for common
                    patterns.
                </p>
            </div>
        </div>
    )
}

