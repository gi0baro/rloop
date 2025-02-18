# RLoop

RLoop is an [AsyncIO](https://docs.python.org/3/library/asyncio.html) selector event loop implemented in Rust on top of the [mio crate](https://github.com/tokio-rs/mio).

> **Warning**: RLoop is currently a work in progress and definitely not suited for *production usage*.

> **Note:** RLoop is available on Unix systems only.

## Installation

```bash
pip install rloop
```

## Usage

```python
import asyncio
import rloop

asyncio.set_event_loop_policy(rloop.EventLoopPolicy())
loop = asyncio.new_event_loop()
asyncio.set_event_loop(loop)
```

## Differences from stdlib

At current time, when compared with the stdlib's event loop, RLoop doesn't support the following features:

- UDP
- Unix Domain Sockets
- SSL
- debugging

RLoop also doesn't implement the following methods:

- `loop.sendfile`
- `loop.connect_accepted_socket`
- `loop.sock_recvfrom`
- `loop.sock_recvfrom_into`
- `loop.sock_sendto`
- `loop.sock_sendfile`

### `call_later` with negative delays

While the stdlib's event loop will use the actual delay of callbacks when `call_later` is used with negative numbers, RLoop will treat those as `call_soon`, and thus the effective order will follow the invocation order, not the delay.

## License

RLoop is released under the BSD License.
