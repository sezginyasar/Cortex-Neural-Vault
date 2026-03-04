package main

/*
#cgo LDFLAGS: -L./ -lcortex_core
#include <stdlib.h>

typedef void* CortexEngine;
typedef void* MemoryVault;

// Mevcut olanlar
extern CortexEngine create_cortex_engine(const char* path);
extern MemoryVault create_vault(const unsigned char* key);
extern unsigned long long save_data(CortexEngine eng, MemoryVault vault, const char* content, unsigned char level, const char* owner);

// YENİ EKLENENLER (Hatanın çözümü burası)
extern char* get_all_data_json(CortexEngine eng, MemoryVault vault);
extern void free_cortex_string(char* ptr);
*/
import "C"
import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
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

			// 1. Durum: Tüm verileri getir (En başa alıyoruz)
			if msgType == "GET_ALL" {
				cJson := C.get_all_data_json(engine, vault)
				goJson := C.GoString(cJson)
				C.free_cortex_string(cJson)
				ws.WriteMessage(websocket.TextMessage, []byte(goJson))
				continue // İşlem bitti, aşağıya bakma
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
	cContent := C.CString(contentStr)
	cOwner := C.CString(ownerStr)
	defer C.free(unsafe.Pointer(cContent))
	defer C.free(unsafe.Pointer(cOwner))

	id := C.save_data(engine, vault, cContent, C.uchar(level), cOwner)

	resp := fmt.Sprintf("Kayıt Başarılı! AI Duydu: %s | ID: %d", contentStr, uint64(id))
	ws.WriteMessage(websocket.TextMessage, []byte(resp))
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
