# sushii-image-server2

Rewritten image server with Rust.

Connects to an external (headless) browser through WebDriver via
[fantoccini](https://github.com/jonhoo/fantoccini) to render HTML templates to
images.

Optionally maintains a pool of persistent WebDriver sessions.

**Note:** This project has been discontinued in
favor of using the original [sushii-image-server](https://github.com/sushiibot/sushii-image-server).
This was written to test out Rust libraries to interact with a
headless browser. However, the features are limited as fantoccini communicates
through WebDriver as compared to puppeteer which uses the DevTools protocol.
Main feature that makes this project not as usable is the fact that WebDriver
can only modify the entire browser window and not the viewport. It also provides
not much additional performance gain.

## Usage

sushii image server requires a WebDriver compatible process running in order
to connect to a headless browser.

An example to run ChromeDriver and Google Chrome with Docker with
[robcherry/docker-chromedriver](https://github.com/RobCherry/docker-chromedriver):

```bash
docker run \
    -p 4444:4444 \
    --privileged \
    -v /dev/shm:/dev/shm \
    robcherry/docker-chromedriver:latest
```

Add `-e CHROMEDRIVER_WHITELISTED_IPS=''` as a parameter in order to run without
authentication.

**Note:** Docker containers running browsers should have either at least
`--shm-size 2g` set, or shared memory of the host `-v /dev/shm:/dev/shm` to
prevent crashes due to a low default shared memory of 64MB.

FireFox via geckodriver is planned to be supported, as Google products and
services would be best avoided. However, I am not sure if geckodriver supports
multiple concurrent sessions.

## Configuration

Configuration options can be set in [`Rocket.toml`](./Rocket.toml). Available
options include Rocket's basic [configuration options](https://rocket.rs/master/guide/configuration/#overview)
as well as options specific to sushii-image-server listed below.

```toml
# required options
webdriver_url = "http://127.0.0.1:4444"

# optional options, default values shown below
## Whether or not to keep browser clients and processes alive on idle
pool_keep_alive = false
## Max number of concurrent browser clients and processes
pool_size = 4
```

You can also set configuration options via environment variables. The same
options are given below in environment variables.

```bash
ROCKET_WEBDRIVER_URL="http://127.0.0.1:4444"
ROCKET_POOL_KEEP_ALIVE=false
ROCKET_POOL_SIZE=4
```

## Example Request

sushii-image-server accepts POST requests with a JSON body to pass data.

```bash
curl localhost:8000/template \
      -d '{"name": "test", "width": 1280, "height": 720, "ctx": {"name": "Bob", "age": 30 }}' > image.png
```

## Benchmarks

With `pool_keep_alive` enabled, it will keep WebDriver sessions persistent and
therefore keep the associated browser process open.  This prevents the need to
re-create WebDriver sessions and need to restart processes each request at the
cost of higher average memory usage. This would be ideal for situations where
there is the need of higher throughput and lower latency.

The following tests use [bombardier](https://github.com/codesenberg/bombardier).

### pool_keep_alive = true

Memory usage is a constant ~500MB.

```bash
$ bombardier
    -c 5 \
    -n 500 \
    --method=POST \
    --body='{"name": "test", "width": 1280, "height": 720, "ctx": {"name": "Bob", "age": 30 }}' \
    http://127.0.0.1:8000/template

Bombarding http://127.0.0.1:8000/template with 50 request(s) using 5 connection(s)
 50 / 50 [=============================================================================================] 100.00% 16/s 3s
Done!
Statistics        Avg      Stdev        Max
  Reqs/sec        17.32      23.20     100.56
  Latency      282.69ms   300.22ms      1.29s
  HTTP codes:
    1xx - 0, 2xx - 50, 3xx - 0, 4xx - 0, 5xx - 0
    others - 0
  Throughput:   177.77KB/s
```

### pool_keep_alive = false

Memory usage is ~60MB at idle.

```bash
$ bombardier
    -c 5 \
    -n 500 \
    --method=POST \
    --body='{"name": "test", "width": 1280, "height": 720, "ctx": {"name": "Bob", "age": 30 }}' \
    http://127.0.0.1:8000/template

Bombarding http://127.0.0.1:8000/template with 50 request(s) using 5 connection(s)
 50 / 50 [=============================================================================================] 100.00% 4/s 10s
Done!
Statistics        Avg      Stdev        Max
  Reqs/sec         5.17      16.56     164.16
  Latency         0.98s   214.42ms      1.57s
  HTTP codes:
    1xx - 0, 2xx - 50, 3xx - 0, 4xx - 0, 5xx - 0
    others - 0
  Throughput:    51.07KB/s
```
