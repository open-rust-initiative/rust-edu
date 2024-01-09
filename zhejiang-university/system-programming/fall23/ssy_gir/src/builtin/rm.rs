use std::fs;

use index;


pub fn cmd_rm(args: &[String]) {
    if let Err(why) = rm(args) {
        println!("Could not rm paths: {:?}", why);
    }
}

  
fn rm(paths: &[String]) -> Result<(), index::Error> {  
    for path in paths {  
        fs::remove_file(path)?;  
    }  
    Ok(())  
} 