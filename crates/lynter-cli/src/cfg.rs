//! Config discovery (walking up from each target) and a per-run cache.

use lynter_core::config::{discover_config, load_config_file, Config, LoadResult};
use lynter_core::scope::Resolver;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct Loaded {
    pub config: Config,
    pub resolver: Resolver,
    pub path: PathBuf,
    pub dir: PathBuf,
}

#[derive(Default)]
pub struct Cache {
    by_path: HashMap<PathBuf, Result<Rc<Loaded>, Vec<String>>>,
    by_dir: HashMap<PathBuf, Option<(PathBuf, Option<String>)>>,
    pub explicit: Option<PathBuf>,
}

fn abs(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

impl Cache {
    pub fn new(explicit: Option<&Path>) -> Cache {
        Cache {
            explicit: explicit.map(abs),
            ..Cache::default()
        }
    }

    fn load_path(&mut self, path: &Path) -> Result<Rc<Loaded>, Vec<String>> {
        if let Some(hit) = self.by_path.get(path) {
            return hit.clone();
        }
        let result = match load_config_file(path) {
            LoadResult::Ok { config, warnings } => {
                // Core predates the `adapters` key and warns that it ignores
                // it; the adapter layer owns and validates it here.
                for w in warnings.iter().filter(|w| !w.contains(".adapters\"")) {
                    eprintln!("lynter: {}: warning: {w}", path.display());
                }
                let adapter_errors = lynter_code::adapters::validate_config(&config.raw);
                if !adapter_errors.is_empty() {
                    let errs: Vec<String> = adapter_errors
                        .into_iter()
                        .map(|e| format!("{}: {e}", path.display()))
                        .collect();
                    self.by_path.insert(path.to_path_buf(), Err(errs.clone()));
                    return Err(errs);
                }
                let resolver = Resolver::new(&config);
                Ok(Rc::new(Loaded {
                    resolver,
                    config: *config,
                    path: path.to_path_buf(),
                    dir: path.parent().map(Path::to_path_buf).unwrap_or_default(),
                }))
            }
            LoadResult::Err(errors) => Err(errors
                .into_iter()
                .map(|e| {
                    if e.path.is_empty() {
                        format!("{}: {}", path.display(), e.message)
                    } else {
                        format!("{}: {}: {}", path.display(), e.path, e.message)
                    }
                })
                .collect()),
        };
        self.by_path.insert(path.to_path_buf(), result.clone());
        result
    }

    /// Config governing a file in `dir`: the explicit one, else the nearest
    /// found walking up.
    pub fn for_dir(&mut self, dir: &Path) -> Result<Rc<Loaded>, Vec<String>> {
        if let Some(p) = self.explicit.clone() {
            return self.load_path(&p);
        }
        let start = abs(dir);
        if !self.by_dir.contains_key(&start) {
            let mut found = None;
            let mut cur = Some(start.as_path());
            while let Some(d) = cur {
                if let Some(f) = discover_config(d) {
                    found = Some((f.path, f.notice));
                    break;
                }
                cur = d.parent();
            }
            if let Some((p, Some(n))) = &found {
                if !self.by_path.contains_key(p) {
                    eprintln!("lynter: {}: {n}", p.display());
                }
            }
            self.by_dir.insert(start.clone(), found);
        }
        match self.by_dir[&start].clone() {
            Some((p, _)) => self.load_path(&p),
            None => Err(vec![format!(
                "no lynter.yaml, linter.yaml, or lappe-linter.yaml found walking up from {}",
                start.display()
            )]),
        }
    }
}
