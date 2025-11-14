use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};

/// 搜索Atlas文件内容
pub fn search_atlas_content(dir_path: &Path, search_text: &str) -> Result<Vec<PathBuf>> {
    let files: Vec<PathBuf> = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("atlas"))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    if files.is_empty() {
        println!("📁 在目录 {:?} 中未找到 .atlas 文件", dir_path);
        return Ok(Vec::new());
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message("搜索Atlas文件中...");

    let results: Vec<PathBuf> = files.par_iter()
        .filter_map(|file_path| {
            pb.inc(1);
            match search_single_atlas_file(file_path, search_text) {
                Ok(true) => Some(file_path.clone()),
                Ok(false) => None,
                Err(e) => {
                    eprintln!("搜索文件 {:?} 时出错: {}", file_path, e);
                    None
                }
            }
        })
        .collect();

    pb.finish_with_message("Atlas搜索完成");
    Ok(results)
}

fn search_single_atlas_file(file_path: &Path, search_text: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)
        .context("无法读取Atlas文件")?;

    Ok(content.to_lowercase().contains(search_text))
}

/// 搜索Skel文件内容
pub fn search_skel_content(dir_path: &Path, search_texts: &[&str]) -> Result<Vec<PathBuf>> {
    let files: Vec<PathBuf> = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("skel"))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    if files.is_empty() {
        println!("📁 在目录 {:?} 中未找到 .skel 文件", dir_path);
        return Ok(Vec::new());
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    pb.set_message("搜索Skel文件中...");

    let search_texts_lower: Vec<String> = search_texts.iter()
        .map(|s| s.to_lowercase())
        .collect();

    let results: Vec<PathBuf> = files.par_iter()
        .filter_map(|file_path| {
            pb.inc(1);
            match search_single_skel_file(file_path, &search_texts_lower) {
                Ok(true) => Some(file_path.clone()),
                Ok(false) => None,
                Err(e) => {
                    eprintln!("搜索文件 {:?} 时出错: {}", file_path, e);
                    None
                }
            }
        })
        .collect();

    pb.finish_with_message("Skel搜索完成");
    Ok(results)
}

fn search_single_skel_file(file_path: &Path, search_texts: &[String]) -> Result<bool> {
    // Skel文件可能是二进制或文本，先尝试UTF-8解码
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => {
            // 如果UTF-8解码失败，尝试读取为字节并查找ASCII字符串
            let bytes = fs::read(file_path).context("无法读取Skel文件")?;

            // 将字节数据转换为字符串，忽略无效的UTF-8序列
            String::from_utf8_lossy(&bytes).to_string()
        }
    };

    // 检查是否包含所有搜索文本
    for search_text in search_texts {
        if !content.to_lowercase().contains(search_text) {
            return Ok(false);
        }
    }

    Ok(true)
}
