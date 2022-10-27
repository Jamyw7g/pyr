use std::{str::FromStr, time::Duration, sync::Once};

use pyo3::{
    prelude::*,
    types::{PyDict, PyTuple},
};
use reqwest::{header::HeaderMap, ClientBuilder, Proxy};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use log::debug;

use crate::{
    common::*,
    error::Error,
    response::{Header, Response},
    types::BoxedBytes,
};

#[pyclass]
#[derive(Debug, Clone)]
pub struct Client {
    client: RClient,
}

static INIT_LOG: Once = Once::new();

#[pymethods]
impl Client {
    #[new]
    #[args(kwargs = "**")]
    fn new(kwargs: Option<&PyDict>) -> PyResult<Self> {
        INIT_LOG.call_once(|| {
            pyo3_log::init();
        });
        let client = build_client(kwargs)?;
        Ok(Self { client })
    }

    // TODO: using macro to reduce verbose writting
    #[args(kwargs = "**")]
    fn get(slf: PyRef<Self>, url: &str, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        Self::request(slf, "GET", url, kwargs)
    }

    #[args(urls = "*", kwargs = "**")]
    fn parallel_get(
        slf: PyRef<Self>,
        urls: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        Self::parallel_request(slf, "GET", urls, kwargs)
    }

    #[args(kwargs = "**")]
    fn head(slf: PyRef<Self>, url: &str, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        Self::request(slf, "HEAD", url, kwargs)
    }

    #[args(urls = "*", kwargs = "**")]
    fn parallel_head(
        slf: PyRef<Self>,
        urls: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        Self::parallel_request(slf, "HEAD", urls, kwargs)
    }

    #[args(kwargs = "**")]
    fn post(slf: PyRef<Self>, url: &str, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        Self::request(slf, "POST", url, kwargs)
    }

    #[args(urls = "*", kwargs = "**")]
    fn parallel_post(
        slf: PyRef<Self>,
        urls: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        Self::parallel_request(slf, "POST", urls, kwargs)
    }

    #[args(kwargs = "**")]
    fn put(slf: PyRef<Self>, url: &str, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        Self::request(slf, "PUT", url, kwargs)
    }

    #[args(urls = "*", kwargs = "**")]
    fn parallel_put(
        slf: PyRef<Self>,
        urls: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        Self::parallel_request(slf, "PUT", urls, kwargs)
    }

    #[args(kwargs = "**")]
    fn delete(slf: PyRef<Self>, url: &str, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        Self::request(slf, "DELETE", url, kwargs)
    }

    #[args(urls = "*", kwargs = "**")]
    fn parallel_delete(
        slf: PyRef<Self>,
        urls: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        Self::parallel_request(slf, "DELETE", urls, kwargs)
    }

    #[args(kwargs = "**")]
    fn request(
        slf: PyRef<Self>,
        method: &str,
        url: &str,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let py = slf.py();
        let client = slf.client.clone();
        let method = RMethod::from_str(method).map_err(|e| Error::AnyError(e.into()))?;
        let url = RUrl::parse(url).map_err(|e| Error::AnyError(e.into()))?;

        let mut timeout = None;
        let mut headers = None;
        if let Some(kwargs) = kwargs {
            if let Some(t) = kwargs.get_item("timeout") {
                timeout = t.extract().ok().map(|v| Duration::from_secs(v));
            }
            if let Some(t) = kwargs.get_item("headers") {
                headers = t.downcast().ok();
            }
        }

        let mut req = RRequest::new(method, url);
        *req.timeout_mut() = timeout;
        if let Some(headers) = build_headers(headers) {
            *req.headers_mut() = headers;
        }

        execute(py, client, req)
    }

    #[args(urls = "*", kwargs = "**")]
    fn parallel_request(
        slf: PyRef<Self>,
        method: &str,
        urls: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let py = slf.py();
        let client = slf.client.clone();
        let method = RMethod::from_str(method).map_err(|e| Error::AnyError(e.into()))?;

        let mut callback = None;
        let mut timeout = None;
        let mut headers = None;

        if let Some(kwargs) = kwargs {
            callback = kwargs.get_item("callback").map(|val| val.into());
            if let Some(t) = kwargs.get_item("timeout") {
                timeout = t.extract().ok().map(|v| Duration::from_secs(v));
            }
            if let Some(t) = kwargs.get_item("headers") {
                headers = t.downcast().ok();
            }
        }

        let headers = build_headers(headers);
        let mut reqs = Vec::with_capacity(urls.len());
        for any in urls.iter() {
            let a_url: &str = any.extract()?;
            let url_res = RUrl::parse(a_url).map_err(|e| Error::AnyError(e.into()))?;
            let mut req = RRequest::new(method.clone(), url_res);
            *req.timeout_mut() = timeout;
            if let Some(map) = headers.clone() {
                *req.headers_mut() = map;
            }
            reqs.push(req);
        }
        parallel_execute(py, client, reqs, callback)
    }

    #[args(kwargs = "**")]
    fn download(slf: PyRef<Self>, url: &str, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let py = slf.py();

        let mut opt_name = None;
        if let Some(kw) = kwargs {
            if let Some(val) = kw.get_item("name") {
                opt_name = val.extract().ok();
                _ = kw.del_item("name")?;
            }
        }
        let name = opt_name.unwrap_or_else(|| format!("{:x}", md5::compute(url.as_bytes())));

        let url = RUrl::parse(url).map_err(|e| Error::AnyError(e.into()))?;
        let client = slf.client.clone();
        let mut req = RRequest::new(RMethod::GET, url);
        let headers = build_headers(kwargs);
        if let Some(headers) = headers {
            *req.headers_mut() = headers;
        }
        pyo3_asyncio::tokio::future_into_py(py, async move {
            debug!("Download a resource.");
            download(client, req, name).await?;
            Ok(())
        })
        .map(|fut| fut.to_object(py))
    }

    #[args(urls = "*", kwargs = "**")]
    fn parallel_download(
        slf: PyRef<Self>,
        urls: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let py = slf.py();

        let mut reqs = Vec::with_capacity(urls.len());
        let headers = build_headers(kwargs);
        for url_name in urls.iter() {
            let item = match url_name.extract::<(&str, Option<&str>)>() {
                Ok(val) => val,
                _ => (url_name.extract()?, None),
            };
            let url = RUrl::parse(item.0).map_err(|e| Error::AnyError(e.into()))?;
            let name = if let Some(val) = item.1 {
                val.to_string()
            } else {
                format!("{:x}", md5::compute(item.0.as_bytes()))
            };
            let mut req = RRequest::new(RMethod::GET, url);
            if let Some(headers) = headers.as_ref() {
                *req.headers_mut() = headers.clone();
            }

            reqs.push((req, name));
        }

        let slf_client = slf.client.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let futs = reqs
                .into_iter()
                .map(|(req, name)| {
                    let client = slf_client.clone();
                    async move {
                        if let Err(e) = download(client, req, name).await {
                            debug!("Download resource with error: {}", e);
                        }
                        Ok::<_, Error>(())
                    }
                })
                .collect::<Vec<_>>();
            debug!("Parallel download {} resource.", futs.len());
            futures::future::join_all(futs).await;
            Ok(())
        })
        .map(|fut| fut.to_object(py))
    }
}

fn build_client(dict: Option<&PyDict>) -> Result<RClient, Error> {
    let mut cb = ClientBuilder::new();
    if let Some(dict) = dict {
        if let Some(val) = dict.get_item("proxy") {
            let proxy: &str = val.extract()?;
            if proxy.to_ascii_lowercase().eq("noproxy") {
                cb = cb.no_proxy();
            } else {
                let proxy = Proxy::all(proxy)?;
                cb = cb.proxy(proxy);
            };
        }
    }

    let client = cb.build()?;
    Ok(client)
}

fn build_headers(headers: Option<&PyDict>) -> Option<HeaderMap> {
    headers.map(|_dict| {
        let header = HeaderMap::new();

        header
    })
}

fn execute(py: Python, client: RClient, req: RRequest) -> PyResult<PyObject> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        debug!("Execute a request.");
        let resp = inner_execute(client, req).await?;
        Ok(resp)
    })
    .map(|fut| fut.to_object(py))
}

fn parallel_execute(
    py: Python,
    client: RClient,
    reqs: Vec<RRequest>,
    callback: Option<PyObject>,
) -> PyResult<PyObject> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let mut futs = Vec::with_capacity(reqs.len());
        for req in reqs.into_iter() {
            let cb = callback.clone();
            let cli = client.clone();
            let fut = async move {
                let resp = inner_execute(cli, req).await?;
                let res = if let Some(cb) = cb {
                    let url = resp.url.clone();
                    let fut = Python::with_gil(|py| {
                        let cb = cb.call1(py, (resp,))?;
                        pyo3_asyncio::tokio::into_future(cb.as_ref(py))
                    })?;
                    debug!("Call callback: {}", url);
                    fut.await?
                } else {
                    Python::with_gil(|py| resp.into_py(py))
                };

                Ok::<_, Error>(res)
            };
            futs.push(fut);
        }
        debug!("Parallel execute {} request.", futs.len());
        let res: Vec<_> = futures::future::join_all(futs)
            .await
            .into_iter()
            .filter_map(|res| {
                match res {
                    Ok(val) => Some(val),
                    Err(e) => {
                        debug!("Execute a request with error: {}", e);
                        None
                    }
                }
            })
            .collect();
        Ok(res)
    })
    .map(|fut| fut.to_object(py))
}

async fn inner_execute(client: RClient, req: RRequest) -> Result<Response, Error> {
    let resp = client.execute(req).await?;
    let url = resp.url().as_str().to_owned();
    let status = resp.status();
    let header = Header::from(resp.headers());
    let content = BoxedBytes::from(resp.bytes().await?.to_vec());

    Ok(Response {
        url,
        content,
        status,
        header,
    })
}

async fn download(client: RClient, req: RRequest, name: String) -> Result<(), Error> {
    let mut buf_writer = BufWriter::new(File::create(&name).await?);
    let mut resp = client.execute(req).await?;
    debug!("Downloading...: {}.", name);
    while let Some(chunk) = resp.chunk().await? {
        buf_writer.write_all(&chunk).await?;
    }

    Ok(())
}
