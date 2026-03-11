#!/bin/sh

node /write-env.js > /usr/share/nginx/html/runtime-env.js

exec nginx -g "daemon off;"
