use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use recovery_engine::engine::RecoveryEngine;

fn main() -> Result<()> {
    let disk_image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join("simulated_disk.img");

    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp")
        .join("recovered");

    println!("RustRecover — scan + extract");
    println!("Imagem de disco: {}", disk_image_path.display());
    println!("Destino:         {}", output_dir.display());
    println!();

    let mut disk_file = File::open(&disk_image_path)
        .with_context(|| format!("Não foi possível abrir '{}'", disk_image_path.display()))?;

    let disk_size = disk_file.metadata()?.len();
    println!("Tamanho do disco: {} bytes ({} setores de 512 B)", disk_size, disk_size / 512);
    println!();

    let engine = RecoveryEngine::new(&output_dir).with_chunk_size(512);

    let carved_files = engine
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
    println!();

    let extracted_files = engine
        .extract_all(&mut disk_file, &carved_files)
        .context("Falha durante a extração")?;

    let saved_paths = engine
        .save_all(&extracted_files)
        .context("Falha ao salvar os arquivos")?;

    println!("Arquivos extraídos para '{}':", output_dir.display());
    for path in &saved_paths {
        println!("  {}", path.file_name().unwrap().to_string_lossy());
    }

    Ok(())
}

