package dev.eagely.voice.service

import org.springframework.stereotype.Service
import reactor.core.publisher.Mono

@Service
class ParsingService(
    private val geocodingService: GeocodingService,
    private val openAIService: OpenAIService,
    private val unitsService: UnitsService,
    private val weatherService: WeatherService,
) {
    fun parse(input: String): Mono<String> {
        return when {
            "units" in input -> Mono.just(unitsService.getUnitsInfo())
            "weather" in input -> {
                geocodingService.getLocation(input.replace(" in ", "").replace("weather", ""))
                    .flatMap { (town, lat, lon) ->
                        weatherService.getForecast(lat, lon).map {
                            "The temperature in $town is ${
                                String.format(
                                    "%.1f",
                                    unitsService.getTemperature(it.temperature)
                                )
                            } degrees ${unitsService.getTemperatureUnits()} with ${it.sky} and a humidity of ${it.humidity} percent."
                        }
                    }
            }

            else -> openAIService.getCompletion(input).map { it.choices.first().message.content!! }
        }
    }
}