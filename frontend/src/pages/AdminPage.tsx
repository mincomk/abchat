import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { DBridgeClient, type User } from '../api/dbridge-api';
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
    <div className="flex flex-col p-5 overflow-y-auto w-full h-full">
      <div className="flex justify-between items-center mb-5 border-b-2 border-[var(--accent-color)] pb-2.5">
        <h2 className="m-0 text-[var(--accent-color)] font-bold text-xl">{t('admin.title')}</h2>
        <Button variant="ghost" className="!h-auto !py-1.5 !px-2.5" onClick={onBack}>{t('admin.back')}</Button>
      </div>
      <div className="flex flex-col gap-[30px]">
        <section>
          <h3 className="mb-2 text-[#666] font-bold">{t('admin.register.title')}</h3>
          <form className="flex gap-2.5 flex-wrap" onSubmit={handleRegister}>
            <Input 
              type="text" 
              placeholder={t('admin.register.username')} 
              className="!h-auto !py-1 !px-1.5"
              value={regUsername} 
              onChange={(e) => setRegUsername(e.target.value)}
              required 
            />
            <Input 
              type="password" 
              placeholder={t('admin.register.password')} 
              className="!h-auto !py-1 !px-1.5"
              value={regPassword} 
              onChange={(e) => setRegPassword(e.target.value)}
              required 
            />
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
        <section>
          <h3 className="mb-2 text-[#666] font-bold">{t('admin.accounts.title')}</h3>
          <div className="overflow-x-auto">
            {loading ? (
              <div className="py-2.5">{t('admin.accounts.loading')}</div>
            ) : error ? (
              <div className="text-[var(--error-color)] py-2.5">{error}</div>
            ) : users.length === 0 ? (
              <div className="py-2.5">{t('admin.accounts.none')}</div>
            ) : (
              <UserTable users={users} currentUsername={currentUsername} onDelete={handleDelete} />
            )}
          </div>
        </section>
      </div>
    </div>
  );
};
