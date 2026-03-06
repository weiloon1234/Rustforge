import type { ComponentType } from 'react'

import { CookbookOverview } from './pages/cookbook/CookbookOverview'
import { Chapter1CrudApiServer } from './pages/cookbook/Chapter1CrudApiServer'
import { AddAdminDatatable } from './pages/cookbook/AddAdminDatatable'
import { Chapter2ValidationDto } from './pages/cookbook/Chapter2ValidationDto'
import { Chapter2AdminAuth } from './pages/cookbook/Chapter2AdminAuth'
import { Chapter3JobsUsage } from './pages/cookbook/Chapter3JobsUsage'
import { Chapter4NotificationsUsage } from './pages/cookbook/Chapter4NotificationsUsage'
import { Chapter5WebsocketChannel } from './pages/cookbook/Chapter5WebsocketChannel'
import { Chapter6WebsocketAuthMiddleware } from './pages/cookbook/Chapter6WebsocketAuthMiddleware'
import { Chapter7CliWorkflow } from './pages/cookbook/Chapter7CliWorkflow'
import { Chapter8EndToEndFlow } from './pages/cookbook/Chapter8EndToEndFlow'
import { Chapter9ProductionHardening } from './pages/cookbook/Chapter9ProductionHardening'
import { Chapter10CachingRecipe } from './pages/cookbook/Chapter10CachingRecipe'
import { Chapter11TestingRecipe } from './pages/cookbook/Chapter11TestingRecipe'
import { Chapter12EventFanOutRecipe } from './pages/cookbook/Chapter12EventFanOutRecipe'
import { ActiveRecord } from './pages/database/ActiveRecord'
import { DbGen } from './pages/database/DbGen'
import { GeneratedApi } from './pages/database/GeneratedApi'
import { Migrations } from './pages/database/Migrations'
import { Schema } from './pages/database/Schema'
import { ModelApiColumns } from './pages/database/model-api/Columns'
import { ModelApiCollections } from './pages/database/model-api/Collections'
import { ModelApiFacade } from './pages/database/model-api/Facade'
import { ModelApiFeatures } from './pages/database/model-api/FrameworkFeatures'
import { ModelApiInsert } from './pages/database/model-api/Insert'
import { ModelApiQuery } from './pages/database/model-api/Query'
import { ModelApiRelations } from './pages/database/model-api/Relations'
import { ModelApiUnsafe } from './pages/database/model-api/UnsafeSql'
import { ModelApiUpdate } from './pages/database/model-api/Update'
import { ModelApiView } from './pages/database/model-api/View'
import { AttachmentsFeature } from './pages/framework-features/AttachmentsFeature'
import { AutoDataTableFeature } from './pages/framework-features/AutoDataTable'
import { FrameworkFeatures } from './pages/framework-features/FrameworkFeatures'
import { LocalizedRelationsFeature } from './pages/framework-features/LocalizedRelationsFeature'
import { MetaFeature } from './pages/framework-features/MetaFeature'
import { RealtimeProtocolStateMachineFeature } from './pages/framework-features/RealtimeProtocolStateMachine'
import { RealtimeWebSocketFeature } from './pages/framework-features/RealtimeWebSocket'
import { Bootstrap } from './pages/getting-started/Bootstrap'
import { Cli } from './pages/getting-started/Cli'
import { Configuration } from './pages/getting-started/Configuration'
import { DirectoryStructure } from './pages/getting-started/DirectoryStructure'
import { Installation } from './pages/getting-started/Installation'
import { Introduction } from './pages/getting-started/Introduction'
import { QuickStart } from './pages/getting-started/QuickStart'
import { HttpLog } from './pages/http/HttpLog'
import { I18n } from './pages/http/I18n'
import { OpenApi } from './pages/http/OpenApi'
import { Requests } from './pages/http/Requests'
import { Responses } from './pages/http/Responses'
import { Routing } from './pages/http/Routing'
import { ValidationRules } from './pages/http/ValidationRules'
import { Jobs } from './pages/async/Jobs'
import { Notifications } from './pages/async/Notifications'
import { Scheduler } from './pages/async/Scheduler'
import { Caching } from './pages/async/Caching'
import { Guards } from './pages/security/Guards'
import { PermissionMatrix } from './pages/security/PermissionMatrix'
import { Permissions } from './pages/security/Permissions'

export interface DocsPage {
    title: string
    href: string
    component: ComponentType
}

export interface DocsSection {
    title: string
    items: DocsPage[]
}

export const docsSections: DocsSection[] = [
    {
        title: 'Start Here',
        items: [
            { title: 'Introduction', href: '#/', component: Introduction },
            { title: 'Installation', href: '#/installation', component: Installation },
            { title: 'Quick Start', href: '#/quick-start', component: QuickStart },
            {
                title: 'Directory Structure',
                href: '#/directory-structure',
                component: DirectoryStructure,
            },
            { title: 'Bootstrap Runtime', href: '#/bootstrap', component: Bootstrap },
            { title: 'Configuration', href: '#/configuration', component: Configuration },
            { title: 'CLI Commands', href: '#/cli', component: Cli },
        ],
    },
    {
        title: 'Framework Features',
        items: [
            { title: 'Overview', href: '#/framework-features', component: FrameworkFeatures },
            { title: 'AutoDataTable', href: '#/feature-autodatatable', component: AutoDataTableFeature },
            { title: 'Meta', href: '#/feature-meta', component: MetaFeature },
            {
                title: 'Attachments',
                href: '#/feature-attachments',
                component: AttachmentsFeature,
            },
            {
                title: 'Localized & Relationships',
                href: '#/feature-localized-relations',
                component: LocalizedRelationsFeature,
            },
            {
                title: 'Realtime / WebSocket',
                href: '#/feature-realtime',
                component: RealtimeWebSocketFeature,
            },
            {
                title: 'Realtime Protocol & Runbook',
                href: '#/feature-realtime-protocol',
                component: RealtimeProtocolStateMachineFeature,
            },
            { title: 'Jobs & Queue', href: '#/jobs', component: Jobs },
            { title: 'Notifications', href: '#/notifications', component: Notifications },
            { title: 'Scheduler & Cron', href: '#/scheduler', component: Scheduler },
            { title: 'Caching', href: '#/caching', component: Caching },
        ],
    },
    {
        title: 'HTTP & API',
        items: [
            { title: 'Routing', href: '#/routing', component: Routing },
            { title: 'Requests & Validation', href: '#/requests', component: Requests },
            {
                title: 'Validation Rules',
                href: '#/validation-rules',
                component: ValidationRules,
            },
            { title: 'Responses', href: '#/responses', component: Responses },
            { title: 'OpenAPI', href: '#/openapi', component: OpenApi },
            { title: 'Guards & Auth', href: '#/auth', component: Guards },
            { title: 'Permissions & AuthZ', href: '#/permissions', component: Permissions },
            { title: 'Permission Matrix', href: '#/permission-matrix', component: PermissionMatrix },
            { title: 'Internationalization', href: '#/i18n', component: I18n },
            { title: 'HTTP & Webhook Logs', href: '#/http-log', component: HttpLog },
        ],
    },
    {
        title: 'Database & Models',
        items: [
            { title: 'Schema Definition', href: '#/schema', component: Schema },
            { title: 'Code Generation', href: '#/db-gen', component: DbGen },
            { title: 'Model API Overview', href: '#/model-api', component: GeneratedApi },
            { title: 'Xxx (Facade)', href: '#/model-api-facade', component: ModelApiFacade },
            { title: 'XxxQuery', href: '#/model-api-query', component: ModelApiQuery },
            { title: 'XxxInsert', href: '#/model-api-insert', component: ModelApiInsert },
            { title: 'XxxUpdate', href: '#/model-api-update', component: ModelApiUpdate },
            {
                title: 'XxxView & Extensions',
                href: '#/model-api-view',
                component: ModelApiView,
            },
            {
                title: 'XxxCol & Filtering',
                href: '#/model-api-columns',
                component: ModelApiColumns,
            },
            {
                title: 'Relations & Joins',
                href: '#/model-api-relations',
                component: ModelApiRelations,
            },
            { title: 'Unsafe SQL', href: '#/model-api-unsafe', component: ModelApiUnsafe },
            { title: 'Collections', href: '#/model-api-collections', component: ModelApiCollections },
            {
                title: 'Framework Features on Models',
                href: '#/model-api-features',
                component: ModelApiFeatures,
            },
            { title: 'ActiveRecord', href: '#/active-record', component: ActiveRecord },
            { title: 'Migrations', href: '#/migrations', component: Migrations },
        ],
    },
    {
        title: 'Cookbook',
        items: [
            { title: 'Overview', href: '#/cookbook', component: CookbookOverview },
            {
                title: 'Build a CRUD Admin Resource',
                href: '#/cookbook/build-crud-admin-resource',
                component: Chapter1CrudApiServer,
            },
            {
                title: 'Add an Admin DataTable',
                href: '#/cookbook/add-admin-datatable',
                component: AddAdminDatatable,
            },
            {
                title: 'Add Validation Contracts',
                href: '#/cookbook/add-validation-contracts',
                component: Chapter2ValidationDto,
            },
            {
                title: 'Add Admin Auth & Permission Gates',
                href: '#/cookbook/add-admin-auth-permission-gates',
                component: Chapter2AdminAuth,
            },
            {
                title: 'Add Jobs',
                href: '#/cookbook/add-jobs',
                component: Chapter3JobsUsage,
            },
            {
                title: 'Add Notifications',
                href: '#/cookbook/add-notifications',
                component: Chapter4NotificationsUsage,
            },
            {
                title: 'Add a Realtime Channel',
                href: '#/cookbook/add-realtime-channel',
                component: Chapter5WebsocketChannel,
            },
            {
                title: 'Add WebSocket Auth',
                href: '#/cookbook/add-websocket-auth',
                component: Chapter6WebsocketAuthMiddleware,
            },
            {
                title: 'Add a Console Workflow',
                href: '#/cookbook/add-console-workflow',
                component: Chapter7CliWorkflow,
            },
            {
                title: 'Build an End-to-End Flow',
                href: '#/cookbook/build-end-to-end-flow',
                component: Chapter8EndToEndFlow,
            },
            {
                title: 'Production Hardening',
                href: '#/cookbook/production-hardening',
                component: Chapter9ProductionHardening,
            },
            {
                title: 'Add Caching',
                href: '#/cookbook/add-caching',
                component: Chapter10CachingRecipe,
            },
            {
                title: 'Test the Flow',
                href: '#/cookbook/test-the-flow',
                component: Chapter11TestingRecipe,
            },
            {
                title: 'Fan-out Events',
                href: '#/cookbook/fan-out-events',
                component: Chapter12EventFanOutRecipe,
            },
        ],
    },
]

export const docsRouteMap: Record<string, ComponentType> = Object.fromEntries(
    docsSections.flatMap((section) =>
        section.items.map((item) => [item.href, item.component] as const)
    )
)
