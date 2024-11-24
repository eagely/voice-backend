package dev.eagely.voice.config

import org.springframework.boot.context.properties.ConfigurationProperties

@ConfigurationProperties(prefix = "app.geocoding")
data class GeocodingConfig(
    val baseUrl: String,
)