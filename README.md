# sushii-image-server2

Rewritten image server with Rust.

Connects to an external (headless) browser through WebDriver via
[fantoccini](https://github.com/jonhoo/fantoccini) to render HTML templates to
images.

Maintains a pool of WebDriver sessions to provide concurrent requests.

## Usage

## Example

```bash
curl localhost:8000/template \
      -d '{"name": "test", "width": 1280, "height": 720, "ctx": {"name": "Bob", "age": 30 }}' > image.png
```
