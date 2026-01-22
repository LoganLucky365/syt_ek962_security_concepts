Swordpro2
swordpro2
Online

LoganLucky — 12.12.2025 10:13
Weitergeleitet
src/app/app.config.ts

export const appConfig: ApplicationConfig = {
  providers: [
    provideFirebaseApp(() => initializeApp(environment.firebase)),
    provideFirestore(() => getFirestore()),
    provideAuth(() => getAuth()),
    provideFunctions(() => getFunctions()),
    provideStorage(() => getStorage()),
    provideMessaging(() => getMessaging()),
    provideAnalytics(() => getAnalytics()),
    ScreenTrackingService,
    UserTrackingService,
    provideRouter(routes)
  ],
};
Weitergeleitet
src/environments/environment.ts

export const environment = {
  firebase: {
    apiKey: "AIzaSyBR72P1g4LNX7nc0LWFPS6oJx3lN8r8bQE",
    authDomain: "friendlychat-96408.firebaseapp.com",
    projectId: "friendlychat-96408",
    storageBucket: "friendlychat-96408.firebasestorage.app",
    messagingSenderId: "718568030174",
    appId: "1:718568030174:web:0c9467bbc2dcbbf59f274f",
    measurementId: "G-FTY14PPRM1"
  },
};
Weitergeleitet
src/firebase-config.js

@@ -0,0 +1,10 @@
// For Firebase JS SDK v7.20.0 and later, measurementId is optional
const firebaseConfig = {
  apiKey: "AIzaSyBR72P1g4LNX7nc0LWFPS6oJx3lN8r8bQE",
  authDomain: "friendlychat-96408.firebaseapp.com",
  projectId: "friendlychat-96408",
  storageBucket: "friendlychat-96408.firebasestorage.app",
  messagingSenderId: "718568030174",
  appId: "1:718568030174:web:0c9467bbc2dcbbf59f274f",
  measurementId: "G-FTY14PPRM1"
};
LoganLucky — 13.01.2026 12:10
gib mal link zu deinen frontend deployment
Swordpro2

 — 13.01.2026 12:12
sex.puerto.cloud
aber ist nd fertig
LoganLucky — 13.01.2026 12:12
och man kein sex.gordlby.at
achso kein login das meinst... aber der light/dark switch is crazy xD
LoganLucky — 20.01.2026 22:36
jo dumme frage aber hast du bei dir in der db überhaupt werte (INSY) mit den Flags F/V?
Swordpro2

 — 20.01.2026 22:50
ich schau grad selber aber ich find iwie nix
LoganLucky — 20.01.2026 22:50
gut ich bin noch ned komplett geistig
Swordpro2

 — 20.01.2026 22:50
Also ich weiß das ich flags hab aber seh keine mit F oder V
LoganLucky — 20.01.2026 22:50
dachte scho kann gar kein sql mehr
LoganLucky — 20.01.2026 22:50
jaja fix
Swordpro2

 — 20.01.2026 22:50
Ja ich hab auch so gedacht das ich komplett besoffen bin
Ich müsdt eigentlich langsam mal das promill badge erstellen
LoganLucky — 20.01.2026 22:52
so jetzt fehlt ma eig nur noch der globale filter drüber und der berricht mit sql funktionen
aber kp von dem
als der berricht
btw wie war rust abgabe?
Swordpro2

 — 20.01.2026 22:53
hab ich nd mehr gemacht
LoganLucky — 20.01.2026 22:53
oh schade
LoganLucky — gestern um 00:06 Uhr
Bild
Swordpro2

 — gestern um 00:18 Uhr
lol
LoganLucky — gestern um 20:43 Uhr
jo wie schauts aus mit den borko kubernetes ek?
und schick ma dann was ich commitn soll
Swordpro2

 — gestern um 21:15 Uhr
sorry hab bisher gepennt
willst du morgen bei graf es durchgehen?
LoganLucky — gestern um 21:16 Uhr
jaja fix machma
aber hast es?
oder soll ich auch das machen/mal anfangen?
Swordpro2

 — gestern um 21:17 Uhr
Ja nein ich kanns machen das geht ja eh leicht
LoganLucky — gestern um 21:27 Uhr
kk mich störts eh ned
LoganLucky — 14:42
# Recherche der Technologien

## Einleitung

Als Junior-Developer erhalten Sie von Ihrem technischen Architekten die Aufgabe, eine Gegenüberstellung von gängigen Authentifikationssystemen zu erstellen. Beachten Sie bei der Auflistung von Vor- und Nachteilen die notwendigen funktionalen Anforderungen:

- Anbindung an ein bestehende Authentifikations-Services
- Token-Authentifizierung, um ein andauerndes Login bei mehreren Services gering zu halten bzw. auf ein einmaliges Authentifizieren zu beschränken
- Verwendung auf verschiedenen Implementierungen und Devices
- Diese Gegenüberstellung soll kurz und bündig auf die gängigen Systeme und Technologien verweisen. Es sollen keine Informationen kopiert sondern nur kurz zusammengefasst werden.

## Single SignOn (SSO)

### Definition

Beim sogenannten Single SignOn handelt es sich ume in Authentifizierungsferfahren, womit sich ein User einmal anmeldet und damit auf verschiedene Softwaresysteme zugreifen kann und sich nicht immer erneut authentifizieren muss.

### Funktionsweise

Coole AI grafik (;

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Benutzer  │────▶│  Identity   │────▶│  Anwendung  │
│             │◀────│  Provider   │◀────│    A/B/C    │
└─────────────┘     └─────────────┘     └─────────────┘
      │                    │
      │  1. Einmalige      │  2. Token ausgestellt
      │     Anmeldung      │     
      │                    │
      └────────────────────┘
```

### Vorteile
- **Userverhalten**: Happy uer -> weniger logins
- **Verwaltung**: Einfachere Administration
- **Audit**: zentrale schnittstelle für alles und allem

### Nachteile
- **SPoF**: Fällt login dingens fällt alles
- **Komplexitäät**: relativ komplexe anwendung
- **Security**: sso komprimitiert -> zugriff auf alles

### Implementierung  
Wird meist durch follgende Protokolle implementiert:
- SAML 2 (Webgeschichten)
- OAuth 2 (API -> gibt JSON zurück)
- Kerberos (Tickets über netzwerk versendet)

---

## Kerberos

### Definition

Unter Kerberos kann man sich ein Protokoll vorstellen, welches seine "übertragungen" symmetrich verschlüsselt und über eine vertrauenswürdigen Drittdienst KDC benutzer authentifiziert.

### Komponenten
1. **Key Distribution Center (KDC)**
    - **Authentication Server (AS)**: User wird authentifiziert
    - **Ticket Granting Server (TGS)**: gibt Ticket aus

2. **Principal**: Benutzer/Dienst mit uniquer Domain
3. **Realm**: administrative Domain

### Authentifizierungsablauf

Ai hat act gekocht

```
┌──────────┐          ┌─────────────┐          ┌──────────┐
│  Client  │          │     KDC     │          │  Server  │
└────┬─────┘          └──────┬──────┘          └────┬─────┘
     │                       │                      │
     │ 1. AS-REQ (Benutzer)  │                      │
     │──────────────────────▶│                      │
     │                       │                      │
     │ 2. AS-REP (TGT)       │                      │
     │◀──────────────────────│                      │
     │                       │                      │
     │ 3. TGS-REQ (TGT +     │                      │
     │    Dienst-Anfrage)    │                      │
     │──────────────────────▶│                      │
     │                       │                      │
     │ 4. TGS-REP (Service   │                      │
     │    Ticket)            │                      │
     │◀──────────────────────│                      │
     │                       │                      │
     │ 5. AP-REQ (Service Ticket)                   │
     │─────────────────────────────────────────────▶│
     │                       │                      │
     │ 6. AP-REP (Zugriff gewährt)                  │
     │◀─────────────────────────────────────────────│
```

### Ticket struct
```
┌─────────────────────────────────────────────────────┐
│                    Ticket                           │
├─────────────────────────────────────────────────────┤
... (317 Zeilen verbleibend)
Einklappen
message.txt
20 kB
﻿
LoganLucky
.loganlucky
 
# Recherche der Technologien

## Einleitung

Als Junior-Developer erhalten Sie von Ihrem technischen Architekten die Aufgabe, eine Gegenüberstellung von gängigen Authentifikationssystemen zu erstellen. Beachten Sie bei der Auflistung von Vor- und Nachteilen die notwendigen funktionalen Anforderungen:

- Anbindung an ein bestehende Authentifikations-Services
- Token-Authentifizierung, um ein andauerndes Login bei mehreren Services gering zu halten bzw. auf ein einmaliges Authentifizieren zu beschränken
- Verwendung auf verschiedenen Implementierungen und Devices
- Diese Gegenüberstellung soll kurz und bündig auf die gängigen Systeme und Technologien verweisen. Es sollen keine Informationen kopiert sondern nur kurz zusammengefasst werden.

## Single SignOn (SSO)

### Definition

Beim sogenannten Single SignOn handelt es sich ume in Authentifizierungsferfahren, womit sich ein User einmal anmeldet und damit auf verschiedene Softwaresysteme zugreifen kann und sich nicht immer erneut authentifizieren muss.

### Funktionsweise

Coole AI grafik (;

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Benutzer  │────▶│  Identity   │────▶│  Anwendung  │
│             │◀────│  Provider   │◀────│    A/B/C    │
└─────────────┘     └─────────────┘     └─────────────┘
      │                    │
      │  1. Einmalige      │  2. Token ausgestellt
      │     Anmeldung      │     
      │                    │
      └────────────────────┘
```

### Vorteile
- **Userverhalten**: Happy uer -> weniger logins
- **Verwaltung**: Einfachere Administration
- **Audit**: zentrale schnittstelle für alles und allem

### Nachteile
- **SPoF**: Fällt login dingens fällt alles
- **Komplexitäät**: relativ komplexe anwendung
- **Security**: sso komprimitiert -> zugriff auf alles

### Implementierung  
Wird meist durch follgende Protokolle implementiert:
- SAML 2 (Webgeschichten)
- OAuth 2 (API -> gibt JSON zurück)
- Kerberos (Tickets über netzwerk versendet)

---

## Kerberos

### Definition

Unter Kerberos kann man sich ein Protokoll vorstellen, welches seine "übertragungen" symmetrich verschlüsselt und über eine vertrauenswürdigen Drittdienst KDC benutzer authentifiziert.

### Komponenten
1. **Key Distribution Center (KDC)**
    - **Authentication Server (AS)**: User wird authentifiziert
    - **Ticket Granting Server (TGS)**: gibt Ticket aus

2. **Principal**: Benutzer/Dienst mit uniquer Domain
3. **Realm**: administrative Domain

### Authentifizierungsablauf

Ai hat act gekocht

```
┌──────────┐          ┌─────────────┐          ┌──────────┐
│  Client  │          │     KDC     │          │  Server  │
└────┬─────┘          └──────┬──────┘          └────┬─────┘
     │                       │                      │
     │ 1. AS-REQ (Benutzer)  │                      │
     │──────────────────────▶│                      │
     │                       │                      │
     │ 2. AS-REP (TGT)       │                      │
     │◀──────────────────────│                      │
     │                       │                      │
     │ 3. TGS-REQ (TGT +     │                      │
     │    Dienst-Anfrage)    │                      │
     │──────────────────────▶│                      │
     │                       │                      │
     │ 4. TGS-REP (Service   │                      │
     │    Ticket)            │                      │
     │◀──────────────────────│                      │
     │                       │                      │
     │ 5. AP-REQ (Service Ticket)                   │
     │─────────────────────────────────────────────▶│
     │                       │                      │
     │ 6. AP-REP (Zugriff gewährt)                  │
     │◀─────────────────────────────────────────────│
```

### Ticket struct
```
┌─────────────────────────────────────────────────────┐
│                    Ticket                           │
├─────────────────────────────────────────────────────┤
│ Realm: TGM.AC.AT                                    │
│ Principal: user@TGM.AC.AT                           │
│ Session Key: [verschlüsselt]                        │
│ Gültig von: 2024-01-15 08:00:00                     │
│ Gültig bis: 2024-01-15 18:00:00                     │
│ Client Address: 192.168.1.100                       │
│ Authorization Data: [optional]                      │
└─────────────────────────────────────────────────────┘
```

### Vorteile
- **Pari Auth**: Client und Server verifizieren sich gegenseitig
- **Keine pw übertragung**: keine pw über netzwerk gesendet
- **Replay**: Tickets zeitliches timout & nur einmal verwendbar
- **Delegierung**: Dienst agiert auf afforderung eines benutzers

### Nachteile
- **Zeitabhängigkeit**: sync Uhren (Micheller xD)
- **Komplexität**: relativ komplex
- **SPoF**: ausfall stört alle services

### Verwendung in Active Directory

Windows AD verwendet Kerberos als primary Auth mechanism, Bei anmeldung an Domäne erhält User direkt einen TGT -> Spoiler wird als nächstes behandelt

---

## Ticket Granting Ticket (TGT)

### Definition

Unter dem sogenannten Ticket Granting Ticket verstehe ich ein spezielles Ticket, welches von Kerberos versendet wird, dass der AS nach einer ersten Authentication ausstellt. Das kann man sich so wie nen Fühererschein vorstellen der bei jeder Ticketcontrolle vorgezeigt werden kann -> nur die engel in blau ist der TGS.

### Funktionsweise

coole ai graph

```
                    ┌─────────────────────────────────┐
                    │              TGT                │
                    ├─────────────────────────────────┤
┌────────────┐      │ Verschlüsselt mit KDC-Schlüssel │      ┌────────────┐
│  Benutzer  │─────▶│ • Benutzer-Principal            │─────▶│    TGS     │
│            │      │ • Session Key                   │      │            │
└────────────┘      │ • Gültigkeitszeitraum           │      └────────────┘
                    │ • Client-Adresse                │
                    └─────────────────────────────────┘
```

### Lebenszyklus
1. **Creation**: Nach auth bei AS
2. **Saving**: im credential cach vom client /tmp/krb5cc_xxxx
3. **Using**: für service ticket request bei TGS
4. **Renewal**: auch vor ablauf erneuerbar
5. **TimeOfDeath**: je nach konfig

### Security
- **Verschlüsselung**: TGT ist mit private Key von KDC encrypted
- **TimeToLIve**: Reduziert "klauen" von keys
- **Forwarding**: TGT kann an andere hosts weitergegebenw erden
- **Proxiable**: erlaubt anfordern von ticket von anderen addressen

---

## Central Authentication Service (CAS)

### Definition

Cantral Authentication Service (CAS) ist ein Protokoll (SSO Protokoll), haben streber von Yale entwickelt, welches ermöglicht in einer WebApp Benutzer gegen einen zentralen Server zu authentifizieren.

### Architektur

AI Art xD

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│  Browser    │◀───────▶│  CAS Server │◀───────▶│  Anwendung  │
│  (Client)   │         │  (IdP)      │         │  (Service)  │
└─────────────┘         └─────────────┘         └─────────────┘
                               │
                               ▼
                        ┌─────────────┐
                        │ User Store  │
                        │ (LDAP/DB)   │
                        └─────────────┘
```

### CAS 3

1. User accesd geschützte Ressource
   User -> Application

2. Aplication redirected to Cas
   User <- 302 Redirect - App

3. Cas Authentication
   User - Data -> CAS

4. Cas Service ticket schicken
   User <- 302 Redirect -  CAS

5. App validates Ticket
   App - /serviceValidate -> CAS 
   App <- XML Response -  CAS 

6. App erstellt session
   App <- Set-Cookie - App

### Ticket-Typen
| Ticket | Beschreibung          | Gültigkeit     |
|--------|-----------------------|----------------|
| **TGT (Ticket Granting Ticket)** | represent sso session | session based  |
| **ST (Service Ticket)** | one time auth         | ca 10 s        |
| **PGT (Proxy Granting Ticket)** | proxy auth            | session based  |
| **PT (Proxy Ticket)** | service proxy ticket  | ca 10 Sekunden |

### Vorteile
- **Easy**: weniger komplex als sam
- **Web focus**: optimized auf web apps
- **Open Source**
- **Flexibel**: verschiedene backend auth methoden

### Nachteile
- **Nur Web**
- **Synchron Validation**: always backend communication
- **Nicht so verbreitet**

---

## Open Authorization 2

### Definition

Unter oAuth 2 verstehe ich ein Framework welches Programmen begrenzten Zugriff auf HTTP Dienste erlaubt -> entweder im Namen der Anwendung oder Ressourcenbesitzers.

### Rollen
| Rolle | Beschreibung        | Beispiel   |
|-------|---------------------|------------|
| **Resource Owner** | Besitzt "Ressource" | User       |
| **Client** | App möchte zugriff  | App        |
| **Authorization Server** | Tokensteller        | Google     |
| **Resource Server** | Hosted ressource    | API Server |

### Grant

Act brauchbar

```
┌──────────┐                              ┌────────────────┐
│  Client  │                              │  Auth Server   │
│  (App)   │                              │   (Google)     │
└────┬─────┘                              └───────┬────────┘
     │                                            │
     │ 1. Redirect zu /authorize                  │
     │───────────────────────────────────────────▶│
     │                                            │
     │ 2. Benutzer authentifiziert sich           │
     │                                            │
     │ 3. Redirect mit Authorization Code         │
     │◀───────────────────────────────────────────│
     │                                            │
     │ 4. POST /token (code + client_secret)      │
     │───────────────────────────────────────────▶│
     │                                            │
     │ 5. Access Token + Refresh Token            │
     │◀───────────────────────────────────────────│
```

### Tokens

| Token | USe                  | Validation Period |
|-------|----------------------|-------------------|
| **Access Token** | API access           | short             |
| **Refresh Token** | renewal access token | long              |
| **ID Token** | user identity        | short             |

### OpenID Connect
OIDC ist Identitätsschicht:
- added JWT
- standard UserInfo Endpoint
- Definiert Scopes openid, profile, email

### Security important
- **HTTPS** 
- **State Parameter** gegen CSRF
- **PKCE** für Clients
- **Token Binding** wenn verfügbar
- kurze token laufzeit

---

## Security Assertion Markup Language (SAML)

### Definition

Unter SAML 2 verstehen wir einen XML Standard für den Austausch von Auth und Autorisierung zwischen verschiedenen Partieen zB Identity Provider/Service Porvider.

### Komponenten
- **Identity Provider (IdP)**: authentifiziert User
- **Service Provider (SP)**: app 
- **Assertion**: XML-Dokument auth informationen
- **Metadata**: XML-Konfiguration für IdP/SP

### Authentication

#### SP initiated SSO
```
┌──────────┐          ┌──────────┐          ┌──────────┐
│  Browser │          │    SP    │          │   IdP    │
└────┬─────┘          └────┬─────┘          └────┬─────┘
     │ 1. GET /protected   │                     │
     │────────────────────▶│                     │
     │                     │                     │
     │ 2. Redirect (SAML AuthnRequest)           │
     │◀────────────────────│                     │
     │                     │                     │
     │ 3. POST AuthnRequest                      │
     │──────────────────────────────────────────▶│
     │                     │                     │
     │ 4. Login-Formular   │                     │
     │◀──────────────────────────────────────────│
     │                     │                     │
     │ 5. Credentials      │                     │
     │──────────────────────────────────────────▶│
     │                     │                     │
     │ 6. POST SAML Response (Assertion)         │
     │◀──────────────────────────────────────────│
     │                     │                     │
     │ 7. POST SAML Response                     │
     │────────────────────▶│                     │
     │                     │                     │
     │ 8. Set-Cookie + Redirect                  │
     │◀────────────────────│                     │
```

#### IdP Initiated SSO

1. User logon on IdP
2. succht application aus
3. IdP sendet saml Response an SP

### Sicherheitsmerkmale
- **XML Signature**
- **XML Encryption**
- **Audience Restriction**: assertion nur für einen sp
- **Time Constraints** 
- **One-Time Use**

---

## Implementierung in bsp applikation

### Übersicht der implementierten Mechanismen

Mehrere Aut möglichkeiten

```
┌─────────────────────────────────────────────────────────────────┐
│                    Auth architektur                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────┐   ┌───────────────┐   ┌───────────────┐      │
│  │ Local Auth    │   │ Google OAuth  │   │ LDAP          │      │
│  │ (Passwort)    │   │ 2             │   │               │      │
│  └───────┬───────┘   └───────┬───────┘   └───────┬───────┘      │
│          │                   │                   │              │
│          ▼                   ▼                   ▼              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   JWT Service                           │    │
│  │   • Token Generation (HS256)                            │    │
│  │   • Token Validation                                    │    │
│  │   • Claims: sub, email, role, exp, iat, iss             │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                  │
│                              ▼                                  │
│                      ┌───────────────┐                          │
│                      │  API Response │                          │
│                      │  Bearer Token │                          │
│                      └───────────────┘                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

[2] **Kerberos auth**
https://datatracker.ietf.org/doc/html/rfc4120

[3] **Microsoft Docs Kerber**
https://docs.microsoft.com/en-us/windows-server/security/kerberos/kerberos-authentication-overview

[4] **Kerberos doc**
https://web.mit.edu/kerberos/krb5-latest/doc/

[5] **Apereo CAS impl**
https://apereo.github.io/cas/development/protocol/CAS-Protocol-Specification.html

[6] **Bearer Token OAUTH**
https://datatracker.ietf.org/doc/html/rfc6750

[7] **OASIS SAML doc**
https://docs.oasis-open.org/security/saml/v2.0/saml-core-2.0-os.pdf

[8] **OpenID Dev Doc** - OpenID Foundation
https://openid.net/specs/openid-connect-core-1_0.html

[11] **OWASP** - Session Management
https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html

[12] **NIST** - Digital Identity Guidelines
https://pages.nist.gov/800-63-3/sp800-63b.html

### Rust crate

[13] **jwt Crate** https://docs.rs/jsonwebtoken/

[14] **oauth2 Crate**https://docs.rs/oauth2/

[15] **ldap3 Crate** https://docs.rs/ldap3/