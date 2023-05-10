# Partition

## Project

* docs :
  * openapi.yml : openapi specs
  * hyper_services.svg : layer stack and where to find related code
* libraries contains dependencies. Especially generated ones.
  * server-lib : generated with openapi generator. Must not be manually edited
* src : partition server (Rust)
* ui : partition ui (React)

Generate `server-lib` with :

```shell
./openapi.sh
```

## Configuration

### Environment variables

You can override configuration using environment variables. The name of the variable follow this convention :

`PARTITION_[TOML_SECTION]_[KEY]`

To add custom headers in response :

`PARTITION_[HEADER_NAME]`

Example of use in a docker compose file :

```yaml
version: "3.1"

services:
  partition:
    image: partition
    environment:
      # Override root listen configuration key
      PARTITION_LISTEN: "0.0.0.0:9000"
      # Override 'path' key in indexing section
      PARTITION_INDEXING_PATH: "/tmp/index"
      # Override 'connection mysql' in database section
      PARTITION_DATABASE_CONNECTION_MYSQL: "db:3306"
      # Add these headers in response
      PARTITION_HEADERS_Access-Control-Allow-Origin: "*"
      PARTITION_HEADERS_Access-Control-Allow-Methods: "*"
      PARTITION_HEADERS_Access-Control-Allow-Headers: "*"

  db:
    image: mariadb:latest
```

## Development

### Running a swagger-ui inside docker

To run `swagger-ui` :
* Update, if needed, `resources/sample.toml` to add `Access-Control-Allow-*` headers
* Update, if needed, `host` in `resources/sample.toml` to put ip address instead of loopback
* Run server `cargo run -- -c resources/sample.toml`
* Run swagger-ui docker image `docker run -p 8001:8080 -e SWAGGER_JSON_URL=http://[content of host confg]/openapi.json swaggerapi/swagger-ui`
* Access swagger-ui with a browser `http://127.0.0.1:8001/`

### Test index analysis

Change analysis in `examples/index_test.rs` and run

```shell
cargo r --example index_test -- "This is a test" "this is another test"
```

When satisfied with the output, change in `src/index.rs`.

### Database

For mysql/mariadb, install `libmysqlclient` :

```shell
apt install libmysqlclient-dev
```

For postgres, install `libpq` :

```shell
apt install libpq-dev
```

## Credits

Icon was found [here](https://www.iconfinder.com/icons/3669472/music_library_ic_icon) and is under MIT license.

Music used for examples and test :
Titre:  Notturno  
Auteur: Ross Bugden  
Source: https://soundcloud.com/rossbugden  
Licence: https://creativecommons.org/licenses/by/4.0/deed.fr 
Téléchargement (3MB): https://auboutdufil.com/?id=635  