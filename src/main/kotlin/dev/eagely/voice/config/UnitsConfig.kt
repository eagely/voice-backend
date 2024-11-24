package dev.eagely.voice.config

import org.springframework.boot.context.properties.ConfigurationProperties

@ConfigurationProperties(prefix = "app.units")
data class UnitsConfig(
    val isMetric: Boolean,
)