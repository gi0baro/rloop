# RLoop

RLoop is an [AsyncIO](https://docs.python.org/3/library/asyncio.html) event loop implemented in Rust on top of the [mio crate](https://github.com/tokio-rs/mio).

> [!WARNING]
> Disclaimer: This is a work in progress and definitely not ready for production usage.

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

## License

RLoop is released under the BSD License.
