import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter11TestingRecipe() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Recipe: Test the Flow
                </h1>
                <p className="text-xl text-gray-500">
                    Test by layer: contract semantics, generator output, datatable behavior, starter output, and app-specific workflows.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Testing strategy</h2>
                <p>
                    Keep tests aligned with the actual framework layers. Do not collapse everything into one large end-to-end test if a smaller layer already owns the guarantee.
                </p>
                <ol>
                    <li>Contract and validation semantics</li>
                    <li>Generator output and typed API shape</li>
                    <li>Feature behavior such as datatable filter/mapping/export parity</li>
                    <li>Starter generation and dependency policy</li>
                    <li>App-specific workflow and handler integration</li>
                </ol>

                <h2>What the repo already tests</h2>
                <ul>
                    <li><code>core-web/tests/rustforge_contract.rs</code>: contract macro, validation rules, wrapper types, and <code>Patch&lt;T&gt;</code> semantics.</li>
                    <li><code>db-gen/tests/template_generation.rs</code>: checked-in fixture parity for generator output.</li>
                    <li><code>db-gen/tests/typed_first_generation.rs</code>: generated API behavior such as typed-first surfaces, enum explained fields, and PK-type correctness.</li>
                    <li><code>core-datatable/src/tests.rs</code>: datatable filter parsing, typed mapping hooks, and record/export shaping.</li>
                    <li><code>scaffold/tests/scaffold_smoke.rs</code>: fresh starter generation plus compile gate.</li>
                    <li><code>scaffold/tests/template_workspace_deps.rs</code>: scaffold template uses git dependencies, not local path crates.</li>
                    <li><code>scaffold/tests/lint_allow_policy.rs</code>: disallowed suppression policy.</li>
                </ul>

                <h2>Layer 1: contract semantics</h2>
                <p>
                    Test DTO behavior where the contract itself owns the rule: wrapper types, async validators, and <code>Option&lt;T&gt;</code> vs <code>Patch&lt;T&gt;</code> semantics.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rustforge_contract]
struct PatchEmailInput {
    #[serde(default)]
    #[rf(email)]
    email: Patch<String>,
}

#[test]
fn patch_email_validation_behaves_as_expected() {
    // missing, null, valid value, invalid value
}`}</code>
                </pre>
                <p>
                    If the rule is reusable across multiple DTOs, prefer a wrapper type and test that wrapper once instead of repeating the same assertions on many request structs.
                </p>

                <h2>Layer 2: generator behavior</h2>
                <p>
                    Test generator guarantees with fixtures and targeted assertions, not by manually inspecting generated source every time.
                </p>
                <ul>
                    <li>Fixture parity for large generated outputs</li>
                    <li>Targeted assertions for important generated symbols and method families</li>
                    <li>Regression tests for PK-type-specific paths such as UUID and string PKs</li>
                </ul>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo test -p db-gen`}</code>
                </pre>

                <h2>Layer 3: feature behavior</h2>
                <p>
                    For framework features like AutoDataTable, test the pipeline behavior directly: filter parsing, typed row hooks, and row-to-record shaping.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[tokio::test]
async fn datatable_uses_typed_mapping_for_query_results() {
    // execute_datatable(...)
    // assert map_row mutation and row_to_record output are both present
}`}</code>
                </pre>
                <p>
                    This is where query/export parity should be tested. The same mapping path must feed both page responses and CSV export.
                </p>

                <h2>Layer 4: starter generation checks</h2>
                <p>
                    Treat scaffold generation as a product surface. Test the starter output directly instead of assuming template edits are safe.
                </p>
                <ul>
                    <li>starter output contains the expected files</li>
                    <li>starter output does not ship <code>Cargo.lock</code></li>
                    <li>fresh generated output compiles</li>
                    <li>template dependencies still point to the framework git repo</li>
                </ul>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo test -p scaffold`}</code>
                </pre>

                <h2>Layer 5: add app-specific tests deliberately</h2>
                <p>
                    The scaffold app does not ship an <code>app/tests/</code> directory yet. Add one when a project-level workflow or permission rule deserves its own regression coverage.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`// app/tests/admin_permissions.rs
#[tokio::test]
async fn normal_admin_cannot_assign_admin_manage() {
    // seed actor + target
    // call workflow or handler
    // assert forbidden
}`}</code>
                </pre>
                <p>
                    Test app business rules where they live. For example, starter admin delegation rules belong in workflow or handler tests, not in generic framework matcher tests.
                </p>

                <h2>Frontend test boundary</h2>
                <p>
                    The scaffold frontend currently enforces <code>typecheck</code> and <code>build</code> as the baseline gate. Add page/store tests when portal-specific UI logic becomes complex enough to deserve its own regression suite.
                </p>
                <ul>
                    <li>typed auth-store helpers</li>
                    <li>datatable column/export behavior</li>
                    <li>permission-gated UI visibility</li>
                    <li>runtime bootstrap parsing helpers</li>
                </ul>

                <h2>Recommended command set</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`cargo test -p core-web --test rustforge_contract
cargo test -p db-gen
cargo test -p scaffold
cargo test -p core-datatable
npm --prefix core-docs/frontend run build
cargo check --workspace`}</code>
                </pre>

                <h2>Practical rule</h2>
                <p>
                    Put the assertion at the layer that actually owns the invariant. If the framework matcher owns the rule, test the matcher. If the scaffold workflow owns the rule, test the workflow. If the generated output shape owns the rule, test generation.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/requests">Requests &amp; Validation</a> for contract-boundary semantics.</li>
                    <li><a href="#/feature-autodatatable">AutoDataTable</a> for the datatable pipeline that feature tests should cover.</li>
                    <li><a href="#/cookbook/build-end-to-end-flow">Build an End-to-End Flow</a> for a vertical slice to regression-test.</li>
                </ul>
            </div>
        </div>
    )
}
