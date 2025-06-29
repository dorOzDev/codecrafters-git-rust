#[derive(Debug, Default)]
pub struct Capabilities {
    pub multi_ack: bool,
    pub multi_ack_detailed: bool,
    pub thin_pack: bool,
    pub side_band: bool,
    pub side_band_64k: bool,
    pub ofs_delta: bool,
    pub shallow: bool,
    pub no_progress: bool,
    pub include_tag: bool,
    pub allow_tip_sha1_in_want: bool,
    pub allow_reachable_sha1_in_want: bool,
    pub no_done: bool,

    pub symref: Option<(String, String)>,
    pub agent: Option<String>,
    pub object_format: Option<String>,
    
    pub other: Vec<String>,

    #[doc(hidden)]
    pub(self) all_caps: Vec<String>,
}

impl Capabilities {

    pub fn to_git_line(&self) -> String {
        self.all_caps.join(" ")
    }
}

pub fn parse_capabilities(cap_strings: Vec<&str>) -> Capabilities {
    let mut caps = Capabilities::default();
    caps.all_caps = Vec::with_capacity(cap_strings.len());

    for cap in cap_strings {
        caps.all_caps.push(cap.to_string());
        match cap {
            "multi_ack" => caps.multi_ack = true,
            "multi_ack_detailed" => caps.multi_ack_detailed = true,
            "thin-pack" => caps.thin_pack = true,
            "side-band" => caps.side_band = true,
            "side-band-64k" => caps.side_band_64k = true,
            "ofs-delta" => caps.ofs_delta = true,
            "shallow" => caps.shallow = true,
            "no-progress" => caps.no_progress = true,
            "include-tag" => caps.include_tag = true,
            "allow-tip-sha1-in-want" => caps.allow_tip_sha1_in_want = true,
            "allow-reachable-sha1-in-want" => caps.allow_reachable_sha1_in_want = true,
            "no-done" => caps.no_done = true,

            _ if cap.starts_with("symref=") => {
                if let Some((from, to)) = cap["symref=".len()..].split_once(':') {
                    caps.symref = Some((from.to_string(), to.to_string()));
                }
            }
            _ if cap.starts_with("agent=") => {
                caps.agent = Some(cap["agent=".len()..].to_string());
            }
            _ if cap.starts_with("object-format=") => {
                caps.object_format = Some(cap["object-format=".len()..].to_string());
            }

            other => caps.other.push(other.to_string()),

        }
    }
    caps
}