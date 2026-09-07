package com.gemwallet.android.domains.price.values

import androidx.compose.runtime.Immutable
import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.Currency

@Immutable
data class PriceValue(
    override val currency: Currency,
    override val value: Double?,
    override val changePercentage: Double?,
    override val valueFormatted: String,
    override val changePercentageFormatted: String,
    override val state: ValueDirection,
) : EquivalentValue
