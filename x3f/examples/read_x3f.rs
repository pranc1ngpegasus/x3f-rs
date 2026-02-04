use std::env;
use std::fs;
use x3f::X3F;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <x3f_file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let data = fs::read(path).expect("Failed to read file");

    let x3f = match X3F::from_bytes(&data) {
        Ok(x3f) => x3f,
        Err(e) => {
            eprintln!("Failed to parse X3F: {:?}", e);
            std::process::exit(1);
        },
    };

    // ヘッダー情報
    let header = x3f.header();
    println!("=== X3F Header ===");
    println!(
        "File type: {:?}",
        String::from_utf8_lossy(header.file_type_identifier())
    );
    println!("Version: {:?}", header.file_format_version());
    println!(
        "Image size: {}x{}",
        u32::from_le_bytes(header.image_columns().try_into().unwrap()),
        u32::from_le_bytes(header.image_rows().try_into().unwrap())
    );
    println!("Rotation: {:?}", header.rotation());

    // 拡張ヘッダー（v2.1以降）
    if let Some(ext) = x3f.extended_header() {
        println!("\n=== Extended Header ===");
        println!(
            "White balance: {:?}",
            String::from_utf8_lossy(ext.white_balance_label_string())
        );
    }

    // ディレクトリ情報
    let dir = x3f.directory();
    let num_entries = u32::from_le_bytes(dir.entry_count().try_into().unwrap());
    println!("\n=== Directory ===");
    println!("Number of entries: {}", num_entries);

    // 各エントリの情報
    println!("\n=== Directory Entries ===");
    for (i, entry) in dir.entries().enumerate() {
        let offset = u32::from_le_bytes(entry.data_offset().try_into().unwrap());
        let length = u32::from_le_bytes(entry.data_length().try_into().unwrap());
        let entry_type = String::from_utf8_lossy(entry.entry_type());
        println!(
            "[{}] Type: {}, Offset: {}, Length: {}",
            i, entry_type, offset, length
        );
    }

    // セクションデータの詳細
    println!("\n=== Section Data ===");
    for entry in dir.entries() {
        let entry_type = String::from_utf8_lossy(entry.entry_type());
        match x3f.section_data(&entry) {
            Some(section) => {
                println!("Section {}: {:?}", entry_type, section);
            },
            None => {
                println!("Section {}: (out of bounds)", entry_type);
            },
        }
    }
}
