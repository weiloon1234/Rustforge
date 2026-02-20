import { useState, useEffect, useSyncExternalStore } from 'react'

function getHash() {
    return window.location.hash || '#/'
}

function subscribe(callback: () => void) {
    window.addEventListener('hashchange', callback)
    return () => window.removeEventListener('hashchange', callback)
}

export function useRouter() {
    const hash = useSyncExternalStore(subscribe, getHash, getHash)

    const navigate = (path: string) => {
        window.location.hash = path
    }

    const isActive = (href: string) => hash === href

    return { hash, navigate, isActive }
}
