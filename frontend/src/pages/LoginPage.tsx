import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { useWatchLayout } from '../hooks/useWatchLayout';

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
    const isWatch = useWatchLayout();
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

    const toggleMode = () => setMode(mode === 'password' ? 'token' : 'password');
    const submitLabel = mode === 'password' ? t('login.go') : t('login.in');
    // Watch layout: one field per line, finger-sized rows, form centred in the circle.
    const fieldClass = isWatch ? 'w-full watch-tap min-w-0' : 'flex-[1_1_80px] min-w-[60px]';

    const modeButton = (
        <Button
            variant="secondary"
            className={isWatch ? 'watch-tap flex-1 !p-0 font-bold' : 'w-5 h-5 !p-0 !text-[8px] font-bold'}
            title={t('login.mode')}
            onClick={toggleMode}
            type="button"
        >
            {mode === 'password' ? 'PW' : 'TK'}
        </Button>
    );

    const submitButton = (
        <Button
            type="submit"
            className={isWatch ? 'watch-tap flex-1' : 'flex-none h-5 !px-1.5'}
            title={submitLabel}
        >
            {submitLabel}
        </Button>
    );

    return (
        <div className={isWatch
            ? 'watch-inset w-full h-full flex flex-col justify-center gap-1.5 py-3'
            : 'w-full h-full flex flex-row flex-wrap gap-0.5 max-w-[400px] p-0.5 items-center justify-center'}>
            <form className={isWatch ? 'flex flex-col gap-1.5 w-full max-w-[300px] mx-auto' : 'flex flex-row flex-wrap gap-0.5 flex-[1_1_auto] justify-center'} onSubmit={handleSubmit}>
                <Input
                    type="text"
                    placeholder={t('login.channel_id')}
                    className={fieldClass}
                    value={channelId}
                    onChange={(e) => setChannelId(e.target.value)}
                    title={t('login.channel_id')}
                    required
                />
                <Input
                    type="text"
                    placeholder={t('login.username')}
                    className={fieldClass}
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    title={t('login.username')}
                    required
                />
                {mode === 'password' ? (
                    <Input
                        type="password"
                        placeholder={t('login.password')}
                        className={fieldClass}
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
                            className={fieldClass}
                            value={nickname}
                            onChange={(e) => setNickname(e.target.value)}
                            title={t('login.nickname')}
                            required
                        />
                        <Input
                            type="password"
                            placeholder={t('login.token')}
                            className={fieldClass}
                            value={token}
                            onChange={(e) => setToken(e.target.value)}
                            title={t('login.token')}
                            required
                        />
                    </>
                )}
                {isWatch ? (
                    <div className="flex gap-1.5">
                        {modeButton}
                        {submitButton}
                    </div>
                ) : (
                    <>
                        {modeButton}
                        {submitButton}
                    </>
                )}
            </form>
        </div>
    );
};
