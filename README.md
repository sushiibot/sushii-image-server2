# sushii-image-server2

Rewritten image server with Rust.

Connects to an external (headless) browser through WebDriver via
[fantoccini](https://github.com/jonhoo/fantoccini) to render HTML templates to
images.

Maintains a pool of WebDriver sessions to provide concurrent requests.

## Usage

sushii image server requires a WebDriver compatible process running in order
to connect to a headless browser.

An example to run ChromeDriver and Google Chrome with Docker:

```bash
docker run \
    -p 4444:4444 \
    --privileged \
    -v /dev/shm:/dev/shm \
    robcherry/docker-chromedriver:latest
```

FireFox via geckodriver to be supported soon, as Google products and services
would be best avoided.

## Example

```bash
curl localhost:8000/template \
      -d '{"name": "test", "width": 1280, "height": 720, "ctx": {"name": "Bob", "age": 30 }}' > image.png
```
