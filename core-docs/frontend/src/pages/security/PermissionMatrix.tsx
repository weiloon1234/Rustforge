export function PermissionMatrix() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Permission Matrix</h1>
                <p className="text-xl text-gray-500">
                    Short operational reference for matching, wildcard behavior, and starter admin delegation rules.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Matcher cheat sheet</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Granted</th>
                            <th>Required</th>
                            <th>Result</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr><td><code>*</code></td><td><code>x.read</code></td><td>allow</td></tr>
                        <tr><td><code>x.*</code></td><td><code>x.manage</code></td><td>allow</td></tr>
                        <tr><td><code>x.manage</code></td><td><code>x.read</code></td><td>allow</td></tr>
                        <tr><td><code>x.read</code></td><td><code>x.manage</code></td><td>deny</td></tr>
                        <tr><td><code>export</code></td><td><code>x.read</code></td><td>deny</td></tr>
                        <tr><td><code>x.manage</code></td><td><code>y.read</code></td><td>deny</td></tr>
                    </tbody>
                </table>

                <h2>Actor matrix in the starter admin area</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Actor</th>
                            <th>Normal permission gates</th>
                            <th>Delegation</th>
                            <th>Admin-table special rules</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td><code>developer</code></td>
                            <td>allow</td>
                            <td>can assign any permission</td>
                            <td>treated as full access</td>
                        </tr>
                        <tr>
                            <td><code>superadmin</code></td>
                            <td>allow</td>
                            <td>can assign any permission</td>
                            <td>treated as full access for starter admin flows</td>
                        </tr>
                        <tr>
                            <td><code>admin</code></td>
                            <td>only owned/implied scopes</td>
                            <td>can only assign owned/implied scopes</td>
                            <td>cannot assign <code>admin.read</code> or <code>admin.manage</code>; cannot edit/delete self in admin CRUD table</td>
                        </tr>
                    </tbody>
                </table>

                <h2>Examples</h2>
                <ul>
                    <li>If an admin owns <code>content_page.manage</code>, they pass checks for <code>content_page.read</code>.</li>
                    <li>If an admin owns <code>content_page.manage</code>, the starter UI may still allow assigning both <code>content_page.manage</code> and <code>content_page.read</code>.</li>
                    <li>If an admin owns <code>admin.manage</code>, starter policy still blocks delegating <code>admin.manage</code> and <code>admin.read</code> to another normal admin.</li>
                    <li>If a token owns <code>export</code> only, it still cannot query feature-specific datatables without that feature’s read/manage permission.</li>
                </ul>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/permissions">Permissions &amp; AuthZ</a> for the full explanation.</li>
                    <li><a href="#/auth">Guards &amp; Auth</a> for token/session flow.</li>
                    <li><a href="#/cookbook/add-admin-auth-permission-gates">Add Admin Auth &amp; Permission Gates</a> for the starter recipe.</li>
                </ul>
            </div>
        </div>
    )
}
