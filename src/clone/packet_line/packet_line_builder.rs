use crate::hash::GitHash;

pub struct UploadPackV2RequestBuilder {
    command: String,
    object_format: String,
    agent: Option<String>,
    wants: Vec<String>,
    shallows: Vec<String>,
    deepen: Option<u32>,
    include_done: bool,
    fetch_options: Vec<String>,

}

impl UploadPackV2RequestBuilder {
    pub fn new() -> Self {
        Self {
            command: "fetch".into(),
            object_format: GitHash::hash_version().to_string(), 
            agent: None,
            wants: Vec::new(),
            shallows: Vec::new(),
            deepen: None,
            include_done: false,
            fetch_options: Vec::new(),
        }
    }

    pub fn command(mut self, cmd: &str) -> Self {
        self.command = cmd.to_owned();
        self
    }

    pub fn fetch_option(mut self, opt: &str) -> Self {
        self.fetch_options.push(opt.to_owned());
        self
    }

    pub fn object_format(mut self, format: &str) -> Self {
        self.object_format = format.to_owned();
        self
    }

    pub fn agent(mut self, agent: &str) -> Self {
        self.agent = Some(agent.to_owned());
        self
    }

    pub fn want(mut self, hash: &str) -> Self {
        self.wants.push(hash.to_owned());
        self
    }

    pub fn shallow(mut self, hash: &str) -> Self {
        self.shallows.push(hash.to_owned());
        self
    }

    pub fn deepen(mut self, depth: u32) -> Self {
        self.deepen = Some(depth);
        self
    }

    pub fn done(mut self) -> Self {
        self.include_done = true;
        self
    }

    pub fn build(self) -> Vec<u8> {
        let mut pkt = Vec::new();

        // --- Phase 1: Command Section ---
        pkt.extend_from_slice(&encode_pkt_line(&format!("command={}", self.command)));
        pkt.extend_from_slice(&encode_pkt_line(&format!("object-format={}", self.object_format)));
        if let Some(agent) = self.agent {
            pkt.extend_from_slice(&encode_pkt_line(&format!("agent={}", agent)));
        }
        pkt.extend_from_slice(b"0001"); 

        for opt in self.fetch_options {
            pkt.extend_from_slice(&encode_pkt_line(&opt));
        }

        // --- Phase 2: Fetch Section ---
        for want in self.wants {
            pkt.extend_from_slice(&encode_pkt_line(&format!("want {}", want)));
        }

        for shallow in self.shallows {
            pkt.extend_from_slice(&encode_pkt_line(&format!("shallow {}", shallow)));
        }

        if let Some(depth) = self.deepen {
            pkt.extend_from_slice(&encode_pkt_line(&format!("deepen {}", depth)));
        }

        if self.include_done {
            pkt.extend_from_slice(&encode_pkt_line("done"));
        }

        pkt.extend_from_slice(b"0000"); // final flush


        pkt
    }
}


fn encode_pkt_line(line: &str) -> Vec<u8> {
    let total_len = line.len() + 4;
    let mut pkt = format!("{:04x}", total_len).into_bytes();
    pkt.extend_from_slice(line.as_bytes());
    pkt
}