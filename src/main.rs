use std::env::current_exe;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};

use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty, Value};

use tokio::io::{
    stdin as get_stdin, stdout as get_stdout, AsyncBufRead, AsyncBufReadExt, AsyncReadExt,
    AsyncWriteExt, BufReader,
};
use tokio::process::Command;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    output_folder: PathBuf,
    binary: PathBuf,
}

#[derive(Default, Debug)]
struct PacketHeader {
    content_length: usize,
}

#[derive(Debug)]
struct Packet {
    header: PacketHeader,
    raw: String,
    formatted: String,
}

const CONFIG_FILE_NAME: &str = "lsp_proxy.toml";

fn get_time_in_millis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

async fn write_to_log(path: impl AsRef<Path>, text: &str, msg_type: &str) -> Result<()> {
    let time = get_time_in_millis();
    let mut path = path.as_ref().to_path_buf();
    path.push(format!("{time}_{msg_type}.json"));

    let mut output = tokio::fs::File::create(&path).await?;
    output.write_all(text.as_bytes()).await?;
    eprintln!("{text}");

    Ok(())
}

fn get_path_of_binary() -> Result<PathBuf> {
    let current_exe_path = current_exe()?;
    let exe_dir_path = current_exe_path.parent();
    let exe_dir_path = exe_dir_path.context("No parent directory of exe.")?;
    Ok(exe_dir_path.to_path_buf())
}

fn get_config(directory: impl AsRef<Path>) -> Result<Config> {
    let mut config_path = directory.as_ref().to_path_buf();
    config_path.push(CONFIG_FILE_NAME);
    let string = std::fs::read_to_string(config_path).context("Trying to open the config file.")?;
    Ok(toml::from_str(&string)?)
}

async fn read_packet_from_input<T>(fp: &mut BufReader<T>) -> Result<Packet>
where
    BufReader<T>: AsyncBufRead,
    BufReader<T>: AsyncBufReadExt,
    T: std::marker::Unpin,
{
    let mut header: PacketHeader = PacketHeader::default();

    loop {
        let mut line = String::new();
        fp.read_line(&mut line).await?;
        if let Some(content_length) = line.strip_prefix("Content-Length: ") {
            header.content_length = content_length
                .trim()
                .parse()
                .context("Trying to parse Content-Length")?;
        } else if line.strip_prefix("Content-Type: ").is_some() {
            // ignored.
        } else if line == "\r\n" {
            break;
        } else {
            bail!("Could not parse input as LSP data.")
        }
    }

    let mut underlying = vec![0u8; header.content_length];
    fp.read_exact(&mut underlying).await.unwrap();

    let raw = String::from_utf8(underlying)?;
    let json: Value = from_str(&raw).context("Could not convert string to json.")?;
    let formatted = to_string_pretty(&json).context("Could not convert string from json.")?;

    Ok(Packet {
        header,
        formatted,
        raw,
    })
}

async fn forwarding_loop<T>(
    mut reader: BufReader<T>,
    mut writer: impl AsyncWriteExt + std::marker::Unpin,
    msg_type: &str,
    output_folder: &Path,
) where
    BufReader<T>: AsyncBufRead,
    BufReader<T>: AsyncBufReadExt,
    T: std::marker::Unpin,
{
    loop {
        let packet = read_packet_from_input(&mut reader).await.unwrap();
        write_to_log(&output_folder, &packet.formatted, msg_type)
            .await
            .unwrap();
        writer
            .write_all(
                format!("Content-Length: {}\r\n\r\n", packet.header.content_length).as_bytes(),
            )
            .await
            .unwrap();
        writer.write_all(&packet.raw.into_bytes()).await.unwrap();
    }
}

async fn async_main() -> Result<()> {
    let binary_path = get_path_of_binary().context("Tried to get path of this binary.")?;
    let config = get_config(binary_path).context("Trying to get the config")?;

    let mut cmd = Command::new(config.binary);

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());

    let mut child = cmd
        .args(&std::env::args().collect::<Vec<_>>()[1..])
        .spawn()
        .context("Tried to spawn rust-analyser binary.")?;

    let stdout = child
        .stdout
        .take()
        .context("Tried to get stdout from binary.")?;

    let subprocess_writer = child
        .stdin
        .take()
        .context("Tried to get stdin from binary.")?;

    let subprocess_reader = BufReader::new(stdout);
    let process_stdin = BufReader::new(get_stdin());
    let process_stdout = get_stdout();

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    let output_folder = config.output_folder.clone();
    tokio::spawn(async move {
        forwarding_loop(
            process_stdin,
            subprocess_writer,
            "server-recv",
            &output_folder,
        )
        .await
    });

    let output_folder = config.output_folder;
    tokio::spawn(async move {
        forwarding_loop(
            subprocess_reader,
            process_stdout,
            "server-send",
            &output_folder,
        )
        .await
    });

    child
        .wait()
        .await
        .context("While trying to run the program.")?;

    Ok(())
}

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { async_main().await })
}
