from asyncio.coroutines import iscoroutine
from functools import wraps
from typing import Callable

from pyr.pyr import Client, Header, Response


def callback(fn: Callable):
    """Wrap any callable to awaitable.
    """
    @wraps(fn)
    async def async_inner(resp: Response):
        ret = fn(resp)
        if iscoroutine(ret):
            return await ret
        else:
            return ret
    return async_inner


async def get(
        url: str, 
        /, *,
        timeout: int = None, 
        proxy: str = None, 
        headers: dict = None, 
        **kwargs
    ) -> Response:
    client = Client(proxy=proxy)
    return await client.get(url, timeout=timeout, headers=headers, **kwargs)


async def head(
        url: str, 
        /, *,
        timeout: int = None, 
        proxy: str = None, 
        headers: dict = None, 
        **kwargs
    ) -> Response:
    client = Client(proxy=proxy)
    return await client.head(url, timeout=timeout, headers=headers, **kwargs)


async def post(
        url: str, 
        /, *,
        timeout: int = None, 
        proxy: str = None, 
        headers: dict = None, 
        **kwargs
    ) -> Response:
    client = Client(proxy=proxy)
    return await client.post(url, timeout=timeout, headers=headers, **kwargs)


async def put(
        url: str, 
        /, *,
        timeout: int = None, 
        proxy: str = None, 
        headers: dict = None, 
        **kwargs
    ) -> Response:
    client = Client(proxy=proxy)
    return await client.put(url, timeout=timeout, headers=headers, **kwargs)


async def delete(
        url: str, 
        /, *,
        timeout: int = None, 
        proxy: str = None, 
        headers: dict = None, 
        **kwargs
    ) -> Response:
    client = Client(proxy=proxy)
    return await client.delete(url, timeout=timeout, headers=headers, **kwargs)