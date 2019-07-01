#[derive(Debug)]
pub struct Options<'a> {
    pub verbose: bool,
    pub base_folder: &'a str,
    pub tcp_suffix: &'a str,
    pub udp_suffix: &'a str,
    pub use_udp: bool,
    pub credential_file: Option<&'a str>,
    pub country: Country,
    pub random: bool,
    pub forced: Option<&'a str>,
}

impl<'a> Options<'a> {
    pub fn new() -> Options<'a> {
        Options {
            verbose: false,
            base_folder: "",
            tcp_suffix: "",
            udp_suffix: "",
            use_udp: true,
            credential_file: None,
            country: Country::Default,
            random: false,
            forced: None,
        }
    }

    pub fn with_verbose(mut self, v: bool) -> Self {
        self.verbose = v;
        self
    }

    pub fn with_base_folder(mut self, base_folder: &'a str) -> Self {
        self.base_folder = base_folder;
        self
    }

    pub fn with_tcp_suffix(mut self, tcp_suffix: &'a str) -> Self {
        self.tcp_suffix = tcp_suffix;
        self
    }

    pub fn with_udp_suffix(mut self, udp_suffix: &'a str) -> Self {
        self.udp_suffix = udp_suffix;
        self
    }

    pub fn with_use_udp(mut self, use_udp: bool) -> Self {
        self.use_udp = use_udp;
        self
    }

    pub fn with_credential_file(mut self, credential_file: Option<&'a str>) -> Self {
        self.credential_file = credential_file;
        self
    }

    pub fn with_country(mut self, country: Country) -> Self {
        self.country = country;
        self
    }

    pub fn with_random(mut self, random: bool) -> Self {
        self.random = random;
        self
    }

    pub fn with_forced(mut self, forced: Option<&'a str>) -> Self {
        self.forced = forced;
        self
    }
}

#[derive(Debug)]
pub enum Country {
    Default,
    Canada,
    US,
}

impl Country {
    pub fn new(country: &str) -> Country {
        match country.to_lowercase().as_str() {
            "canada" | "ca" => Country::Canada,
            "us" | "usa" => Country::US,
            _ => Country::Default,
        }
    }

    pub fn to_str(&self) -> Option<&'static str> {
        match self {
            Country::Canada => Some("ca"),
            Country::US => Some("us"),
            _ => None,
        }
    }
}
pub fn vprint(v: bool, msg: &str) {
    if v {
        println!("{}", msg);
    }
}
