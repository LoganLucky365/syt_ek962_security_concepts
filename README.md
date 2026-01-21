[![Maintenance](https://img.shields.io/badge/Maintained%3F-no-red.svg)](https://GitHub.com/Naereen/StrapDown.js/graphs/commit-activity)
[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
[![GitHub license](https://img.shields.io/github/license/Naereen/StrapDown.js.svg)](https://github.com/Naereen/StrapDown.js/blob/master/LICENSE)
![Promille](https://img.shields.io/badge/Promille-0.67‰-yellow)

# SYT EK 962 Security Concepts

## Aufgabenstellung

**Recherche**
Als Junior-Developer erhalten Sie von Ihrem technischen Architekten die Aufgabe, eine Gegenüberstellung von gängigen Authentifikationssystemen zu erstellen. Beachten Sie bei der Auflistung von Vor- und Nachteilen die notwendigen funktionalen Anforderungen:

Anbindung an ein bestehende Authentifikations-Services
Token-Authentifizierung, um ein andauerndes Login bei mehreren Services gering zu halten bzw. auf ein einmaliges Authentifizieren zu beschränken
Verwendung auf verschiedenen Implementierungen und Devices
Diese Gegenüberstellung soll kurz und bündig auf die gängigen Systeme und Technologien verweisen. Es sollen keine Informationen kopiert sondern nur kurz zusammengefasst werden.

**Implementierung**
Um das in Entwicklung befindliche Online-Portal zur Bereitstellung von Informationen des Windparks entsprechend einer breiten Community schmackhaft zu machen, und trotzdem eine Authentifizierung anzubieten, werden Sie beauftragt einen Prototypen für Spring-Boot zu entwickeln, der sich über Sozialen-Netzwerken authentifizieren kann (Facebook, Google+, etc.).

Es soll ebenfalls eine Implementierung bereitgestellt werden, die sich an einen aktiven AD-Service verbinden und entsprechend die Rollen von bestimmten Gruppen unterscheiden kann. Dabei sollen zwei Bereiche festgelegt werden, die nur von einer bestimmten Gruppe betreten werden kann.

Die Prototypen sollen klar und einfach gehalten werden und mit entsprechender Dokumentation versehen sein, um eine einfache Integrierung in das bestehende Spring-Boot Projekt zu ermöglichen.

### Recherchen

In eigene .md FIles im root dir ausgelagert

## Dokumentation der durchgeführten Arbeitsschritte

### Misc


### Social Media Authentication (Google)

jwt secret erstellen

```bash
export JWT_SECRET=$(openssl rand -hex 32)
```

#### Sign In

Einlogen als user

```http
POST /auth/signin
Content-Type: application/json

{
    "email": "admin@admin.com",
    "password": "admin123456789"
}
```

```shell
curl -X POST localhost:8080/auth/signin \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@admin.com",
    "password": "admin123456789"
  }'
```

Response 200

Error Response 401

#### Register User

Create new user account & only as admin

```http
POST /auth/admin/register
Authorization: Bearer jwt_tocken_placeholder
Content-Type: application/json

{
    "name": "John Doe",
    "email": "john.doe@example.com",
    "password": "pwhash",
    "role": "user"
}
```

```shell
curl -X POST localhost:8080/auth/admin/register \
  -H "Authorization: Bearer jwt_tocken_placeholder" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Rainer Zufall",
    "email": "rainer.zufall@nobody.ru",
    "password": "pwhash",
    "role": "user"
  }'
```

Response 2001

Error Response 401, 403, 409

#### Verify Token

validate jwt

```http
POST /auth/verify
Content-Type: application/json

{
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

```shell
curl -X POST localhost:8080/auth/verify \
  -H "Content-Type: application/json" \
  -d '{
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }'

```

Response 200

Error Response 403

#### Config

config from .env -> env.example

| Variable | Default | Description        |
|----------|---------|--------------------|
| `HOST` | `127.0.0.1` | Server address     |
| `PORT` | `8080` | port               |
| `DATABASE_URL` | `sqlite:./auth.db?mode=rwc` | dbpath             |
| `JWT_SECRET` | (random) | min 32 chars       |
| `JWT_EXPIRATION_SECS` | `3600` | token period       |
| `JWT_ISSUER` | `auth-service` | token issuer claim |
| `INITIAL_ADMIN_CONFIG` | `initial_admin.json` | initial admin conf |
| `RUST_LOG` | `info,sqlx=warn` | logging            |

#### Admin setup

On startup admin account gets created -> abholen aus json

```json
{
    "name": "bofh",
    "email": "admin@bofh.com",
    "password_hash": "$argon2id$v=19$m=65536,t=3,p=4$randomsalt$hashedpassword"
}
```

Generate the hash:
```bash
cargo run --bin hash_password
```

#### Password Hashing Argon2

Warum Argon2 -> is goated (und Roschger sagt) -> und owasp btw

| Parameter | Value | Purpose                              |
|-----------|------|--------------------------------------|
| Memory | 64 | owasp sagt                           |
| Iterations | 3 | Time cost                            |
| Parallelism | 4 | cpu util threads                     |
| Algorithm | Argon2id | sowohl side channel als auch gpu res |

#### JWT

Generiert HS256
sub email role exp iat iss
expires one hour

### User per AD

Um die user über das AD einbeziehen zu können greife ich auf ldap zurück.

#### Architektur

ldap wird über user binding angesprochen. Daher
- keine service ac auf ad
- kein admin zugang auf ad
- authentifizierung direkt mit seinen userdaten
- muss logischerweise im sleben netz wie controller sein

AI grafik act kinda cool

```
┌─────────────┐       ┌─────────────────┐       ┌────────────────────┐
│   Client    │──────>│   Auth Service  │──────>│  Active Directory  │
│             │       │                 │       │  (dc-01.tgm.ac.at) │
│  username   │       │  1. LDAP Bind   │       │                    │
│  password   │       │  2. User Search │       │  Port 389 (LDAP)   │
│             │       │  3. Get Attrs   │       │  Port 636 (LDAPS)  │
└─────────────┘       └─────────────────┘       └────────────────────┘
```

#### Konfig

Umgebungsvar

| Variable | Beschreibung   | Beispiel | Erforderlich |
|----------|----------------|----------|--------------|
| `LDAP_URL` | Serverurl      | `ldap://dc-01.tgm.ac.at:389` | Ja |
| `LDAP_USER_BASE_DN` | Domain Name    | `OU=Users,DC=tgm,DC=ac,DC=at` | Ja |
| `LDAP_DOMAIN` | Domain Name UPN | `tgm.ac.at` | Ja |
| `LDAP_USE_UPN` | UPN Format     | `true` | Nein (default: true) |
| `LDAP_USE_STARTTLS` | STARTTLS       | `false` | Nein (default: false) |
| `LDAP_USERNAME_ATTRIBUTE` | Attribut Username | `sAMAccountName` | Nein (default: sAMAccountName) |
| `LDAP_TIMEOUT_SECS` | Timeout        | `10` | Nein (default: 10) |
| `LDAP_ADMIN_GROUP` | Gruppe admin   | `Domain Admins` | Nein |

#### Beispielflow von ai find ich ganz cool

```
Client sendet: username + password
     │
     ▼
Service baut Bind-DN:
  - UPN Format: username@tgm.ac.at
  - oder DN Format: CN=username,OU=Users,DC=tgm,DC=ac,DC=at
     │
     ▼
LDAP simple_bind() mit User-Credentials
     │
     ├── Erfolg: Credentials sind gültig
     │
     └── Fehler: Ungültige Credentials (401 Unauthorized)
```

#### Bind fragt attr an

| AD Attribut | Verwendung          |
|-------------|---------------------|
| `cn` | common name         |
| `displayName` | name                |
| `mail` | email               |
| `sAMAccountName` | windowUsername      |
| `userPrincipalName` | UPN user@domain.com |
| `memberOf` | group               |

#### locales syncen
1. User scho in db
2. benutzer anlegen
3. ad groupes adminrechte vergeben
4. JWT token generieren

#### API

**/auth/ldap/signin**

authentifizieren gegen ad

**Request:**
```json
{
    "username": "mustermann",
    "password": "geheim123"
}
```

**Response (Erfolg):**
```json
{
    "token": "eyJhbGciOiJIUzI1NiIs...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "user": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Max Mustermann",
        "email": "mustermann@tgm.ac.at",
        "role": "user",
        "created_at": "2025-01-21T10:30:00Z"
    }
}
```

**Response (Fehler):**
```json
{
    "error": "Invalid credentials"
}
```
#### ldapsearch

```bash
ldapsearch -x -H ldap://dc-01.tgm.ac.at:389 -b "DC=tgm,DC=ac,DC=at"

telnet dc-01.tgm.ac.at 389
```

#### Crate

```toml
ldap3 = { version = "0.11", default-features = false, features = ["tls-rustls"] }
```

### Tests

```bash
cargo test
```

### Test logging

```bash
RUST_LOG=debug cargo run
```

Test Reports

## Quellen

[1] “Doc OAuth Crate” Accessed: Jan. 21, 2026. [Online]. Available: https://crates.io/crates/oauth2

[2] “Herocu docs” Accessed: Jan. 09, 2026. [Online]. Available: https://devcenter.heroku.com/

[3] “google cloud access” Accessed: Jan. 21, 2026. [Online]. Available: https://console.cloud.google.com/apis/credentials

[4] “Diesel CSV documentation” Accessed: Jan. 19, 2026. [Online]. Available: https://diesel.readthedocs.io/en/latest/csv.html

[5] "badges doc" Accessed: Jan. 20, 2026. [Online]. Available: https://naereen.github.io/badges/