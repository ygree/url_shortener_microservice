# URL shortener microservice

## Run

```bash
RUST_LOG=url_shortener_microservice=debug cargo watch -x "run"
```

Ask to create a short url for `www.youtube.com/watch?v=lL9zveDz8H12`:

```bash
curl -X POST -i "localhost:3000/www.youtube.com/watch?v=lL9zveDz8H12"
```

Get short or original url:

```bash
curl -i "localhost:3000/www.youtube.com/watch?v=lL9zveDz8H12"
```

```
/XY
```


Get original url by short:

```bash
curl -i "localhost:3000/XY"
 ```

```
/www.youtube.com/watch?v=lL9zveDz8H12
```


## Challenges shipping to prod

This service implementation could be packed into a docker image for deployment.

Currently, it uses KVService and UniqueIdGen mock service implementations.
In order to scale this it requires persistent scalable implementations of KVService and UniqueIdGen.

UniqueIdGen (required to generate a new short url) can become a bottleneck if number of write request will become significant.

If a persistent KVService performance is not enough a caching could be added to first check the cash for lookups as opposed to reading from KV-store every time. Could be implemented with Redis or similar solution.


### KVService

KV-store that allows to store key-value pairs and retrieve them by key.
Now, it uses in-mem HashMap implementation. It can be replaced with a persistent KV-Store, e.g. MongoDB.


### UniqueIdGen

Used for a unique number generation. 
In-mem implementation uses atomic value. 
It could be replaced with KV-Store to obtain and return next unique value. It could require using optimistic lock or similar mechanism.
