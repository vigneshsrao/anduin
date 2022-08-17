use std::convert::{TryFrom, From, Into};
use std::net::TcpStream;
use std::path::Path;
use crate::httpresponse::{HttpResponse, ResponseCode};
use crate::httpserver::Logs;
use std::io::Read;


/// A dummy error to wrap all the parsing related errors so that we can use the
/// `?` syntax for various different while parsing a Request
#[derive(Debug)]
struct ParseError;
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error while parsing the Request")
    }
}

impl std::error::Error for ParseError{}

type ParseResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
enum RequestMethod {
    Options,    // Section 9.2
    Get    ,    // Section 9.3
    Head   ,    // Section 9.4
    Post   ,    // Section 9.5
    Put    ,    // Section 9.6
    Delete ,    // Section 9.7
    Trace  ,    // Section 9.8
    Connect,    // Section 9.9
}

impl TryFrom<&str> for RequestMethod {
    type Error = ParseError;

    fn try_from(string: &str) -> Result<Self, Self::Error> {
        match string {
            "OPTIONS" => Ok(RequestMethod::Options),
            "GET"     => Ok(RequestMethod::Get)    ,
            "HEAD"    => Ok(RequestMethod::Head)   ,
            "POST"    => Ok(RequestMethod::Post)   ,
            "PUT"     => Ok(RequestMethod::Put)    ,
            "DELETE"  => Ok(RequestMethod::Delete) ,
            "TRACE"   => Ok(RequestMethod::Trace)  ,
            "CONNECT" => Ok(RequestMethod::Connect),
            _         => Err(ParseError),
        }
    }
}

impl From<&RequestMethod> for &str {
    fn from(method: &RequestMethod) -> &'static str {
        match method {
            RequestMethod::Options  => "OPTIONS",
            RequestMethod::Get      => "GET",
            RequestMethod::Head     => "HEAD",
            RequestMethod::Post     => "POST",
            RequestMethod::Put      => "PUT",
            RequestMethod::Delete   => "DELETE",
            RequestMethod::Trace    => "TRACE",
            RequestMethod::Connect  => "CONNECT",
        }
    }
}


pub struct HttpRequest<'a> {
    method:     RequestMethod,
    uri:        &'a str,
    version:    &'a str,
    data:       Option<String>,
    logs:       &'a mut Logs,
}

impl HttpRequest<'_> {

    /// Parse the HTTP request string to create an HttpRequest. On success, it
    /// returns an HttpRequest, otherwise returns an Error. If this returns an
    /// Error, then the server must respond with a BadRequest response code, as
    /// if we are unable to parse the request, then it means that the request is
    /// malformed.
    pub fn parse<'a>(request: &'a String, stream: &'a mut TcpStream, logs: &'a mut Logs)
                     -> ParseResult<HttpRequest<'a>> {

        // println!("{}", request);
        let mut lines = request.lines();
        let request_line: Vec<&str>  = lines.next()
                                            .ok_or(ParseError)?
                                            .split(' ').collect();

        macro_rules! index {
            ($idx:expr) => {
                *request_line.get($idx).ok_or(ParseError)?
            };
        }

        let method   = RequestMethod::try_from(index!(0))?;
        let uri      = index!(1);
        let version  = HttpRequest::validate_version(index!(2))?;
        let mut data = None;

        let mut size = 0;

        for line in lines {
            if line.contains("Content-Length: ") {
                size = line[16..].parse::<usize>()?;
            }
        }

        if size != 0 {
            let mut v = Vec::<u8>::with_capacity(size);
            let mut tmp = [0u8; 1];

            for _ in 0..size {
                stream.read_exact(&mut tmp)?;
                v.push(tmp[0]);
            }

            data = Some(String::from_utf8(v).unwrap().to_string());
            // println!("data = {}",data);
        }

        Ok(HttpRequest {
            method:     method,
            uri:        uri,
            version:    version,
            data:       data,
            logs:       logs,
        })

    }

    /// Validate the http version part of the the request line. If successful,
    /// then it returns the argument itself, otherwise returns an Error
    fn validate_version(version_str: &str) -> ParseResult<&str> {
        let ver_str = if let Some(prefix) = version_str.strip_prefix("HTTP") {
            prefix
        } else {
            version_str.strip_prefix("http").ok_or(ParseError)?
        };

        let ver_str = ver_str.strip_prefix("/").ok_or(ParseError)?;

        let num: Vec<&str> = ver_str.split('.').collect();

        let _major: u8 = num.get(0).ok_or(ParseError)?.parse()?;
        let _minor: u8 = num.get(1).ok_or(ParseError)?.parse()?;

        Ok(version_str)
    }

    pub fn handle_method(&mut self) -> HttpResponse {
        let method_name: &str = (&self.method).into();
        self.log(&format!("{} {} {}", method_name, self.uri, self.version));
        match &self.method {
            RequestMethod::Get  => self.handle_get(),
            RequestMethod::Post => self.handle_post(),
            _                   => {
                HttpResponse::new(ResponseCode::NotImplemented,
                                  None, None)
            }
        }
    }

    fn log(&mut self, logs: &String) {
            self.logs.log(logs);
    }


    fn _unhex(&self) -> String {
        let mut uri = String::new();
        let mut chars = self.uri.chars();
        chars.next().unwrap();
        loop {

            let char = chars.next();
            if char.is_none() {
                break;
            }

            let char = char.unwrap();

            if char != '%' {
                uri.push(char);
                continue;
            }

            let mut tmp = String::new();
            tmp.push(chars.next().unwrap());
            tmp.push(chars.next().unwrap());

            uri.push(u8::from_str_radix(&tmp, 16).unwrap() as char);

            // if self.uri.get(i).unwrap() != "%" {
            //     i += 1;
            //     continue;
            // }

        }

        uri
    }


    fn handle_post(&self) -> HttpResponse {
        if self.data.is_some() {
            let data = self.data.as_ref().unwrap();
            if data.contains("----------") {
                println!("\n\n\n\n\n\n");
            }
            println!("{}", data);
        }
        HttpResponse::new(ResponseCode::OK,
                          Some("OK".as_bytes().to_vec()),
                          Some(String::from("text/plain")))
    }

    fn handle_get(&self) -> HttpResponse {
        println!("GET {}", self.uri);
        self.read_uri_path()
            .map(|data| HttpResponse::new(ResponseCode::OK,
                                          Some(data),
                                          Some(self.get_content_type())))
            .unwrap_or( HttpResponse::new(ResponseCode::NotFound,
                                          None, None))

    }

    fn get_content_type(&self) -> String {
        let path = Path::new(self.uri);
        if path.extension() == None {
            return String::from("application/octet-stream");
        }

        let extension = path.extension().unwrap();
        let extension = match extension.to_str().unwrap() {
            "html" => "text/html",
            "txt"  => "text/plain",
            "css"  => "text/css",
            "csv"  => "text/csv",

            "xml"  => "application/xml",
            "zip"  => "application/zip",
            "js"   => "application/javascript",
            "json" => "application/json",

            "gif"  => "image/gif",
            "jpeg" => "image/jpeg",
            "jpg"  => "image/jpeg",
            "png"  => "image/png",
            "tiff" => "image/tiff",

            "mpeg" => "video/mpeg",
            "mp4"  => "video/mp4",

            _ => "application/octet-stream"
        };

        String::from(extension)
    }

    /// Read a file from the file system. This is very hackish right now and
    /// does properly parse the URI's. TODO: Clean this code up to properly
    /// handle all the cases
    fn read_uri_path(&self) -> std::io::Result<Vec<u8>> {
        let path = ".".to_string() + self.uri;
        let metadata = std::fs::metadata(&path)?;
        let path = if metadata.is_dir() {
            path + "/index.html"
        } else {
            path
        };
        std::fs::read(path)
    }

}
