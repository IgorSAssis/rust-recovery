use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use file_carver::scanner::Scanner;
use file_carver::signature::{JPEG_SIGNATURE, PNG_SIGNATURE};

fn main() -> Result<()> {
    let disk_image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join("simulated_disk.img");

    println!("RustRecover — teste de scanner");
    println!("Imagem de disco: {}", disk_image_path.display());
    println!();

    let mut disk_file = File::open(&disk_image_path)
        .with_context(|| format!("Não foi possível abrir '{}'", disk_image_path.display()))?;

    let disk_size = disk_file.metadata()?.len();
    println!("Tamanho do disco: {} bytes ({} setores de 512 B)", disk_size, disk_size / 512);
    println!();

    let carved_files = Scanner::new()
        .add_signature(&JPEG_SIGNATURE)
        .add_signature(&PNG_SIGNATURE)
        .with_chunk_size(512)
        .scan(&mut disk_file)
        .context("Falha durante o escaneamento")?;

    if carved_files.is_empty() {
        println!("Nenhum arquivo encontrado.");
        return Ok(());
    }

    println!("Arquivos encontrados: {}", carved_files.len());
    println!("{:-<55}", "");
    println!("{:<6}  {:<12}  {:<12}  {}", "Tipo", "Início", "Fim", "Tamanho");
    println!("{:-<55}", "");

    for carved_file in &carved_files {
        println!(
            "{:<6}  {:<12}  {:<12}  {} bytes",
            carved_file.kind.name(),
            carved_file.offset_start,
            carved_file.offset_end,
            carved_file.size(),
        );
    }

    println!("{:-<55}", "");

    Ok(())
}

