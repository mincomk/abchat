import React from 'react';
import { useTranslation } from 'react-i18next';
import { type User } from '../../api/dbridge-api';
import { Button } from '../ui/Button';

interface UserTableProps {
  users: User[];
  currentUsername: string;
  onDelete: (username: string) => void;
}

export const UserTable: React.FC<UserTableProps> = ({ users, currentUsername, onDelete }) => {
  const { t } = useTranslation();
  return (
    <table className="w-full border-collapse mt-2.5">
      <thead>
        <tr>
          <th className="text-left p-2 border-b border-[var(--border-color)] text-[var(--secondary-text-color)] text-[12px]">{t('admin.accounts.username')}</th>
          <th className="text-left p-2 border-b border-[var(--border-color)] text-[var(--secondary-text-color)] text-[12px]">{t('admin.accounts.nickname')}</th>
          <th className="text-left p-2 border-b border-[var(--border-color)] text-[var(--secondary-text-color)] text-[12px]">{t('admin.accounts.admin')}</th>
          <th className="text-left p-2 border-b border-[var(--border-color)] text-[var(--secondary-text-color)] text-[12px]">{t('admin.accounts.actions')}</th>
        </tr>
      </thead>
      <tbody>
        {users.map((user) => (
          <tr key={user.username}>
            <td className="p-2 border-b border-[var(--border-color)]">{user.username}</td>
            <td className="p-2 border-b border-[var(--border-color)]">{user.nickname}</td>
            <td className="p-2 border-b border-[var(--border-color)]">{user.is_admin ? t('common.yes') : t('common.no')}</td>
            <td className="p-2 border-b border-[var(--border-color)]">
              {user.username !== currentUsername && (
                <Button variant="danger" className="!text-[10px] !h-auto !py-0.5 !px-1.5" onClick={() => onDelete(user.username)}>
                  {t('admin.accounts.delete')}
                </Button>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
};
