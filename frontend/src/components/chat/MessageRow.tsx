import React from 'react';

export interface ChatMessage {
  id: string;
  type: 'user' | 'system';
  systemType?: 'info' | 'error';
  sender?: {
    username: string;
    nickname: string;
  };
  content: string;
  timestamp: number;
}

interface MessageRowProps {
  msg: ChatMessage;
}

const hashString = (str: string): number => {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        hash = (hash << 5) - hash + str.charCodeAt(i);
        hash |= 0;
    }
    return Math.abs(hash);
};

export const MessageRow: React.FC<MessageRowProps> = ({ msg }) => {
    const isSystem = msg.type === 'system';
    
    const senderColor = isSystem 
        ? (msg.systemType === 'error' ? 'text-[var(--error-color)]' : 'text-[var(--accent-color)]')
        : [
            'text-[#fff]', 'text-[#f00]', 'text-[#0f0]', 'text-[#ff0]', 
            'text-[#00f]', 'text-[#f0f]', 'text-[#0ff]', 'text-[#fa0]', 'text-[#a0f]'
        ][hashString(msg.sender!.nickname) % 9];

    return (
        <div className="flex gap-1.5 leading-tight">
            {isSystem ? (
                <>
                    <span className="text-[#666] text-[10px] min-w-[45px]">[SYSTEM]</span>
                    <span className={`flex-1 break-all ${senderColor}`}>{msg.content}</span>
                </>
            ) : (
                <>
                    <span className="text-[#666] text-[10px] min-w-[45px]">
                        {new Date(msg.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </span>
                    <span className={`font-bold min-w-[80px] text-right ${senderColor}`}>
                        &lt;{msg.sender!.nickname}&gt;
                    </span>
                    <span className="flex-1 break-all">{msg.content}</span>
                </>
            )}
        </div>
    );
};
