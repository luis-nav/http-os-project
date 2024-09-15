# Developer instructions on how to setup the DE

## Docker Setup
### Build app:
```docker build . -t httprust```
### Run app:
```docker run --cpus 0.5 -p 8080:8080 --name http_instance httprust```

