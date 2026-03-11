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
