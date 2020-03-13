//#[macro_use]
extern crate log;
extern crate curl;
extern crate serde;
//#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde::{
    Serialize,
    de::DeserializeOwned,
};
use curl::easy::{Easy2,List,WriteError,Handler,ReadError};

const DEFAULT_USER_AGENT: &'static str = "user-agent: jttp-client/0.1";

struct Buffer {
    read_data: Vec<u8>,
    write_data: Vec<u8>,
}
impl Buffer {
    fn new() -> Buffer {
        Buffer {
            read_data: Vec::new(),
            write_data: Vec::new(),
        }
    }
    fn reset(&mut self) {
        self.read_data.clear();
        self.write_data.clear();
    }
    fn get(&self) -> &[u8] {
        &self.read_data[..]
    }
    fn _set(&mut self, mut data: Vec<u8>) {
        std::mem::swap(&mut self.write_data, &mut data);
    }
}
impl Handler for Buffer {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.read_data.extend_from_slice(data);
        Ok(data.len())
    }
    fn read(&mut self, data: &mut [u8]) -> Result<usize, ReadError> {
        let mut sz = self.write_data.len();
        if sz>0 {
            if data.len() >= sz {
                data[0..sz].clone_from_slice(&self.write_data[0..sz]);
                self.write_data.clear();
            } else {
                sz = data.len();
                data[0..sz].clone_from_slice(&self.write_data[0..sz]);
                let tail = self.write_data.split_off(sz);
                self.write_data = tail;
            }
        }
        Ok(sz)
    }
}

#[derive(Debug)]
pub enum Error {
    CurlGet(curl::Error),
    CurlPut(curl::Error),
    CurlPost(curl::Error),
    CurlPutData(curl::Error),
    CurlPostData(curl::Error),
    CurlUrl(curl::Error),
    CurlHeaders(curl::Error),
    CurlParseHeader(curl::Error),
    CurlPerformGet(curl::Error),
    CurlPerformPut(curl::Error),
    CurlPerformPost(curl::Error),
    CurlCode(curl::Error),
    Http{ code: u32, message: String },
    Json(serde_json::Error),
}

pub struct JttpRequest {
    url: String,
    headers: Vec<String>,
    request: Easy2<Buffer>,
}
impl JttpRequest {
    pub fn new(url: &str) -> JttpRequest {
        JttpRequest {
            url: url.to_string(),
            request: Easy2::new(Buffer::new()),
            headers: vec![DEFAULT_USER_AGENT.to_string()],
        }
    }
    fn prepare_headers(&self) -> Result<List,Error> {
        let mut headers = List::new();
        for h in &self.headers {
            headers.append(h).map_err(Error::CurlParseHeader)?;
        }
        Ok(headers)
    }
    pub fn post<T: Serialize, R: DeserializeOwned>(&mut self, request: &T) -> Result<R,Error> {
        self.request.reset();
        self.request.get_mut().reset();
        let headers = self.prepare_headers()?;
        self.request.post(true).map_err(Error::CurlPost)?;
        self.request.http_headers(headers).map_err(Error::CurlHeaders)?;
        self.request.url(&self.url).map_err(Error::CurlUrl)?;
        self.request.post_fields_copy(&serde_json::to_vec(request).map_err(Error::Json)?).map_err(Error::CurlPostData)?;
        self.request.perform().map_err(Error::CurlPerformPost)?;
        let code = self.request.response_code().map_err(Error::CurlCode)?;
        match (code,serde_json::from_slice(self.request.get_ref().get()).map_err(Error::Json)) {
            (_,Ok(r)) => Ok(r),
            (200,Err(e)) => Err(e),
            (_,Err(_)) => Err(Error::Http{ code: code, message: String::from_utf8_lossy(self.request.get_ref().get()).to_string() }),                       
        }
    }
}

/*

// PUT
        self.request.reset();
        self.request.get_mut().reset();
        let mut headers = self.prepare_headers()?;

                self.request.put(true).map_err(Error::CurlPut)?;
                self.request.get_mut().set(data);
                self.request.upload(true).map_err(Error::CurlPutData)?;
                // extra header
                headers.append(&format!("name: {}",value)).map_err(Error::CurlParseHeader)?; 
                self.request.http_headers(headers).map_err(Error::CurlHeaders)?;
                let url = format!("{}?param={}",self.url,self.request.url_encode(..));
                self.request.url(&url).map_err(Error::CurlUrl)?;
                self.request.perform().map_err(Error::CurlPerformPut)?;

           
// GET
                self.request.reset();
                self.request.get_mut().reset();
                self.request.get(true).map_err(Error::CurlGet)?;
                let mut headers = self.prepare_headers()?;
                self.request.http_headers(headers).map_err(Error::CurlHeaders)?;
                let url = format!("{}?param={}",self.url,self.request.url_encode(..));
                self.request.url(&url).map_err(Error::CurlUrl)?;
                self.request.perform().map_err(Error::CurlPerformGet)?;
 */

