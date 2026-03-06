import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter3JobsUsage() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Recipe: Add Jobs
                </h1>
                <p className="text-xl text-gray-500">
                    Register jobs once, dispatch them from workflows, and let the worker own retries, backoff, and failed-job handling.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Add durable background processing without leaking queue logic into handlers or breaking DB write integrity.
                </p>

                <h2>Step 1: Define the job</h2>
                <p>
                    Put concrete jobs under <code>app/src/internal/jobs/</code>. The job payload should be serializable, minimal, and stable enough to survive retries.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub struct RebuildSearchIndexJob {
    pub article_id: i64,
}

#[async_trait]
impl Job for RebuildSearchIndexJob {
    const NAME: &'static str = "rebuild_search_index";
    const QUEUE: &'static str = "default";

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        // reload typed model, recompute index, persist result
        Ok(())
    }
}`}</code>
                </pre>

                <h2>Step 2: Register the job once</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub fn register_jobs(worker: &mut Worker) {
    worker.register::<RebuildSearchIndexJob>();
}`}</code>
                </pre>

                <h2>Step 3: Dispatch from the workflow, not the handler</h2>
                <p>
                    The workflow should own the decision that a job is needed. Handlers stay thin and should not become queue orchestration layers.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub async fn publish_article(...) -> Result<ArticleView, AppError> {
    let row = article
        .update()
        .where_id(Op::Eq, id)
        .set_status(ArticleStatus::Published)
        .save()
        .await?;

    job_buffer.push(RebuildSearchIndexJob { article_id: id })?;
    Ok(row)
}`}</code>
                </pre>

                <h2>Step 4: Use outbox-style buffering when the job must commit with the write</h2>
                <p>
                    If the job must only exist when the DB write commits, use the durable buffering path instead of pushing directly from a request handler.
                </p>
                <ul>
                    <li>DB write succeeds + job enqueue succeeds together</li>
                    <li>worker/outbox sweeper dispatches later</li>
                    <li>failed-job storage captures terminal failures</li>
                </ul>

                <h2>Step 5: Operate through the worker and failed-job API</h2>
                <p>
                    The framework exposes queue and failed-job APIs so operators can inspect queues, list failed jobs, and retry failed work.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./bin/worker
curl http://127.0.0.1:3000/_jobs/failed
curl -X POST http://127.0.0.1:3000/_jobs/failed/<JOB_ID>/retry`}</code>
                </pre>

                <h2>Queue choice</h2>
                <p>
                    Separate queue names by workload when it helps operations, for example <code>default</code>, <code>mail</code>, <code>realtime</code>, or <code>maintenance</code>. Do not create queue names casually if there is no operational reason to separate them.
                </p>

                <h2>Verification</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo check -p app
./bin/worker
# trigger the workflow that dispatches the job
# confirm worker logs show job execution
# inspect /_jobs/failed when forcing a failure`}</code>
                </pre>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/jobs">Job Queue</a> for the framework-level runtime view.</li>
                    <li><a href="#/notifications">Notifications</a> if the job is part of a notification flow.</li>
                    <li><a href="#/cookbook/test-the-flow">Test the Flow</a> for where to put job and failed-job regression tests.</li>
                </ul>
            </div>
        </div>
    )
}
