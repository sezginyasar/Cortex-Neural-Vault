package main

/*
#cgo LDFLAGS: -L./ -lcortex_core
#include <stdlib.h>
#include <stdbool.h>

typedef void* CortexEngine;
typedef void* MemoryVault;

// Mevcut olanlar
extern CortexEngine create_cortex_engine(const char* path);
extern MemoryVault create_vault(const unsigned char* key);
extern unsigned long long save_data(CortexEngine eng, MemoryVault vault, const char* content, unsigned char level, const char* owner);

// YENİ EKLENENLER (Hatanın çözümü burası)
extern char* get_all_data_json(CortexEngine eng, MemoryVault vault);
extern char* get_all_metadata_json(CortexEngine eng);
extern char* get_data_by_id_json(CortexEngine eng, MemoryVault vault, unsigned long long id);
extern char* search_vault(CortexEngine eng, MemoryVault vault, const char* query, unsigned long long start_ts, unsigned long long end_ts);
extern char* get_trash_bin_json(CortexEngine eng);
extern bool soft_delete_cell(CortexEngine eng, unsigned long long id);
extern bool restore_cell(CortexEngine eng, unsigned long long id);
extern unsigned long long trigger_garbage_collector(CortexEngine eng, unsigned long long retention_seconds);
extern void free_cortex_string(char* ptr);

// 🧠 Nöron Grafı FFI Köprüleri
extern void add_sentence_neural(CortexEngine eng, unsigned long long sentence_id, const char* sentence, const char* category);
extern char* search_neural_vault(CortexEngine eng, const char* query, const char* category);
extern char* get_sentence_synapses_json(CortexEngine eng, const char* sentence);
extern char* get_all_neurons_json(CortexEngine eng);
extern char* get_sentence_synapses_by_id_json(CortexEngine eng, unsigned long long sentence_id);
*/
import "C"
import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strconv"
	"strings" // ADD THIS
	"time"
	"unsafe"

	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true }, // Vue 3'ten erişim için
}

// Rust motoru ve vault global (veya bir struct içinde) tutulmalı
var engine C.CortexEngine
var vault C.MemoryVault

func handleConnections(w http.ResponseWriter, r *http.Request) {
	ws, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		return
	}
	defer ws.Close()

	var audioBuffer []byte

	for {
		messageType, p, err := ws.ReadMessage()
		if err != nil {
			break
		}

		if messageType == websocket.BinaryMessage {
			audioBuffer = append(audioBuffer, p...)
		} else if messageType == websocket.TextMessage {
			var msg map[string]interface{}
			if err := json.Unmarshal(p, &msg); err != nil {
				continue
			}

			fmt.Printf("Gelen Mesaj: %+v\n", msg)

			msgType, _ := msg["type"].(string)

			// 1. Durum: Sadece hafif üst verileri (Metadataları) getirir. Şifre çözmez.
			if msgType == "GET_METADATA" {
				cJson := C.get_all_metadata_json(engine)
				goJson := C.GoString(cJson)
				C.free_cortex_string(cJson)
				ws.WriteMessage(websocket.TextMessage, []byte(goJson))
				continue
			}

			// 🧠 1.5 Durum: Nöron listesini (Sözlüğü) getirir
			if msgType == "GET_ALL_NEURONS" {
				cJson := C.get_all_neurons_json(engine)
				goJson := C.GoString(cJson)
				C.free_cortex_string(cJson)

				var neurons []map[string]interface{}
				json.Unmarshal([]byte(goJson), &neurons)

				rsp := map[string]interface{}{
					"type": "ALL_NEURONS",
					"data": neurons,
				}
				rspBytes, _ := json.Marshal(rsp)
				ws.WriteMessage(websocket.TextMessage, rspBytes)
				continue
			}

			// 2. Durum: AI sadece belirli bir veriyi (ID) istediğinde çalışır. Şifresini çözer.
			if msgType == "GET_DATA_BY_ID" {
				if idStr, ok := msg["id"].(string); ok {
					id, err := strconv.ParseUint(idStr, 10, 64)
					if err == nil {
						cJson := C.get_data_by_id_json(engine, vault, C.ulonglong(id))
						goJson := C.GoString(cJson)
						C.free_cortex_string(cJson)

						// 🧠 Sinapsları çek (Cümledeki kelimeler için)
						var rawRec map[string]interface{}
						json.Unmarshal([]byte(goJson), &rawRec)
						content, _ := rawRec["content"].(string)

						var synapses map[string]interface{}
						var primes []interface{}
						if content != "" {
							cContent := C.CString(content)
							cSynapseJson := C.get_sentence_synapses_json(engine, cContent)
							goSynapseJson := C.GoString(cSynapseJson)
							C.free_cortex_string(cSynapseJson)
							C.free(unsafe.Pointer(cContent))

							json.Unmarshal([]byte(goSynapseJson), &synapses)

							// 🧠 Asal Sayı Synapses Çek
							cPrimesJson := C.get_sentence_synapses_by_id_json(engine, C.ulonglong(id))
							goPrimesJson := C.GoString(cPrimesJson)
							C.free_cortex_string(cPrimesJson)
							json.Unmarshal([]byte(goPrimesJson), &primes)
						}

						// Zenginleştirilmiş yanıt
						rsp := map[string]interface{}{
							"id":          idStr,
							"content":     content,
							"synapses":    synapses,
							"primes":      primes, // 🧠 Yeni
							"owner":       rawRec["owner"],
							"sensitivity": rawRec["sensitivity"],
						}
						rspBytes, _ := json.Marshal(rsp)
						ws.WriteMessage(websocket.TextMessage, rspBytes)
					} else {
						ws.WriteMessage(websocket.TextMessage, []byte("Hata: Gecersiz ID formati"))
					}
				}
				continue
			}

			// Eski sistem: Tüm verileri getir (Test ve uyumluluk için var)
			if msgType == "GET_ALL" {
				cJson := C.get_all_data_json(engine, vault)
				goJson := C.GoString(cJson)
				C.free_cortex_string(cJson)
				ws.WriteMessage(websocket.TextMessage, []byte(goJson))
				continue // İşlem bitti, aşağıya bakma
			}

			// YENİ EKLENEN: AI_QUERY (Hafıza tabanlı asistan sorusu)
			if msgType == "AI_QUERY" {
				question, ok := msg["content"].(string)
				if !ok {
					ws.WriteMessage(websocket.TextMessage, []byte(`{"type":"ERROR", "text":"Soru eksik!"}`))
					continue
				}

				// 1. Rust'tan sorguya uyan (Top 20) hafızaları bul (In-Memory RAG)
				cQuery := C.CString(question)
				startTs, endTs := parseDateRange(question) // 🧠 Yeni
				cJson := C.search_vault(engine, vault, cQuery, C.ulonglong(startTs), C.ulonglong(endTs))
				goJson := C.GoString(cJson)
				C.free_cortex_string(cJson)

				var memoryList []string
				var sources []string // 🧠 Yeni: Kaynak Citation listesi
				var rawRecords []map[string]interface{}
				if err := json.Unmarshal([]byte(goJson), &rawRecords); err == nil {
					for _, r := range rawRecords {
						if content, ok := r["content"].(string); ok && content != "" {
							memoryList = append(memoryList, content)
							
							idStr := "0"
							if idVal, ok := r["id"]; ok {
								idStr = fmt.Sprintf("%v", idVal)
							}
							sources = append(sources, fmt.Sprintf("#%s: %s", idStr, content))
						}
					}
				}

				// 🧠 2. Sinaps Grafından da arayalım (Nöron Bağlantıları)
				cCategory := C.CString("genel") 
				cNeuralJson := C.search_neural_vault(engine, cQuery, cCategory)
				goNeuralJson := C.GoString(cNeuralJson)
				C.free_cortex_string(cNeuralJson)
				C.free(unsafe.Pointer(cCategory))
				C.free(unsafe.Pointer(cQuery)) // C.CString(question) bitti

				var neuralWords []string
				if err := json.Unmarshal([]byte(goNeuralJson), &neuralWords); err == nil && len(neuralWords) > 0 {
					// Yapay zekaya sinapslardan gelen kelimeleri de ipucu/bağlam olarak besle
					memoryList = append(memoryList, fmt.Sprintf("[Nöron Sinaps İpuçları]: %s", strings.Join(neuralWords, ", ")))
					sources = append(sources, fmt.Sprintf("[Sinaps Grafı İpucu]: %s", strings.Join(neuralWords, ", ")))
				}

				// 2. Groq'a gönder (chat.go)
				answer, err := QueryAI(question, memoryList)
				if err != nil {
					rspBytes, _ := json.Marshal(map[string]string{
						"type": "ERROR",
						"text": err.Error(),
					})
					ws.WriteMessage(websocket.TextMessage, rspBytes)
				} else {
					rspBytes, _ := json.Marshal(map[string]interface{}{
						"type":    "AI_RESPONSE",
						"text":    answer,
						"sources": sources, // 🧠 Yeni
					})
					ws.WriteMessage(websocket.TextMessage, rspBytes)
				}
				continue
			}

			// 2. Durum: Ses kaydı bitti
			if msgType == "STOP_RECORDING" {
				if len(audioBuffer) > 0 {
					text, err := TranscribeAudio(audioBuffer)
					if err != nil {
						ws.WriteMessage(websocket.TextMessage, []byte("AI Hatası: "+err.Error()))
					} else {
						saveToRust(ws, text, "Sezgin (Voice)", 1)
					}
				}
				audioBuffer = nil
				continue // İşlem bitti
			}

			// ÇÖP KUTUSU ÖZELLİKLERİ
			if msgType == "GET_TRASH_BIN" {
				cJson := C.get_trash_bin_json(engine)
				goJson := C.GoString(cJson)
				C.free_cortex_string(cJson)

				var records []interface{}
				json.Unmarshal([]byte(goJson), &records)

				rspBytes, _ := json.Marshal(map[string]interface{}{
					"type": "TRASH_BIN_DATA",
					"data": records,
				})
				ws.WriteMessage(websocket.TextMessage, rspBytes)
				continue
			}

			if msgType == "DELETE_DATA" {
				idStr, ok := msg["id"].(string)
				if !ok {
					if idf, isFloat := msg["id"].(float64); isFloat {
						idStr = strconv.FormatFloat(idf, 'f', -1, 64)
					}
				}
				if id, err := strconv.ParseUint(idStr, 10, 64); err == nil {
					success := bool(C.soft_delete_cell(engine, C.ulonglong(id)))
					if success {
						ws.WriteMessage(websocket.TextMessage, []byte(`{"type":"SYSTEM_INFO", "text":"Kayıt çöp kutusuna taşındı."}`))
					}
				}
				continue
			}

			if msgType == "RESTORE_DATA" {
				idStr, ok := msg["id"].(string)
				if !ok {
					if idf, isFloat := msg["id"].(float64); isFloat {
						idStr = strconv.FormatFloat(idf, 'f', -1, 64)
					}
				}
				if id, err := strconv.ParseUint(idStr, 10, 64); err == nil {
					success := bool(C.restore_cell(engine, C.ulonglong(id)))
					if success {
						ws.WriteMessage(websocket.TextMessage, []byte(`{"type":"SYSTEM_INFO", "text":"Kayıt arşive geri yüklendi!"}`))
					}
				}
				continue
			}

			if msgType == "EMPTY_TRASH" {
				// Hemen hepsini sil (retention: 0 saniye)
				deletedCount := C.trigger_garbage_collector(engine, 0)
				ws.WriteMessage(websocket.TextMessage, []byte(fmt.Sprintf(`{"type":"SYSTEM_INFO", "text":"Çöp kutusu tamamen boşaltıldı! (%d kayıt fiziki olarak imha edildi)"}`, int(deletedCount))))
				continue
			}

			// 3. Durum: Normal metin kaydı (TEXT_DATA)
			// Sadece msgType "TEXT_DATA" ise veya type alanı belirtilmişse buraya girsin
			content, ok1 := msg["content"].(string)
			owner, ok2 := msg["owner"].(string)

			if ok1 && ok2 {
				level := 1.0
				if val, ok := msg["level"].(float64); ok {
					level = val
				}
				saveToRust(ws, content, owner, byte(level))
			} else {
				// Eğer hiçbir tipe uymuyorsa hata dönebilirsin
				ws.WriteMessage(websocket.TextMessage, []byte("Hata: Bilinmeyen veya eksik veri formatı."))
			}
		}
	}
}

// Kod tekrarını önlemek için yardımcı fonksiyon
func saveToRust(ws *websocket.Conn, contentStr, ownerStr string, level byte) {
	// Bağlam Zenginleştirme (Context Enrichment)
	currentTime := time.Now().Format("02.01.2006 Monday 15:04")
	enrichedContent := fmt.Sprintf("[Kayıt Zamanı: %s] %s", currentTime, contentStr)

	cContent := C.CString(enrichedContent)
	cOwner := C.CString(ownerStr)
	defer C.free(unsafe.Pointer(cContent))
	defer C.free(unsafe.Pointer(cOwner))

	id := C.save_data(engine, vault, cContent, C.uchar(level), cOwner)

	// 🧠 Nöron Grafına da ekle
	category := "genel"
	cleanContent := contentStr
	if strings.HasPrefix(contentStr, "iş:") {
		category = "iş"
		cleanContent = strings.TrimSpace(strings.TrimPrefix(contentStr, "iş:"))
	} else if strings.HasPrefix(contentStr, "aile:") {
		category = "aile"
		cleanContent = strings.TrimSpace(strings.TrimPrefix(contentStr, "aile:"))
	} else if strings.HasPrefix(contentStr, "özel:") {
		category = "özel"
		cleanContent = strings.TrimSpace(strings.TrimPrefix(contentStr, "özel:"))
	}

	cCleanContent := C.CString(cleanContent)
	cCategory := C.CString(category)
	defer C.free(unsafe.Pointer(cCleanContent))
	defer C.free(unsafe.Pointer(cCategory))

	C.add_sentence_neural(engine, id, cCleanContent, cCategory)

	// Yeni Chat UI'ın parse edebilmesi için JSON yapısında dön
	respMsg := map[string]string{
		"type": "SYSTEM_INFO",
		"text": fmt.Sprintf("✅ Hafızaya Başarıyla Mühürlendi: '%s'", contentStr),
	}
	respBytes, _ := json.Marshal(respMsg)
	ws.WriteMessage(websocket.TextMessage, respBytes)
}

func parseDateRange(question string) (uint64, uint64) {
	now := time.Now()
	q := strings.ToLower(question)

	if strings.Contains(q, "son 3 gün") {
		return uint64(now.Add(-3 * 24 * time.Hour).Unix()), uint64(now.Unix())
	}
	if strings.Contains(q, "son 1 hafta") || strings.Contains(q, "geçen hafta") {
		return uint64(now.Add(-7 * 24 * time.Hour).Unix()), uint64(now.Unix())
	}
	if strings.Contains(q, "dün") {
		yesterday := now.Add(-24 * time.Hour)
		start := time.Date(yesterday.Year(), yesterday.Month(), yesterday.Day(), 0, 0, 0, 0, now.Location())
		end := time.Date(yesterday.Year(), yesterday.Month(), yesterday.Day(), 23, 59, 59, 0, now.Location())
		return uint64(start.Unix()), uint64(end.Unix())
	}
	if strings.Contains(q, "bugün") {
		start := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, now.Location())
		return uint64(start.Unix()), uint64(now.Unix())
	}
	return 0, 0
}

func main() {
	// 1. Rust Motorunu Hazırla
	dbPath := C.CString("main_db.cognitive")
	engine = C.create_cortex_engine(dbPath)
	C.free(unsafe.Pointer(dbPath))

	key := [32]byte{7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7}
	vault = C.create_vault((*C.uchar)(unsafe.Pointer(&key[0])))

	// 2. HTTP Server & WebSocket
	http.HandleFunc("/ws", handleConnections)
	fmt.Println("CortexCore Orkestratör 8080 portunda dinleniyor...")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
