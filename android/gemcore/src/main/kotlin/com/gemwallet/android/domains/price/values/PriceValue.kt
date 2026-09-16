package com.gemwallet.android.domains.price.values

import androidx.compose.runtime.Immutable
import uniffi.gemstone.GemValueTone
import com.wallet.core.primitives.Currency

@Immutable
data class PriceValue(
    override val currency: Currency,
    override val value: Double?,
    override val changePercentage: Double?,
    override val valueFormatted: String,
    override val changePercentageFormatted: String,
    override val state: GemValueTone,
) : EquivalentValue
