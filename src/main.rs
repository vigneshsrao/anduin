use std::net::TcpListener;
use std::ffi::CString;
use std::os::raw::c_char;
use std::env;

mod cmdline;
mod threadpool;
mod httpserver;
mod httprequest;
mod httpresponse;
mod defaults;

use cmdline::CmdLineOptions;
use threadpool::Threadpool;
use httpserver::HttpConnection;

/// Initialize a sandbox for the http server. Note that this function is should
/// be run as root and only run once at the very beginning. This is going to
/// chroot the process and then drop the privilages to those of an unprivilaged
/// user
fn initialize_sandbox() -> Result<(), ()> {

    extern "C" {
        fn chroot(path: *const c_char) -> i32;
        fn setuid(uid:  i32) -> i32;
        fn setgid(gid:  i32) -> i32;
        fn perror(message: *const c_char) -> i32;
    }

    // A helper macro to check the return value of a libc function and print
    // an error meessage in case the libc function failed.
    macro_rules! check {
        ($retval:expr, $message:expr) => {
            if $retval == !0 {
                let message = format!("[Error] initialize_sandbox: {}"
                                      , $message);
                let message = CString::new(message)
                    .expect("CString: failed to create message");
                unsafe { perror(message.as_ptr()) };
                return Err(());
            }
        };
    }

    let current_dir = env::current_dir().unwrap();
    env::set_current_dir(&current_dir).expect("unable to move into chroot dir");
    let current_dir = current_dir.into_os_string()
                                 .into_string()
                                 .expect("[Error] failed to get CWD");
    let chroot_path =
        CString::new(current_dir)
        .expect("CString: Failed to create path");

    let success = unsafe { chroot(chroot_path.as_ptr()) };
    check!(success ,"Failed to chroot");

    let success = unsafe { setuid(1000) };
    check!(success, "Failed to setuid");

    let success = unsafe { setgid(1000) };
    check!(success, "Failed to setgid");

    Ok(())
}


fn main()  {

    // Parse the cmd line options
    let cmdlineopts = match CmdLineOptions::parse() {
        Ok(cmd)  => cmd,
        Err(err) => {
            println!("Invalid cmd line syntax found: {}", err);
            CmdLineOptions::help();
            return;
        }
    };

    // Create a new thread pool to handle the connections
    let pool = Threadpool::new(4);

    let address  = format!("{}:{}", cmdlineopts.host, cmdlineopts.port);
    let listener = match TcpListener::bind(address) {
        Ok(listener) => listener,
        Err(err)     => {
            println!("failed to bind to given host/port: {}", err);
            return;
        },
    };


    // Initialize the chroot sandbox and fail hard if this fails
    if cmdlineopts.sandbox {
        let _ = initialize_sandbox().map_err(|_| {
            println!("[!] WARNING! Initialize sandbox failed!");
            println!("[!] Running as unsandboxed. Server is vulnerable to bugs!!!");
        });
            // .expect("Failed to initialize sandbox");
    }

    println!("[+] Listening on port {}:{}", cmdlineopts.host, cmdlineopts.port);

    for stream in listener.incoming() {
        pool.execute(|| {
            HttpConnection::handle_connection(stream);
        });
    }
}
