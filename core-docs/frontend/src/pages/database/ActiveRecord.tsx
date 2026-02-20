import { useEffect } from 'react'
import Prism from 'prismjs'

export function ActiveRecord() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Active Record</h1>
                <p className="text-xl text-gray-500">
                    Eloquent-like ORM for Rust with full type safety.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    The generated models provide a fluent, chainable API for database
                    operations. Field/enum/relation usage is strongly typed at compile time.
                    For complex edge cases, raw SQL is available only through explicit{' '}
                    <code>unsafe_sql()</code> wrappers.
                </p>
                <p>
                    For a complete generated method reference grouped by struct, see{' '}
                    <a href="#/model-api">Model API</a>.
                </p>
                <p>
                    For dedicated framework feature guides (Meta, Attachments, and Localized +
                    Relationships), see <a href="#/framework-features">Framework Features</a>.
                </p>

                {/* Quick Reference */}
                <div className="bg-gray-50 border rounded-lg p-6 my-6">
                    <h3 className="mt-0">Quick Reference</h3>
                    <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <strong>Entry Point</strong>
                            <pre className="bg-gray-900 text-gray-100 p-2 rounded mt-1">
                                <code>{`let model = Article::new(&pool, None);`}</code>
                            </pre>
                        </div>
                        <div>
                            <strong>Query Builder</strong>
                            <pre className="bg-gray-900 text-gray-100 p-2 rounded mt-1">
                                <code>{`model.query().where_status(...).get().await?`}</code>
                            </pre>
                        </div>
                        <div>
                            <strong>Insert Builder</strong>
                            <pre className="bg-gray-900 text-gray-100 p-2 rounded mt-1">
                                <code>{`model.insert().set_title(...).save().await?`}</code>
                            </pre>
                        </div>
                        <div>
                            <strong>Update Builder</strong>
                            <pre className="bg-gray-900 text-gray-100 p-2 rounded mt-1">
                                <code>{`model.update().where_id(...).set_status(...).save().await?`}</code>
                            </pre>
                        </div>
                    </div>
                </div>

                {/* Simple CRUD */}
                <h2>Simple CRUD Operations</h2>

                <h3>Create</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use crate::generated::localized::Locale;

let article = model.insert()
    .set_status(ArticleStatus::Draft)
    .set_title_lang(Locale::En, "Hello World")
    .set_content_lang(Locale::En, "Article content...")
    .save()
    .await?;`}</code>
                </pre>

                <h3>Read</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`// Find by ID
let article = model.find(123).await?;

// Get all
let articles = model.query().get().await?;

// First result
let first = model.query()
    .where_status(Op::Eq, ArticleStatus::Published)
    .first()
    .await?;`}</code>
                </pre>

                <h3>Update</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`model.update()
    .where_key(123)
    .set_status(ArticleStatus::Published)
    .save()
    .await?;`}</code>
                </pre>

                <h3>Delete</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`// Soft delete
model.delete(123).await?;

// Restore
model.restore(123).await?;`}</code>
                </pre>

                <h3 className="text-lg font-bold text-gray-900 mt-6">Soft Deletes (Fluent API)</h3>
                <p>
                    For more control, you can use the query builder provided by <code>db-gen</code>.
                </p>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-2">
                     <div className="border p-4 rounded-lg bg-gray-50">
                        <h4 className="font-bold text-gray-900 m-0">Bulk Delete</h4>
                        <pre className="bg-gray-900 text-gray-100 p-2 rounded mt-2 text-xs overflow-x-auto">
                            <code className="language-rust">{`// Delete all drafts
model.query()
    .where_status(Op::Eq, "draft")
    .delete()
    .await?;`}</code>
                        </pre>
                    </div>
                    <div className="border p-4 rounded-lg bg-gray-50">
                        <h4 className="font-bold text-gray-900 m-0">Restore Deleted</h4>
                        <pre className="bg-gray-900 text-gray-100 p-2 rounded mt-2 text-xs overflow-x-auto">
                            <code className="language-rust">{`// Restore specific ID
model.query()
    .only_deleted() // or .with_deleted()
    .where_id(Op::Eq, 123)
    .restore()
    .await?;`}</code>
                        </pre>
                    </div>
                </div>

                {/* Localized Fields */}
                <h2>Localized Fields</h2>
                <p>
                    For fields listed in <code>multilang = ["field_name"]</code> in schema, the
                    generated model provides strongly typed setters using the <code>Locale</code>{' '}
                    enum.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use crate::generated::localized::Locale;

// Set individual locales
model.insert()
    .set_title_lang(Locale::En, "English Title")
    .set_title_lang(Locale::Zh, "Chinese Title")
    .save()
    .await?;
    
// Set from MultiLang struct (e.g. from request DTO)
model.insert()
    .set_title_langs(request.title_translations)
    .save()
    .await?;`}</code>
                </pre>

                {/* WHERE Conditions */}
                <h2>WHERE Conditions</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`// Type-safe column filters
model.query()
    .where_status(Op::Eq, ArticleStatus::Published)
    .where_ranking(Op::Gte, 5)
    .where_created_at(Op::Gt, some_datetime)
    .get().await?;

// IN clause
model.query()
    .where_in(ArticleCol::Id, &[1, 2, 3, 4, 5])
    .get().await?;

// Raw WHERE (explicit special-case path)
use core_db::common::sql::RawClause;

model.query()
    .unsafe_sql()
    .where_raw(RawClause::new(
        "title ILIKE ? OR content ILIKE ?",
        ["%rust%", "%rust%"],
    )?)
    .done()
    .get().await?;`}</code>
                </pre>

                {/* Ordering & Pagination */}
                <h2>Ordering & Pagination</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`// Order by column
model.query()
    .order_by(ArticleCol::CreatedAt, OrderDir::Desc)
    .get().await?;

// Pagination with count
let page = model.query()
    .where_status(Op::Eq, ArticleStatus::Published)
    .paginate(1, 0)  // Pass 0 to use DEFAULT_PER_PAGE (from .env)
    .await?;

// Page contains: data, total, per_page, current_page, last_page
println!("Total: {}, Pages: {}", page.total, page.last_page);`}</code>
                </pre>

                {/* Operators */}
                <h2>Available Operators (Op)</h2>
                <div className="overflow-x-auto">
                    <table className="min-w-full text-sm">
                        <thead>
                            <tr className="bg-gray-100">
                                <th className="px-4 py-2 text-left">Op Enum</th>
                                <th className="px-4 py-2 text-left">SQL</th>
                                <th className="px-4 py-2 text-left">Example</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td className="px-4 py-2"><code>Op::Eq</code></td>
                                <td className="px-4 py-2">=</td>
                                <td className="px-4 py-2">Equal</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-2"><code>Op::Ne</code></td>
                                <td className="px-4 py-2">{'<>'}</td>
                                <td className="px-4 py-2">Not equal</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-2"><code>Op::Lt</code></td>
                                <td className="px-4 py-2">{'<'}</td>
                                <td className="px-4 py-2">Less than</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-2"><code>Op::Gt</code></td>
                                <td className="px-4 py-2">{'>'}</td>
                                <td className="px-4 py-2">Greater than</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-2"><code>Op::Like</code></td>
                                <td className="px-4 py-2">LIKE</td>
                                <td className="px-4 py-2">Pattern match</td>
                            </tr>
                            <tr>
                                <td className="px-4 py-2"><code>Op::Ilike</code></td>
                                <td className="px-4 py-2">ILIKE</td>
                                <td className="px-4 py-2">Case-insensitive LIKE</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
                {/* Transactions */}
                <h2>Transactions</h2>
                <p>
                    Pool-backed <code>insert.save()</code> and <code>update.save()</code> are
                    auto-atomic by default. The generator wraps base row + localized/meta/attachment
                    writes in one transaction.
                </p>
                <p>
                    Use an explicit transaction when you need a larger cross-model unit of work.
                </p>

                <div className="bg-blue-50 border-l-4 border-blue-400 p-4 my-4">
                    <p className="text-sm text-blue-700">
                        <strong>Note:</strong> In explicit transaction mode, you still commit manually.
                        If the function returns early with <code>?</code> before commit, SQLx rolls it back on drop.
                    </p>
                </div>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_db::common::sql::DbConn;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn create_user_with_post(ctx: &BootContext) -> Result<()> {
    // 1. Start the transaction
    let tx = ctx.db.begin().await?;
    
    // 2. Wrap in Arc<Mutex<...>> for shared tx access.
    let tx_lock = Arc::new(Mutex::new(tx));

    // 3. Create models using the transaction connection
    // Note: We use DbConn::tx(tx_lock.clone()) explicitly
    let user_model = User::new(DbConn::tx(tx_lock.clone()), None);
    let post_model = Post::new(DbConn::tx(tx_lock.clone()), None);

    // 4. Perform operations (Atomic)
    let user = user_model.insert()
        .set_email("test@example.com")
        .save()
        .await?;

    post_model.insert()
        .set_user_id(user.id)
        .set_title("First Post")
        .save()
        .await?;

    // 5. Commit
    let tx = Arc::try_unwrap(tx_lock)
        .map_err(|_| anyhow::anyhow!("tx scope still in use"))?
        .into_inner();
    tx.commit().await?;
    
    Ok(())
}`}</code>
                </pre>
            </div>
        </div>
    )
}
