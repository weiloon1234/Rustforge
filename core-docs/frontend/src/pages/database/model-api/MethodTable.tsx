export function MethodTable({
    rows,
}: {
    rows: Array<{ method: string; returns: string; notes: string }>
}) {
    return (
        <div className="overflow-x-auto">
            <table className="min-w-full text-sm border-collapse border border-gray-200">
                <thead className="bg-gray-100">
                    <tr>
                        <th className="border p-2 text-left">Method</th>
                        <th className="border p-2 text-left">Returns</th>
                        <th className="border p-2 text-left">Notes</th>
                    </tr>
                </thead>
                <tbody>
                    {rows.map((row) => (
                        <tr key={row.method}>
                            <td className="border p-2 font-mono">{row.method}</td>
                            <td className="border p-2 font-mono">{row.returns}</td>
                            <td className="border p-2">{row.notes}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    )
}
