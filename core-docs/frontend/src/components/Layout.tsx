import { useState, type ReactNode } from 'react'
import { Sidebar } from './Sidebar'
import { Header } from './Header'

interface LayoutProps {
    children: ReactNode
}

export function Layout({ children }: LayoutProps) {
    const [isSidebarOpen, setIsSidebarOpen] = useState(false)

    return (
        <div className="min-h-screen bg-gray-50 text-gray-900 font-sans">
            <div className="flex h-screen overflow-hidden">
                <Sidebar
                    isOpen={isSidebarOpen}
                    onClose={() => setIsSidebarOpen(false)}
                />

                <div className="flex-1 flex flex-col min-w-0 overflow-hidden relative">
                    <Header onOpenSidebar={() => setIsSidebarOpen(true)} />

                    <main className="flex-1 overflow-y-auto pt-16 lg:pt-0 p-4 lg:p-10 scroll-smooth">
                        <div className="max-w-4xl mx-auto pb-20">{children}</div>
                    </main>
                </div>
            </div>
        </div>
    )
}
