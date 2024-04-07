use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::prelude::*;

const MAX_FREQUENCY: i32 = 500;
pub struct CacheSystem {
    cache: HashMap<String, Vec<u8>>,
    max_size: u64,
    cur_size: u64,
    frequency: HashMap<String, i32>,
}

impl CacheSystem {
    pub fn new(max_size: u64) -> Self {
        CacheSystem {
            cache: HashMap::new(),
            max_size,
            cur_size: 0,
            frequency: HashMap::new(),
        }
    }

    pub fn add(&mut self, path: &str, content: &str) {
        self.add_frequency(path);

        if self.is_full() {
            let least_path = self.least_frequent(path);
            dbg!(&least_path);
            if least_path != path {
                self.delete_cache(&least_path);
                if !self.is_full() {
                    self.add_cache(path, content);
                }
            }
        } else {
            self.add_cache(path, content);
        }
    }

    pub fn get(&mut self, path: &str) -> String {
        let bytes = self.cache.get(path).unwrap();
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut s = String::new();
        decoder.read_to_string(&mut s).unwrap();
        self.add_frequency(path);
        dbg!(self.least_frequent(path));
        dbg!(self.is_full());
        dbg!(&self.frequency);
        let keys: Vec<String> = self.cache.keys().cloned().collect();
        dbg!(keys);

        dbg!(s.len());
        s
    }

    pub fn delete_cache(&mut self, path: &str) {
        let deleted = self.cache.remove(path).unwrap();
        self.cur_size -= deleted.len() as u64 * 8;
    }

    pub fn add_cache(&mut self, path: &str, content: &str) {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(content.as_bytes()).unwrap();
        let bytes = encoder.finish().unwrap();
        self.cur_size += 8 * bytes.len() as u64;
        self.cache.insert(path.to_string(), bytes);
    }

    pub fn add_frequency(&mut self, path: &str) {
        let handle = self.frequency.entry(path.to_string()).or_insert(0);
        *handle += 1;
        if *handle >= MAX_FREQUENCY {
            self.reset_frequency();
        }
    }

    pub fn least_frequent(&self, other: &str) -> String {
        let mut candidates: Vec<(String, i32)> = self
            .cache
            .keys()
            .map(|x| (x.to_string(), *self.frequency.get(x).unwrap()))
            .collect();
        candidates.push((
            other.to_owned(),
            self.frequency.get(other).unwrap().to_owned(),
        ));
        let a = candidates.par_drain(..).reduce(
            || (String::new(), i32::MAX),
            |acc, x| {
                if x.1 < acc.1 {
                    x
                } else {
                    acc
                }
            },
        );

        a.0.to_string()
    }

    pub fn is_full(&self) -> bool {
        dbg!(self.cur_size, self.max_size);
        if self.cur_size >= self.max_size {
            true
        } else {
            false
        }
    }

    fn reset_frequency(&mut self) {
        let fre_sum = self.frequency.values().cloned().par_bridge().reduce(|| 0, |acc, x| acc + x);
        let keys: Vec<String> = self.frequency.keys().cloned().collect();
        for path in keys.into_iter() {
            let tmp = self.frequency.get(&path).unwrap().to_owned() * 100  / fre_sum;
            if tmp <= 0 {
                self.frequency.remove(&path);
                if self.cache.contains_key(&path) {
                    self.delete_cache(&path);
                }
            } else {
                let handle = self.frequency.get_mut(&path).unwrap();
                *handle = tmp;
            }
        }
        dbg!(fre_sum);
    }

    pub fn contains_key(&self, path: &str) -> bool {
        if self.cache.contains_key(path) {
            true
        } else {
            false
        }
    }

    pub fn get_cache(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }
}
