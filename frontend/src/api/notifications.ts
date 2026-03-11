import { DBridgeClient, type NotificationMode as BackendNotificationMode } from './dbridge-api';

export type NotificationPermissionStatus = 'granted' | 'denied' | 'default';

export async function checkNotificationPermission(): Promise<NotificationPermissionStatus> {
    if (!('Notification' in window)) {
        return 'denied';
    }
    return Notification.permission as NotificationPermissionStatus;
}

function urlBase64ToUint8Array(base64String: string) {
    const padding = '='.repeat((4 - base64String.length % 4) % 4);
    const base64 = (base64String + padding)
        .replace(/-/g, '+')
        .replace(/_/g, '/');

    const rawData = window.atob(base64);
    const outputArray = new Uint8Array(rawData.length);

    for (let i = 0; i < rawData.length; ++i) {
        outputArray[i] = rawData.charCodeAt(i);
    }
    return outputArray;
}

export async function requestNotificationPermission(client: DBridgeClient): Promise<NotificationPermissionStatus> {
    if (!('Notification' in window) || !('serviceWorker' in navigator)) {
        throw new Error('Push notifications are not supported in this browser.');
    }

    const permission = await Notification.requestPermission();
    if (permission !== 'granted') {
        return permission as NotificationPermissionStatus;
    }

    try {
        const registration = await navigator.serviceWorker.ready;
        const publicVapidKey = await client.getVapidPublicKey();
        const convertedVapidKey = urlBase64ToUint8Array(publicVapidKey);

        const subscription = await registration.pushManager.subscribe({
            userVisibleOnly: true,
            applicationServerKey: convertedVapidKey
        });

        // Convert subscription to our format
        const subJson = subscription.toJSON();
        if (!subJson.endpoint || !subJson.keys?.p256dh || !subJson.keys?.auth) {
            throw new Error('Failed to get subscription keys');
        }

        await client.subscribeNotifications({
            endpoint: subJson.endpoint,
            p256dh: subJson.keys.p256dh,
            auth: subJson.keys.auth
        });

        return 'granted';
    } catch (err) {
        console.error('Failed to subscribe to push notifications:', err);
        throw err;
    }
}

export type NotificationMode = 'all' | 'critical' | 'off';

export interface NotificationSettings {
    mode: NotificationMode;
}

function toBackendMode(mode: NotificationMode): BackendNotificationMode {
    switch (mode) {
        case 'all': return 'All';
        case 'critical': return 'Critical';
        case 'off': return 'Off';
    }
}

export function fromBackendMode(mode: BackendNotificationMode): NotificationMode {
    switch (mode) {
        case 'All': return 'all';
        case 'Critical': return 'critical';
        case 'Off': return 'off';
    }
}

export async function saveNotificationSettings(client: DBridgeClient, settings: NotificationSettings): Promise<void> {
    await client.updateNotificationSettings({
        notification_mode: toBackendMode(settings.mode)
    });
}

export async function getNotificationSettings(client: DBridgeClient): Promise<NotificationSettings> {
    const response = await client.getNotificationSettings();
    return {
        mode: fromBackendMode(response.notification_mode)
    };
}
