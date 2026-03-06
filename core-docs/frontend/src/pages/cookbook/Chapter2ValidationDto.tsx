import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter2ValidationDto() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Recipe: Add Validation Contracts
                </h1>
                <p className="text-xl text-gray-500">
                    Keep contract DTOs as SSOT for runtime validation, OpenAPI schema, and TypeScript export.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Define DTOs once in Rust and reuse them across handler boundary validation, OpenAPI docs, and
                    frontend type generation.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Contract modules: <code>app/src/contracts/api/v1/admin/*.rs</code>
                    </li>
                    <li>
                        Datatable contracts: <code>app/src/contracts/datatable/admin/*.rs</code>
                    </li>
                    <li>
                        Scoped constants in each datatable contract: <code>SCOPED_KEY</code> +{' '}
                        <code>ROUTE_PREFIX</code>
                    </li>
                    <li>
                        Handler boundary extractors: <code>ContractJson&lt;T&gt;</code> /{' '}
                        <code>AsyncContractJson&lt;T&gt;</code>
                    </li>
                    <li>
                        Type export pipeline: <code>app/build.rs</code> +{' '}
                        <code>app/src/bin/export-types.rs</code>
                    </li>
                </ul>

                <h3>DTO baseline</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[rustforge_contract]
#[derive(ts_rs::TS)]
#[ts(export, export_to = "admin/types/")]
pub struct AdminPasswordUpdateInput {
    #[rf(length(min = 8, max = 128))]
    pub current_password: String,

    #[rf(length(min = 8, max = 128))]
    #[rf(must_match(other = "password_confirmation"))]
    pub password: String,

    #[rf(length(min = 8, max = 128))]
    pub password_confirmation: String,
}`}</code>
                </pre>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>
                        Use wrapper rule types under <code>app/src/contracts/types/</code> for reusable project rules.
                    </li>
                    <li>
                        For PATCH uniqueness checks using path ID, use hidden target field + explicit async validate in
                        handler before workflow.
                    </li>
                    <li>
                        Add new portal DTO folders under <code>app/src/contracts/api/v1/&lt;portal&gt;/</code>; no export
                        registry edits needed because discovery is build-time automatic.
                    </li>
                </ul>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`cargo check -p app
make gen-types
ls frontend/src/admin/types
cat frontend/src/shared/types/platform.ts`}</code>
                </pre>
            </div>
        </div>
    )
}
