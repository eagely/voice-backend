package dev.eagely.voice.service

import org.springframework.beans.factory.annotation.Value
import org.springframework.http.MediaType
import org.springframework.stereotype.Service
import org.springframework.web.reactive.function.BodyInserters
import org.springframework.web.reactive.function.client.WebClient
import reactor.core.publisher.Mono

@Service
class TranscriptionService(
    @Value("\${app.whisper.base-url}") private val baseUrl: String
) {
    private val webClient = WebClient.create(baseUrl)

    fun getWhisperTranscription(bytes: ByteArray): Mono<String> {
        return webClient.post()
            .uri("/whisper")
            .contentType(MediaType.APPLICATION_OCTET_STREAM)
            .body(BodyInserters.fromValue(bytes))
            .retrieve()
            .bodyToMono(String::class.java)
    }
}