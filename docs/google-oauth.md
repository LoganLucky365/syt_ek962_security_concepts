# Google OAuth Implementation

This document describes the Google OAuth 2.0 integration for the authentication service.

## Overview

The service supports Google OAuth 2.0 authorization code flow, allowing users to authenticate using their Google accounts. When a user authenticates via Google, the service:

1. Redirects the user to Google's authorization page
2. Receives an authorization code via callback
3. Exchanges the code for access tokens
4. Fetches the user's profile from Google
5. Creates or retrieves the user in the database
6. Issues a JWT token for API access

## Setup

### 1. Google Cloud Console Configuration

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Navigate to **APIs & Services** > **Credentials**
4. Click **Create Credentials** > **OAuth client ID**
5. Select **Web application** as the application type
6. Add authorized redirect URIs:
   - Development: `http://localhost:8080/auth/google/callback`
   - Production: `https://your-domain.com/auth/google/callback`
7. Copy the **Client ID** and **Client Secret**

### 2. Environment Variables

Add the following to your `.env` file:

```bash
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-client-secret
GOOGLE_REDIRECT_URI=http://localhost:8080/auth/google/callback
```

For production, update `GOOGLE_REDIRECT_URI` to use HTTPS and your actual domain.

## API Endpoints

### GET /auth/google/login

Initiates the Google OAuth flow by returning the authorization URL.

**Request:**
```
GET /auth/google/login
```

**Response (200 OK):**
```json
{
    "authorization_url": "https://accounts.google.com/o/oauth2/v2/auth?client_id=...&redirect_uri=...&response_type=code&scope=openid%20email%20profile&state=abc123...",
    "state": "abc123..."
}
```

**Response (400 Bad Request) - if Google OAuth not configured:**
```json
{
    "error": "Google OAuth is not configured"
}
```

**Client Usage:**
1. Call this endpoint
2. Store the `state` value for CSRF verification
3. Redirect the user to `authorization_url`

### GET /auth/google/callback

Handles the OAuth callback from Google. This endpoint is called by Google after user authorization.

**Request:**
```
GET /auth/google/callback?code=4/0ABC...&state=abc123...
```

**Query Parameters:**

| Parameter | Type   | Required | Description                    |
|-----------|--------|----------|--------------------------------|
| code      | string | Yes      | Authorization code from Google |
| state     | string | Yes      | CSRF state parameter           |

**Response (200 OK):**
```json
{
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "user": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "John Doe",
        "email": "john@gmail.com",
        "role": "user",
        "created_at": "2024-01-15T10:30:00Z"
    },
    "is_new_user": true
}
```

**Error Responses:**

| Status | Description                                    |
|--------|------------------------------------------------|
| 400    | Invalid or expired authorization code          |
| 400    | Email not verified by Google                   |
| 409    | Email already registered with different method |

## Authentication Flow

```
┌──────────┐     ┌─────────────┐     ┌────────────┐     ┌──────────────┐
│  Client  │     │ Auth Service│     │   Google   │     │   Database   │
└────┬─────┘     └──────┬──────┘     └─────┬──────┘     └──────┬───────┘
     │                  │                  │                   │
     │ GET /auth/google/login              │                   │
     │─────────────────>│                  │                   │
     │                  │                  │                   │
     │ authorization_url + state           │                   │
     │<─────────────────│                  │                   │
     │                  │                  │                   │
     │ Redirect to Google                  │                   │
     │────────────────────────────────────>│                   │
     │                  │                  │                   │
     │ User authenticates                  │                   │
     │<────────────────────────────────────│                   │
     │                  │                  │                   │
     │ GET /auth/google/callback?code=...  │                   │
     │─────────────────>│                  │                   │
     │                  │                  │                   │
     │                  │ Exchange code    │                   │
     │                  │─────────────────>│                   │
     │                  │                  │                   │
     │                  │ Access token     │                   │
     │                  │<─────────────────│                   │
     │                  │                  │                   │
     │                  │ GET /userinfo    │                   │
     │                  │─────────────────>│                   │
     │                  │                  │                   │
     │                  │ User profile     │                   │
     │                  │<─────────────────│                   │
     │                  │                  │                   │
     │                  │ Find/Create user │                   │
     │                  │─────────────────────────────────────>│
     │                  │                  │                   │
     │                  │ User record      │                   │
     │                  │<─────────────────────────────────────│
     │                  │                  │                   │
     │ JWT token + user │                  │                   │
     │<─────────────────│                  │                   │
     │                  │                  │                   │
```

## User Management

### New Users

When a user authenticates via Google for the first time:
- A new user record is created with `auth_provider = 'google'`
- The Google user ID is stored in `external_id`
- The user's name and email are populated from Google's profile
- Default role is `user`
- `is_new_user: true` is returned in the response

### Existing Users

When a user with an existing Google account authenticates:
- The user is looked up by `external_id` (Google's `sub` claim)
- `is_new_user: false` is returned in the response

### Email Conflicts

If a user tries to authenticate via Google with an email that's already registered with a different authentication method (e.g., local password), the request is rejected with a 409 Conflict error. This prevents account hijacking and ensures users authenticate with their original method.

## Security Considerations

### CSRF Protection

The `state` parameter is generated using cryptographically secure randomness. Clients should:
1. Store the `state` value before redirecting to Google
2. Verify the `state` value matches after the callback
3. Reject requests with mismatched state values

### Email Verification

Only users with verified emails (`email_verified: true` from Google) are accepted. This prevents account creation with unverified email addresses.

### Token Storage

Google access tokens and refresh tokens are NOT stored. Only the user's profile information (sub, email, name) is retained. This minimizes the impact of a potential database breach.

### HTTPS

In production, always use HTTPS for the redirect URI. Google will reject HTTP redirect URIs for production OAuth credentials.

## Troubleshooting

### "Google OAuth is not configured"

Ensure the following environment variables are set:
- `GOOGLE_CLIENT_ID`
- `GOOGLE_CLIENT_SECRET`

### "Failed to exchange authorization code"

- Verify the authorization code hasn't expired (they're single-use and short-lived)
- Check that the redirect URI matches exactly what's configured in Google Cloud Console
- Ensure the client secret is correct

### "Email not verified by Google"

The user's Google account has an unverified email address. They need to verify their email with Google first.

### "Email already registered with a different authentication method"

The email is already associated with a local account. The user should sign in with their original method (email/password) or contact support to link accounts.

## Files Modified/Created

| File                        | Description                           |
|-----------------------------|---------------------------------------|
| `src/auth/google.rs`        | Google OAuth provider implementation  |
| `src/auth/mod.rs`           | Export GoogleAuthProvider             |
| `src/handlers/oauth.rs`     | HTTP handlers for OAuth endpoints     |
| `src/handlers/mod.rs`       | Export oauth module                   |
| `src/handlers/auth.rs`      | Updated AppState and routes           |
| `src/config.rs`             | GoogleOAuthConfig struct              |
| `src/error.rs`              | OAuthError variant                    |
| `src/models/user.rs`        | User::new_external() constructor      |
| `src/main.rs`               | Initialize GoogleAuthProvider         |
| `.env.example`              | Google OAuth environment variables    |
