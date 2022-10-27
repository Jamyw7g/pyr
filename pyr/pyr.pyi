from typing import Awaitable, Tuple, Optional, List, Any


class Client:
    def __init__(self, **kwargs) -> None:
        pass

    def request(
        self, 
        method: str, 
        url: str, 
        /, *,
        timeout: int, 
        headers: dict, 
        **kwargs
    ) -> Response:
        pass

    def parallel_request(
        self, 
        method: str, 
        urls: Tuple[str],
        /, *,
        timeout: int, 
        headers: dict, 
        callback: Awaitable, 
        **kwargs
    ) -> List[Response, Any]:
        pass

    def get(
        self, 
        url: str, 
        /, *,
        timeout: int, 
        headers: dict, 
        **kwargs
    ) -> Response:
        pass

    def parallel_get(
        self, 
        urls: Tuple[str], 
        /, *,
        timeout: int, 
        headers: dict, 
        callback: Awaitable, 
        **kwargs
    ) -> List[Response, Any]:
        pass

    def head(
        self, 
        url: str, 
        /, *,
        timeout: int, 
        headers: dict, 
        **kwargs
    ) -> Response:
        pass

    def parallel_head(
        self, 
        urls: Tuple[str], 
        /, *,
        timeout: int, 
        headers: dict, 
        callback: Awaitable, 
        **kwargs
    ) -> List[Response, Any]:
        pass

    def post(
        self, 
        url: str, 
        /, *,
        timeout: int, 
        headers: dict, 
        **kwargs
    ) -> Response:
        pass

    def parallel_post(
        self, 
        urls: Tuple[str], 
        /, *,
        timeout: int, 
        headers: dict, 
        callback: Awaitable, 
        **kwargs
    ) -> List[Response, Any]:
        pass

    def put(
        self, 
        url: str, 
        /, *,
        timeout: int, 
        headers: dict, 
        **kwargs
    ) -> Response:
        pass

    def parallel_put(
        self, 
        urls: Tuple[str], 
        /, *,
        timeout: int, 
        headers: dict, 
        callback: Awaitable, 
        **kwargs
    ) -> List[Response, Any]:
        pass

    def delete(
        self, 
        url: str, 
        /, *,
        timeout: int, 
        headers: dict, 
        **kwargs
    ) -> Response:
        pass

    def parallel_delete(
        self, 
        urls: Tuple[str], 
        /, *,
        timeout: int, 
        headers: dict, 
        callback: Awaitable, 
        **kwargs
    ) -> List[Response, Any]:
        pass

    def download(
        self, 
        url: str, 
        /, *,
        name: Optional[str], 
        **kwargs
    ) -> None:
        pass

    def parallel_delete(
        self, 
        urls: Tuple[str, Tuple[str]], 
        /,
        **kwargs
    ) -> None:
        pass 

class Response:
    @property.getter
    def url(self) -> str:
        pass

    @property.getter
    def content(self) -> bytes:
        pass

    def headers(self) -> Header:
        pass

    def ok(self) -> bool:
        pass


class Header:
    pass