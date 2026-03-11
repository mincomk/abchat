import { Client, Events, GatewayIntentBits, Message, TextChannel } from 'discord.js';
import 'dotenv/config';
import { logger } from './lib/logger.js';
import { pubClient, subClient } from './lib/redis.js';
import { MessageSchema } from './lib/types.js';
import { preprocess } from './lib/preprocess.js';

const DISCORD_CHANNEL_ID = process.env['DISCORD_CHANNEL_ID'];
const ABCHAT_CHANNEL_ID = process.env['ABCHAT_CHANNEL_ID'] || 'bot:messages';

const mainTopic = 'chat:' + ABCHAT_CHANNEL_ID

const client = new Client({
    intents: [
        GatewayIntentBits.Guilds,
        GatewayIntentBits.GuildMessages,
        GatewayIntentBits.MessageContent,
    ],
});

let sentIds: string[] = [];

client.once(Events.ClientReady, (readyClient) => {
    logger.info(`Ready! Logged in as ${readyClient.user.tag}`);
});

// --- Redis -> Discord ---
subClient.subscribe(mainTopic, (err) => {
    if (err) {
        logger.error({ err }, 'Failed to subscribe to Redis channel');
    } else {
        logger.info(`Listening for messages on Redis channel: ${ABCHAT_CHANNEL_ID}`);
    }
});

subClient.on('message', async (topic, message) => {
    if (topic !== mainTopic) return;

    try {
        const data: MessageSchema = JSON.parse(message);

        if (sentIds.includes(data.id)) {
            sentIds = sentIds.filter(id => id != data.id)
        } else {
            const discordChannel = client.channels.cache.get(DISCORD_CHANNEL_ID!);

            if (discordChannel instanceof TextChannel) {
                await discordChannel.send(`**${data.sender.nickname}**: ${data.content}`);
            }
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
        content: message.guild ? preprocess(message.guild, message.content) : message.content,
        timestamp: message.createdTimestamp,
        channel_id: ABCHAT_CHANNEL_ID,
        sender: {
            username: message.author.username,
            nickname: message.member?.displayName || message.author.username,
            is_admin: false, // All discord users treated as non-admin
        },
    };

    try {
        sentIds.push(message.id)
        await pubClient.publish(mainTopic, JSON.stringify(messageData));

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
