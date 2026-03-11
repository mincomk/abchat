import { Client, Events, GatewayIntentBits, Message, TextChannel } from 'discord.js';
import 'dotenv/config';
import { logger } from './lib/logger.js';
import { pubClient, subClient } from './lib/redis.js';
import { MessageSchema } from './lib/types.js';

const DISCORD_CHANNEL_ID = process.env['DISCORD_CHANNEL_ID'];
const ABCHAT_CHANNEL_ID = process.env['ABCHAT_CHANNEL_ID'] || 'bot:messages';

const client = new Client({
    intents: [
        GatewayIntentBits.Guilds,
        GatewayIntentBits.GuildMessages,
        GatewayIntentBits.MessageContent,
    ],
});

client.once(Events.ClientReady, (readyClient) => {
    logger.info(`Ready! Logged in as ${readyClient.user.tag}`);
});

// --- Redis -> Discord ---
subClient.subscribe(ABCHAT_CHANNEL_ID, (err) => {
    if (err) {
        logger.error({ err }, 'Failed to subscribe to Redis channel');
    } else {
        logger.info(`Listening for messages on Redis channel: ${ABCHAT_CHANNEL_ID}`);
    }
});

subClient.on('message', async (channel, message) => {
    if (channel !== ABCHAT_CHANNEL_ID) return;

    try {
        const data: MessageSchema = JSON.parse(message);
        const discordChannel = await client.channels.fetch(DISCORD_CHANNEL_ID!);

        if (discordChannel instanceof TextChannel) {
            await discordChannel.send(`**${data.sender.nickname}**: ${data.content}`);
        }
    } catch (err) {
        logger.error({ err, message }, 'Error processing Redis message');
    }
});

// --- Discord -> Redis ---
client.on(Events.MessageCreate, async (message: Message) => {
    if (message.author.bot || message.channelId !== DISCORD_CHANNEL_ID) return;

    const messageData: MessageSchema = {
        id: message.id,
        content: message.content,
        timestamp: Math.floor(message.createdTimestamp / 1000),
        channel_id: ABCHAT_CHANNEL_ID,
        sender: {
            username: message.author.username,
            nickname: message.member?.displayName || message.author.username,
            is_admin: false, // All discord users treated as non-admin
        },
    };

    try {
        await pubClient.publish(ABCHAT_CHANNEL_ID, JSON.stringify(messageData));
        logger.debug({ messageId: message.id }, 'Published message to Redis');
    } catch (err) {
        logger.error({ err }, 'Failed to publish message to Redis');
    }
});

// --- Startup ---
const token = process.env['DISCORD_TOKEN'];

if (!token || !DISCORD_CHANNEL_ID) {
    logger.fatal('Missing DISCORD_TOKEN or DISCORD_CHANNEL_ID in environment variables.');
    process.exit(1);
}

client.login(token).catch((err) => {
    logger.error({ err }, 'Failed to login to Discord');
    process.exit(1);
});
