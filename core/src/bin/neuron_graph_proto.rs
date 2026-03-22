use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Synapse {
    pub target_id: u64,
    pub category: String,
}

pub struct NeuralVault {
    // 🧠 Sözlük (Dictionary): Kelime <-> ID Eşleşmesi
    pub word_to_id: HashMap<String, u64>,
    pub id_to_word: HashMap<u64, String>,
    
    // 🔗 Sinapslar (Graph): Kaynak Nöron -> Hedef Nöronlar Listesi
    pub synapses: HashMap<u64, Vec<Synapse>>,
    pub next_id: u64,
}

impl NeuralVault {
    pub fn new() -> Self {
        NeuralVault {
            word_to_id: HashMap::new(),
            id_to_word: HashMap::new(),
            synapses: HashMap::new(),
            next_id: 1,
        }
    }

    // Kelimeyi nörona çevirir (Yoksa yenisini oluşturur)
    pub fn get_or_create_neuron(&mut self, word: &str) -> u64 {
        let clean_word = word.to_lowercase().trim().to_string();
        if let Some(&id) = self.word_to_id.get(&clean_word) {
            return id;
        }
        let id = self.next_id;
        self.word_to_id.insert(clean_word.clone(), id);
        self.id_to_word.insert(id, clean_word);
        self.next_id += 1;
        id
    }

    // Cümleyi nöron zincirine ve sinaps bağlantılarına çevirir
    pub fn add_sentence(&mut self, sentence: &str, category: &str) {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        let mut prev_id: Option<u64> = None;

        for word in words {
            let id = self.get_or_create_neuron(word);
            
            if let Some(p_id) = prev_id {
                // Sinaps (Bağlantı) Ekle: Önceki Nöron -> Mevcut Nöron
                let edge = Synapse {
                    target_id: id,
                    category: category.to_string(),
                };
                self.synapses.entry(p_id).or_insert(Vec::new()).push(edge);
            }
            prev_id = Some(id);
        }
    }

    // Arama: Kelime ve Kategoriye göre bağlı diğer nöronları bulur
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

    // İndeks İle Geri Dönüşüm (Integrity Test)
    pub fn reconstruct_from_node(&self, start_word: &str, category: &str) -> String {
        let mut sentence = vec![start_word.to_string()];
        let mut current_word = start_word.to_lowercase();

        loop {
            if let Some(&id) = self.word_to_id.get(&current_word) {
                if let Some(edges) = self.synapses.get(&id) {
                    // Kategoriye uyan ilk bağlantıyı takip et (Basit zincirleme)
                    if let Some(next_edge) = edges.iter().find(|e| e.category == category) {
                        if let Some(next_word) = self.id_to_word.get(&next_edge.target_id) {
                            sentence.push(next_word.clone());
                            current_word = next_word.clone();
                            continue;
                        }
                    }
                }
            }
            break; // Daha fazla bağlantı yoksa bitir
        }
        sentence.join(" ")
    }
}

fn main() {
    let mut vault = NeuralVault::new();

    // 🔬 TEST 1: Bilgileri Nöronlara Parçalama & Sinaps Oluşturma
    vault.add_sentence("Rapor sunumu yarın saat 14:00'da", "iş");
    vault.add_sentence("Akşam marketten ekmek ve süt al", "kişisel");
    vault.add_sentence("Haftasonu ailece pikniğe gidilecek", "aile");

    println!("--- 🧠 NEURAL VAULT PROTOTYPE ---");
    println!("Toplam Benzersiz Kelime (Nöron): {}", vault.word_to_id.len());
    
    // 🔬 TEST 2: Kategoriye Göre İleri Arama
    println!("\n🔍 [Arama] 'iş' kategorisinde 'Rapor' nöronuna bağlı olanlar:");
    let res = vault.search("rapor", "iş");
    println!("➡ Sonuç: {:?}", res); // ["sunumu"]

    println!("\n🔍 [Arama] 'kişisel' kategorisinde 'Ekmek' nöronuna bağlı olanlar:");
    let res2 = vault.search("ekmek", "kişisel");
    println!("➡ Sonuç: {:?}", res2); // ["ve"]

    // 🔬 TEST 3: Cümle Analizi ve İleri Yönlü Yol Bulma (Reconstruction)
    println!("\n🧭 [Yol Takibi] 'Rapor' nöronundan başlayıp 'iş' sinapslarını takip et:");
    let full_story = vault.reconstruct_from_node("rapor", "iş");
    println!("➡ Oluşturulan Cümle: \"{}\"", full_story); // "rapor sunumu yarın saat 14:00'da"

    // 🛡️ TEST 4: Güvenlik Gösterimi (Sözlüksüz Veritabanı)
    println!("\n🛡️ [Güvenlik] Sözlük (Dictionary) Olmadan Sinaps Dosyası:");
    println!("(Sadece anlamsız sayılar ve etiketler barındırır)");
    for (src_id, edges) in &vault.synapses {
        for edge in edges {
             println!("  [ID: {}] -({:^8})-> [ID: {}]", src_id, edge.category, edge.target_id);
        }
    }
}
