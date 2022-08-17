use crate::defaults::Defaults;

#[allow(dead_code)]
pub enum ResponseCode {
    Continue                     ,  // Section 10.1.1
    SwitchingProtocols           ,  // Section 10.1.2:
    OK                           ,  // Section 10.2.1:
    Created                      ,  // Section 10.2.2:
    Accepted                     ,  // Section 10.2.3:
    NonAuthoritativeInformation  ,  // Section 10.2.4:
    NoContent                    ,  // Section 10.2.5:
    ResetContent                 ,  // Section 10.2.6:
    PartialContent               ,  // Section 10.2.7:
    MultipleChoices              ,  // Section 10.3.1:
    MovedPermanently             ,  // Section 10.3.2:
    Found                        ,  // Section 10.3.3:
    SeeOther                     ,  // Section 10.3.4:
    NotModified                  ,  // Section 10.3.5:
    UseProxy                     ,  // Section 10.3.6:
    TemporaryRedirect            ,  // Section 10.3.8:
    BadRequest                   ,  // Section 10.4.1:
    Unauthorized                 ,  // Section 10.4.2:
    PaymentRequired              ,  // Section 10.4.3:
    Forbidden                    ,  // Section 10.4.4:
    NotFound                     ,  // Section 10.4.5:
    MethodNotAllowed             ,  // Section 10.4.6:
    NotAcceptable                ,  // Section 10.4.7:
    ProxyAuthenticationRequired  ,  // Section 10.4.8:
    RequestTimeout               ,  // Section 10.4.9:
    Conflict                     ,  // Section 10.4.10:
    Gone                         ,  // Section 10.4.11:
    LengthRequired               ,  // Section 10.4.12:
    PreconditionFailed           ,  // Section 10.4.13:
    RequestEntityTooLarge        ,  // Section 10.4.14:
    RequestURITooLarge           ,  // Section 10.4.15:
    UnsupportedMediaType         ,  // Section 10.4.16:
    RequestedRangeNotSatisfiable ,  // Section 10.4.17:
    ExpectationFailed            ,  // Section 10.4.18:
    InternalServerError          ,  // Section 10.5.1:
    NotImplemented               ,  // Section 10.5.2:
    BadGateway                   ,  // Section 10.5.3:
    ServiceUnavailable           ,  // Section 10.5.4:
    GatewayTimeOut               ,  // Section 10.5.5:
    HttpVersionNotSupported      ,  // Section 10.5.6:
}

impl ResponseCode {
    pub fn get(&self) -> (u16, &'static str) {

        match self {
            ResponseCode::Continue                     => (100, "Continue "),
            ResponseCode::SwitchingProtocols           => (101, "Switching Protocols "),
            ResponseCode::OK                           => (200, "OK "),
            ResponseCode::Created                      => (201, "Created "),
            ResponseCode::Accepted                     => (202, "Accepted "),
            ResponseCode::NonAuthoritativeInformation  => (203, "Non-Authoritative Information "),
            ResponseCode::NoContent                    => (204, "No Content "),
            ResponseCode::ResetContent                 => (205, "Reset Content "),
            ResponseCode::PartialContent               => (206, "Partial Content "),
            ResponseCode::MultipleChoices              => (300, "Multiple Choices "),
            ResponseCode::MovedPermanently             => (301, "Moved Permanently "),
            ResponseCode::Found                        => (302, "Found "),
            ResponseCode::SeeOther                     => (303, "See Other "),
            ResponseCode::NotModified                  => (304, "Not Modified "),
            ResponseCode::UseProxy                     => (305, "Use Proxy "),
            ResponseCode::TemporaryRedirect            => (307, "Temporary Redirect "),
            ResponseCode::BadRequest                   => (400, "Bad Request "),
            ResponseCode::Unauthorized                 => (401, "Unauthorized "),
            ResponseCode::PaymentRequired              => (402, "Payment Required "),
            ResponseCode::Forbidden                    => (403, "Forbidden "),
            ResponseCode::NotFound                     => (404, "Not Found "),
            ResponseCode::MethodNotAllowed             => (405, "Method Not Allowed "),
            ResponseCode::NotAcceptable                => (406, "Not Acceptable "),
            ResponseCode::ProxyAuthenticationRequired  => (407, "Proxy Authentication Required "),
            ResponseCode::RequestTimeout               => (408, "Request Time-out "),
            ResponseCode::Conflict                     => (409, "Conflict "),
            ResponseCode::Gone                         => (410, "Gone "),
            ResponseCode::LengthRequired               => (411, "Length Required "),
            ResponseCode::PreconditionFailed           => (412, "Precondition Failed "),
            ResponseCode::RequestEntityTooLarge        => (413, "Request Entity Too Large "),
            ResponseCode::RequestURITooLarge           => (414, "Request-URI Too Large "),
            ResponseCode::UnsupportedMediaType         => (415, "Unsupported Media Type "),
            ResponseCode::RequestedRangeNotSatisfiable => (416, "Requested range not satisfiable "),
            ResponseCode::ExpectationFailed            => (417, "Expectation Failed "),
            ResponseCode::InternalServerError          => (500, "Internal Server Error "),
            ResponseCode::NotImplemented               => (501, "Not Implemented "),
            ResponseCode::BadGateway                   => (502, "Bad Gateway "),
            ResponseCode::ServiceUnavailable           => (503, "Service Unavailable "),
            ResponseCode::GatewayTimeOut               => (504, "Gateway Time-out "),
            ResponseCode::HttpVersionNotSupported      => (505, "Http Version not supported "),
        }
    }
}


pub struct HttpResponse {
    pub status:     ResponseCode,
    data:           Option<Vec<u8>>,
    content_type:   Option<String>,
}

impl HttpResponse {

    /// Create a new HTTP Response
    pub fn new(status: ResponseCode, data: Option<Vec<u8>>,
               content_type: Option<String>) -> Self {
        Self {
            status:       status,
            data:         data,
            content_type: content_type,
        }
    }

    /// Convert the response into a string that can be sent back to the client
    pub fn response(&mut self) -> Vec<u8> {

        let (status_code, status_message) = self.status.get();
        let holder;

        let (content_length, data) = if let Some(data) = &self.data {
            (data.len(), data)
        } else {
            self.content_type = Some(String::from("text/html"));
            holder = Defaults::default_message(&self.status);
            (holder.len(), &holder)
        };

        // Create a response string. Note that it is safe to unwrap content_type
        // as by now we would have filled it in already.
        let mut response =
            format!("HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                    status_code, status_message,
                    self.content_type.as_ref().unwrap(),
                    content_length).into_bytes();

        response.extend(data);

        response
    }
}
