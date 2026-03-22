use aes_gcm::{
    AeadCore, Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use zeroize::Zeroize;

pub fn is_prime(n: u64) -> bool {
    if n <= 1 { return false; }
    if n <= 3 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}

pub fn next_prime(n: u64) -> u64 {
    let mut p = n + 1;
    while !is_prime(p) {
        p += 1;
    }
    p
}

pub fn tokenize(sentence: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();

    for c in sentence.chars() {
        if c.is_whitespace() {
            if !current_word.is_empty() {
                tokens.push(current_word.clone());
                current_word.clear();
            }
        } else if !c.is_alphanumeric() {
            if !current_word.is_empty() {
                tokens.push(current_word.clone());
                current_word.clear();
            }
            tokens.push(c.to_string());
        } else {
            current_word.push(c);
        }
    }
    if !current_word.is_empty() {
        tokens.push(current_word);
    }
    tokens
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SensitivityLevel {
    Level1,
    Level2,
    Level3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CellMetadata {
    pub id: u64,
    pub sensitivity: SensitivityLevel,
    pub owner_id: String,
    #[serde(default)]
    pub created_at: u64, // 🧠 Yeni
    pub deleted_at: Option<u64>,
}

impl CellMetadata {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Serileştirme hatası")
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).expect("Hücre üst verisi çözülemedi")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CortexCell {
    pub id: u64,
    pub content: Vec<u8>,
    pub sensitivity: SensitivityLevel,
    pub owner_id: String,
    #[serde(default)]
    pub created_at: u64, // 🧠 Yeni
    pub deleted_at: Option<u64>,
}

// Geriye dönük uyumluluk veya yedek kullanım için
impl CortexCell {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Serileştirme hatası")
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).expect("Hücre verisi çözülemedi")
    }
}

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct MemoryVault {
    key: [u8; 32],
}

impl MemoryVault {
    pub fn new(raw_key: [u8; 32]) -> Self {
        Self { key: raw_key }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .expect("Encryption failed");
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        result
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.key));
        let (nonce_part, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_part);
        cipher
            .decrypt(nonce, ciphertext)
            .expect("Decryption failed")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Synapse {
    pub target_id: u64,
    pub category: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NeuralVault {
    pub word_to_id: HashMap<String, u64>,
    pub id_to_word: HashMap<u64, String>,
    pub synapses: HashMap<u64, Vec<Synapse>>,
    pub next_id: u64,
    pub sentence_synapses: HashMap<u64, Vec<u64>>, // 🧠 Cümle ID -> Kelime Prime ID'leri
}

impl NeuralVault {
    pub fn new() -> Self {
        NeuralVault {
            word_to_id: HashMap::new(),
            id_to_word: HashMap::new(),
            synapses: HashMap::new(),
            next_id: 2, // 2'den başlatıyoruz (Asal sayı sayacı)
            sentence_synapses: HashMap::new(),
        }
    }

    pub fn get_or_create_neuron(&mut self, word: &str) -> u64 {
        let clean_word = word.to_lowercase().trim().to_string();
        if let Some(&id) = self.word_to_id.get(&clean_word) {
            return id;
        }
        let id = self.next_id;
        self.word_to_id.insert(clean_word.clone(), id);
        self.id_to_word.insert(id, clean_word);
        self.next_id = next_prime(self.next_id);
        id
    }

    pub fn add_sentence(&mut self, sentence_id: u64, sentence: &str, category: &str) {
        let words = tokenize(sentence);
        let mut prev_id: Option<u64> = None;
        let mut prime_ids = Vec::new();

        for word in &words {
            let id = self.get_or_create_neuron(word);
            prime_ids.push(id);
            
            if let Some(p_id) = prev_id {
                let edge = Synapse {
                    target_id: id,
                    category: category.to_string(),
                };
                self.synapses.entry(p_id).or_insert(Vec::new()).push(edge);
            }
            prev_id = Some(id);
        }

        // 🧠 Cümle ID'sine göre kullanılan tüm asal sayıları (kelimeleri) kaydet
        self.sentence_synapses.insert(sentence_id, prime_ids);
    }

    pub fn search(&self, query_word: &str, category: &str) -> Vec<String> {
        let mut results = Vec::new();
        let query_lower = query_word.to_lowercase();

        if let Some(&id) = self.word_to_id.get(&query_lower) {
            if let Some(edges) = self.synapses.get(&id) {
                for edge in edges {
                    if edge.category == category {
                        if let Some(word) = self.id_to_word.get(&edge.target_id) {
                            results.push(word.clone());
                        }
                    }
                }
            }
        }
        results
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let bytes = bincode::serialize(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut file = File::create(path)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let vault: NeuralVault = bincode::deserialize(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(vault)
    }
}

// Veritabanı Motoru: Storage + Index
pub struct CortexEngine {
    storage_path: String,
    // ID -> (Metadata, Content Payload Offset, Payload Length)
    index: HashMap<u64, (CellMetadata, u64, u64)>, 
    pub neural_graph: NeuralVault, // Bellek içi nöron grafı
}

impl CortexEngine {
    pub fn new(storage_path: &str) -> Self {
        let graph_path = format!("{}.graph", storage_path);
        let neural_graph = NeuralVault::load_from_file(&graph_path).unwrap_or_else(|_| NeuralVault::new());

        let mut engine = CortexEngine {
            storage_path: storage_path.to_string(),
            index: HashMap::new(),
            neural_graph,
        };

        if std::path::Path::new(storage_path).exists() {
            if let Err(e) = engine.restore_index() {
                eprintln!("Indeks yukleme hatasi: {}", e);
            }
        }

        engine
    }

    // YENİ MİMARİ: Dosyayı baştan sona tarayıp HashMap'i O(1) bellek kullanarak doldurur
    // Verileri (Payload'ları) okumadan atlar (seek), sadece Metadata'yı okur
    fn restore_index(&mut self) -> std::io::Result<()> {
        let mut file = File::open(&self.storage_path)?;
        let mut pos = 0;
        let file_len = file.metadata()?.len();

        while pos < file_len {
            file.seek(SeekFrom::Start(pos))?;

            // 1. Önce üst verinin (Metadata) uzunluğunu oku (8 byte)
            let mut len_bytes = [0u8; 8];
            if file.read_exact(&mut len_bytes).is_err() {
                break;
            }
            let meta_len = u64::from_le_bytes(len_bytes);

            // 2. Özel Metadata parçasını oku
            let mut meta_buffer = vec![0u8; meta_len as usize];
            file.read_exact(&mut meta_buffer)?;
            let meta = CellMetadata::from_bytes(&meta_buffer);

            // 3. İçerik (Payload) uzunluğunu oku (8 byte)
            let mut content_len_bytes = [0u8; 8];
            file.read_exact(&mut content_len_bytes)?;
            let content_len = u64::from_le_bytes(content_len_bytes);

            // 4. İçerik tam olarak nerede başlıyor onu kaydet
            let content_pos = file.stream_position()?;

            // 5. İndekse kaydet: "Bu ID'li şifreli veri content_pos'ta başlıyor ve content_len byte boyutunda"
            self.index.insert(meta.id, (meta, content_pos, content_len));

            // 6. Şifreli büyük veriyi OKUMADAN KÖRKÜTÜK ATLA (Performans artışı burada asıl!)
            file.seek(SeekFrom::Current(content_len as i64))?;

            // 7. Yeni blok başlangıcı
            pos += 8 + meta_len + 8 + content_len;
        }
        println!("CortexCore: {} adet kayit indexlendi.", self.index.len());
        Ok(())
    }

    pub fn save_cell(&mut self, cell: &CortexCell) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.storage_path)?;

        let meta = CellMetadata {
            id: cell.id,
            sensitivity: cell.sensitivity.clone(),
            owner_id: cell.owner_id.clone(),
            created_at: cell.created_at,
            deleted_at: cell.deleted_at,
        };

        let meta_bytes = meta.to_bytes();
        let content_bytes = &cell.content;

        // Dosya formatı: [MetaLen: 8byte] + [Meta] + [ContentLen: 8byte] + [Content]
        file.write_all(&(meta_bytes.len() as u64).to_le_bytes())?;
        file.write_all(&meta_bytes)?;

        file.write_all(&(content_bytes.len() as u64).to_le_bytes())?;
        
        let content_pos = file.stream_position()?;
        file.write_all(content_bytes)?;

        // Bellekteki indeksi güncelle
        self.index.insert(cell.id, (meta, content_pos, content_bytes.len() as u64));
        Ok(())
    }

    pub fn get_cell(&self, id: u64) -> Option<CortexCell> {
        let (meta, pos, content_len) = self.index.get(&id)?;
        let mut file = File::open(&self.storage_path).ok()?;

        file.seek(SeekFrom::Start(*pos)).ok()?;

        let mut buffer = vec![0u8; *content_len as usize];
        file.read_exact(&mut buffer).ok()?;

        Some(CortexCell {
            id: meta.id,
            sensitivity: meta.sensitivity.clone(),
            owner_id: meta.owner_id.clone(),
            content: buffer,
            created_at: meta.created_at,
            deleted_at: meta.deleted_at,
        })
    }
}

pub fn create_secure_cell(
    vault: &MemoryVault,
    content: &str,
    level: SensitivityLevel,
    owner: &str,
) -> CortexCell {
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();

    CortexCell {
        id: rand::random::<u64>(),
        content: vault.encrypt(content.as_bytes()),
        sensitivity: level,
        owner_id: owner.to_string(),
        created_at: now_ts,
        deleted_at: None,
    }
}

use std::ffi::{CStr, CString}; 
use std::os::raw::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn create_cortex_engine(path: *const c_char) -> *mut CortexEngine {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = c_str.to_str().unwrap();
    let engine = Box::new(CortexEngine::new(path_str));
    Box::into_raw(engine)
}

#[unsafe(no_mangle)]
pub extern "C" fn save_data(
    engine_ptr: *mut CortexEngine,
    vault_ptr: *mut MemoryVault,
    content: *const c_char,
    level: u8,
    owner: *const c_char,
) -> u64 {
    let engine = unsafe { &mut *engine_ptr };
    let vault = unsafe { &*vault_ptr };
    let c_content = unsafe { CStr::from_ptr(content) }.to_str().unwrap();
    let c_owner = unsafe { CStr::from_ptr(owner) }.to_str().unwrap();

    let s_level = match level {
        1 => SensitivityLevel::Level1,
        2 => SensitivityLevel::Level2,
        _ => SensitivityLevel::Level3,
    };

    let cell = create_secure_cell(vault, c_content, s_level, c_owner);
    let id = cell.id;
    engine.save_cell(&cell).unwrap();
    id
}

#[unsafe(no_mangle)]
pub extern "C" fn create_vault(key_ptr: *const u8) -> *mut MemoryVault {
    let mut key = [0u8; 32];
    unsafe {
        std::ptr::copy_nonoverlapping(key_ptr, key.as_mut_ptr(), 32);
    }
    let vault = Box::new(MemoryVault::new(key));
    Box::into_raw(vault)
}

// YENİ EKLENDİ - SADECE ÜST VERİ LİSTESİ DÖNER (ŞİFRE ÇÖZÜŞÜ YAPMAZ)
#[unsafe(no_mangle)]
pub extern "C" fn get_all_metadata_json(engine_ptr: *mut CortexEngine) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    
    let mut results = Vec::new();

    for (id, (meta, _, _)) in &engine.index {
        if meta.deleted_at.is_none() {
            results.push(serde_json::json!({
                "id": id.to_string(),
                "owner": meta.owner_id, 
                "sensitivity": format!("{:?}", meta.sensitivity)
            }));
        }
    }
    
    let json_str = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
    CString::new(json_str).unwrap().into_raw()
}

// ESKİ FONKSİYONU GÜNCELLEDİK - GERİYE DÖNÜK GO ORCHESTRATOR BOZULMASIN DİYE EKLENDİ 
// (BÜTÜN DATABASE'İN ŞİFRESİNİ DECRYPT EDER, GEÇİŞ AŞAMASINDA ÇALIŞABİLMESİ İÇİN)
#[unsafe(no_mangle)]
pub extern "C" fn get_all_data_json(engine_ptr: *mut CortexEngine, vault_ptr: *mut MemoryVault) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let vault = unsafe { &*vault_ptr };
    
    let mut results = Vec::new();

    for &id in engine.index.keys() {
        if let Some(cell) = engine.get_cell(id) {
            if cell.deleted_at.is_none() {
                let decrypted_bytes = vault.decrypt(&cell.content);
                results.push(serde_json::json!({
                    "id": cell.id.to_string(),
                    "content": String::from_utf8_lossy(&decrypted_bytes).to_string(),
                    "owner": cell.owner_id, 
                    "sensitivity": format!("{:?}", cell.sensitivity)
                }));
            }
        }
    }
    
    let json_str = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
    CString::new(json_str).unwrap().into_raw()
}


// YENİ EKLENDİ - SADECE İSTENEN ID'NİN DETAYINI VE İÇERİĞİNİ ŞİFRESİNİ ÇÖZÜP DÖNDÜRÜR
#[unsafe(no_mangle)]
pub extern "C" fn get_data_by_id_json(
    engine_ptr: *mut CortexEngine, 
    vault_ptr: *mut MemoryVault, 
    id: u64
) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let vault = unsafe { &*vault_ptr };
    
    if let Some(cell) = engine.get_cell(id) {
        let decrypted_bytes = vault.decrypt(&cell.content);
        let json_str = serde_json::to_string(&serde_json::json!({
            "id": cell.id.to_string(),
            "content": String::from_utf8_lossy(&decrypted_bytes).to_string(),
            "owner": cell.owner_id,
            "sensitivity": format!("{:?}", cell.sensitivity)
        })).unwrap_or_else(|_| "{}".to_string());
        
        return CString::new(json_str).unwrap().into_raw();
    }
    
	    CString::new("{}").unwrap().into_raw()
}


// YENİ EKLENDİ - IN-MEMORY RAG SEARCH: Tüm vault'u ramde çözer, kelime arar, top 20'yi döner
#[unsafe(no_mangle)]
pub extern "C" fn search_vault(
    engine_ptr: *mut CortexEngine, 
    vault_ptr: *mut MemoryVault, 
    query_ptr: *const c_char,
    start_ts: u64, // 🧠 Yeni: Başlangıç Zamanı
    end_ts: u64    // 🧠 Yeni: Bitiş Zamanı
) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let vault = unsafe { &*vault_ptr };
    
    let c_query = unsafe { CStr::from_ptr(query_ptr) }.to_str().unwrap_or("");
    let query_lower = c_query.to_lowercase();
    let keywords: Vec<&str> = query_lower.split_whitespace().collect();
    
    if keywords.is_empty() {
        return CString::new("[]").unwrap().into_raw();
    }

    let mut scored_results = Vec::new();

    for (&id, (meta, _, _)) in &engine.index {
        if meta.deleted_at.is_some() {
            continue;
        }

        // 🧠 Tarih Filtresi
        if start_ts > 0 && meta.created_at < start_ts {
            continue;
        }
        if end_ts > 0 && meta.created_at > end_ts {
            continue;
        }

        if let Some(cell) = engine.get_cell(id) {
            let decrypted_bytes = vault.decrypt(&cell.content);
            let content_str = String::from_utf8_lossy(&decrypted_bytes).to_string();
            let content_lower = content_str.to_lowercase();
            
            let mut score = 0;
            for &kw in &keywords {
                let char_count = kw.chars().count();
                if char_count > 2 {
                    let prefix_len = std::cmp::min(5, char_count);
                    let prefix: String = kw.chars().take(prefix_len).collect();
                    if content_lower.contains(&prefix) {
                        score += 1;
                    }
                }
            }
            if score > 0 {
                scored_results.push((score, cell.id, content_str, cell.owner_id));
            }
        }
    }
    scored_results.sort_by(|a, b| b.0.cmp(&a.0));
    let mut final_results = Vec::new();
    // Top 20 kaydı al ve JSON'a çevir
    for (score, id, content, owner) in scored_results.into_iter().take(20) {
        final_results.push(serde_json::json!({
            "id": id.to_string(),
            "content": content,
            "owner": owner,
            "score": score
        }));
    }
    
    let json_str = serde_json::to_string(&final_results).unwrap_or_else(|_| "[]".to_string());
    CString::new(json_str).unwrap().into_raw()
}


// ÇÖP KUTUSU ÖZELLİKLERİ YENİ EKLENDİ

#[unsafe(no_mangle)]
pub extern "C" fn get_trash_bin_json(engine_ptr: *mut CortexEngine) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let mut results = Vec::new();
    
    for (id, (meta, _, _)) in &engine.index {
        if let Some(del_time) = meta.deleted_at {
            results.push(serde_json::json!({
                "id": id.to_string(),
                "owner": meta.owner_id, 
                "sensitivity": format!("{:?}", meta.sensitivity),
                "deleted_at": del_time
            }));
        }
    }
    
    let json_str = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
    CString::new(json_str).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn soft_delete_cell(engine_ptr: *mut CortexEngine, id: u64) -> bool {
    let engine = unsafe { &mut *engine_ptr };
    if let Some(mut cell) = engine.get_cell(id) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        cell.deleted_at = Some(now); // soft delete timestamp
        if engine.save_cell(&cell).is_ok() {
            return true;
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn restore_cell(engine_ptr: *mut CortexEngine, id: u64) -> bool {
    let engine = unsafe { &mut *engine_ptr };
    if let Some(mut cell) = engine.get_cell(id) {
        cell.deleted_at = None; // restore
        if engine.save_cell(&cell).is_ok() {
            return true;
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn trigger_garbage_collector(engine_ptr: *mut CortexEngine, retention_seconds: u64) -> usize {
    let engine = unsafe { &mut *engine_ptr };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    
    let mut valid_cells = Vec::new();
    let mut deleted_count = 0;
    
    // Geçerli kayıtları topla
    for &id in engine.index.keys() {
        if let Some(cell) = engine.get_cell(id) {
            if let Some(del_time) = cell.deleted_at {
                if now.saturating_sub(del_time) >= retention_seconds {
                    deleted_count += 1;
                    continue; // 30 Günü doldurmuşsa yeni db'ye yazma (Fiziksel İmha)
                }
            }
            valid_cells.push(cell);
        }
    }
    
    if deleted_count > 0 {
        // Diski fiziksel olarak temizle ve yeni yapıyı kaydet
        if std::fs::write(&engine.storage_path, b"").is_ok() {
            engine.index.clear();
            for cell in valid_cells {
                let _ = engine.save_cell(&cell);
            }
        }
    }
    
    deleted_count
}

#[unsafe(no_mangle)]
pub extern "C" fn free_cortex_string(ptr: *mut c_char) {
    if ptr.is_null() { return; }
    unsafe {
        let _ = CString::from_raw(ptr);
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn add_sentence_neural(
    engine_ptr: *mut CortexEngine,
    sentence_id: u64, // 🧠 Cümle ID'si eklendi
    sentence_ptr: *const c_char,
    category_ptr: *const c_char,
) {
    let engine = unsafe { &mut *engine_ptr };
    let c_sentence = unsafe { CStr::from_ptr(sentence_ptr) }.to_str().unwrap_or("");
    let c_category = unsafe { CStr::from_ptr(category_ptr) }.to_str().unwrap_or("genel");
    
    engine.neural_graph.add_sentence(sentence_id, c_sentence, c_category);
    
    // Değişikliği diske kaydet
    let graph_path = format!("{}.graph", engine.storage_path);
    let _ = engine.neural_graph.save_to_file(&graph_path);
}

#[unsafe(no_mangle)]
pub extern "C" fn search_neural_vault(
    engine_ptr: *mut CortexEngine,
    query_ptr: *const c_char,
    category_ptr: *const c_char,
) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let c_query = unsafe { CStr::from_ptr(query_ptr) }.to_str().unwrap_or("");
    let c_category = unsafe { CStr::from_ptr(category_ptr) }.to_str().unwrap_or("genel");
    
    let results = engine.neural_graph.search(c_query, c_category);
    let json_str = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
    CString::new(json_str).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_sentence_synapses_json(
    engine_ptr: *mut CortexEngine,
    sentence_ptr: *const c_char,
) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let c_sentence = unsafe { CStr::from_ptr(sentence_ptr) }.to_str().unwrap_or("");
    let words = tokenize(c_sentence);
    
    let mut results: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

    for word in &words {
        let clean_word = word.to_lowercase();
        if let Some(&id) = engine.neural_graph.word_to_id.get(&clean_word) {
            if let Some(edges) = engine.neural_graph.synapses.get(&id) {
                let mut edge_list = Vec::new();
                for edge in edges {
                    if let Some(target_word) = engine.neural_graph.id_to_word.get(&edge.target_id) {
                        edge_list.push(serde_json::json!({
                            "source_id": id,
                            "target_id": edge.target_id,
                            "target": target_word,
                            "category": edge.category
                        }));
                    }
                }
                if !edge_list.is_empty() {
                    results.insert(clean_word, edge_list);
                }
            }
        }
    }

    let json_str = serde_json::to_string(&results).unwrap_or_else(|_| "{}".to_string());
    CString::new(json_str).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_sentence_synapses_by_id_json(
    engine_ptr: *mut CortexEngine,
    sentence_id: u64,
) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let mut primes = Vec::new();
    
    if let Some(list) = engine.neural_graph.sentence_synapses.get(&sentence_id) {
        primes = list.clone();
    }
    
    let json_str = serde_json::to_string(&primes).unwrap_or_else(|_| "[]".to_string());
    CString::new(json_str).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_all_neurons_json(engine_ptr: *mut CortexEngine) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let mut results = Vec::new();

    for (id, word) in &engine.neural_graph.id_to_word {
        results.push(serde_json::json!({
            "id": id,
            "word": word
        }));
    }

    // ID'ye göre sırala
    results.sort_by(|a, b| {
        let a_id = a["id"].as_u64().unwrap_or(0);
        let b_id = b["id"].as_u64().unwrap_or(0);
        a_id.cmp(&b_id)
    });

    let json_str = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
    CString::new(json_str).unwrap().into_raw()
}

#[cfg(test)]
mod cortex_tests {
    use super::*;

    #[test]
    fn test_engine_with_index() {
        let path = "engine_test.cortex";
        // Clean up before test
        std::fs::remove_file(path).ok();
        
        let mut engine = CortexEngine::new(path);
        let vault = MemoryVault::new([7u8; 32]);

        let cell = create_secure_cell(&vault, "Özel Bilgi", SensitivityLevel::Level3, "Sezgin");
        let cell_id = cell.id;

        engine.save_cell(&cell).expect("Kayıt hatası");

        let loaded = engine
            .get_cell(cell_id)
            .expect("İndeks hatası: Kayıt bulunamadı");
        let decrypted = vault.decrypt(&loaded.content);

        assert_eq!("Özel Bilgi", String::from_utf8(decrypted).unwrap());
        
        std::fs::remove_file(path).ok();
    }
}
