import { Redis } from 'ioredis';
import { logger } from './logger.js';

const redisUrl = process.env['REDIS_URL'] || 'redis://localhost:6379';

export const pubClient = new Redis(redisUrl);
export const subClient = new Redis(redisUrl);

pubClient.on('error', (err: Error) => logger.error({ err }, 'Redis Pub Client Error'));
subClient.on('error', (err: Error) => logger.error({ err }, 'Redis Sub Client Error'));

pubClient.on('connect', () => logger.info('Redis Pub Client Connected'));
subClient.on('connect', () => logger.info('Redis Sub Client Connected'));
