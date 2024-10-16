package dev.eagely.voice.service

import org.springframework.stereotype.Service
import reactor.core.publisher.Mono

@Service
class ParsingService(
    private val weatherService: WeatherService,
    private val geocodingService: GeocodingService,
    private val openAIService: OpenAIService
) {
    fun parse(input: String): Mono<String> {
        return when {
            "weather" in input -> {
                geocodingService.getLocation(input.replace(" in ", "").replace("weather", "")).flatMap {
                    (town, lat, lon) -> weatherService.getForecast(lat, lon).map {
                        "The temperature in $town is ${String.format("%.1f", it.temperature)} degrees Celsius with ${it.sky} and a humidity of ${it.humidity} percent."
                    }
                }
            }
            else -> openAIService.getCompletion(input).map { it.choices.first().message.content!! }
        }
    }
}