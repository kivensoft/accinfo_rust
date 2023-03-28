use std::io::Write;

use anyhow::Context;
#[cfg(feature="simple_config_parser")]
pub use simple_config_parser::Config;
pub use getopts::{Options, Matches};
pub use anyhow;

// const Z: &str = "\x1b[0m";
// const K: &str = "\x1b[30m";
// const R: &str = "\x1b[31m";
// const G: &str = "\x1b[32m";
// const Y: &str = "\x1b[33m";
// const B: &str = "\x1b[34m";
// const M: &str = "\x1b[35m";
// const C: &str = "\x1b[36m";
// const W: &str = "\x1b[37m";

#[cfg(not(feature="simple_config_parser"))]
pub struct Config {}

#[macro_export]
#[doc(hidden)]
macro_rules! __set_opt_flag {
    ($opts:expr, $short_opt:literal, $long_opt:literal, $opt_name:literal, $desc:literal, $val:expr, bool) => {
        let s = format!("{} (\x1b[34mdefault: \x1b[32m{}\x1b[0m)", $desc, $val);
        $opts.optflag($short_opt, $long_opt, &s)
    };
    ($opts:expr, $short_opt:literal, $long_opt:literal, $opt_name:literal, $desc:literal, $val:expr, $_:ty) => {
        let s = match $val.len() {
            0 => String::from($desc),
            _ => format!("{} (\x1b[34mdefault: \x1b[32m{}\x1b[0m)", $desc, $val),
        };
        $opts.optopt($short_opt, $long_opt, &s, $opt_name)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __get_opt_value {
    ($matches:expr, "help", $out_val:expr, $t:ty) => {};
    ($matches:expr, "conf-file", $out_val:expr, $t:ty) => {};
    ($matches:expr, $name:expr, $out_val:expr, String) => {
        if let Some(s) = $matches.opt_str($name) {
            $out_val = s;
        }
    };
    ($matches:expr, $name:expr, $out_val:expr, bool) => {
        if $matches.opt_present($name) {
            $out_val = true;
        }
    };
    ($matches:expr, $name:expr, $out_val:expr, $t:ty) => {
        if let Some(s) = $matches.opt_str($name) {
            $out_val = s.parse::<$t>().with_context(
                || format!("program argument {} is not a numbe", $name))?;
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature="simple_config_parser")]
macro_rules! __get_cfg_value {
    ($cfg: expr, "conf-file", $out_val: expr, $t:ty) => {};
    ($cfg: expr, $name: expr, $out_val: expr, String) => {
        if let Ok(s) = $cfg.get_str($name) {
            $out_val = s;
        }
    };
    ($cfg: expr, $name: expr, $out_val: expr, bool) => {
        if let Ok(s) = $cfg.get_str($name) {
            $out_val = s.to_lowercase() == "true";
        }
    };
    ($cfg: expr, $name: expr, $out_val: expr, $t:ty) => {
        if let Ok(s) = $cfg.get_str($name) {
            $out_val = s.parse::<$t>().with_context(
                || format!("app config file key {} is not a number", $name))?;
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[cfg(not(feature="simple_config_parser"))]
macro_rules! __get_cfg_value {
    ($($arg:tt)*) => {};
}

/// Application Parameter Definition Macro.
///
/// # Examples
///
/// ```
/// appconfig::appconfig_define!(AppConf,
///     log_level: String => ["L",  "log-level", "LogLevel", "log level(trace/debug/info/warn/error/off)"],
///     log_file : String => ["F",  "log-file", "LogFile", "log filename"],
///     log_max  : String => ["M",  "log-max", "LogFileMaxSize", "log file max size(unit: k/m/g)"],
///     listen   : String => ["l",  "", "Listen", "http service ip:port"],
///     debug    : bool   => ["",  "debug", "", "debug mode"],
/// );
///
/// impl Default for AppConf {
///     fn default() -> Self {
///         AppConf {
///             log_level: String::from("info"),
///             log_file : String::new(),
///             log_max  : String::from("10m"),
///             listen   : String::from("0.0.0.0:8080"),
///             debug    : false,
///         }
///     }
/// }
///
/// let mut ac = AppConf::default();
/// if !appconfig::parse_args(&ac, "example application")? {
///     return;
/// }
/// ```
#[macro_export]
macro_rules! appconfig_define {
    ( $struct_name:ident, $($field:ident : $type:tt =>
            [$short_opt:literal, $long_opt:tt, $opt_name:literal, $desc:literal]$(,)?)+ ) => {

        #[derive(Debug)]
        pub struct $struct_name {
            $( pub $field: $type,)*
        }

        impl $crate::AppConfig for $struct_name {
            fn to_opts(&self) -> $crate::Options {
                let mut opts = $crate::Options::new();
                $( $crate::__set_opt_flag!(opts, $short_opt, $long_opt, $opt_name, $desc, self.$field, $type); )*
                opts
            }

            fn set_from_getopts(&mut self, matches: &$crate::Matches) -> $crate::anyhow::Result<()> {
                $( $crate::__get_opt_value!(matches, $long_opt, self.$field, $type); )*
                Ok(())
            }

            fn set_from_cfg(&mut self, cfg: &$crate::Config) -> $crate::anyhow::Result<()> {
                $( $crate::__get_cfg_value!(cfg, $long_opt, self.$field, $type); )*
                Ok(())
            }
        }
    };
}


const C_HELP: &str = "help";
#[cfg(feature="simple_config_parser")]
const C_CONF_FILE: &str = "conf-file";
const C_VERSION: &str = "version";

pub trait AppConfig {
    fn to_opts(&self) -> getopts::Options;
    fn set_from_getopts(&mut self, matches: &getopts::Matches) -> anyhow::Result<()>;
    fn set_from_cfg(&mut self, cfg: &Config) -> anyhow::Result<()>;
}

pub fn print_banner(banner: &str, use_color: bool) {

    if banner.is_empty() { return; }

    let mut rng = rand::thread_rng();
    let mut text = String::new();
    let mut dyn_color: [u8; 5] = [b'\x1b', b'[', b'3', b'0', b'm'];
    let mut n = 0;

    for line in banner.lines() {
        if use_color {
            loop {
                let i = rand::Rng::gen_range(&mut rng, 1..8);
                if n != i { n = i; break }
            }
            dyn_color[3] = b'0' + n;
            text.push_str(std::str::from_utf8(&dyn_color).unwrap());
        }
        text.push_str(line);
        text.push('\n');
    }

    if use_color {
        text.push_str("\x1b[0m");
    }
    text.push('\n');

    std::io::stdout().write_all(text.as_bytes()).unwrap();
}

/// Parsing configuration from command line parameters and configuration files
/// and populate it with the variable `ac`
///
/// If the return value is Ok(false), it indicates that the program needs to be terminated immediately.
///
/// Arguments:
///
/// * `app_config`: Output variable, which will be filled after parameter parsing
/// * `banner`: application banner
///
/// Returns:
///
/// Ok(true): success, Ok(false): require terminated, Err(e): error
///
#[inline]
pub fn parse_args<T>(app_config: &mut T, banner: &str) -> anyhow::Result<bool>
        where T: AppConfig {
    parse_args_ext(app_config, banner, |_| true)
}


/// Parsing configuration from command line parameters and configuration files
/// and populate it with the variable `ac`
///
/// If the return value is false, it indicates that the program needs to be terminated immediately.
///
/// * `app_config`: application config variable
/// * `banner`: application banner
/// * `f`: A user-defined callback function that checks the validity of parameters.
/// If it returns false, this function will print the help information and return Ok(false)
///
/// Returns:
///
/// Ok(true): success, Ok(false): require terminated, Err(e): error
///
pub fn parse_args_ext<T, F>(app_config: &mut T, version: &str, f: F) -> anyhow::Result<bool>
        where T: AppConfig, F: Fn(&T) -> bool {

    let mut args = std::env::args();
    let prog = args.next().unwrap();

    let mut opts = app_config.to_opts();
    opts.optflag("h", C_HELP, "this help");
    #[cfg(feature="simple_config_parser")]
    opts.optopt("c",  C_CONF_FILE, "set configuration file", "ConfigFile");
    if version.len() > 0 {
        opts.optflag("",  C_VERSION, "show version and exit");
    }

    let matches = match opts.parse(args).context("parse program arguments failed") {
        Ok(m) => m,
        Err(e) => {
            print_usage(&prog, &opts);
            return Err(e);
        },
    };

    if matches.opt_present(C_HELP) {
        print_usage(&prog, &opts);
        return Ok(false);
    }

    if version.len() > 0 && matches.opt_present(C_VERSION) {
        println!("{version}");
        return Ok(false);
    }

    // 参数设置优先级：命令行参数 > 配置文件参数
    // 因此, 先从配置文件读取参数覆盖缺省值, 然后用命令行参数覆盖
    // 从配置文件读取参数, 如果环境变量及命令行未提供配置文件参数, 则允许读取失败, 否则, 读取失败返回错误
    #[cfg(feature="simple_config_parser")]
    get_from_config_file(app_config, &matches, &prog)?;

    // 从命令行读取参数
    app_config.set_from_getopts(&matches)?;

    if !f(app_config) {
        print_usage(&prog, &opts);
        return Ok(false);
    }

    // print_banner(banner, true);

    Ok(true)
}

fn print_usage(prog: &str, opts: &getopts::Options) {
    let path = std::path::Path::new(prog);
    let prog = path.file_name().unwrap().to_str().unwrap();
    let brief = format!("\nUsage: \x1b[36m{} \x1b[33m{}\x1b[0m", &prog, "[options]");
    println!("{}", opts.usage(&brief));
}

#[cfg(feature="simple_config_parser")]
fn get_from_config_file<T: AppConfig>(ac: &mut T, matches: &Matches, prog: &str) -> anyhow::Result<()> {
    let mut conf_is_set = false;
    let mut conf_file = String::new();
    if let Some(cf) = matches.opt_str(C_CONF_FILE) {
        conf_is_set = true;
        conf_file = cf;
    }
    if !conf_is_set {
        let mut path = std::path::PathBuf::from(prog);
        path.set_extension("conf");
        conf_file = path.to_str().ok_or(anyhow::anyhow!("program name error"))?.to_owned();
    }
    match Config::new().file(&conf_file) {
        Ok(cfg) => ac.set_from_cfg(&cfg),
        Err(_) => {
            match conf_is_set {
                true => anyhow::bail!("can't read app config file {conf_file}"),
                false => Ok(())
            }
        },
    }
}