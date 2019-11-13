use async_std::fs;
use async_std::path::PathBuf;
use async_std::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

async fn work_through_directory(
    dir: &async_std::path::PathBuf,
    toplist: Arc<Mutex<HashMap<String, usize>>>,
    subdirectories: Arc<Mutex<VecDeque<async_std::path::PathBuf>>>,
) -> std::io::Result<()> {

    let mut entries = fs::read_dir(dir).await?;
    while let Some(res) = entries.next().await {
        let entry: async_std::fs::DirEntry = res?;
        let entry_type = entry.file_type().await?;
        let entry_path = entry.path();
        let _entry_fname = entry.file_name(); // without the leading path
        let entry_fname_str = _entry_fname.to_string_lossy();
        if entry_type.is_dir() {
            // Add to queue for traversing
            subdirectories.lock().unwrap().push_back(entry_path);
         } else {
            // Add to top list of filenames

            // split at the "." and then give me the last item
            let extension = entry_fname_str.rsplit('.').next();
            if let Some(ext) = extension {
                let extstr = ext.to_string();
                // this unwrap is will only fail unlocking when it's poisoned
                // i.e., when a thread that has unlocked it suddenly panics,
                // in which case we probably want to abort anyway
                toplist.lock().unwrap().entry(extstr).and_modify(|e| *e += 1).or_insert(1);
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    async_std::task::block_on(async {
        let subdirectories: Arc<Mutex<VecDeque<async_std::path::PathBuf>>> = Arc::new(Mutex::new(VecDeque::new()));
        let toplist: Arc<Mutex<HashMap<String, usize>>> = Arc::new(Mutex::new(HashMap::new()));

        let args = std::env::args().collect::<Vec<_>>();
        let p = match args.get(1) {
            Some(dir) => PathBuf::from(dir),
            _ => PathBuf::from(".")
        };

        subdirectories.lock().unwrap().push_back(p);

        while !subdirectories.lock().unwrap().is_empty() {
            let popped = subdirectories.lock().unwrap().pop_front();
            let d = popped.unwrap();
            let result = work_through_directory( &d, toplist.clone(), subdirectories.clone()).await;
            if let Err(e) = result {
                panic!("ERROR!\n\tError '{:?}' on Path {:?}", e, d.display());
            }

        }

        let toplistvec = toplist.lock().unwrap();
        let mut counted_vec: Vec<_> = toplistvec.iter().collect();
        counted_vec.sort_by(|a, b| a.1.cmp(b.1).reverse());

        let top100 = counted_vec.iter().take(100);
        println!("Top Ext Count");
        for (idx, extension_entry) in top100.enumerate() {
            println!("{} {} {}", idx+1, extension_entry.0, extension_entry.1);
        }
        Ok(())
    })
}
