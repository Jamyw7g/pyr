import asyncio
import logging
from pyr import Client, Response, callback, get


logging.basicConfig(level=logging.DEBUG)

@callback
def show_url(resp: Response):
    pass

@callback
async def async_show(resp: Response):
    pass

async def test_client():
    client = Client(proxy="noproxy")
    await client.parallel_request('get', "https://www.baidu.com", "http://qq.com", timeout=10, callback=show_url)
    await client.parallel_get("https://www.baidu.com", timeout=5, callback=async_show)

loop = asyncio.get_event_loop()
loop.run_until_complete(test_client())
res = loop.run_until_complete(get("http://www.baidu.com", timeout=10, proxy="http://127.0.0.1:7890"))