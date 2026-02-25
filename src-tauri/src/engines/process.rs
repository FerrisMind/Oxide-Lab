use log::info;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub struct ManagedProcess {
    pub process: Child,
    pub port: u16,
    pub health_url: String,
    pub is_running: Arc<AtomicBool>,
}

impl ManagedProcess {
    pub fn new(
        command_path: &str,
        args: &[String],
        port: u16,
        health_endpoint: &str,
    ) -> std::io::Result<Self> {
        info!("Starting process: {} with args: {:?}", command_path, args);

        let child = Command::new(command_path)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let health_url = format!("http://127.0.0.1:{}{}", port, health_endpoint);

        Ok(Self {
            process: child,
            port,
            health_url,
            is_running: Arc::new(AtomicBool::new(true)),
        })
    }

    pub async fn wait_ready(&self) -> Result<(), String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
            .map_err(|e| e.to_string())?;

        let start = Instant::now();
        let timeout = Duration::from_secs(30);

        info!("Waiting for engine at {}...", self.health_url);

        while start.elapsed() < timeout {
            if let Ok(resp) = client.get(&self.health_url).send().await
                && resp.status().is_success()
            {
                info!("Engine ready at {}", self.health_url);
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err(format!(
            "Engine timed out after {:?} at {}",
            timeout, self.health_url
        ))
    }

    pub fn kill(&mut self) -> Result<(), String> {
        info!("Stopping engine on port {}", self.port);
        self.is_running.store(false, Ordering::SeqCst);
        self.process.kill().map_err(|e| e.to_string())?;
        self.process.wait().map_err(|e| e.to_string())?;
        Ok(())
    }
}
