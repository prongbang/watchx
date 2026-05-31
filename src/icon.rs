use std::path::Path;

pub fn get_file_icon(path: &Path) -> &'static str {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    match extension {
        // Programming Languages - เรียงตามความนิยม
        "js" | "jsx" => "🟨",            // JavaScript - สีเหลืองแทน JS
        "ts" | "tsx" => "🔷",            // TypeScript - สีฟ้าแทน TS
        "py" => "🐍",                    // Python
        "html" | "htm" => "🌈",          // HTML - สีสันมากขึ้น
        "css" | "scss" | "sass" => "🎨", // CSS
        "java" => "☕",                  // Java
        "php" => "🐘",                   // PHP
        "rs" => "🦀",                    // Rust
        "go" => "🐹",                    // Go
        "rb" => "💎",                    // Ruby
        "c" => "⚡",                     // C
        "cpp" | "cc" | "cxx" => "⚡⚡",  // C++
        "cs" => "🔷",                    // C#
        "swift" => "🦅",                 // Swift
        "kt" | "kts" => "🧩",            // Kotlin
        "dart" => "🎯",                  // Dart
        "lua" => "🌙",                   // Lua
        "r" => "📊",                     // R
        "pl" => "🐪",                    // Perl
        "scala" => "🔺",                 // Scala
        "clj" => "⚙️",                   // Clojure
        "ex" | "exs" => "💧",            // Elixir
        "hs" => "λ️",                     // Haskell

        // Data & Config
        "json" => "📋",                      // JSON
        "xml" => "🔄",                       // XML
        "yml" | "yaml" => "⚙️",              // YAML
        "toml" => "📦",                      // TOML
        "ini" => "🔧",                       // INI
        "csv" => "📑",                       // CSV
        "sql" => "💾",                       // SQL
        "db" | "sqlite" | "sqlite3" => "🗃️", // Database files

        // Web Assets
        "svg" => "🖌️",          // SVG (vector)
        "jpg" | "jpeg" => "📸", // JPEG (photos)
        "png" => "🖼️",          // PNG (images with transparency)
        "gif" => "🎞️",          // GIF (animated)
        "webp" => "🏞️",         // WebP
        "ico" => "🏷️",          // Icon

        // Documents
        "md" | "markdown" => "📝", // Markdown
        "txt" => "📄",             // Text
        "pdf" => "📕",             // PDF
        "doc" | "docx" => "📘",    // Word
        "xls" | "xlsx" => "📗",    // Excel
        "ppt" | "pptx" => "📙",    // PowerPoint
        "odt" => "📃",             // OpenDocument Text
        "rtf" => "📜",             // Rich Text Format

        // Development Tools
        "sh" | "bash" | "zsh" => "🐚",                 // Shell
        "bat" | "cmd" => "⌨️",                         // Windows Batch/Command
        "ps1" => "💻",                                 // PowerShell
        "git" | "gitignore" | "gitattributes" => "🌱", // Git
        "docker" | "dockerfile" => "🐳",               // Docker
        "makefile" => "🛠️",                            // Makefile
        "lock" => "🔒",                                // Lock files
        "env" => "🔐",                                 // Environment variables

        // Media
        "mp4" | "mov" | "avi" | "mkv" | "webm" => "🎬", // Video
        "mp3" | "wav" | "ogg" | "flac" | "m4a" => "🎵", // Audio
        "ttf" | "otf" | "woff" | "woff2" => "🔤",       // Fonts

        // Archives
        "zip" => "🤐",        // ZIP
        "rar" => "📦",        // RAR
        "tar" => "📚",        // TAR
        "gz" | "tgz" => "🗜️", // GZIP
        "7z" => "🧰",         // 7-Zip

        // Logs & Temp Files
        "log" => "📊",          // Log files
        "tmp" | "temp" => "⏱️", // Temporary files
        "bak" => "💾",          // Backup files
        "cache" => "⚡",        // Cache files

        // Mobile & App Development
        "apk" => "📱",       // Android Package
        "ipa" => "🍎",       // iOS App
        "plist" => "📋",     // Property List (Apple)
        "xcodeproj" => "⌨️", // Xcode Project

        // 3D & Design
        "obj" | "fbx" | "blend" => "🧊", // 3D models
        "psd" | "ai" | "sketch" => "🎭", // Design files

        // Default - for everything else
        _ => "📄",
    }
}
