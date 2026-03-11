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

const FormattedMessage: React.FC<{ content: string }> = ({ content }) => {
    const regex = /<(@&|@|#)([^:]+):([^>]+)>/g;
    const parts: (string | React.ReactNode)[] = [];
    let lastIndex = 0;
    let match;

    while ((match = regex.exec(content)) !== null) {
        if (match.index > lastIndex) {
            parts.push(content.slice(lastIndex, match.index));
        }

        const prefix = match[1];
        const name = match[2];
        const id = match[3];

        let colorClass = '';
        if (prefix === '@&') {
            colorClass = 'text-[#faa61a]'; // Role - Orange
        } else if (prefix === '@') {
            colorClass = 'text-[#00aaff]'; // User - Cyan
        } else if (prefix === '#') {
            colorClass = 'text-[#a0f]';   // Channel - Purple
        }

        parts.push(
            <span key={`${id}-${match.index}`} className={`${colorClass} font-semibold cursor-default`}>
                @{name}
            </span>
        );

        lastIndex = regex.lastIndex;
    }

    if (lastIndex < content.length) {
        parts.push(content.slice(lastIndex));
    }

    return <>{parts.length > 0 ? parts : content}</>;
};

export const MessageRow: React.FC<MessageRowProps> = ({ msg }) => {
    const isSystem = msg.type === 'system';

    const senderColor = isSystem
        ? (msg.systemType === 'error' ? 'text-[var(--error-color)]' : 'text-[var(--accent-color)]')
        : [
            'text-[var(--text-color)]', 'text-[#f00]', 'text-[#0f0]', 'text-[#ff0]',
            'text-[#00f]', 'text-[#f0f]', 'text-[#0ff]', 'text-[#fa0]', 'text-[#a0f]'
        ][hashString(msg.sender!.nickname) % 9];

    return (
        <div className="flex gap-1.5 leading-tight">
            {isSystem ? (
                <>
                    <span className="text-[var(--secondary-text-color)] text-[10px] min-w-[45px]">[SYSTEM]</span>
                    <span className={`flex-1 break-all ${senderColor}`}>
                        <FormattedMessage content={msg.content} />
                    </span>
                </>
            ) : (
                <>
                    <span className="text-[var(--secondary-text-color)] text-[10px] min-w-[45px]">
                        {new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </span>
                    <span className={`font-bold min-w-[80px] text-right ${senderColor}`}>
                        &lt;{msg.sender!.nickname}&gt;
                    </span>
                    <span className="flex-1 break-all">
                        <FormattedMessage content={msg.content} />
                    </span>
                </>
            )}
        </div>
    );
};
