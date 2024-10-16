package dev.eagely.voice.controller

import dev.eagely.voice.service.ParsingService
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestBody
import org.springframework.web.bind.annotation.RestController
import reactor.core.publisher.Mono

@RestController
class ProcessController(
    private val parsingService: ParsingService,
) {
    @PostMapping("/process")
    fun process(@RequestBody content: String): Mono<ResponseEntity<String>> {
        return parsingService.parse(content).map { ResponseEntity.ok(it) }
    }
}