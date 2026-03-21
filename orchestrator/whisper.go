package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"os"
)

// Groq veya OpenAI uyumlu response yapısı
type WhisperResponse struct {
	Text string `json:"text"`
}

func TranscribeAudio(audioData []byte) (string, error) {
	// 1. Geçici bir ses dosyası oluştur (Whisper genelde dosya bekler)
	tempFile, err := os.CreateTemp("", "cortex-audio-*.ogg")
	if err != nil {
		return "", err
	}
	defer os.Remove(tempFile.Name()) // İşlem bitince sil
	tempFile.Write(audioData)
	tempFile.Close()

	// 2. Multipart form verisi hazırla
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)

	part, err := writer.CreateFormFile("file", tempFile.Name())
	if err != nil {
		return "", err
	}

	fileHandle, _ := os.Open(tempFile.Name())
	io.Copy(part, fileHandle)

	// Model parametresini ekle
	writer.WriteField("model", "whisper-large-v3")
	writer.Close()
	
	apiKey := os.Getenv("GROQ_API_KEY")
	if apiKey == "" {
		return "", fmt.Errorf("Sistem Hatası: GROQ_API_KEY bulunamadı. Lütfen export GROQ_API_KEY komutunu girip 'air' sunucusunu YENİDEN başlatın")
	}
	
	// 3. API İsteği (Groq örneği)
	req, err := http.NewRequest("POST", "https://api.groq.com/openai/v1/audio/transcriptions", body)
	if err != nil {
		return "", err
	}

	// BURAYA KENDİ API KEY'İNİ EKLEMELİSİN
	req.Header.Set("Authorization", "Bearer "+apiKey)
	req.Header.Set("Content-Type", writer.FormDataContentType())

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("Groq API Hatası (HTTP %d): %s", resp.StatusCode, string(bodyBytes))
	}

	// 4. Yanıtı oku
	var result WhisperResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	return result.Text, nil
}
