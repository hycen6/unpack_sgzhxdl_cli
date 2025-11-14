use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Read;
use walkdir::WalkDir;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};

pub fn restore_file_extensions(dir_path: &Path) -> Result<()> {
    let files: Vec<PathBuf> = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message("恢复文件扩展名中...");

    files.par_iter().for_each(|file_path| {
        if let Err(e) = restore_single_file_extension(file_path) {
            eprintln!("处理文件 {:?} 时出错: {}", file_path, e);
        }
        pb.inc(1);
    });

    pb.finish_with_message("文件扩展名恢复完成");
    Ok(())
}

fn restore_single_file_extension(file_path: &Path) -> Result<()> {
    if file_path.extension().is_some() {
        return Ok(()); // 已有扩展名，跳过
    }

    let file_type = detect_file_type(file_path)?;

    if let Some(extension) = file_type {
        let mut new_path = file_path.to_path_buf();
        new_path.set_extension(extension);

        // 重名文件添加自增数字后缀
        let mut counter = 1;
        while new_path.exists() {
            let stem = file_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            new_path = file_path.with_file_name(format!("{}_{}.{}", stem, counter, extension));
            counter += 1;
        }

        fs::rename(file_path, &new_path)?;
        println!("✓ {} -> {}",
            file_path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
            new_path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        );
    }

    Ok(())
}

fn detect_file_type(file_path: &Path) -> Result<Option<&'static str>> {
    let mut file = File::open(file_path).context("无法打开文件")?;
    let mut header = [0u8; 64]; // 增加到64字节以更好检测XML头部
    let bytes_read = file.read(&mut header).context("无法读取文件头")?;

    if bytes_read >= 8 && &header[0..8] == b"\x89PNG\r\n\x1a\n" {
        return Ok(Some("png"));
    }

    // 将头部解码为UTF-8进行初步检测
    if let Ok(header_str) = std::str::from_utf8(&header[..bytes_read]) {
        let trimmed = header_str.trim();

        if trimmed.starts_with("<?xml") || trimmed.starts_with("<!DOCTYPE") {
            return Ok(Some("xml"));
        }

        // 检查其他XML格式（如plist等）
        if trimmed.starts_with('<') {
            // 进一步验证是否为有效的XML格式
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let content_trimmed = content.trim();
                if content_trimmed.starts_with("<?xml") ||
                   content_trimmed.starts_with("<!DOCTYPE") ||
                   content_trimmed.contains("</plist>") ||
                   content_trimmed.contains("<dict>") ||
                   (content_trimmed.starts_with('<') && content_trimmed.ends_with('>')) {
                    return Ok(Some("xml"));
                }
            }
        }

        // 初步简单JSON检测
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            // 进一步验证是否为有效JSON（读取整个文件）
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                    return Ok(Some("json"));
                }
            }
        }
    }

    if can_read_to_string(file_path)? {
        Ok(Some("atlas"))
    } else {
        Ok(Some("skel"))
    }
}

fn can_read_to_string(file_path: &Path) -> Result<bool> {
    match std::fs::read_to_string(file_path) {
        Ok(_) => {
            Ok(true)
        }
        Err(_) => {
            Ok(false)
        }
    }
}

/// 按扩展名组织文件
pub fn organize_files_by_extension(source_dir: &Path, extension: &str, target_dir: &Path) -> Result<()> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    let extension = if !extension.starts_with('.') {
        format!(".{}", extension)
    } else {
        extension.to_string()
    };

    let files: Vec<PathBuf> = WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == extension.trim_start_matches('.'))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    if files.is_empty() {
        println!("📁 未找到扩展名为 {} 的文件", extension);
        return Ok(());
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message(format!("移动 {} 文件中...", extension));

    for file_path in files {
        if let Err(e) = move_single_file(&file_path, target_dir) {
            eprintln!("移动文件 {:?} 时出错: {}", file_path, e);
        }
        pb.inc(1);
    }

    pb.finish_with_message(format!("{} 文件移动完成", extension));
    Ok(())
}

fn move_single_file(file_path: &Path, target_dir: &Path) -> Result<()> {
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("无效的文件名"))?;

    let target_path = target_dir.join(file_name);

    // 如果目标文件已存在，添加数字后缀
    let final_target_path = if target_path.exists() {
        let stem = Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = Path::new(file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let mut counter = 1;
        let mut new_target = target_dir.join(format!("{}_{}.{}", stem, counter, extension));
        while new_target.exists() {
            counter += 1;
            new_target = target_dir.join(format!("{}_{}.{}", stem, counter, extension));
        }
        new_target
    } else {
        target_path
    };

    fs::rename(file_path, &final_target_path)?;
    println!("✓ 移动: {} -> {}",
        file_name,
        final_target_path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
    );

    Ok(())
}

/// 根据尺寸重命名PNG文件
pub fn rename_png_by_size(dir_path: &Path) -> Result<()> {
    let files: Vec<PathBuf> = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message("重命名PNG文件中...");

    for file_path in files {
        if let Err(e) = rename_single_png(&file_path) {
            eprintln!("重命名PNG文件 {:?} 时出错: {}", file_path, e);
        }
        pb.inc(1);
    }

    pb.finish_with_message("PNG文件重命名完成");
    Ok(())
}

fn rename_single_png(file_path: &Path) -> Result<()> {
    let (width, height) = get_png_dimensions(file_path)?;

    let new_name = format!("size_{}x{}.png", width, height);
    let new_path = file_path.parent()
        .unwrap_or(file_path)
        .join(&new_name);

    // 如果新文件名已存在，添加数字后缀
    let final_new_path = if new_path.exists() {
        let mut counter = 1;
        let mut new_path_with_counter = file_path.parent()
            .unwrap_or(file_path)
            .join(format!("size_{}x{}_{}.png", width, height, counter));

        while new_path_with_counter.exists() {
            counter += 1;
            new_path_with_counter = file_path.parent()
                .unwrap_or(file_path)
                .join(format!("size_{}x{}_{}.png", width, height, counter));
        }
        new_path_with_counter
    } else {
        new_path
    };

    fs::rename(file_path, &final_new_path)?;

    println!("✓ {} -> {}",
        file_path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
        final_new_path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
    );

    Ok(())
}

fn get_png_dimensions(file_path: &Path) -> Result<(u32, u32)> {
  
    let mut file = File::open(file_path).context("无法打开PNG文件")?;

    // 检查PNG签名
    let mut signature = [0u8; 8];
    file.read_exact(&mut signature).context("无法读取PNG签名")?;

    if signature != [137, 80, 78, 71, 13, 10, 26, 10] {
        anyhow::bail!("不是有效的PNG文件");
    }

    // 读取IHDR chunk
    let mut chunk_header = [0u8; 8];
    file.read_exact(&mut chunk_header).context("无法读取IHDR chunk")?;

    // 验证chunk类型
    if &chunk_header[4..8] != b"IHDR" {
        anyhow::bail!("无法找到IHDR chunk");
    }

    // 读取宽度和高度
    let mut ihdr_data = [0u8; 8];
    file.read_exact(&mut ihdr_data).context("无法读取IHDR数据")?;

    let width = u32::from_be_bytes([ihdr_data[0], ihdr_data[1], ihdr_data[2], ihdr_data[3]]);
    let height = u32::from_be_bytes([ihdr_data[4], ihdr_data[5], ihdr_data[6], ihdr_data[7]]);

    Ok((width, height))
}