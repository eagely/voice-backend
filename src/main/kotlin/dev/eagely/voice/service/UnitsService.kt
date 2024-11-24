package dev.eagely.voice.service

import dev.eagely.voice.config.UnitsConfig
import org.springframework.stereotype.Service

@Service
class UnitsService(
    private val unitsConfig: UnitsConfig,
) {
    fun getUnitsInfo() = if (unitsConfig.isMetric) "Using metric units" else "Using imperial units"
    fun getTemperature(celsius: Double) = if (unitsConfig.isMetric) celsius else celsius * 9 / 5 + 32
    fun getTemperatureUnits() = if (unitsConfig.isMetric) "°C" else "°F"
}