use std::{str::FromStr, sync::Once, time::Duration};

use log::debug;
use pyo3::{
    exceptions::asyncio::CancelledError,
    prelude::*,
    types::{PyDict, PyTuple},
};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    ClientBuilder, Proxy,
};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

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
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

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
            tokio::select! {
                res = download(client, req, name) => { _ = res?; Ok(()) }
                _ = tokio::signal::ctrl_c() => {
                    debug!("Terminate a download with ctrl-c");
                    Err(CancelledError::new_err("Cancelled a future in Rust."))
                }
            }
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
            tokio::select! {
                _ = futures::future::join_all(futs) => {}
                _ = tokio::signal::ctrl_c() => {
                    debug!("Terminate parallel download with ctrl-c");
                    return Err(CancelledError::new_err("Cancelled some futures in Rust."));
                }
            };
            Ok(())
        })
        .map(|fut| fut.to_object(py))
    }
}

fn build_client(dict: Option<&PyDict>) -> Result<RClient, Error> {
    let mut cb = ClientBuilder::new();
    if let Some(dict) = dict {
        if let Some(val) = dict.get_item("proxy") {
            match val.extract::<&str>().ok() {
                Some(p) if p == "noproxy" => cb = cb.no_proxy(),
                Some(p) => cb = cb.proxy(Proxy::all(p)?),
                _ => debug!("Nonsupport proxy."),
            }
        }
        match dict.get_item("user-agent").map(|v| v.extract::<&str>()) {
            Some(Ok(ua)) => cb = cb.user_agent(ua),
            _ => cb = cb.user_agent(APP_USER_AGENT),
        }
        if let Some(Ok(v)) = dict.get_item("verbose").map(|v| v.extract::<bool>()) {
            cb = cb.connection_verbose(v);
        }
    }

    let client = cb.build()?;
    Ok(client)
}

fn build_headers(headers: Option<&PyDict>) -> Option<HeaderMap> {
    headers.map(|dict| {
        let mut header = HeaderMap::new();
        for (key, val) in dict.iter() {
            let key = key
                .extract::<&str>()
                .ok()
                .map(|k| HeaderName::from_str(k).ok());
            let val = val
                .extract::<&str>()
                .ok()
                .map(|v| HeaderValue::from_str(v).ok());
            match (&key, &val) {
                (Some(Some(ref k)), Some(Some(ref v))) => _ = header.insert(k, v.clone()),
                _ => debug!("Incorrect header: val: {:?}, val: {:?}", key, val),
            }
        }
        header
    })
}

fn execute(py: Python, client: RClient, req: RRequest) -> PyResult<PyObject> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        debug!("Execute a request.");
        tokio::select! {
            resp = inner_execute(client, req) => {
                Ok(resp?)
            }
            _ = tokio::signal::ctrl_c() => {
                debug!("Terminate a execute with ctrl-c");
                Err(CancelledError::new_err("Cancelled a futures in Rust."))
            }
        }
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
        let futs_res = tokio::select! {
            futs_res = futures::future::join_all(futs) => { futs_res }
            _ = tokio::signal::ctrl_c() => {
                debug!("Terminate parallel execute with ctrl-c");
                return Err(CancelledError::new_err("Cancelled some futures in Rust."));
            }
        };
        let res: Vec<_> = futs_res
            .into_iter()
            .filter_map(|res| match res {
                Ok(val) => Some(val),
                Err(e) => {
                    debug!("Execute a request with error: {}", e);
                    None
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
