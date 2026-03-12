import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '../../ui/Button';
import { SettingsCard } from './SettingsCard';

export const MiscSettingsCard: React.FC = () => {
    const { t } = useTranslation();
    const [status, setStatus] = useState<string | null>(null);

    const handleClearCache = async () => {
        try {
            // Source - https://stackoverflow.com/a/54451080
            // Posted by abraham
            // Retrieved 2026-03-12, License - CC BY-SA 4.0
            const cacheNames = await caches.keys();
            await Promise.all(cacheNames.map(cacheName => caches.delete(cacheName)));
            
            setStatus(t('misc.clear_cache_success'));
            setTimeout(() => setStatus(null), 3000);
        } catch (err) {
            console.error('Failed to clear cache:', err);
        }
    };

    return (
        <SettingsCard title={t('misc.title')}>
            <div className="flex flex-col gap-2">
                <Button variant="secondary" className="!h-6 !text-[9px]" onClick={handleClearCache}>
                    {t('misc.clear_cache')}
                </Button>
                {status && (
                    <div className="text-[8px] text-[var(--success-color)]">
                        {status}
                    </div>
                )}
            </div>
        </SettingsCard>
    );
};
