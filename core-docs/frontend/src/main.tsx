import './app.css'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'

// PrismJS
import 'prismjs'
import 'prismjs/themes/prism-tomorrow.css' // Dark theme
import 'prismjs/components/prism-rust'
import 'prismjs/components/prism-toml'
import 'prismjs/components/prism-bash'
import 'prismjs/components/prism-json'
import 'prismjs/components/prism-sql'

createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <App />
    </StrictMode>,
)
