# Gitache

Gitache is a web service that serves raw files from GitHub, GitLab and Bitbucket
with the proper `Content-Type` headers. Requests to a branch will be redirected
to the branches `HEAD`. Requests to a specific commit will also set long time
cache headers, so the service can be put behind a CDN like Cloudflare.

The endpoints follow the pattern `/<service>/<user>/<repo>/<gitref>/<file>`
where `<service>` is one of `github`, `gitlab` or `bitbucket`, `<gitref>` is the
name of the branch or a commit hash.

## Building and Running

The code can be built natively using `cargo build --release` or as a Docker
image using `docker build .`

The easiest way to run the service is by using `docker-compose up -d` and
exposing port `8080`.


## API Limits

To get the `HEAD` of a requested branch, Gitache sends a request to the
requested service's API. To prevent running into rate limiting issues with the
GitHub API, an OAuth2 App should be created and the client ID and secret can be
set via the `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET` environment variables.

## Variables

| Environment Variable   | CLI Flag         | Description                     |
| ---                    | ---              | ---                             |
| `GITHUB_CLIENT_SECRET` | `--gh-secret`    | GitHub OAuth2 secret (optional) |
| `GITHUB_CLIENT_ID`     | `--gh-id`        | GH OAuth2 Client ID (optional)  |
| `CF_ZONE_IDENT`        | `--cf-zone`      | Cloudflare Zone identifier      |
| `CF_AUTH_USER`         | `--cf-auth-user` | CF API User (`X-Auth-Email`)    |
| `CF_AUTH_KEY`          | `--cf-auth-key`  | CF API Key (`X-Auth-Key`)       |
| `HOSTNAME`             | `--hostname`     | Hostname (default: `gitcdn.tk`) |
