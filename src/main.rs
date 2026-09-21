use clap::{ArgGroup, Parser};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};
use tmtch::AppPaths;

#[derive(Parser, Debug)]
#[command(
    name = "tmrch",
    version,
    about = "拡張子ごとのテンプレートで新規ファイルを作成する"
)]
#[command(group(ArgGroup::new("mode").args(["file", "edit", "list"]).required(true).multiple(false)))]
struct Cli {
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
    #[arg(long, value_name = "EXTENSION")]
    edit: Option<String>,
    #[arg(long)]
    list: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let paths = match AppPaths::from_system() {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    match run(cli, &paths) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli, paths: &AppPaths) -> Result<(), String> {
    if let Some(file) = cli.file {
        return create_file(&file, paths);
    }
    if let Some(extension) = cli.edit {
        return edit_template(&extension, paths);
    }
    if cli.list {
        return list_templates(paths);
    }
    unreachable!()
}

fn extension_key(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    if name.starts_with('.') && !name[1..].contains('.') {
        return None;
    }
    let (_, key) = name.rsplit_once('.')?;
    (!key.is_empty()).then(|| key.to_ascii_lowercase())
}

fn normalize_extension(value: &str) -> Option<String> {
    let key = value.trim().trim_start_matches('.').to_ascii_lowercase();
    (!key.is_empty() && !key.contains('/') && !key.contains('\\')).then_some(key)
}

fn create_file(path: &Path, paths: &AppPaths) -> Result<(), String> {
    if path.is_dir() {
        return Err(format!(
            "{} はディレクトリです。ファイルパスを指定してください。",
            path.display()
        ));
    }
    let bytes = match extension_key(path) {
        Some(key) => paths
            .read_template(&key)
            .map_err(|error| error.to_string())?
            .unwrap_or_default(),
        None => Vec::new(),
    };
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(path).map_err(|error| {
        format!(
            "{} を作成できません: {error}。親ディレクトリと既存ファイルを確認してください。",
            path.display()
        )
    })?;
    use std::io::Write;
    file.write_all(&bytes)
        .map_err(|error| format!("{} へ書き込めません: {error}", path.display()))
}

fn list_templates(paths: &AppPaths) -> Result<(), String> {
    let directory = paths.templates_dir();
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{} を一覧できません: {error}", directory.display())),
    };
    let mut templates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_file()
        {
            templates.push((
                entry.file_name().to_string_lossy().into_owned(),
                entry
                    .path()
                    .canonicalize()
                    .map_err(|error| error.to_string())?,
            ));
        }
    }
    templates.sort_by(|left, right| left.0.cmp(&right.0));
    for (extension, path) in templates {
        println!("{extension}\t{}", path.display());
    }
    Ok(())
}

fn edit_template(value: &str, paths: &AppPaths) -> Result<(), String> {
    let extension = normalize_extension(value)
        .ok_or_else(|| "拡張子を指定してください。例: rs または .rs".to_owned())?;
    paths.initialize().map_err(|error| error.to_string())?;
    let target = paths.template_file(&extension);
    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&target)
        .map_err(|error| format!("{} を作成できません: {error}", target.display()))?;
    let command = paths.load_config().map_err(|error| error.to_string())?.editor.or_else(|| env::var("VISUAL").ok()).or_else(|| env::var("EDITOR").ok()).ok_or_else(|| "editor が設定されていません。config.toml の editor、VISUAL、または EDITOR を設定してください。".to_owned())?;
    let arguments = parse_command(&command)?;
    let (program, arguments) = arguments.split_first().unwrap();
    let status = Command::new(program)
        .args(arguments)
        .arg(&target)
        .status()
        .map_err(|error| format!("エディタ {program} を起動できません: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "エディタが終了コード {:?} で終了しました。",
            status.code()
        ))
    }
}

fn parse_command(input: &str) -> Result<Vec<String>, String> {
    let mut arguments = Vec::new();
    let mut argument = String::new();
    let mut quote = None;
    let mut started = false;
    let mut escaped = false;
    for character in input.chars() {
        if escaped {
            argument.push(character);
            started = true;
            escaped = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            started = true;
            continue;
        }
        match (quote, character) {
            (Some(current), character) if current == character => quote = None,
            (None, '\'' | '"') => {
                quote = Some(character);
                started = true;
            }
            (None, character) if character.is_whitespace() => {
                if started {
                    arguments.push(std::mem::take(&mut argument));
                    started = false;
                }
            }
            (_, character) => {
                argument.push(character);
                started = true;
            }
        }
    }
    if escaped || quote.is_some() {
        return Err("editor の引用符またはエスケープが閉じられていません".into());
    }
    if started {
        arguments.push(argument);
    }
    if arguments.is_empty() {
        Err("editor が空です".into())
    } else {
        Ok(arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_extensions_and_ignores_dotfiles() {
        assert_eq!(extension_key(Path::new("Main.RS")), Some("rs".into()));
        assert_eq!(extension_key(Path::new(".gitignore")), None);
        assert_eq!(normalize_extension(".RS"), Some("rs".into()));
    }
    #[test]
    fn parses_quoted_and_escaped_editor_arguments() {
        assert_eq!(
            parse_command("'my editor' --wait \"\" \"a\\\"b\"").unwrap(),
            vec!["my editor", "--wait", "", "a\"b"]
        );
    }
}
