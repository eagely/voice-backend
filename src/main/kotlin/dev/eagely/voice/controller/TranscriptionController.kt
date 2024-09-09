package dev.eagely.voice.controller

import dev.eagely.voice.service.TranscriptionService
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestBody
import org.springframework.web.bind.annotation.RestController
import reactor.core.publisher.Mono

@RestController
class TranscriptionController(
    private val transcriptionService: TranscriptionService
) {
    @PostMapping("/whisper")
    fun whisper(@RequestBody bytes: ByteArray): Mono<ResponseEntity<String>> {
        return transcriptionService.getWhisperTranscription(bytes).map { ResponseEntity.ok(it) }
    }
}