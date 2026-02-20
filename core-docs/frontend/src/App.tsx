import { Layout } from './components/Layout'
import { useRouter } from './hooks/useRouter'

// Getting Started
import { Introduction } from './pages/getting-started/Introduction'
import { Installation } from './pages/getting-started/Installation'
import { QuickStart } from './pages/getting-started/QuickStart'
import { DirectoryStructure } from './pages/getting-started/DirectoryStructure'
import { Bootstrap } from './pages/getting-started/Bootstrap'
import { Configuration } from './pages/getting-started/Configuration'
import { Cli } from './pages/getting-started/Cli'

// Cookbook
import { CookbookOverview } from './pages/cookbook/CookbookOverview'
import { Chapter1CrudApiServer } from './pages/cookbook/Chapter1CrudApiServer'
import { Chapter2AdminAuth } from './pages/cookbook/Chapter2AdminAuth'
import { Chapter3JobsUsage } from './pages/cookbook/Chapter3JobsUsage'
import { Chapter4NotificationsUsage } from './pages/cookbook/Chapter4NotificationsUsage'
import { Chapter5WebsocketChannel } from './pages/cookbook/Chapter5WebsocketChannel'
import { Chapter6WebsocketAuthMiddleware } from './pages/cookbook/Chapter6WebsocketAuthMiddleware'
import { Chapter7CliWorkflow } from './pages/cookbook/Chapter7CliWorkflow'
import { Chapter7EndToEndFlow } from './pages/cookbook/Chapter7EndToEndFlow'
import { Chapter8ProductionHardening } from './pages/cookbook/Chapter8ProductionHardening'

// Framework Features
import { FrameworkFeatures } from './pages/framework-features/FrameworkFeatures'
import { MetaFeature } from './pages/framework-features/MetaFeature'
import { AttachmentsFeature } from './pages/framework-features/AttachmentsFeature'
import { LocalizedRelationsFeature } from './pages/framework-features/LocalizedRelationsFeature'
import { AutoDataTableFeature } from './pages/framework-features/AutoDataTable'
import { RealtimeWebSocketFeature } from './pages/framework-features/RealtimeWebSocket'
import { RealtimeProtocolStateMachineFeature } from './pages/framework-features/RealtimeProtocolStateMachine'

// Database
import { Schema } from './pages/database/Schema'
import { DbGen } from './pages/database/DbGen'
import { ActiveRecord } from './pages/database/ActiveRecord'
import { Migrations } from './pages/database/Migrations'
import { GeneratedApi } from './pages/database/GeneratedApi'
import { ModelApiFacade } from './pages/database/model-api/Facade'
import { ModelApiQuery } from './pages/database/model-api/Query'
import { ModelApiInsert } from './pages/database/model-api/Insert'
import { ModelApiUpdate } from './pages/database/model-api/Update'
import { ModelApiView } from './pages/database/model-api/View'
import { ModelApiColumns } from './pages/database/model-api/Columns'
import { ModelApiRelations } from './pages/database/model-api/Relations'
import { ModelApiUnsafe } from './pages/database/model-api/UnsafeSql'
import { ModelApiCollections } from './pages/database/model-api/Collections'
import { ModelApiFeatures } from './pages/database/model-api/FrameworkFeatures'

// HTTP
import { Routing } from './pages/http/Routing'
import { Requests } from './pages/http/Requests'
import { ValidationRules } from './pages/http/ValidationRules'
import { Responses } from './pages/http/Responses'
import { OpenApi } from './pages/http/OpenApi'
import { I18n } from './pages/http/I18n'
import { HttpLog } from './pages/http/HttpLog'

// Async & Security
import { Guards } from './pages/security/Guards'
import { Permissions } from './pages/security/Permissions'
import { Jobs } from './pages/async/Jobs'
import { Notifications } from './pages/async/Notifications'
import { Scheduler } from './pages/async/Scheduler'

import type { ComponentType } from 'react'

const routeMap: Record<string, ComponentType> = {
    // Getting Started
    '#/': Introduction,
    '#/installation': Installation,
    '#/quick-start': QuickStart,
    '#/directory-structure': DirectoryStructure,
    '#/bootstrap': Bootstrap,
    '#/configuration': Configuration,
    '#/cli': Cli,

    // Cookbook
    '#/cookbook': CookbookOverview,
    '#/cookbook-chapter-1-crud-api-server': Chapter1CrudApiServer,
    '#/cookbook-chapter-2-validation-dto': Chapter2AdminAuth,
    '#/cookbook-chapter-2-admin-auth': Chapter2AdminAuth,
    '#/cookbook-chapter-3-jobs-usage': Chapter3JobsUsage,
    '#/cookbook-chapter-4-notifications': Chapter4NotificationsUsage,
    '#/cookbook-chapter-5-websocket-channel': Chapter5WebsocketChannel,
    '#/cookbook-chapter-6-websocket-auth-middleware': Chapter6WebsocketAuthMiddleware,
    '#/cookbook-chapter-7-cli-workflow': Chapter7CliWorkflow,
    '#/cookbook-chapter-8-end-to-end-flow': Chapter7EndToEndFlow,
    '#/cookbook-chapter-9-production-hardening': Chapter8ProductionHardening,

    // compatibility aliases for existing shared links
    '#/cookbook-chapter-7-end-to-end-flow': Chapter7EndToEndFlow,
    '#/cookbook-chapter-8-production-hardening': Chapter8ProductionHardening,

    // Framework Features
    '#/framework-features': FrameworkFeatures,
    '#/feature-meta': MetaFeature,
    '#/feature-attachments': AttachmentsFeature,
    '#/feature-localized-relations': LocalizedRelationsFeature,
    '#/feature-autodatatable': AutoDataTableFeature,
    '#/feature-realtime': RealtimeWebSocketFeature,
    '#/feature-realtime-protocol': RealtimeProtocolStateMachineFeature,

    // Database
    '#/schema': Schema,
    '#/db-gen': DbGen,
    '#/generated-model-api': GeneratedApi,
    '#/model-api': GeneratedApi,
    '#/model-api-facade': ModelApiFacade,
    '#/model-api-query': ModelApiQuery,
    '#/model-api-insert': ModelApiInsert,
    '#/model-api-update': ModelApiUpdate,
    '#/model-api-view': ModelApiView,
    '#/model-api-columns': ModelApiColumns,
    '#/model-api-relations': ModelApiRelations,
    '#/model-api-unsafe': ModelApiUnsafe,
    '#/model-api-collections': ModelApiCollections,
    '#/model-api-features': ModelApiFeatures,
    '#/active-record': ActiveRecord,
    '#/migrations': Migrations,

    // HTTP
    '#/routing': Routing,
    '#/requests': Requests,
    '#/validation-rules': ValidationRules,
    '#/responses': Responses,
    '#/openapi': OpenApi,
    '#/i18n': I18n,
    '#/http-log': HttpLog,

    // Security & Async
    '#/auth': Guards,
    '#/permissions': Permissions,
    '#/jobs': Jobs,
    '#/notifications': Notifications,
    '#/scheduler': Scheduler,
}

import { useEffect } from 'react'
import Prism from 'prismjs'

export default function App() {
    const { hash } = useRouter()
    const ActiveComponent = routeMap[hash]

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
