import React from 'react';
import { useTranslation } from 'react-i18next';
import { DBridgeClient, type User } from '../../../api/dbridge-api';
import { Button } from '../../ui/Button';
import { PasswordChangeCard } from './PasswordChangeCard';
import { NotificationSettingsCard } from './NotificationSettingsCard';
import { NicknameChangeCard } from './NicknameChangeCard';
import { MiscSettingsCard } from './MiscSettingsCard';

interface SettingsOverlayProps {
    client: DBridgeClient;
    onClose: () => void;
    onUpdateUser?: (user: User) => void;
    username: string;
    /** Watch layout: settings become a full-screen scrollable panel instead of a strip. */
    watch?: boolean;
}

export const SettingsOverlay: React.FC<SettingsOverlayProps> = ({ client, onClose, onUpdateUser, username, watch = false }) => {
    const { t } = useTranslation();

    return (
        <div className={watch
            ? 'watch-panel fixed inset-0 z-50 overflow-y-auto bg-[var(--bg-color)] py-4 pb-12 flex flex-col gap-2 items-center'
            : 'bg-[var(--header-bg)] border-b border-[var(--border-color)] p-2.5 flex flex-col sm:flex-row flex-wrap gap-4 items-start'}>
            {watch && (
                <Button variant="ghost" className="watch-tap w-[160px] flex-none mt-3" onClick={onClose}>
                    {t('change_pwd.close')}
                </Button>
            )}
            <div className={watch ? 'w-full max-w-[320px] flex flex-col gap-2' : 'contents'}>
                <NicknameChangeCard client={client} onUpdateUser={onUpdateUser} username={username} />
                <PasswordChangeCard client={client} onSuccess={onClose} />
                <NotificationSettingsCard client={client} />
                <MiscSettingsCard />

                {!watch && (
                    <div className="ml-auto w-full sm:w-auto flex justify-end">
                        <Button variant="ghost" className="!h-4 !px-2 !text-[9px]" onClick={onClose}>
                            {t('change_pwd.close')}
                        </Button>
                    </div>
                )}
            </div>
        </div>
    );
};
