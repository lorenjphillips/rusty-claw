pub struct RuntimeModeReport {
    pub mode: String,
    pub connected: bool,
    pub detail: String,
}

impl RuntimeModeReport {
    pub fn as_text(&self) -> String {
        format!(
            "mode={}\nconnected={}\ndetail={}",
            self.mode, self.connected, self.detail
        )
    }
}

pub fn run_remote_mode(target: &str) -> RuntimeModeReport {
    RuntimeModeReport {
        mode: "remote".into(),
        connected: true,
        detail: format!("Remote control placeholder prepared for {}", target),
    }
}

pub fn run_ssh_mode(target: &str) -> RuntimeModeReport {
    RuntimeModeReport {
        mode: "ssh".into(),
        connected: true,
        detail: format!("SSH proxy placeholder prepared for {}", target),
    }
}

pub fn run_teleport_mode(target: &str) -> RuntimeModeReport {
    RuntimeModeReport {
        mode: "teleport".into(),
        connected: true,
        detail: format!("Teleport resume/create placeholder prepared for {}", target),
    }
}

pub struct DirectModeReport {
    pub mode: String,
    pub target: String,
    pub active: bool,
}

impl DirectModeReport {
    pub fn as_text(&self) -> String {
        format!(
            "mode={}\ntarget={}\nactive={}",
            self.mode, self.target, self.active
        )
    }
}

pub fn run_direct_connect(target: &str) -> DirectModeReport {
    DirectModeReport {
        mode: "direct-connect".into(),
        target: target.into(),
        active: true,
    }
}

pub fn run_deep_link(target: &str) -> DirectModeReport {
    DirectModeReport {
        mode: "deep-link".into(),
        target: target.into(),
        active: true,
    }
}
