export interface User {
    username: string;
    nickname: string;
    is_admin: boolean;
}

export interface MessageSchema {
    id: string;
    content: string;
    timestamp: number;
    channel_id: string;
    sender: User;
}
