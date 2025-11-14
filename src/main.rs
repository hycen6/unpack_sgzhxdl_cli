use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;
use std::fs;
use dialoguer::{Select, Input, Confirm};
use console::Style;
use walkdir::WalkDir;

mod file_operations;
mod search;

use file_operations::*;
use search::*;

#[derive(Parser)]
#[command(name = "三幻Spine动态立绘还原工具")]
#[command(about = "一个用于三幻Spine动态立绘还原的CLI工具")]
struct Cli {
    /// 工作目录路径
    #[arg(short, long)]
    work_dir: Option<PathBuf>,
}

struct AppState {
    work_dir: PathBuf,
    atlas_dir: PathBuf,
    skels_dir: PathBuf,
}

impl AppState {
    fn new(work_dir: PathBuf) -> Self {
        let parent_dir = work_dir.parent().unwrap_or(&work_dir).to_path_buf();
        Self {
            work_dir: work_dir.clone(),
            atlas_dir: parent_dir.join("atlas"),
            skels_dir: parent_dir.join("skels"),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let work_dir = if let Some(dir) = cli.work_dir {
        dir
    } else {
        select_work_directory()?
    };

    let state = AppState::new(work_dir);
    let green = Style::new().green();

    if !state.work_dir.exists() {
        anyhow::bail!("工作目录不存在: {:?}", state.work_dir);
    }

    println!("✅ 工作目录设置为: {:?}", green.apply_to(state.work_dir.display()));

    loop {
        show_main_menu(&state)?;
    }
}

fn select_work_directory() -> Result<PathBuf> {
    let cyan = Style::new().cyan();
    let blue = Style::new().blue();
    println!("{}", cyan.apply_to("=== 欢迎使用三幻Spine动态立绘还原工具 ==="));
    println!();
    println!("{}", blue.apply_to("=== 操作指南 ==="));
    println!("1. 方向键选择操作");
    println!("2. 回车键确认选择");
    println!();
    println!("请选择工作目录(解压的美术资源目录，例如: miniRes):");

    // 获取当前目录的一级文件夹
    let current_dir = std::env::current_dir()?;
    let mut folders = Vec::new();

    if let Ok(entries) = fs::read_dir(&current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(folder_name) = path.file_name() {
                    if let Some(name_str) = folder_name.to_str() {
                        folders.push((name_str.to_string(), path.clone()));
                    }
                }
            }
        }
    }

    if folders.is_empty() {
        println!("当前目录下没有找到文件夹，请手动输入工作目录路径:");
        let input: String = Input::new()
            .with_prompt("工作目录路径")
            .interact()?;

        let path = PathBuf::from(input.trim());
        return Ok(path);
    }

    // 添加"其他"选项，让用户手动输入路径
    let mut display_items: Vec<String> = folders.iter().map(|(name, _)| name.clone()).collect();
    display_items.push("其他（手动输入路径）".to_string());

    let selection = Select::new()
        .items(&display_items)
        .default(0)
        .interact()?;

    if selection == folders.len() {
        // 用户选择了"其他"，手动输入路径
        let input: String = Input::new()
            .with_prompt("请输入工作目录路径")
            .interact()?;

        let path = PathBuf::from(input.trim());
        if !path.exists() {
            anyhow::bail!("目录不存在: {:?}", path);
        }
        Ok(path)
    } else {
        // 用户选择了现有文件夹
        let (_folder_name, path) = &folders[selection];
        Ok(path.clone())
    }
}

fn show_main_menu(state: &AppState) -> Result<()> {
    let blue = Style::new().blue();
    let green = Style::new().green();

    println!();
    println!("{}", blue.apply_to("=== 操作指南 ==="));
    println!("1. 方向键选择操作");
    println!("2. 回车键确认选择");
    println!();

    let items = vec![
        "恢复文件扩展名",
        "归类文件 (.atlas 和 .skel)",
        "重命名PNG文件（按尺寸）",
        "搜索Atlas内容",
        "搜索Skel内容",
        "显示当前工作目录信息",
        "退出"
    ];

    let selection = Select::new()
        .items(&items)
        .default(0)
        .interact()?;

    match selection {
        0 => handle_restore_extensions(state),
        1 => handle_organize_files(state),
        2 => handle_rename_png_files(state),
        3 => handle_search_atlas(state),
        4 => handle_search_skel(state),
        5 => show_work_dir_info(state),
        6 => {
            println!("{}", green.apply_to("感谢使用！"));
            std::process::exit(0);
        }
        _ => unreachable!(),
    }?;

    Ok(())
}

fn show_work_dir_info(state: &AppState) -> Result<()> {
    let green = Style::new().green();
    let yellow = Style::new().yellow();

    println!();
    println!("{}", green.apply_to("=== 工作目录信息 ==="));
    println!("{}: {}", yellow.apply_to("工作目录"), state.work_dir.display());
    println!("{}: {}", yellow.apply_to("Atlas目录"), state.atlas_dir.display());
    println!("{}: {}", yellow.apply_to("Skels目录"), state.skels_dir.display());

    // 统计工作目录文件数量
    if state.work_dir.exists() {
        let file_count = WalkDir::new(&state.work_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();

        println!("{}: {}", yellow.apply_to("工作目录文件数量"), file_count);
    }

    if state.atlas_dir.exists() {
        let atlas_count = WalkDir::new(&state.atlas_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();

        println!("{}: {}", yellow.apply_to("Atlas文件数量"), atlas_count);
    }

    if state.skels_dir.exists() {
        let skels_count = WalkDir::new(&state.skels_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();

        println!("{}: {}", yellow.apply_to("Skels文件数量"), skels_count);
    }

    println!();
    Ok(())
}

fn handle_restore_extensions(state: &AppState) -> Result<()> {
    let yellow = Style::new().yellow();
    let green = Style::new().green();

    if !state.work_dir.exists() {
        anyhow::bail!("工作目录不存在: {:?}", state.work_dir);
    }

    println!("{}", yellow.apply_to("📁 正在恢复文件扩展名..."));

    let proceed = Confirm::new()
        .with_prompt("确定要恢复工作目录中所有文件的扩展名吗？")
        .default(true)
        .interact()?;

    if !proceed {
        return Ok(());
    }

    restore_file_extensions(&state.work_dir)?;
    println!("{}", green.apply_to("✅ 文件扩展名恢复完成！"));

    Ok(())
}

fn handle_organize_files(state: &AppState) -> Result<()> {
    let yellow = Style::new().yellow();
    let green = Style::new().green();

    if !state.work_dir.exists() {
        anyhow::bail!("工作目录不存在: {:?}", state.work_dir);
    }

    println!("{}", yellow.apply_to("🗂️  正在归类文件..."));

    let proceed = Confirm::new()
        .with_prompt("确定要将.atlas和.skel文件移动到上级目录对应的文件夹吗？")
        .default(true)
        .interact()?;

    if !proceed {
        return Ok(());
    }

    // 移动atlas文件
    if !state.atlas_dir.exists() {
        fs::create_dir_all(&state.atlas_dir)?;
    }
    organize_files_by_extension(&state.work_dir, ".atlas", &state.atlas_dir)?;

    // 移动skel文件
    if !state.skels_dir.exists() {
        fs::create_dir_all(&state.skels_dir)?;
    }
    organize_files_by_extension(&state.work_dir, ".skel", &state.skels_dir)?;

    println!("{}", green.apply_to("✅ 文件归类完成！"));
    println!("📁 Atlas文件已移动到: {}", state.atlas_dir.display());
    println!("📁 Skel文件已移动到: {}", state.skels_dir.display());

    Ok(())
}

fn handle_rename_png_files(state: &AppState) -> Result<()> {
    let yellow = Style::new().yellow();
    let green = Style::new().green();

    if !state.work_dir.exists() {
        anyhow::bail!("工作目录不存在: {:?}", state.work_dir);
    }

    println!("{}", yellow.apply_to("🖼️  正在重命名PNG文件..."));

    let proceed = Confirm::new()
        .with_prompt("确定要按照尺寸重命名工作目录中的所有PNG文件吗？")
        .default(true)
        .interact()?;

    if !proceed {
        return Ok(());
    }

    rename_png_by_size(&state.work_dir)?;
    println!("{}", green.apply_to("✅ PNG文件重命名完成！"));

    Ok(())
}

fn handle_search_atlas(state: &AppState) -> Result<()> {
    let yellow = Style::new().yellow();

    if !state.atlas_dir.exists() {
        anyhow::bail!("Atlas目录不存在: {:?}", state.atlas_dir);
    }

    println!("{}", yellow.apply_to("🔍 搜索Atlas内容"));

    let search_text: String = Input::new()
        .with_prompt("请输入搜索内容（例如：2017,1937）")
        .interact()?;

    if search_text.trim().is_empty() {
        println!("⚠️  搜索内容不能为空");
        return Ok(());
    }

    let results = search_atlas_content(&state.atlas_dir, &search_text)?;

    if results.is_empty() {
        println!("❌ 未找到匹配的内容");
    } else {
        println!("✅ 找到 {} 个匹配的文件:", results.len());
        for result in results {
            println!("  📄 {}", result.display());
        }
    }

    Ok(())
}

fn handle_search_skel(state: &AppState) -> Result<()> {
    let yellow = Style::new().yellow();

    if !state.skels_dir.exists() {
        anyhow::bail!("Skels目录不存在: {:?}", state.skels_dir);
    }

    println!("{}", yellow.apply_to("🔍 搜索Skel内容"));

    println!("请输入搜索内容（支持多个，用空格分隔，例如：biaoqing_jiangdongzhizhi biaoqing_yansu）:");
    let search_input: String = Input::new()
        .with_prompt("搜索内容")
        .interact()?;

    let search_texts: Vec<&str> = search_input.trim().split_whitespace().collect();

    if search_texts.is_empty() || (search_texts.len() == 1 && search_texts[0].is_empty()) {
        println!("⚠️  搜索内容不能为空");
        return Ok(());
    }

    let results = search_skel_content(&state.skels_dir, &search_texts)?;

    if results.is_empty() {
        println!("❌ 未找到匹配的文件");
    } else {
        println!("✅ 找到 {} 个匹配的文件:", results.len());
        for result in results {
            println!("  📄 {}", result.display());
        }
    }

    Ok(())
}