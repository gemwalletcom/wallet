package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Entity
import androidx.room.PrimaryKey
import com.wallet.core.primitives.Currency

@Entity(tableName = "currency_rates")
data class DbFiatRate(@PrimaryKey val currency: Currency, val rate: Double)
