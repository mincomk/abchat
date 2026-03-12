import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { DBridgeClient, type User } from '../api/dbridge-api';
import { generateRandomPassword } from '../util/password';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { UserTable } from '../components/admin/UserTable';

interface AdminProps {
  client: DBridgeClient;
  currentUsername: string;
  onBack: () => void;
}

export const AdminPage: React.FC<AdminProps> = ({ client, currentUsername, onBack }) => {
  const { t } = useTranslation();
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [regUsername, setRegUsername] = useState('');
  const [regPassword, setRegPassword] = useState('');
  const [regNickname, setRegNickname] = useState('');

  const [pwdTargetUsername, setPwdTargetUsername] = useState<string | null>(null);
  const [newPwd, setNewPwd] = useState('');
  const [retypeNewPwd, setRetypeNewPwd] = useState('');

  const [nickTargetUsername, setNickTargetUsername] = useState<string | null>(null);
  const [newNick, setNewNick] = useState('');

  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  const loadUsers = async () => {
    setLoading(true);
    try {
      const data = await client.listUsers();
      setUsers(data);
      setError(null);
    } catch (err: any) {
      setError(t('admin.errors.failed_load', { error: err.message }));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadUsers();
  }, [client]);

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      await client.registerUser({ username: regUsername, password: regPassword, nickname: regNickname });
      setRegUsername('');
      setRegPassword('');
      setRegNickname('');
      loadUsers();
    } catch (err: any) {
      setError(t('admin.errors.failed_register', { error: err.message }));
    }
  };

  const handleAdminChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!pwdTargetUsername) return;
    setError(null);
    setSuccessMsg(null);
    if (newPwd !== retypeNewPwd) {
        setError(t('change_pwd.mismatch'));
        return;
    }
    try {
      await client.adminChangePassword(pwdTargetUsername, newPwd);
      setSuccessMsg(t('admin.errors.change_pwd_success'));
      setNewPwd('');
      setRetypeNewPwd('');
      setPwdTargetUsername(null);
    } catch (err: any) {
      setError(err.message);
    }
  };

  const handleAdminChangeNickname = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!nickTargetUsername) return;
    setError(null);
    setSuccessMsg(null);
    try {
      await client.adminChangeNickname(nickTargetUsername, newNick);
      setSuccessMsg(t('admin.errors.change_nickname_success'));
      setNewNick('');
      setNickTargetUsername(null);
      loadUsers();
    } catch (err: any) {
      setError(err.message);
    }
  };

  const handleToggleAdmin = async (username: string, is_admin: boolean) => {
    setError(null);
    setSuccessMsg(null);
    try {
      await client.updateUserAdmin(username, is_admin);
      setSuccessMsg(t('admin.errors.toggle_admin_success'));
      loadUsers();
    } catch (err: any) {
      setError(err.message);
    }
  };

  const handleDelete = async (username: string) => {
    if (!confirm(t('admin.errors.confirm_delete', { username }))) return;
    setError(null);

    try {
      await client.deleteUser(username);
      loadUsers();
    } catch (err: any) {
      setError(t('admin.errors.failed_delete', { error: err.message }));
    }
  };

  return (
    <div className="flex flex-col p-5 overflow-y-auto w-full h-full bg-[var(--bg-color)]">
      <div className="flex justify-between items-center mb-5 border-b-2 border-[var(--accent-color)] pb-2.5">
        <h2 className="m-0 text-[var(--accent-color)] font-bold text-xl">{t('admin.title')}</h2>
        <Button variant="ghost" className="!h-auto !py-1.5 !px-2.5" onClick={onBack}>{t('admin.back')}</Button>
      </div>
      <div className="flex flex-col gap-[30px]">
        <section>
          <h3 className="mb-2 text-[var(--secondary-text-color)] font-bold">{t('admin.register.title')}</h3>
          <form className="flex gap-2.5 flex-wrap" onSubmit={handleRegister}>
            <Input 
              type="text" 
              placeholder={t('admin.register.username')} 
              className="!h-auto !py-1 !px-1.5"
              value={regUsername} 
              onChange={(e) => setRegUsername(e.target.value)}
              required 
            />
            <div className="flex gap-1">
              <Input 
                type="password" 
                placeholder={t('admin.register.password')} 
                className="!h-auto !py-1 !px-1.5"
                value={regPassword} 
                onChange={(e) => setRegPassword(e.target.value)}
                required 
              />
              <Button 
                type="button" 
                variant="secondary" 
                className="!h-auto !py-1 !px-2"
                onClick={() => setRegPassword(generateRandomPassword())}
              >
                {t('admin.register.gen')}
              </Button>
            </div>
            <Input 
              type="text" 
              placeholder={t('admin.register.nickname')} 
              className="!h-auto !py-1 !px-1.5"
              value={regNickname} 
              onChange={(e) => setRegNickname(e.target.value)}
              required 
            />
            <Button type="submit" className="!h-auto !py-1 !px-4">{t('admin.register.submit')}</Button>
          </form>
        </section>
        {pwdTargetUsername && (
          <section className="bg-[var(--header-bg)] p-3 border border-[var(--border-color)] rounded">
            <h3 className="mb-2 text-[var(--accent-color)] font-bold">
              {t('admin.errors.change_pwd_title', { username: pwdTargetUsername })}
            </h3>
            <form className="flex gap-2.5" onSubmit={handleAdminChangePassword}>
              <Input
                type="password"
                placeholder={t('admin.register.password')}
                className="!h-auto !py-1 !px-1.5 flex-1"
                value={newPwd}
                onChange={(e) => setNewPwd(e.target.value)}
                required
              />
              <Input
                type="password"
                placeholder={t('change_pwd.retype')}
                className="!h-auto !py-1 !px-1.5 flex-1"
                value={retypeNewPwd}
                onChange={(e) => setRetypeNewPwd(e.target.value)}
                required
              />
              <Button
                type="button"
                variant="secondary"
                className="!h-auto !py-1 !px-2"
                onClick={() => {
                    const p = generateRandomPassword();
                    setNewPwd(p);
                    setRetypeNewPwd(p);
                }}
              >
                {t('admin.register.gen')}
              </Button>
              <Button type="submit" variant="ghost" className="!h-auto !py-1 !px-4">{t('change_pwd.submit')}</Button>
              <Button
                type="button"
                variant="ghost"
                className="!h-auto !py-1 !px-4"
                onClick={() => { setPwdTargetUsername(null); setNewPwd(''); }}
              >
                {t('change_pwd.cancel')}
              </Button>
            </form>
          </section>
        )}
        {nickTargetUsername && (
          <section className="bg-[var(--header-bg)] p-3 border border-[var(--border-color)] rounded">
            <h3 className="mb-2 text-[var(--accent-color)] font-bold">
              {t('admin.errors.change_nickname_title', { username: nickTargetUsername })}
            </h3>
            <form className="flex gap-2.5" onSubmit={handleAdminChangeNickname}>
              <Input
                type="text"
                placeholder={t('admin.register.nickname')}
                className="!h-auto !py-1 !px-1.5 flex-1"
                value={newNick}
                onChange={(e) => setNewNick(e.target.value)}
                required
              />
              <Button type="submit" variant="ghost" className="!h-auto !py-1 !px-4">{t('change_nickname.submit')}</Button>
              <Button
                type="button"
                variant="ghost"
                className="!h-auto !py-1 !px-4"
                onClick={() => { setNickTargetUsername(null); setNewNick(''); }}
              >
                {t('change_pwd.cancel')}
              </Button>
            </form>
          </section>
        )}
        {successMsg && (
          <div className="bg-green-900/20 text-green-500 p-2 border border-green-900/50 rounded text-[12px]">
            {successMsg}
          </div>
        )}
        <section>
          <h3 className="mb-2 text-[var(--secondary-text-color)] font-bold">{t('admin.accounts.title')}</h3>
          <div className="overflow-x-auto">
            {loading ? (
              <div className="py-2.5">{t('admin.accounts.loading')}</div>
            ) : error ? (
              <div className="text-[var(--error-color)] py-2.5">{error}</div>
            ) : users.length === 0 ? (
              <div className="py-2.5">{t('admin.accounts.none')}</div>
            ) : (
              <UserTable 
                users={users} 
                currentUsername={currentUsername} 
                onDelete={handleDelete} 
                onChangePassword={setPwdTargetUsername} 
                onChangeNickname={setNickTargetUsername}
                onToggleAdmin={handleToggleAdmin}
              />
            )}
          </div>
        </section>
      </div>
    </div>
  );
};
