# DBridge Web Provider API

The Web Provider exposes both a WebSocket interface for real-time messaging and an HTTP interface for authentication and historical message retrieval.

## Authentication

All endpoints (except login) require authentication using a JSON Web Token (JWT).

- **HTTP**: Pass the token in the `Authorization` header: `Authorization: Bearer <your_jwt_token>`
- **WebSocket**: Send an `identify` command immediately after connecting.

## HTTP Endpoints

### Login
Authenticates a user and returns a JWT.
- **URL**: `POST /auth/login`
- **Authentication**: None
- **Body**:
```json
{
  "username": "jdoe",
  "password": "yourpassword"
}
```
- **Response**:
```json
{
  "token": "<jwt_token>",
  "user": {
    "username": "jdoe",
    "nickname": "John Doe",
    "is_admin": false
  }
}
```

### Change Password
Changes the password of the authenticated user.
- **URL**: `POST /auth/change-password`
- **Authentication**: Required (Bearer Token)
- **Body**:
```json
{
  "old_password": "current_password",
  "new_password": "new_password"
}
```

## Push Notifications

### Get VAPID Public Key
Retrieves the VAPID public key needed for the browser's `PushManager` subscription.
- **URL**: `GET /notifications/vapid-key`
- **Authentication**: None
- **Response**:
```json
{
  "public_key": "B..."
}
```

### Subscribe
Saves a new push notification subscription for the authenticated user.
- **URL**: `POST /notifications/subscribe`
- **Authentication**: Required (Bearer Token)
- **Body**:
```json
{
  "endpoint": "https://fcm.googleapis.com/fcm/send/...",
  "p256dh": "...",
  "auth": "..."
}
```

### Unsubscribe
Removes all push subscriptions for the authenticated user.
- **URL**: `POST /notifications/unsubscribe`
- **Authentication**: Required (Bearer Token)
- **Response**: `200 OK`

### Get Settings
Retrieves the current notification settings.
- **URL**: `GET /notifications/settings`
- **Authentication**: Required (Bearer Token)
- **Response Example**:
```json
{
  "settings": {
    "notification_mode": "All"
  }
}
```

### Update Settings
Updates the notification preferences.
- **URL**: `PUT /notifications/settings`
- **Authentication**: Required (Bearer Token)
- **Body**:
```json
{
  "settings": {
    "notification_mode": "Critical"
  }
}
```

## Admin Endpoints

All admin endpoints require an Admin Bearer Token.

### Register User
Creates a new user account.
- **URL**: `POST /admin/register`
- **Authentication**: Required (Admin Bearer Token)
- **Body**:
```json
{
  "username": "newuser",
  "password": "securepassword",
  "nickname": "New User"
}
```

### List Accounts
Retrieves all registered user accounts.
- **URL**: `GET /admin/accounts`
- **Authentication**: Required (Admin Bearer Token)
- **Response Example**:
```json
[
  {
    "username": "admin",
    "nickname": "Administrator",
    "is_admin": true
  },
  {
    "username": "jdoe",
    "nickname": "John Doe",
    "is_admin": false
  }
]
```

### Update Admin Status
Promotes or demotes a user account.
- **URL**: `PATCH /admin/accounts/:username/admin`
- **Authentication**: Required (Admin Bearer Token)
- **Body**:
```json
{
  "is_admin": true
}
```

### Change User Password
Changes the password of a user account.
- **URL**: `POST /admin/accounts/:username/password`
- **Authentication**: Required (Admin Bearer Token)
- **Body**:
```json
{
  "new_password": "new_secure_password"
}
```

### Delete Account
Deletes a user account. Cannot delete yourself.
- **URL**: `DELETE /admin/accounts/:username`
- **Authentication**: Required (Admin Bearer Token)
- **Response**: `204 No Content`

### Get Messages
Retrieves historical messages for the web platform.
- **URL**: `GET /channels/{channel_id}/messages`
- **Authentication**: Required (Bearer Token)
- **Path Parameters**:
    - `channel_id` (string): Filter messages by a specific remote channel ID.
- **Query Parameters**:
    - `limit` (integer, optional): Maximum number of messages to return. Default: `50`, Maximum: `100`.
    - `offset` (integer, optional): Number of messages to skip for pagination. Default: `0`.

**Response Example (`200 OK`)**:
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "sender": {
      "username": "jdoe",
      "nickname": "John Doe",
    },
    "content": "Hello everyone!",
    "timestamp": 1710000000,
    "channel_id": "general"
  }
]
```

## WebSocket Interface

- **URL**: `ws://<address>/ws/<channel_id>`

### Client Commands

#### Identify
Must be the first message sent.
```json
{
  "type": "identify",
  "token": "<jwt_token>"
}
```

#### Send Message
```json
{
  "type": "send_message",
  "content": "Your message here"
}
```

### Server Messages

The server broadcasts messages to all connected clients in a channel:
```json
{
  "id": "...",
  "sender": { "username": "...", "nickname": "...", "is_admin": false },
  "content": "...",
  "timestamp": 123456789
}
```
