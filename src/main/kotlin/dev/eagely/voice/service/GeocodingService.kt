package dev.eagely.voice.service

import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.registerKotlinModule
import dev.eagely.voice.model.Geocode
import org.springframework.beans.factory.annotation.Value
import org.springframework.stereotype.Service
import org.springframework.web.reactive.function.client.WebClient
import reactor.core.publisher.Mono

@Service
class GeocodingService(
    @Value("\${app.geocoding.base-url}") private val baseUrl: String
) {
    private val webClient = WebClient.create(baseUrl)
    private val mapper = ObjectMapper().registerKotlinModule()

    fun getLocation(location: String): Mono<Geocode> {
        return webClient
            .get()
            .uri {
                it
                    .queryParam("q", location)
                    .queryParam("format", "json")
                    .queryParam("limit", "1")
                    .build()
            }
            .retrieve()
            .bodyToMono(String::class.java)
            .map {
                println(it)
                val geocode = mapper.readTree(it).get(0)
                Geocode(
                    geocode.get("name").asText(),
                    geocode.get("lat").asDouble(),
                    geocode.get("lon").asDouble()
                )
            }
    }
}