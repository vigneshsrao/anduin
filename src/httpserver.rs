use std::net::{TcpStream, SocketAddr};
use std::io::prelude::{Read, Write};

use crate::httprequest::HttpRequest;
use crate::httpresponse::{HttpResponse, ResponseCode};


// Helper function to unwrap `Result`'s. In case of an error that will
// not let us proceed, we should never kill the server. Instead this
// will return the value if there was no error, and if there was an
// error, then it will just log the error message on the screen and
// return out of the function, hence stopping this connection.
macro_rules! unwrap {
    ($variable:ident, $message:expr) => {
        match $variable {
            Ok(value)   => value,
            Err(error)  => {
                println!("[Error] HttpConnection: {} : {}",
                            $message, error);
                return;
            }
        }
    };
}

pub struct Logs(String);
impl Logs {
    pub fn new() -> Self {
        Logs(String::from(""))
    }

    pub fn none() -> &'static str {
        &""
    }

    pub fn log<T: AsRef<str>>(&mut self, log: &T) {
        if !self.0.is_empty() {
            self.0 += " -- ";
        }
        self.0 += log.as_ref();
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn append<T: AsRef<str>>(&mut self, log: &T) {
        self.0.insert_str(0, " -- ");
        self.0.insert_str(0, log.as_ref());
    }
}

pub struct HttpConnection {
    stream:         TcpStream,
    remote_addr:    SocketAddr,
    // data:           String,
    logs:           Option<Logs>,
}

impl HttpConnection {


    pub fn handle_connection(stream: std::io::Result<TcpStream>) {
        // println!("listen");

        let mut stream  = unwrap!(stream, "Failed to get TCP Stream");
        let remote_addr = stream.peer_addr();
        let remote_addr = unwrap!(remote_addr, "Failed to get remote addr");

        // We need to keep reading bytes until we get a \r\n\r\n. The following
        // loop is basically a state machine that does that.
        // Heres the state machine (done to try out emacs `artist-mode` :D)
        //
        //
        //                      Anything Else
        //           +----------------------------------+
        //           V                                  |
        //           V                                  |
        //           V                                  |
        //           V                                  |
        //        ---+---                            ---+---
        //      -/       \-                        -/       \-
        //     /  state 0  \        \r            /  state 1  \
        //     |           +--------------------->|           |
        //     \   Normal  /                      \    \r     /
        //      -\ state /-                        -\       /-
        //        ---+---                            ---+---
        //           ^                                  |
        //           ^                                  |
        //           ^                                  | \n
        //           ^                                  |
        //           ^                                  |
        //           ^                                  V
        //           ^                               ---+---
        //           ^                             -/       \-
        //           ^       Anything else        /  state 2  \
        //           +<---------------------------|           |
        //           ^                            \   \r\n    /
        //           ^                             -\       /-
        //           ^                               ---+---
        //           ^                                  |
        //           ^                                  |
        //           ^                                  | \r
        //           ^                                  |
        //           ^                                  |
        //           ^                                  V
        //           ^                               ---+---
        //           ^                             -/       \-
        //           ^        Anything Else       /  state 3  \
        //           +<---------------------------|           |
        //                                        \  \r\n\r   /
        //                                         -\       /-
        //                                           ---+---
        //                                              |
        //                                              |
        //                                              | \n
        //                                              |
        //                                              |
        //                                              V
        //                                           ---+---
        //                                         -/       \-
        //                                        /  state 4  \          DONE
        //                                        |           |---------->>>>
        //                                        \  \r\n\r\n /
        //                                         -\       /-
        //                                           -------
        //                                          DONE STATE
        //
        //
        const STATE_0: u8 = 0;
        const STATE_1: u8 = 1;
        const STATE_2: u8 = 2;
        const STATE_3: u8 = 3;
        const STATE_4: u8 = 4;

        let mut data   = [0u8; 1024];
        let mut tmp    = [0u8; 1];
        let mut idx    = 0;
        let mut state  = STATE_0;

        macro_rules! get_byte {
            () => {
                let ret = stream.read_exact(&mut tmp);
                if ret.is_err() {
                    break;
                }
                tmp[0];
            };
        }

        loop {


            if idx >= 1024 {
                break;
            }

            get_byte!();
            let byte = tmp[0];

            data[idx] = byte;

            if byte as char == '\r' {
                if state == STATE_0 {
                    state = STATE_1;
                } else if state == STATE_2 {
                    state = STATE_3;
                } else {
                    state = STATE_0;
                }
            } else if byte as char == '\n' {
                if state == STATE_1 {
                    state = STATE_2;
                } else if state == STATE_3 {
                    state = STATE_4;
                } else {
                    state = STATE_0;
                }
            } else {
                state = STATE_0;
            }

            if state == STATE_4 {
                break;
            }

            idx += 1;
        }

        let data = String::from_utf8((&data).to_vec()).unwrap().to_string();

        let mut logs = Logs::new();

        let response = HttpRequest::parse(&data, &mut stream, &mut logs)
            .as_mut()
            .map(|req| req.handle_method())
            .unwrap_or(HttpResponse::new(ResponseCode::BadRequest, None, None));

        let mut connection = Self {
            stream:         stream,
            remote_addr:    remote_addr,
            // data:           data,
            logs:            None,
        };
        connection.logs = Some(logs);
        connection.send_response(response);
        connection.send_logs();
    }

    fn log(&mut self, logs: &String) {
        self.logs.as_mut().map(|log| log.log(logs));
    }

    fn send_response(&mut self, mut response: HttpResponse) {

        let (resp_code, resp_msg) = response.status.get();

        self.log(&format!("{} {}", resp_code, resp_msg));

        let response = response.response();

        let success = self.stream.write(&response);
        unwrap!(success, "Unable to Send Response");
    }

    fn send_logs(&mut self) {

        let addr = &self.remote_addr.to_string();
        self.logs.as_mut().map(|logs| logs.append(addr));

        let _logs = self.logs.as_ref()
                            .map(|logs| logs.as_str())
                            .unwrap_or(Logs::none());

        println!("{}", _logs);
    }
}
