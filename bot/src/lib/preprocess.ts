import { Guild } from "discord.js";

export function preprocess(guild: Guild, text: string): string {
    // replace role, user, channel mention.

    // role mention
    const roleMentionRegex = /<@&(\d+)>/g;
    text = text.replace(roleMentionRegex, (_, id) => {
        const role = guild.roles.cache.get(id);
        const roleName = role ? role.name : "unknown role";

        return `<@&${roleName}:${id}>`
    });

    // user mention
    const userMentionRegex = /<@!?(\d+)>/g;
    text = text.replace(userMentionRegex, (_, id) => {
        const member = guild.members.cache.get(id);
        const name = member ? member.displayName : "unknown user";

        return `<@${name}:${id}>`
    });

    // channel mention
    const channelMentionRegex = /<#(\d+)>/g;
    text = text.replace(channelMentionRegex, (_, id) => {
        const channel = guild.channels.cache.get(id);
        const name = channel ? channel.name : "unknown channel"

        return `<#${name}:${id}>`
    });

    // remove starting line of #, ##, ###, ####. start + # or \n + #
    const headingRegex = /(^#{1,4}|\n#{1,4}) /g;
    text = text.replace(headingRegex, "");

    return text;
}
