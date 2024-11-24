package dev.eagely.voice.service

import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.registerKotlinModule
import dev.eagely.voice.config.WeatherConfig
import dev.eagely.voice.model.Weather
import org.springframework.stereotype.Service
import org.springframework.web.reactive.function.client.WebClient
import reactor.core.publisher.Mono

@Service
class WeatherService(
    private val weatherConfig: WeatherConfig,
) {
    private val apiKey = System.getenv("OPENWEATHER_API_KEY")
    private val webClient = WebClient.create(weatherConfig.baseUrl)

    fun getForecast(lat: Double, lon: Double): Mono<Weather> {
        return webClient.get().uri {
                it.queryParam("appid", apiKey).queryParam("lat", lat).queryParam("lon", lon).build()
            }.retrieve().bodyToMono(String::class.java).map {
                val mapper = ObjectMapper().registerKotlinModule()
                val cur = mapper.readTree(it).get("current")
                Weather(
                    cur.get("temp").asDouble() - 273.15,
                    cur.get("weather").get(0).get("description").asText(),
                    cur.get("humidity").asText()
                )
            }
    }
}