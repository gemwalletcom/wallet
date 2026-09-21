package com.gemwallet.android.testkit

import com.gemwallet.android.domains.price.values.EquivalentValue
import com.wallet.core.primitives.Currency

fun mockEquivalentValue(value: Double? = 1000.0, changePercentage: Double? = 0.0, currency: Currency = Currency.USD): EquivalentValue = object : EquivalentValue {
    override val currency: Currency = currency
    override val value: Double? = value
    override val changePercentage: Double? = changePercentage
}
