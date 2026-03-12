import axios, { type AxiosInstance, isAxiosError } from 'axios';
import { ENV } from '../env';

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

const getBaseUrl = (): string => {
    let baseUrl = ENV.VITE_API_BASE_URL;
    if (!baseUrl) {
        baseUrl = `${window.location.protocol}//${window.location.host}/api`;
    }
    return baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`;
};

const getWsBaseUrl = (): string => {
    let baseUrl = ENV.VITE_WS_BACKEND_URL
    if (!baseUrl) {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        baseUrl = `${protocol}//${window.location.host}/api`;
    }
    return baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`;
};

export interface MessageUser {
    username: string;
    nickname: string;
}

export type NotificationMode = 'All' | 'Critical' | 'Off';

export interface UserSettings {
    notification_mode: NotificationMode;
}

export interface UserSettingsRequest {
    settings: UserSettings;
}

export interface UserSettingsResponse {
    settings: UserSettings;
}


export interface SubscriptionRequest {
    endpoint: string;
    p256dh: string;
    auth: string;
}

export interface VapidPublicKeyResponse {
    public_key: string;
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
    #axios: AxiosInstance;

    constructor(channelId: string) {
        this.#channelId = channelId;
        this.#axios = axios.create({
            baseURL: getBaseUrl(),
        });

        this.#axios.interceptors.request.use((config) => {
            if (this.#token) {
                config.headers.Authorization = `Bearer ${this.#token}`;
            }
            return config;
        });
    }

    get channelId() {
        return this.#channelId;
    }

    setCredentials(token: string, nickname: string, username: string, is_admin: boolean = false) {
        this.#token = token;
        this.#nickname = nickname;
        localStorage.setItem(`dbridge_auth_${this.#channelId}`, JSON.stringify({ token, nickname, username, is_admin }));
    }

    updateStoredNickname(nickname: string) {
        this.#nickname = nickname;
        const stored = this.getStoredCredentials();
        if (stored) {
            stored.nickname = nickname;
            localStorage.setItem(`dbridge_auth_${this.#channelId}`, JSON.stringify(stored));
        }
    }

    getStoredCredentials() {
        const stored = localStorage.getItem(`dbridge_auth_${this.#channelId}`);
        if (stored) {
            try {
                const creds = JSON.parse(stored);
                if (creds.token) {
                    this.#token = creds.token;
                }
                if (creds.nickname) {
                    this.#nickname = creds.nickname;
                }
                return creds;
            } catch {
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
        try {
            const response = await this.#axios.post<LoginResponse>('auth/login', { username, password });
            return response.data;
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data.message || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Login failed: ${message}`);
        }
    }

    async listUsers(): Promise<User[]> {
        if (!this.#token) throw new Error('Not authenticated');

        try {
            const response = await this.#axios.get<User[]>('admin/accounts');
            return response.data;
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to list users: ${message}`);
        }
    }

    async registerUser(data: Record<string, unknown>): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');

        try {
            await this.#axios.post('admin/register', data);
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to register user: ${message}`);
        }
    }

    async changePassword(old_password: string, new_password: string): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');

        try {
            await this.#axios.post('auth/change-password', { old_password, new_password });
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to change password: ${message}`);
        }
    }

    async updateNickname(nickname: string): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');

        try {
            await this.#axios.post('auth/change-nickname', { nickname });
            this.updateStoredNickname(nickname);
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to change nickname: ${message}`);
        }
    }

    async adminChangePassword(username: string, new_password: string): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');

        try {
            await this.#axios.post(`admin/accounts/${username}/password`, { new_password });
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to change user password: ${message}`);
        }
    }

    async adminChangeNickname(username: string, nickname: string): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');

        try {
            await this.#axios.post(`admin/accounts/${username}/nickname`, { nickname });
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to change user nickname: ${message}`);
        }
    }

    async updateUserAdmin(username: string, is_admin: boolean): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');

        try {
            await this.#axios.patch(`admin/accounts/${username}/admin`, { is_admin });
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to update user admin status: ${message}`);
        }
    }

    async deleteUser(username: string): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');

        try {
            await this.#axios.delete(`admin/accounts/${username}`);
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to delete user: ${message}`);
        }
    }

    async subscribeNotifications(data: SubscriptionRequest): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');
        try {
            await this.#axios.post('notifications/subscribe', data);
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to subscribe: ${message}`);
        }
    }

    async unsubscribeNotifications(): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');
        try {
            await this.#axios.post('notifications/unsubscribe');
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to unsubscribe: ${message}`);
        }
    }

    async getNotificationSettings(): Promise<UserSettingsResponse> {
        if (!this.#token) throw new Error('Not authenticated');
        try {
            const response = await this.#axios.get<UserSettingsResponse>('notifications/settings');
            return response.data;
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to get settings: ${message}`);
        }
    }

    async updateNotificationSettings(data: UserSettingsRequest): Promise<void> {
        if (!this.#token) throw new Error('Not authenticated');
        try {
            await this.#axios.put('notifications/settings', data);
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to update settings: ${message}`);
        }
    }

    async getVapidPublicKey(): Promise<string> {
        try {
            const response = await this.#axios.get<VapidPublicKeyResponse>('notifications/vapid-key');
            return response.data.public_key;
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data?.message || error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to get VAPID key: ${message}`);
        }
    }

    async getHistoricalMessages(limit = 50, offset = 0): Promise<Message[]> {
        if (!this.#token) {
            throw new Error('Identification token is missing');
        }

        try {
            const response = await this.#axios.get<Message[]>(`channels/${this.#channelId}/messages`, {
                params: { limit, offset }
            });
            return response.data;
        } catch (error) {
            let message = 'Unknown error';
            if (isAxiosError(error)) {
                message = error.response?.data || error.message;
            } else if (error instanceof Error) {
                message = error.message;
            }
            throw new Error(`Failed to fetch messages: ${message}`);
        }
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

        const url = `${getWsBaseUrl()}ws/${this.#channelId}`;

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
            } catch {
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
