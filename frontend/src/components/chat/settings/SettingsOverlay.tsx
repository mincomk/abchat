import React from 'react';
import { useTranslation } from 'react-i18next';
import { DBridgeClient } from '../../../api/dbridge-api';
import { Button } from '../../ui/Button';
import { PasswordChangeCard } from './PasswordChangeCard';
import { NotificationSettingsCard } from './NotificationSettingsCard';

interface SettingsOverlayProps {
    client: DBridgeClient;
    onClose: () => void;
}

export const SettingsOverlay: React.FC<SettingsOverlayProps> = ({ client, onClose }) => {
    const { t } = useTranslation();

    return (
        <div className="bg-[var(--header-bg)] border-b border-[var(--border-color)] p-2.5 flex flex-wrap gap-4 items-start">
            <PasswordChangeCard client={client} onSuccess={onClose} />
            <NotificationSettingsCard />
            
            <div className="ml-auto">
                <Button variant="ghost" className="!h-4 !px-2 !text-[9px]" onClick={onClose}>
                    {t('change_pwd.cancel')}
                </Button>
            </div>
        </div>
    );
};
