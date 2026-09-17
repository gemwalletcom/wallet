package com.gemwallet.android.ui.components.list_item

import com.gemwallet.android.domains.percentage.formatAsPercentage
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemValidatorRow

data class ValidatorRowUIModel(
    val id: String,
    val name: String,
    val imageUrl: String?,
    val placeholder: String,
    val aprText: String,
)

fun GemValidatorRow.uiModel() = ValidatorRowUIModel(
    id = validator.id,
    name = name,
    imageUrl = imageUrl,
    placeholder = placeholder,
    aprText = validator.apr.formatApr(),
)

fun Double.formatApr(): String = formatAsPercentage(style = GemPercentageStyle.UNSIGNED)
