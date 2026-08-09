package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Entity
import androidx.room.PrimaryKey
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate

@Entity(tableName = "currency_rates")
data class DbFiatRate(
    @PrimaryKey val currency: Currency,
    val rate: Double,
)

fun DbFiatRate.toDTO(): FiatRate {
    return FiatRate(currency, rate)
}

fun FiatRate.toRecord(): DbFiatRate {
    return DbFiatRate(symbol, rate)
}

fun List<DbFiatRate>.toDTO() = map { it.toDTO() }

fun List<FiatRate>.toRecord() = map { it.toRecord() }
