import { useRouter } from '@/hooks/useRouter'
import { cn } from '@/lib/utils'

interface SidebarProps {
    isOpen: boolean
    onClose: () => void
}

const sections = [
    {
        title: 'Getting Started',
        items: [
            { title: 'Introduction', href: '#/' },
            { title: 'Installation', href: '#/installation' },
            { title: 'Quick Start', href: '#/quick-start' },
            { title: 'Directory Structure', href: '#/directory-structure' },
        ],
    },
    {
        title: 'Core Architecture',
        items: [
            { title: 'Bootstrap', href: '#/bootstrap' },
            { title: 'Configuration', href: '#/configuration' },
            { title: 'CLI Commands', href: '#/cli' },
        ],
    },
    {
        title: 'Cookbook',
        items: [
            { title: 'Overview', href: '#/cookbook' },
            { title: 'Chapter 1: CRUD API', href: '#/cookbook-chapter-1-crud-api-server' },
            {
                title: 'Chapter 2A: DTO + Validation',
                href: '#/cookbook-chapter-2-validation-dto',
            },
            {
                title: 'Chapter 2B: Admin Auth',
                href: '#/cookbook-chapter-2-admin-auth',
            },
            { title: 'Chapter 3: Jobs Usage', href: '#/cookbook-chapter-3-jobs-usage' },
            { title: 'Chapter 4: Notifications', href: '#/cookbook-chapter-4-notifications' },
            { title: 'Chapter 5: WebSocket Channel', href: '#/cookbook-chapter-5-websocket-channel' },
            {
                title: 'Chapter 6: WebSocket Auth',
                href: '#/cookbook-chapter-6-websocket-auth-middleware',
            },
            { title: 'Chapter 7: CLI Workflow', href: '#/cookbook-chapter-7-cli-workflow' },
            { title: 'Chapter 8: End-to-End', href: '#/cookbook-chapter-8-end-to-end-flow' },
            {
                title: 'Chapter 9: Hardening',
                href: '#/cookbook-chapter-9-production-hardening',
            },
        ],
    },
    {
        title: 'Framework Features',
        items: [
            { title: 'Overview', href: '#/framework-features' },
            { title: 'Meta', href: '#/feature-meta' },
            { title: 'Attachments', href: '#/feature-attachments' },
            { title: 'Localized + Relationships', href: '#/feature-localized-relations' },
            { title: 'Realtime / WebSocket', href: '#/feature-realtime' },
            { title: 'Realtime Protocol + Runbook', href: '#/feature-realtime-protocol' },
        ],
    },
    {
        title: 'DataTable',
        items: [{ title: 'AutoDataTable', href: '#/feature-autodatatable' }],
    },
    {
        title: 'Database & Models',
        items: [
            { title: 'Schema Definition', href: '#/schema' },
            { title: 'Code Generation', href: '#/db-gen' },
            { title: 'Model API Overview', href: '#/model-api' },
            { title: 'Xxx (Facade)', href: '#/model-api-facade' },
            { title: 'XxxQuery', href: '#/model-api-query' },
            { title: 'XxxInsert', href: '#/model-api-insert' },
            { title: 'XxxUpdate', href: '#/model-api-update' },
            { title: 'XxxView', href: '#/model-api-view' },
            { title: 'XxxCol / Filtering', href: '#/model-api-columns' },
            { title: 'Relations / Joins', href: '#/model-api-relations' },
            { title: 'Unsafe SQL Escape', href: '#/model-api-unsafe' },
            { title: 'Collections', href: '#/model-api-collections' },
            { title: 'Meta / Attach / Localized', href: '#/model-api-features' },
            { title: 'ActiveRecord', href: '#/active-record' },
            { title: 'Migrations', href: '#/migrations' },
        ],
    },
    {
        title: 'HTTP & API',
        items: [
            { title: 'Routing', href: '#/routing' },
            { title: 'Requests & Validation', href: '#/requests' },
            { title: 'Validation Rules', href: '#/validation-rules' },
            { title: 'Responses', href: '#/responses' },
            { title: 'OpenAPI', href: '#/openapi' },
            { title: 'Internationalization', href: '#/i18n' },
            { title: 'HTTP & Webhook Logs', href: '#/http-log' },
        ],
    },
    {
        title: 'Security',
        items: [
            { title: 'Guards & Auth', href: '#/auth' },
            { title: 'Permissions & AuthZ', href: '#/permissions' },
        ],
    },
    {
        title: 'Async & Jobs',
        items: [
            { title: 'Job Queue', href: '#/jobs' },
            { title: 'Notifications', href: '#/notifications' },
            { title: 'Cron Scheduler', href: '#/scheduler' },
        ],
    },
]

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
                    {sections.map((section) => (
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
