package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
	"time"
)

type ChatMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type ChatRequest struct {
	Model    string        `json:"model"`
	Messages []ChatMessage `json:"messages"`
}

type ChatResponse struct {
	Choices []struct {
		Message ChatMessage `json:"message"`
	} `json:"choices"`
}

func QueryAI(prompt string, memories []string) (string, error) {
	apiKey := os.Getenv("GROQ_API_KEY")
	if apiKey == "" {
		return "", fmt.Errorf("Sistem Hatası: GROQ_API_KEY bulunamadı. Lütfen terminalde ayarlayıp sunucuyu yeniden başlatın.")
	}

	// Hafızayı (Context) birleştirelim
	contextStr := strings.Join(memories, "\n- ")
	currentSysTime := time.Now().Format("02.01.2006 Monday 15:04")

	systemPrompt := fmt.Sprintf("Sen Cortex Neural Vault'sun. 'Hafıza (Kayıtlar)' bölümünü senin mutlak gerçeklerin kabul et. Kullanıcının sorusuna akıcı, anadilde (Türkçe) net cevap ver. Eğer hafızada karşılığı yoksa, uydurmak yerine 'Üzgünüm, buna dair kaydım yok' de.\n\nÖNEMLİ ZAMAN KURALI: Eğer kullanıcı bir etkinliğin/planın zamanını sorarsa, sadece 'Perşembe' demek yerine o günün net tarihini matematiksel olarak hesapla ve açıkça belirt. Hesaplamayı, BUGÜN'ün tarih gününe ve [Kayıt Zamanı]'na bakarak yap (Örn: 'Sorguladığınız toplantı önümüzdeki Perşembe, 26 Mart 2026 tarihindedir').\n\n[BUGÜNÜN SİSTEM TARİH/SAAT BİLGİSİ: %s]\n\n--- Hafıza (Kayıtlar) ---\n- %s", currentSysTime, contextStr)

	reqData := ChatRequest{
		Model: "llama-3.3-70b-versatile", // Veya "llama-3.1-70b-versatile"
		Messages: []ChatMessage{
			{Role: "system", Content: systemPrompt},
			{Role: "user", Content: prompt},
		},
	}

	bodyBytes, err := json.Marshal(reqData)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequest("POST", "https://api.groq.com/openai/v1/chat/completions", bytes.NewBuffer(bodyBytes))
	if err != nil {
		return "", err
	}

	req.Header.Set("Authorization", "Bearer "+apiKey)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		errBody, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("Groq Chat API Hatası (HTTP %d): %s", resp.StatusCode, string(errBody))
	}

	var result ChatResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", err
	}

	if len(result.Choices) > 0 {
		return result.Choices[0].Message.Content, nil
	}

	return "Model cevap oluşturamadı.", nil
}
