# ExactAuth
[ExactOnline](https://start.exactonline.nl/) OAuth2 authorization server.
Authenticates users using MrFriendly MrAuth server

## Running locally
Exact Online requires the redirect URI for OAuth2 to be HTTPS. [See more](proxy/README.md)

## Environmental variables
The following environmental variables must be set to run this server
```bash
# MySQL credentials
MYSQL_HOST=
MYSQL_USER=
MYSQL_PASSWORD=
MYSQL_DB=
# Exact OAuth2 credentials
EXACT_CLIENT_ID=
EXACT_CLIENT_SECRET=
REDIRECT_URI=
# MrAuth server URL. Should *not* end with a '/'
MRAUTH_URL=
```