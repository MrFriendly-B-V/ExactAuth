# Proxy
ExactOnline requires a HTTPS URL.
During testing, especially while on the go, a traditional setup using one of our domains
isn't very good.

This directory contains a signed certificate for `mrf.local`, along with a proxy
to tunnel requests. The proxy will run on port `8443`, and proxy to port `8081`

Make sure to add `mrf.local` to your `/etc/hosts`, this can be done with the Make target `setup-hosts`

The proxy can then be run with the Make target `run-proxy`

Sometimes the CA cert of `mkcert` needs to be manually installed into the browser,
the CA can be found at `~/.local/share/mkcert/rootCA.pem`

## Recreating the certificate
```bash
sudo apt install -y libnss3-tools mkcert certutil
# In this directory
mkcert mrf.local
```

>Reference [https://web.dev/how-to-use-local-https/](https://web.dev/how-to-use-local-https/)