// Proprietary data formats and templates must be provided by the user at build time. If they are provided, we can succesfully convert them into Rust that will be referenced. If not, the build will fail.

use std::{env, fs, path::Path};

use serde::{self, Deserialize};

fn main() {
    let character_2d_id_path = Path::new("assets/lookup/character2ds.json");

    if character_2d_id_path.exists() {
        let json_data =
            fs::read_to_string(character_2d_id_path).expect("Failed to read character2ds.json");

        // Parse JSON into Vec<Character2DS>
        let characters: Vec<Character2DS> =
            serde_json::from_str(&json_data).expect("JSON deserialization failed");

        // Generate Rust code to embed as static array
        let mut code =
            String::from("use super::Character2DS;\npub static CHARACTERS: &[Character2DS] = &[\n");

        for c in &characters {
            code.push_str(&format!(
                "    Character2DS {{
                    id: {},
                    character_type: \"{}\".to_string(),
                    is_next_grade: {},
                    character_id: \"{}\".to_string(),
                    unit: \"{}\".to_string(),
                    is_enabled_flip_display: {},
                    asset_name: {},
                }},\n",
                c.id,
                c.character_type,
                c.is_next_grade,
                c.character_id,
                c.unit,
                c.is_enabled_flip_display,
                match &c.asset_name {
                    Some(val) => format!("Some({val:?})"),
                    None => "None".to_string(),
                },
            ));
        }
        code.push_str("];\n");

        fs::write("src/character2ds_gen.rs", code).expect("Failed to write generated Rust file");
    } else {
        eprintln!(
            "Missing {}! Mod generation and injection will NOT work without this file. To obtain it, please reference the README.",
            character_2d_id_path.display()
        );
        let code = "pub static CHARACTERS: &[Character2DS] = &[];\n";
        fs::write("src/character2ds_gen.rs", code).expect("Failed to write fallback Rust file");
    }
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/lookup/character2ds.json");
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Character2DS {
    id: i32,
    character_type: String,
    is_next_grade: bool,
    character_id: i32,
    unit: String,
    is_enabled_flip_display: bool,
    asset_name: Option<String>,
}
