import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '../../ui/Button';
import { SettingsCard } from './SettingsCard';
import { DBridgeClient } from '../../../api/dbridge-api';
import { 
    requestNotificationPermission, 
    saveNotificationSettings, 
    getNotificationSettings,
    checkNotificationPermission,
    type NotificationMode 
} from '../../../api/notifications';

interface NotificationSettingsCardProps {
    client: DBridgeClient;
}

export const NotificationSettingsCard: React.FC<NotificationSettingsCardProps> = ({ client }) => {
    const { t } = useTranslation();
    const [mode, setMode] = useState<NotificationMode>('all');
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<string | null>(null);
    const [permission, setPermission] = useState<string>('default');

    useEffect(() => {
        const init = async () => {
            try {
                const status = await checkNotificationPermission();
                setPermission(status);
                
                const settings = await getNotificationSettings(client);
                setMode(settings.mode);
            } catch (err) {
                console.error('Failed to load notification settings', err);
            }
        };
        init();
    }, [client]);

    const handleAllow = async () => {
        setError(null);
        setSuccess(null);
        try {
            const status = await requestNotificationPermission(client);
            setPermission(status);
            if (status === 'granted') {
                setSuccess(t('notifications.permission_granted'));
            } else {
                setError(t('notifications.permission_denied'));
            }
        } catch (err: any) {
            setError(err.message || 'Error requesting permission');
        }
    };

    const handleModeChange = async (newMode: NotificationMode) => {
        const oldMode = mode;
        setMode(newMode);
        try {
            await saveNotificationSettings(client, { mode: newMode });
        } catch (err: any) {
            setMode(oldMode);
            setError(err.message || 'Error saving settings');
        }
    };

    return (
        <SettingsCard title={t('notifications.title')}>
            <div className="flex flex-col gap-2">
                <Button 
                    variant={permission === 'granted' ? 'ghost' : 'secondary'} 
                    className="!h-6 !text-[9px]" 
                    onClick={handleAllow}
                    disabled={permission === 'granted'}
                >
                    {permission === 'granted' ? t('notifications.enabled') : t('notifications.allow')}
                </Button>

                <div className="flex flex-col gap-1">
                    <span className="text-[9px] text-[var(--secondary-text-color)]">
                        {t('notifications.mode')}
                    </span>
                    <div className="flex gap-1 flex-wrap">
                        {(['all', 'critical', 'off'] as NotificationMode[]).map((m) => (
                            <Button
                                key={m}
                                variant={mode === m ? 'primary' : 'ghost'}
                                className={`!h-5 !px-1.5 !text-[8px] ${
                                    m === 'off' && mode !== 'off' ? 'opacity-70 hover:opacity-100' : ''
                                } ${m === 'off' && mode === 'off' ? '!bg-red-900 !text-white' : ''}`}
                                onClick={() => handleModeChange(m)}
                            >
                                {t(`notifications.mode_${m}`)}
                            </Button>
                        ))}
                    </div>
                    {mode === 'off' && (
                        <span className="text-[7px] text-red-500 leading-tight">
                            {t('notifications.off_warning')}
                        </span>
                    )}
                </div>

                {error && (
                    <div className="text-[8px] text-[var(--error-color)]">
                        {error}
                    </div>
                )}
                {success && (
                    <div className="text-[8px] text-green-500">
                        {success}
                    </div>
                )}
            </div>
        </SettingsCard>
    );
};
