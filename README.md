Authenticating Reverse Proxy
============================

This is a simple reverse proxy that can be used to expose a
service using HTTP Basic authentication and GitHub Personal Access Tokens as an auth backend.

Other auth methods may be supported in the future.

The org token parameter is optional, but without it, only users who have their org membership publicly visible will be
able to login.

Usage:

```shell
docker run -it -e TARGET=http://example.com \
    -e AUTH_METHOD=GH_BASIC \
    -e GH_ORG=your-org \
    -e GH_ORG_TOKEN=ghp_XXXXX \
    -e BIND=0.0.0.0:8080 \
    -p 8080:8080 \
    ghcr.io/richardstephens/authenticating-reverse-proxy:latest
```