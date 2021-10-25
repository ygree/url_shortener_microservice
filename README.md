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
