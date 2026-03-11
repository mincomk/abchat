export type NotificationPermissionStatus = 'granted' | 'denied' | 'default';

export async function checkNotificationPermission(): Promise<NotificationPermissionStatus> {
    // TODO: Implement actual permission check
    return 'default';
}

export async function requestNotificationPermission(): Promise<NotificationPermissionStatus> {
    // TODO: Implement actual permission request
    console.log('Requesting notification permission...');
    return 'granted';
}

export type NotificationMode = 'all' | 'critical' | 'off';

export interface NotificationSettings {
    mode: NotificationMode;
}

export async function saveNotificationSettings(settings: NotificationSettings): Promise<void> {
    // TODO: Implement saving settings to backend or local storage
    console.log('Saving notification settings:', settings);
}
