use aes_gcm::{
    AeadCore, Aes256Gcm, Key, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use zeroize::Zeroize;

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

// Veritabanı Motoru: Storage + Index
pub struct CortexEngine {
    storage_path: String,
    // ID -> (Metadata, Content Payload Offset, Payload Length)
    index: HashMap<u64, (CellMetadata, u64, u64)>, 
}

impl CortexEngine {
    pub fn new(storage_path: &str) -> Self {
        let mut engine = CortexEngine {
            storage_path: storage_path.to_string(),
            index: HashMap::new(),
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
    CortexCell {
        id: rand::random::<u64>(),
        content: vault.encrypt(content.as_bytes()),
        sensitivity: level,
        owner_id: owner.to_string(),
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


// YENİ EKLENDİ - IN-MEMORY RAG SEARCH: Tüm vault'u ramde çözer, kelime arar, top 5'i döner
#[unsafe(no_mangle)]
pub extern "C" fn search_vault(
    engine_ptr: *mut CortexEngine, 
    vault_ptr: *mut MemoryVault, 
    query_ptr: *const c_char
) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let vault = unsafe { &*vault_ptr };
    
    let c_query = unsafe { CStr::from_ptr(query_ptr) }.to_str().unwrap_or("");
    let query_lower = c_query.to_lowercase();
    let keywords: Vec<&str> = query_lower.split_whitespace().collect();
    
    // Eğer çok kısa veya boş bir soruysa tüm verileri dönmeyelim, boş dönelim.
    if keywords.is_empty() {
        return CString::new("[]").unwrap().into_raw();
    }

    let mut scored_results = Vec::new();

    for (&id, (meta, _, _)) in &engine.index {
        if meta.deleted_at.is_some() {
            continue;
        }
        if let Some(cell) = engine.get_cell(id) {
            let decrypted_bytes = vault.decrypt(&cell.content);
            let content_str = String::from_utf8_lossy(&decrypted_bytes).to_string();
            let content_lower = content_str.to_lowercase();
            
            let mut score = 0;
            for &kw in &keywords {
                let char_count = kw.chars().count();
                // 2 harften büyükleri sorgula, Türkçe ekleri(sondan eklemeli) atlatmak için ilk 5 harfi kök say
                if char_count > 2 {
                    let prefix_len = std::cmp::min(5, char_count);
                    let prefix: String = kw.chars().take(prefix_len).collect();
                    if content_lower.contains(&prefix) {
                        score += 1;
                    }
                }
            }
            
            // Eğer en az 1 kelime eşleştiyse, listeye ekle
            if score > 0 {
                scored_results.push((score, cell.id, content_str, cell.owner_id));
            }
        }
    }
    
    // Yüksek skordan düşüğe sırala
    scored_results.sort_by(|a, b| b.0.cmp(&a.0));
    
    let mut final_results = Vec::new();
    // Top 5 kaydı al ve JSON'a çevir
    for (score, id, content, owner) in scored_results.into_iter().take(5) {
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
