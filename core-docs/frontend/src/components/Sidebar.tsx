import { useRouter } from '@/hooks/useRouter'
import { cn } from '@/lib/utils'
import { docsSections } from '@/docsRegistry'

interface SidebarProps {
    isOpen: boolean
    onClose: () => void
}

export function Sidebar({ isOpen, onClose }: SidebarProps) {
    const { isActive } = useRouter()

    return (
        <>
            <aside
                className={cn(
                    'fixed inset-y-0 left-0 z-50 w-64 bg-white border-r border-gray-200 transform transition-transform duration-200 ease-in-out lg:relative lg:translate-x-0 h-screen overflow-y-auto',
                    isOpen ? 'translate-x-0' : '-translate-x-full'
                )}
            >
                <div className="h-16 flex items-center px-6 border-b border-gray-200">
                    <span className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-orange-600 to-amber-600">
                        Rustforge
                    </span>
                </div>

                <nav className="p-4 space-y-8">
                    {docsSections.map((section) => (
                        <div key={section.title}>
                            <h4 className="mb-2 px-2 text-sm font-semibold text-gray-900 uppercase tracking-wider">
                                {section.title}
                            </h4>
                            <div className="space-y-1">
                                {section.items.map((item) => (
                                    <a
                                        key={item.href}
                                        href={item.href}
                                        onClick={onClose}
                                        className={cn(
                                            'block px-2 py-1.5 text-sm rounded-md transition-colors',
                                            isActive(item.href)
                                                ? 'bg-orange-50 text-orange-700 font-medium'
                                                : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
                                        )}
                                    >
                                        {item.title}
                                    </a>
                                ))}
                            </div>
                        </div>
                    ))}
                </nav>
            </aside>

            {isOpen && (
                <div
                    className="fixed inset-0 bg-black/50 z-40 lg:hidden"
                    onClick={onClose}
                    aria-hidden="true"
                />
            )}
        </>
    )
}
