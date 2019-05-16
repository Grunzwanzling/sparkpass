use crate::transform;
use std::path;
use std::fs;
use std::str;

pub struct Options {
    pub args: Vec<String>,
    pub key: String,
    pub repo: String,
    pub verbose: bool,
    pub recursive: bool,
    pub force: bool,
    pub line: i32,
}

pub enum TreeNode {
    Node(String, Vec<TreeNode>),
    Leaf(String),
}

pub fn print_tree(tree: &TreeNode, prefix: String, last: bool, level: i32) {
     match tree {
        TreeNode::Leaf(name) => {
            if level > 0 {
                print!("{}", prefix);

                if last {
                    println!("└── {}", name);
                }else{
                    println!("├── {}", name);
                }
            }else{
                println!("{}", name);
            }
        },
        TreeNode::Node(name, children) => {
            if level != 0 { 
                print!("{}", prefix);
                if last {
                    println!("└── {}", name);
                }else{
                    println!("├── {}", name);
                }
            }else {
                println!("{}", name);
            }

            let mut i = 0;
            for c in children {
                i+=1;
                let mut prefix_new = prefix.clone();
                if level > 0 {
                    if !last {
                        prefix_new.push_str("│   ");
                    }else{
                        prefix_new.push_str("    ");
                    }
                }

                if i != children.len() {
                    print_tree(c, prefix_new, false, level+1);
                }else{
                    print_tree(c, prefix_new, true, level+1);
                }
            }
        }
    }
}

//remove slashes at the start and end
pub fn prepare_entry_path(path: &str) -> &str {
    let mut tmp = path.trim_start_matches("/");
    tmp = tmp.trim_end_matches("/");

    return tmp;
}

pub fn get_tree_from_path(p: &path::Path, is_clear: bool, enc_params: &transform::EncryptionParams) -> Result<TreeNode, String> {
    if p.is_file() {
        let filename = p.file_name().unwrap().to_str().unwrap();
        return Ok(TreeNode::Leaf(transform::retransform_entry(enc_params, filename)));
    }

    let it = match fs::read_dir(p) {
        Ok(iter) => iter,
        Err(_) => return Err("Couldnt read directory".to_owned()),
    };

    let mut result = Vec::new();

    for entry in it {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => return Err("Conversion error. Not UTF-8?".to_owned()),
        };

    
        let entryp = &entry.path();
        
        match get_tree_from_path(entryp, false, enc_params) {
            Ok(node) => {
                result.push(node);
                },
            Err(e) => {
                return Err(e);
            },
        }
    }

    let filename = p.file_name().unwrap().to_str().unwrap();

    let dirname = if !is_clear {
        transform::retransform_entry(enc_params, filename)
    }else{
        filename.to_owned()
    };
    return Ok(TreeNode::Node(dirname.to_owned(), result));
}

pub fn get_all_entries_in_path(p: &path::Path) -> Result<Vec<(String,bool)>, String> {
    let it = match fs::read_dir(p) {
        Ok(iter) => iter,
        Err(_) => return Err("Couldnt read directory".to_owned()),
    };

    let mut result = Vec::new();

    for entry in it {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => return Err("Conversion error. Not UTF-8?".to_owned()),
        };

        let dirp = &entry.path();
        let x = match path::Path::new(dirp).file_name(){
            Some(s) => match s.to_owned().to_str() {
                Some(s) => s.to_owned(),
                None => return Err("Conversion error. Not UTF-8?".to_owned()),
            },
            None => return Err("No Filename? Empty path?".to_owned()),
        };
        result.push((x, entry.path().is_dir()));
    }

    return Ok(result);
}

pub fn add_entry(prefix : &path::Path, p: &path::Path, content: &str, overwrite: bool, enc_params: &transform::EncryptionParams) -> Result<(), String> {
    let trans_path = transform::transform_path(enc_params, p.to_str().unwrap()).join("/");
    let full_path = prefix.join(trans_path.clone());

    let exists = match fs::metadata(full_path.clone()) {
        Ok(_) => true,
        Err(_) => false,
    };

    if exists && !overwrite {
        return Err("Entry exists already".to_owned())
    }else{
        let full_path_dir = full_path.as_path().parent().unwrap();
        match fs::create_dir_all(full_path_dir) {
            Ok(_) => {},
            Err(_) => {
                return Err("An error occured while creating necessary parent directories".to_owned());
            }
        }
    }

    let trans_content = transform::transform_entry(enc_params, content);
    match fs::write(full_path, trans_content) {
        Ok(_) => {},
        Err(_) => {
            return Err("An error occured while writing the content to the file".to_owned());
        }
    }

    return Ok(())
}

pub fn show_entry(prefix: &path::Path, p: &path::Path, enc_params: &transform::EncryptionParams) -> Result<String, String> {
    let trans_path = transform::transform_path(enc_params, p.to_str().unwrap()).join("/");
    let full_path = prefix.join(trans_path);

    if full_path.is_dir() {
        return Err("Is dir".to_owned());
    }

    let exists = match fs::metadata(full_path.as_path()) {
        Ok(_) => true,
        Err(_) => false,
    };

    if !exists {
        return Err("Entry does not exist".to_owned());
    }

    let res = match fs::read(full_path) {
        Ok(r) => r,
        Err(_) => {
            return Err("An error occured while reading the entry from the file".to_owned());
        }
    };


    let content = str::from_utf8(res.as_slice()).unwrap().to_owned();
    let clear_content = transform::retransform_entry(enc_params, content.as_str());

    return Ok(clear_content);
}