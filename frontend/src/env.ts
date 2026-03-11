export type Env = {
    VITE_API_BASE_URL: string
    VITE_WS_BACKEND_URL: string
}

export const ENV = getEnv()

function getEnv(): Env {
    if ('env' in window && typeof window.env == 'object' && window.env && 'VITE_API_BASE_URL' in window.env && 'VITE_WS_BACKEND_URL' in window.env) {
        return Object.assign({}, window.env) as Env;
    } else {
        return {
            VITE_API_BASE_URL: import.meta.env.VITE_API_BASE_URL!,
            VITE_WS_BACKEND_URL: import.meta.env.VITE_WS_BACKEND_URL!,
        }
    }
}
