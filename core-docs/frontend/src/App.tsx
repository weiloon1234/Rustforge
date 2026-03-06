import { useEffect } from 'react'
import Prism from 'prismjs'

import { Layout } from './components/Layout'
import { docsRouteMap } from './docsRegistry'
import { useRouter } from './hooks/useRouter'

export default function App() {
    const { hash } = useRouter()
    const ActiveComponent = docsRouteMap[hash]

    useEffect(() => {
        Prism.highlightAll()
    }, [hash, ActiveComponent])

    return (
        <Layout>
            {ActiveComponent ? (
                <ActiveComponent />
            ) : (
                <div className="prose prose-orange max-w-none">
                    <h1>404</h1>
                    <p>Page not found.</p>
                </div>
            )}
        </Layout>
    )
}
