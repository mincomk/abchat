import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { DBridgeClient } from '../../../api/dbridge-api';
import { Button } from '../../ui/Button';
import { Input } from '../../ui/Input';
import { SettingsCard } from './SettingsCard';

interface PasswordChangeCardProps {
    client: DBridgeClient;
    onSuccess?: () => void;
}

export const PasswordChangeCard: React.FC<PasswordChangeCardProps> = ({ client, onSuccess }) => {
    const { t } = useTranslation();
    const [oldPwd, setOldPwd] = useState('');
    const [newPwd, setNewPwd] = useState('');
    const [retypePwd, setRetypePwd] = useState('');
    const [pwdStatus, setPwdStatus] = useState<{ type: 'success' | 'error', msg: string } | null>(null);

    const handlePasswordChange = async (e: React.FormEvent) => {
        e.preventDefault();
        setPwdStatus(null);
        if (newPwd !== retypePwd) {
            setPwdStatus({ type: 'error', msg: t('change_pwd.mismatch') });
            return;
        }
        try {
            await client.changePassword(oldPwd, newPwd);
            setPwdStatus({ type: 'success', msg: t('change_pwd.success') });
            setOldPwd('');
            setNewPwd('');
            setRetypePwd('');
            if (onSuccess) {
                setTimeout(onSuccess, 2000);
            }
        } catch (err: any) {
            setPwdStatus({ type: 'error', msg: err.message });
        }
    };

    return (
        <SettingsCard title={t('change_pwd.title')}>
            <form className="flex flex-col gap-1.5" onSubmit={handlePasswordChange}>
                <Input
                    type="password"
                    placeholder={t('change_pwd.old')}
                    value={oldPwd}
                    onChange={e => setOldPwd(e.target.value)}
                    required
                />
                <Input
                    type="password"
                    placeholder={t('change_pwd.new')}
                    value={newPwd}
                    onChange={e => setNewPwd(e.target.value)}
                    required
                />
                <Input
                    type="password"
                    placeholder={t('change_pwd.retype')}
                    value={retypePwd}
                    onChange={e => setRetypePwd(e.target.value)}
                    required
                />
                <Button type="submit" variant="ghost" className="!h-6">{t('change_pwd.submit')}</Button>
            </form>
            {pwdStatus && (
                <div className={`text-[9px] ${pwdStatus.type === 'success' ? 'text-green-500' : 'text-[var(--error-color)]'}`}>
                    {pwdStatus.msg}
                </div>
            )}
        </SettingsCard>
    );
};
