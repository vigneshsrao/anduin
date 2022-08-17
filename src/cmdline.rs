use std::{env, process};

#[derive(Debug)]
struct CmdLineError(&'static str);
impl std::fmt::Display for CmdLineError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl std::error::Error for CmdLineError {}

type CmdLineResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Structure to hold the command line arguments
#[derive(Debug)]
pub struct CmdLineOptions {
    pub host:      String,
    pub port:      u32,
    pub sandbox:   bool,
}

impl Default for CmdLineOptions {
    fn default() -> Self {
        Self {
            host:       String::from("0.0.0.0"),
            port:       8000,
            sandbox:    true,
        }
    }
}

impl CmdLineOptions {

    /// Parse the command line arguments into a CmdLineOptions struct
    pub fn parse() -> CmdLineResult<Self> {

        let args: Vec<String> = env::args().collect();
        let mut cmdline = Self::default();
        let mut skip = false;

        for (idx, val) in args[1..].iter().enumerate() {

            if skip {
                skip = false;
                continue;
            }

            match val.as_str() {

                "-i" |
                "--host" => {
                    cmdline.host = if let Some(iface) = args.get(idx + 2) {
                        skip = true;
                        iface.to_string()
                    } else {
                        return Err(
                            Box::new(CmdLineError("No host provided")));
                    };
                },


                "-p" |
                "--port" => {
                    cmdline.port = if let Some(port) = args.get(idx + 2) {
                        skip = true;
                        if let Ok(port) = port.parse::<u32>() {
                            port
                        } else {
                            return Err(
                                Box::new(CmdLineError("Invalid Port")));
                        }
                    } else {
                        return Err(
                            Box::new(CmdLineError("No port provided")));
                    }
                },

                "--no-sandbox" => {
                    cmdline.sandbox = false;
                },

                "-h" |
                "--help" => {
                    CmdLineOptions::help();
                    process::exit(0);
                },

                others => println!("Invalid arg passed: {}", others),
            };
        }

        Ok(cmdline)

    }


    /// Function to print out the help menu
    pub fn help() {
        println!("
Usage: anduin [OPTIONS]

Options are:

-i, --host <IP>         The IP address on which to host the server [default: 0.0.0.0]
-p, --port <PORT>       The port on which to run server [default: 8000]
--no-sandbox            Run the server without the sandbox
-h, --help              Print this help information and exit
");
    }

}
