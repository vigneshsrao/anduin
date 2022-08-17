use crate::httpresponse::ResponseCode;

pub struct Defaults;

impl Defaults {
    pub fn default_message(code: &ResponseCode) -> Vec<u8> {
        let (code, message) = code.get();
        format!("
<html>
  <title>Anduin Error</title>
  <body>
    <h1> {} {} </h1>
  </body>
</html>", code, message).into_bytes()
    }
}
