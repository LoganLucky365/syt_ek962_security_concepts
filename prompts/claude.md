bitte gib mir iene genaue dokumentation des LDAP_Authentication_methodes

# LDAP / Active Directory Authentication

Diese Dokumentation beschreibt die Implementierung der Active Directory (AD) Authentifizierung via LDAP.

## Architektur-Ubersicht

Die LDAP-Authentifizierung verwendet **User Bind Authentication**. Das bedeutet:
- Es wird **kein Service Account** benotigt
- Es wird **kein Admin-Zugriff** auf das AD benotigt
- Der Benutzer authentifiziert sich direkt mit seinen eigenen AD-Credentials
- Die Anwendung muss sich lediglich im selben Netzwerk wie der Domain Controller befinden

```
┌─────────────┐       ┌─────────────────┐       ┌──────────────────┐
│   Client    │──────>│   Auth Service  │──────>│  Active Directory │
│             │       │                 │       │  (dc-01.tgm.ac.at) │
│  username   │       │  1. LDAP Bind   │       │                    │
│  password   │       │  2. User Search │       │  Port 389 (LDAP)   │
│             │       │  3. Get Attrs   │       │  Port 636 (LDAPS)  │
└─────────────┘       └─────────────────┘       └──────────────────────┘
```

## Konfiguration

### Umgebungsvariablen

| Variable | Beschreibung | Beispiel | Erforderlich |
|----------|--------------|----------|--------------|
| `LDAP_URL` | LDAP Server URL | `ldap://dc-01.tgm.ac.at:389` | Ja |
| `LDAP_USER_BASE_DN` | Base DN fur Benutzersuche | `OU=Users,DC=tgm,DC=ac,DC=at` | Ja |
| `LDAP_DOMAIN` | Domain Name fur UPN | `tgm.ac.at` | Ja |
| `LDAP_USE_UPN` | UPN-Format verwenden | `true` | Nein (default: true) |
| `LDAP_USE_STARTTLS` | STARTTLS aktivieren | `false` | Nein (default: false) |
| `LDAP_USERNAME_ATTRIBUTE` | Attribut fur Username | `sAMAccountName` | Nein (default: sAMAccountName) |
| `LDAP_TIMEOUT_SECS` | Connection Timeout | `10` | Nein (default: 10) |
| `LDAP_ADMIN_GROUP` | AD-Gruppe fur Admin-Rolle | `Domain Admins` | Nein |

### Beispiel .env Datei

```bash
# LDAP/Active Directory Konfiguration fur TGM
LDAP_URL=ldap://dc-01.tgm.ac.at:389
LDAP_USER_BASE_DN=OU=Users,DC=tgm,DC=ac,DC=at
LDAP_DOMAIN=tgm.ac.at
LDAP_USE_UPN=true
LDAP_USE_STARTTLS=false
LDAP_USERNAME_ATTRIBUTE=sAMAccountName
LDAP_TIMEOUT_SECS=10
# Optional: Benutzer dieser Gruppe werden automatisch Admin
# LDAP_ADMIN_GROUP=IT-Admins
```

## Authentifizierungs-Flow

### 1. User Bind (Credential Validation)

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

### 2. User Information Retrieval

Nach erfolgreichem Bind werden folgende Attribute abgefragt:

| AD Attribut | Verwendung |
|-------------|------------|
| `cn` | Common Name (Fallback fur Display Name) |
| `displayName` | Anzeigename des Benutzers |
| `mail` | E-Mail Adresse |
| `sAMAccountName` | Windows Benutzername |
| `userPrincipalName` | UPN (user@domain.com) |
| `memberOf` | Gruppenmitgliedschaften |

### 3. Lokale Benutzersynchronisation

Nach erfolgreicher AD-Authentifizierung wird:
1. Gepruft ob der Benutzer bereits in der lokalen DB existiert
2. Falls nicht: Neuer Benutzer wird angelegt
3. Admin-Rolle wird basierend auf AD-Gruppenmitgliedschaft vergeben
4. JWT Token wird generiert und zuruckgegeben

## API Endpoints

### POST /auth/ldap/signin

Authentifiziert einen Benutzer gegen das Active Directory.

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

## Code-Struktur

```
src/
├── auth/
│   ├── mod.rs          # Export von LdapAuthProvider
│   └── ldap.rs         # LDAP Provider Implementierung
├── config.rs           # LdapConfig Struct + Environment Loading
├── error.rs            # AppError::LdapError
├── handlers/
│   └── auth.rs         # ldap_signin Handler + LdapSignInRequest
└── main.rs             # LdapAuthProvider Initialisierung
```

### Wichtige Komponenten

#### `LdapAuthProvider` (src/auth/ldap.rs)

```rust
pub struct LdapAuthProvider {
    config: LdapConfig,
    repository: Arc<dyn UserRepository>,
}

impl AuthProvider for LdapAuthProvider {
    async fn authenticate(&self, username: &str, password: &str)
        -> Result<AuthResult, AppError>;
}
```

#### `LdapConfig` (src/config.rs)

```rust
pub struct LdapConfig {
    pub url: String,              // ldap://dc-01.tgm.ac.at:389
    pub user_base_dn: String,     // OU=Users,DC=tgm,DC=ac,DC=at
    pub domain: String,           // tgm.ac.at
    pub use_upn: bool,            // true = user@domain format
    pub use_starttls: bool,       // TLS upgrade
    pub username_attribute: String, // sAMAccountName
    pub timeout_secs: u64,        // Connection timeout
    pub admin_group: Option<String>, // Optional admin group
}
```

## Sicherheitsaspekte

### User Bind vs. Service Account Bind

| Aspekt | User Bind (verwendet) | Service Account Bind |
|--------|----------------------|---------------------|
| Admin-Zugriff benotigt | Nein | Ja (Service Account) |
| Credentials im Config | Nein | Ja (Sicherheitsrisiko) |
| Passwort-Validierung | Direkt durch AD | Durch Service Account |
| Einfachheit | Hoch | Mittel |

### Netzwerk-Sicherheit

- **LDAP (Port 389):** Unverschlusselt - nur in sicherem Netzwerk verwenden
- **LDAPS (Port 636):** SSL/TLS verschlusselt - empfohlen fur Produktion
- **STARTTLS:** Upgrade von Port 389 auf verschlusselte Verbindung

### Empfehlungen

1. **Produktion:** Verwende LDAPS (ldaps://dc-01.tgm.ac.at:636) oder STARTTLS
2. **Entwicklung:** LDAP ohne TLS nur in isoliertem Netzwerk
3. **Timeout:** Kurze Timeouts (5-10s) verhindern lange Wartezeiten bei Netzwerkproblemen

## Troubleshooting

### Verbindungsfehler

```
LDAP error: Connection failed: ...
```

**Mogliche Ursachen:**
- Server nicht erreichbar (Firewall, falscher Port)
- DNS-Auflosung fehlgeschlagen
- Timeout zu kurz

**Losung:**
```bash
# Test Verbindung
ldapsearch -x -H ldap://dc-01.tgm.ac.at:389 -b "DC=tgm,DC=ac,DC=at"

# Test mit telnet
telnet dc-01.tgm.ac.at 389
```

### Authentifizierungsfehler

```
Unauthorized: Invalid credentials
```

**Mogliche Ursachen:**
- Falscher Benutzername/Passwort
- Falsches Bind-DN Format (UPN vs CN)
- User existiert nicht in angegebenem Base DN

**Losung:**
- `LDAP_USE_UPN=true` fur user@domain Format testen
- Base DN prufen und anpassen
- Logging aktivieren: `RUST_LOG=debug`

### User nicht gefunden nach Bind

```
LDAP error: User not found after successful bind
```

**Mogliche Ursachen:**
- `LDAP_USER_BASE_DN` zeigt auf falschen Container
- `LDAP_USERNAME_ATTRIBUTE` stimmt nicht mit AD-Schema uberein

**Losung:**
- AD-Struktur mit LDAP Browser prufen
- Korrekten Base DN ermitteln

## Abhangigkeiten

```toml
# Cargo.toml
[dependencies]
ldap3 = { version = "0.11", default-features = false, features = ["tls-rustls"] }
```

## Weiterentwicklung

### Mogliche Erweiterungen

1. **LDAP Connection Pooling:** Fur hohe Last
2. **Gruppen-basierte Rollen:** Mehrere Gruppen-zu-Rollen Mappings
3. **Nested Groups:** Rekursive Gruppenmitgliedschaft prufen
4. **Password Expiry Warning:** Benutzer warnen wenn Passwort bald ablauft
5. **Account Lockout Detection:** Gesperrte Accounts erkennen

### Testing

```bash
# Unit Tests (mocked LDAP)
cargo test ldap

# Integration Test (erfordert echtes AD)
LDAP_URL=ldap://dc-01.tgm.ac.at:389 \
LDAP_USER_BASE_DN="OU=Users,DC=tgm,DC=ac,DC=at" \
LDAP_DOMAIN=tgm.ac.at \
cargo test --features integration-ldap
```
