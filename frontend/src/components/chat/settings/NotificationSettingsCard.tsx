import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '../../ui/Button';
import { SettingsCard } from './SettingsCard';
import { 
    requestNotificationPermission, 
    saveNotificationSettings, 
    type NotificationMode 
} from '../../../api/notifications';

export const NotificationSettingsCard: React.FC = () => {
    const { t } = useTranslation();
    const [mode, setMode] = useState<NotificationMode>('all');
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<string | null>(null);

    const handleAllow = async () => {
        setError(null);
        setSuccess(null);
        try {
            const status = await requestNotificationPermission();
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
        setMode(newMode);
        try {
            await saveNotificationSettings({ mode: newMode });
        } catch (err: any) {
            setError(err.message || 'Error saving settings');
        }
    };

    return (
        <SettingsCard title={t('notifications.title')}>
            <div className="flex flex-col gap-2">
                <Button 
                    variant="secondary" 
                    className="!h-6 !text-[9px]" 
                    onClick={handleAllow}
                >
                    {t('notifications.allow')}
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
