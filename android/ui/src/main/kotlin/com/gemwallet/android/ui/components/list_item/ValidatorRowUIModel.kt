package com.gemwallet.android.ui.components.list_item

import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemValidatorRow
import uniffi.gemstone.formattedPercentage

data class ValidatorRowUIModel(val id: String, val name: String, val imageUrl: String?, val placeholder: String, val apr: GemLocalizedText)

fun GemValidatorRow.uiModel() = ValidatorRowUIModel(
    id = validator.id,
    name = name,
    imageUrl = imageUrl,
    placeholder = placeholder,
    apr = apr,
)

fun Double.aprText(): GemLocalizedText = GemLocalizedText.Apr(takeIf { it > 0.0 }?.let { formattedPercentage(it, GemPercentageStyle.UNSIGNED) })
