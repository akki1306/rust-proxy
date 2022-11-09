# Proxy server

This is a simple http proxy server implementation in rust using tokio library. It supports following features: 

1. Support for HTTPS.
2. Block configured sites (in a pre-defined txt file).
3. CLI client for configuring blocked sites.
4. O(1) lookup for cached sites implemented using HashMap.

## Future improvements to be done:

1. Support pattern matching for blocked sites.
2. Implement caching
3. Add new blocked site without server restarts.

### Libraries used: 

1. tokio
2. dns-lookup
3. reqwest
