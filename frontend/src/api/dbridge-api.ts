export interface User {
    username: string;
    nickname: string;
    is_admin?: boolean;
}

export interface Message {
    id: string;
    sender: User;
    content: string;
    timestamp: number;
    channel_id?: string;
}

export type ServerMessage = Message | string;

export interface LoginResponse {
    token: string;
    user: User;
}

export class DBridgeClient {
    #ws: WebSocket | null = null;
    #onMessageCallback: ((msg: Message) => void) | null = null;
    #onErrorCallback: ((error: string) => void) | null = null;
    #onOpenCallback: (() => void) | null = null;
    #onCloseCallback: (() => void) | null = null;
    #channelId: string;
    #reconnectTimer: number | null = null;
    #token: string | null = null;
    #nickname: string | null = null;
    #shouldReconnect = true;

    constructor(channelId: string) {
        this.#channelId = channelId;
    }

    get channelId() {
        return this.#channelId;
    }

    setCredentials(token: string, nickname: string, username: string, is_admin: boolean = false) {
        this.#token = token;
        this.#nickname = nickname;
        localStorage.setItem(`dbridge_auth_${this.#channelId}`, JSON.stringify({ token, nickname, username, is_admin }));
    }

    getStoredCredentials() {
        const stored = localStorage.getItem(`dbridge_auth_${this.#channelId}`);
        if (stored) {
            try {
                return JSON.parse(stored);
            } catch (e) {
                return null;
            }
        }
        return null;
    }

    clearCredentials() {
        this.#token = null;
        this.#nickname = null;
        localStorage.removeItem(`dbridge_auth_${this.#channelId}`);
    }

    async login(username: string, password: string): Promise<LoginResponse> {
        let baseUrl = import.meta.env.VITE_API_BASE_URL;

        if (!baseUrl) {
            baseUrl = `${window.location.protocol}//${window.location.host}`;
        }

        const url = new URL('/auth/login', baseUrl);

        const response = await fetch(url.toString(), {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ username, password })
        });

        if (!response.ok) {
            const text = await response.text();
            throw new Error(`Login failed: ${text}`);
        }

        return await response.json();
    }

    async listUsers(): Promise<User[]> {
        if (!this.#token) throw new Error('Not authenticated');

        let baseUrl = import.meta.env.VITE_API_BASE_URL;
        if (!baseUrl) baseUrl = `${window.location.protocol}//${window.location.host}`;

        const url = new URL('/admin/accounts', baseUrl);
        const response = await fetch(url.toString(), {
            headers: { 'Authorization': `Bearer ${this.#token}` }
        });

        if (!response.ok) {
            const text = await response.text();
            throw new Error(`Failed to list users: ${text}`);
        }

        return await response.json();
    }

    async registerUser(data: any): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');

        let baseUrl = import.meta.env.VITE_API_BASE_URL;
        if (!baseUrl) baseUrl = `${window.location.protocol}//${window.location.host}`;

        const url = new URL('/admin/register', baseUrl);
        const response = await fetch(url.toString(), {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${this.#token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });

        if (!response.ok) {
            const text = await response.text();
            throw new Error(`Failed to register user: ${text}`);
        }
    }

    async deleteUser(username: string): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');

        let baseUrl = import.meta.env.VITE_API_BASE_URL;
        if (!baseUrl) baseUrl = `${window.location.protocol}//${window.location.host}`;

        const url = new URL(`/admin/accounts/${username}`, baseUrl);
        const response = await fetch(url.toString(), {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${this.#token}` }
        });

        if (!response.ok) {
            const text = await response.text();
            throw new Error(`Failed to delete user: ${text}`);
        }
    }

    async getHistoricalMessages(limit = 50, offset = 0): Promise<Message[]> {
        if (!this.#token) {
            throw new Error('Identification token is missing');
        }

        let baseUrl = import.meta.env.VITE_API_BASE_URL;

        if (!baseUrl) {
            baseUrl = `${window.location.protocol}//${window.location.host}`;
        }

        const url = new URL(`/channels/${this.#channelId}/messages`, baseUrl);
        url.searchParams.set('limit', limit.toString());
        url.searchParams.set('offset', offset.toString());

        const response = await fetch(url.toString(), {
            headers: {
                'Authorization': `Bearer ${this.#token}`
            }
        });

        if (!response.ok) {
            const text = await response.text();
            throw new Error(`Failed to fetch messages: ${text}`);
        }

        return await response.json();
    }

    connect() {
        if (this.#ws) {
            this.#ws.onopen = null;
            this.#ws.onmessage = null;
            this.#ws.onerror = null;
            this.#ws.onclose = null;
            this.#ws.close();
        }

        this.#shouldReconnect = true;
        this.#clearReconnectTimer();

        let baseUrl = import.meta.env.VITE_WS_BACKEND_URL;

        if (!baseUrl) {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const host = window.location.host;
            baseUrl = `${protocol}//${host}`;
        }

        const url = `${baseUrl}/ws/${this.#channelId}`;

        console.log(`Connecting to ${url}...`);
        this.#ws = new WebSocket(url);

        this.#ws.onopen = () => {
            console.log('WS Connected. Identifying...');
            if (this.#token && this.#nickname) {
                this.#ws?.send(JSON.stringify({
                    type: 'identify',
                    token: this.#token,
                    nickname: this.#nickname
                }));
            }
            this.#onOpenCallback?.();
        };

        this.#ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if (data.id && data.sender && data.content) {
                    this.#onMessageCallback?.(data as Message);
                } else {
                    this.#onErrorCallback?.(event.data);
                }
            } catch (e) {
                this.#onErrorCallback?.(event.data);
            }
        };

        this.#ws.onerror = () => {
            this.#onErrorCallback?.('WebSocket connection error');
        };

        this.#ws.onclose = () => {
            console.log('WS Closed.');
            this.#onCloseCallback?.();
            if (this.#shouldReconnect) {
                this.#scheduleReconnect();
            }
        };
    }

    #scheduleReconnect() {
        this.#clearReconnectTimer();
        this.#reconnectTimer = window.setTimeout(() => {
            console.log('Attempting to reconnect...');
            this.connect();
        }, 3000);
    }

    #clearReconnectTimer() {
        if (this.#reconnectTimer !== null) {
            clearTimeout(this.#reconnectTimer);
            this.#reconnectTimer = null;
        }
    }

    sendMessage(content: string) {
        if (this.#ws?.readyState === WebSocket.OPEN) {
            this.#ws.send(JSON.stringify({
                type: 'send_message',
                content
            }));
        }
    }

    onMessage(callback: (msg: Message) => void) {
        this.#onMessageCallback = callback;
    }

    onError(callback: (error: string) => void) {
        this.#onErrorCallback = callback;
    }

    onOpen(callback: () => void) {
        this.#onOpenCallback = callback;
    }

    onClose(callback: () => void) {
        this.#onCloseCallback = callback;
    }

    disconnect() {
        this.#shouldReconnect = false;
        this.#clearReconnectTimer();
        this.#ws?.close();
    }
}
