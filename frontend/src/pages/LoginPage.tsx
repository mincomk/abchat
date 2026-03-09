import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';

export interface LoginData {
    username: string;
    channelId: string;
    password?: string;
    token?: string;
    nickname?: string;
}

interface LoginProps {
    onLogin: (data: LoginData) => void;
}

export const LoginPage: React.FC<LoginProps> = ({ onLogin }) => {
    const { t } = useTranslation();
    const [mode, setMode] = useState<'password' | 'token'>('password');
    const [channelId, setChannelId] = useState('main');
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [nickname, setNickname] = useState('');
    const [token, setToken] = useState('');

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (mode === 'password') {
            onLogin({ channelId, username, password });
        } else {
            onLogin({ channelId, username, nickname, token });
        }
    };

    return (
        <div className="w-full h-full flex flex-row flex-wrap gap-0.5 max-w-[400px] p-0.5 items-center justify-center">
            <form className="flex flex-row flex-wrap gap-0.5 flex-[1_1_auto] justify-center" onSubmit={handleSubmit}>
                <Button
                    variant="secondary"
                    className="w-5 h-5 !p-0 !text-[8px] font-bold"
                    title={t('login.mode')}
                    onClick={() => setMode(mode === 'password' ? 'token' : 'password')}
                    type="button"
                >
                    {mode === 'password' ? 'PW' : 'TK'}
                </Button>
                <Input
                    type="text"
                    placeholder={t('login.channel_id')}
                    className="flex-[1_1_80px] min-w-[60px]"
                    value={channelId}
                    onChange={(e) => setChannelId(e.target.value)}
                    title={t('login.channel_id')}
                    required
                />
                <Input
                    type="text"
                    placeholder={t('login.username')}
                    className="flex-[1_1_80px] min-w-[60px]"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    title={t('login.username')}
                    required
                />
                {mode === 'password' ? (
                    <Input
                        type="password"
                        placeholder={t('login.password')}
                        className="flex-[1_1_80px] min-w-[60px]"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        title={t('login.password')}
                        required
                    />
                ) : (
                    <>
                        <Input
                            type="text"
                            placeholder={t('login.nickname')}
                            className="flex-[1_1_80px] min-w-[60px]"
                            value={nickname}
                            onChange={(e) => setNickname(e.target.value)}
                            title={t('login.nickname')}
                            required
                        />
                        <Input
                            type="password"
                            placeholder={t('login.token')}
                            className="flex-[1_1_80px] min-w-[60px]"
                            value={token}
                            onChange={(e) => setToken(e.target.value)}
                            title={t('login.token')}
                            required
                        />
                    </>
                )}
                <Button type="submit" className="flex-none h-5 !px-1.5" title={mode === 'password' ? t('login.go') : t('login.in')}>
                    {mode === 'password' ? t('login.go') : t('login.in')}
                </Button>
            </form>
        </div>
    );
};
