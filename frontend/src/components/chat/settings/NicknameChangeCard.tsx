import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { DBridgeClient, type User } from '../../../api/dbridge-api';
import { Button } from '../../ui/Button';
import { Input } from '../../ui/Input';
import { SettingsCard } from './SettingsCard';

interface NicknameChangeCardProps {
    client: DBridgeClient;
    username: string;
    onUpdateUser?: (user: User) => void;
}

export const NicknameChangeCard: React.FC<NicknameChangeCardProps> = ({ client, username, onUpdateUser }) => {
    const { t } = useTranslation();
    const [nickname, setNickname] = useState('');
    const [status, setStatus] = useState<{ type: 'success' | 'error', msg: string } | null>(null);

    const handleNicknameChange = async (e: React.FormEvent) => {
        e.preventDefault();
        setStatus(null);
        try {
            await client.updateNickname(nickname);
            setStatus({ type: 'success', msg: t('change_nickname.success') });
            if (onUpdateUser) {
                const creds = client.getStoredCredentials();
                onUpdateUser({
                    username,
                    nickname,
                    is_admin: creds?.is_admin
                });
            }
            setNickname('');
        } catch (err: any) {
            setStatus({ type: 'error', msg: err.message });
        }
    };

    return (
        <SettingsCard title={t('change_nickname.title')}>
            <form className="flex flex-col gap-1.5" onSubmit={handleNicknameChange}>
                <Input
                    type="text"
                    placeholder={t('change_nickname.new')}
                    value={nickname}
                    onChange={e => setNickname(e.target.value)}
                    required
                />
                <Button type="submit" variant="ghost" className="!h-6">{t('change_nickname.submit')}</Button>
            </form>
            {status && (
                <div className={`text-[9px] ${status.type === 'success' ? 'text-[var(--success-color)]' : 'text-[var(--error-color)]'}`}>
                    {status.msg}
                </div>
            )}
        </SettingsCard>
    );
};
