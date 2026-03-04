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
pub struct CortexCell {
    pub id: u64,
    pub content: Vec<u8>,
    pub sensitivity: SensitivityLevel,
    pub owner_id: String,
}

// Hata aldığın to_bytes ve from_bytes metodlarını buraya topladık
impl CortexCell {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Serileştirme hatası")
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        // bincode::deserialize(bytes).expect("De-serileştirme hatası")
        // bincode::deserialize(bytes).unwrap()
        // bincode kullanarak byte dizisinden struct oluştur
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
    index: HashMap<u64, u64>, // ID -> File Position
}

impl CortexEngine {
    // pub fn new(path: &str) -> Self {
    //     Self {
    //         storage_path: path.to_string(),
    //         index: HashMap::new(),
    //     }
    // }
    pub fn new(storage_path: &str) -> Self {
        let mut engine = CortexEngine {
            storage_path: storage_path.to_string(),
            index: HashMap::new(),
        };

        // Eğer dosya zaten varsa, indeksi geri yükle (Persistence)
        if std::path::Path::new(storage_path).exists() {
            if let Err(e) = engine.restore_index() {
                eprintln!("Indeks yukleme hatasi: {}", e);
            }
        }

        engine
    }

    // Dosyayı baştan sona tarayıp HashMap'i dolduran gizli kahraman
    fn restore_index(&mut self) -> std::io::Result<()> {
        let mut file = File::open(&self.storage_path)?;
        let mut pos = 0;
        let file_len = file.metadata()?.len();

        while pos < file_len {
            file.seek(SeekFrom::Start(pos))?;

            // 1. Önce verinin uzunluğunu oku (8 byte)
            let mut len_bytes = [0u8; 8];
            if file.read_exact(&mut len_bytes).is_err() {
                break;
            }
            let len = u64::from_le_bytes(len_bytes);

            // 2. Veriyi oku (CortexCell'i deserializer etmek için)
            let mut buffer = vec![0u8; len as usize];
            file.read_exact(&mut buffer)?;

            // 3. Verinin içinden ID'yi çıkar (Şifreli olsa bile ID açıkta olmalı veya meta veride durmalı)
            let cell = CortexCell::from_bytes(&buffer);

            // 4. İndekse kaydet: "Bu ID'li veri dosyanın 'pos' noktasında başlıyor"
            self.index.insert(cell.id, pos);

            // 5. Bir sonraki verinin başlangıcına zıpla (8 byte uzunluk bilgisi + verinin kendisi)
            pos += 8 + len;
        }
        println!("CortexCore: {} adet kayit indexlendi.", self.index.len());
        Ok(())
    }

    pub fn save_cell(&mut self, cell: &CortexCell) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.storage_path)?;

        let bytes = cell.to_bytes();
        let pos = file.metadata()?.len();

        file.write_all(&(bytes.len() as u64).to_le_bytes())?;
        file.write_all(&bytes)?;

        // Bellekteki indeksi güncelle
        self.index.insert(cell.id, pos);
        Ok(())
    }

    pub fn get_cell(&self, id: u64) -> Option<CortexCell> {
        let pos = self.index.get(&id)?;
        let mut file = File::open(&self.storage_path).ok()?;

        file.seek(SeekFrom::Start(*pos)).ok()?;

        let mut len_bytes = [0u8; 8];
        file.read_exact(&mut len_bytes).ok()?;
        let len = u64::from_le_bytes(len_bytes);

        let mut buffer = vec![0u8; len as usize];
        file.read_exact(&mut buffer).ok()?;

        Some(CortexCell::from_bytes(&buffer))
    }

    pub fn get_all_cells(&self) -> Vec<CortexCell> {
        let mut cells = Vec::new();
        for &id in self.index.keys() {
            if let Some(cell) = self.get_cell(id) {
                cells.push(cell);
            }
        }
        cells
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

#[unsafe(no_mangle)]
pub extern "C" fn get_all_data_json(engine_ptr: *mut CortexEngine, vault_ptr: *mut MemoryVault) -> *mut c_char {
    let engine = unsafe { &*engine_ptr };
    let vault = unsafe { &*vault_ptr };
    
    let mut results = Vec::new();

    for &id in engine.index.keys() {
        if let Some(cell) = engine.get_cell(id) {
            // Hata Çözümü: vault.decrypt doğrudan Vec<u8> döndüğü için let ile alıyoruz
            let decrypted_bytes = vault.decrypt(&cell.content);
            
            results.push(serde_json::json!({
                "id": cell.id,
                "content": String::from_utf8_lossy(&decrypted_bytes).to_string(),
                "owner": cell.owner_id, 
                "sensitivity": format!("{:?}", cell.sensitivity)
            }));
        }
    }
    
    let json_str = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
    CString::new(json_str).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn free_cortex_string(ptr: *mut c_char) {
    if ptr.is_null() { return; }
    unsafe {
        // Pointer'ı tekrar CString'e çevirip kapsam dışına (drop) çıkarıyoruz
        let _ = CString::from_raw(ptr);
    };
}

#[cfg(test)]
mod cortex_tests {
    use super::*;

    #[test]
    fn test_engine_with_index() {
        let path = "engine_test.cortex";
        let mut engine = CortexEngine::new(path);
        let vault = MemoryVault::new([7u8; 32]);

        let cell = create_secure_cell(&vault, "Özel Bilgi", SensitivityLevel::Level3, "Sezgin");
        let cell_id = cell.id;

        engine.save_cell(&cell).expect("Kayıt hatası");

        // İndeks üzerinden anında bulma
        let loaded = engine
            .get_cell(cell_id)
            .expect("İndeks hatası: Kayıt bulunamadı");
        let decrypted = vault.decrypt(&loaded.content);

        assert_eq!("Özel Bilgi", String::from_utf8(decrypted).unwrap());
        std::fs::remove_file(path).ok();
    }
}
