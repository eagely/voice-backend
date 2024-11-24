package dev.eagely.voice.config

import org.springframework.boot.context.properties.ConfigurationProperties

@ConfigurationProperties(prefix = "app.weather")
data class WeatherConfig(
    val baseUrl: String,
)