import { Menu } from 'lucide-react'

interface HeaderProps {
    onOpenSidebar: () => void
}

export function Header({ onOpenSidebar }: HeaderProps) {
    return (
        <header className="lg:hidden fixed top-0 left-0 right-0 z-30 h-16 bg-white border-b border-gray-200 flex items-center px-4">
            <button
                onClick={onOpenSidebar}
                className="p-2 rounded-md hover:bg-gray-100"
                aria-label="Open menu"
            >
                <Menu className="w-6 h-6 text-gray-600" />
            </button>
            <span className="ml-3 text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-orange-600 to-amber-600">
                Rustforge
            </span>
        </header>
    )
}
