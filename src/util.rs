//! sysvinit model — System V init system.

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub runlevel: u32,
    pub status: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Runlevel {
    pub level: u32,
    pub services: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub services: Vec<Service>,
    pub runlevels: Vec<Runlevel>,
    pub current: u32,
}

impl Default for Model {
    fn default() -> Self { Self { services: vec![], runlevels: vec![], current: 3 } }
}

impl Model {
    pub fn new() -> Self { Self::default() }

    pub fn add_service(&mut self, name: &str, runlevel: u32) {
        if !self.services.iter().any(|s| s.name == name) {
            self.services.push(Service { name: name.into(), runlevel, status: "stopped".into(), pid: None });
        }
    }

    pub fn start(&mut self, name: &str) -> bool {
        if let Some(s) = self.services.iter_mut().find(|s| s.name == name) {
            s.status = "running".into();
            s.pid = Some(1000);
            true
        } else { false }
    }

    pub fn stop(&mut self, name: &str) -> bool {
        if let Some(s) = self.services.iter_mut().find(|s| s.name == name) {
            s.status = "stopped".into();
            s.pid = None;
            true
        } else { false }
    }

    pub fn set_runlevel(&mut self, level: u32) -> bool {
        if (0..=6).contains(&level) || level == 2 || level == 4 { self.current = level; true } else { false }
    }

    pub fn list(&self) -> &[Service] { &self.services }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn service_runlevel() {
        let mut s = Model::new();
        s.add_service("ssh", 2);
        assert!(s.start("ssh"));
        assert!(s.set_runlevel(5));
        assert!(s.stop("ssh"));
    }
}
