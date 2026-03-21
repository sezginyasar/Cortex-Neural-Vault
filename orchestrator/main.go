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
extern char* search_vault(CortexEngine eng, MemoryVault vault, const char* query);
extern char* get_trash_bin_json(CortexEngine eng);
extern bool soft_delete_cell(CortexEngine eng, unsigned long long id);
extern bool restore_cell(CortexEngine eng, unsigned long long id);
extern unsigned long long trigger_garbage_collector(CortexEngine eng, unsigned long long retention_seconds);
extern void free_cortex_string(char* ptr);
*/
import "C"
import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strconv"
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

			// 2. Durum: AI sadece belirli bir veriyi (ID) istediğinde çalışır. Şifresini çözer.
			if msgType == "GET_DATA_BY_ID" {
				if idStr, ok := msg["id"].(string); ok {
					id, err := strconv.ParseUint(idStr, 10, 64)
					if err == nil {
						cJson := C.get_data_by_id_json(engine, vault, C.ulonglong(id))
						goJson := C.GoString(cJson)
						C.free_cortex_string(cJson)
						ws.WriteMessage(websocket.TextMessage, []byte(goJson))
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

				// 1. Rust'tan sorguya uyan (Top 5) hafızaları bul (In-Memory RAG)
				cQuery := C.CString(question)
				cJson := C.search_vault(engine, vault, cQuery)
				goJson := C.GoString(cJson)
				C.free_cortex_string(cJson)
				C.free(unsafe.Pointer(cQuery))

				var memoryList []string
				var rawRecords []map[string]interface{}
				if err := json.Unmarshal([]byte(goJson), &rawRecords); err == nil {
					for _, r := range rawRecords {
						if content, ok := r["content"].(string); ok && content != "" {
							memoryList = append(memoryList, content)
						}
					}
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
					rspBytes, _ := json.Marshal(map[string]string{
						"type": "AI_RESPONSE",
						"text": answer,
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

	C.save_data(engine, vault, cContent, C.uchar(level), cOwner)

	// Yeni Chat UI'ın parse edebilmesi için JSON yapısında dön
	respMsg := map[string]string{
		"type": "SYSTEM_INFO",
		"text": fmt.Sprintf("✅ Hafızaya Başarıyla Mühürlendi: '%s'", contentStr),
	}
	respBytes, _ := json.Marshal(respMsg)
	ws.WriteMessage(websocket.TextMessage, respBytes)
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
