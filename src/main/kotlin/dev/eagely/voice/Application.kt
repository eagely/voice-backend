package dev.eagely.voice

import dev.eagely.voice.config.GeocodingConfig
import dev.eagely.voice.config.UnitsConfig
import dev.eagely.voice.config.WeatherConfig
import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.context.properties.EnableConfigurationProperties
import org.springframework.boot.runApplication

@SpringBootApplication
@EnableConfigurationProperties(
    WeatherConfig::class,
    GeocodingConfig::class,
    UnitsConfig::class,
)
class Application

fun main(args: Array<String>) {
    runApplication<Application>(*args)
}