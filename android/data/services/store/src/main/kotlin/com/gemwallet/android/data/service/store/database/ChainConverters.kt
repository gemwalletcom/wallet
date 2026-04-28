package com.gemwallet.android.data.service.store.database

import androidx.room.TypeConverter
import com.wallet.core.primitives.Chain

class ChainConverters {
    @TypeConverter
    fun fromChain(value: Chain): String = value.string

    @TypeConverter
    fun toChain(value: String): Chain {
        return Chain.entries.firstOrNull { it.string == value }
            ?: throw IllegalArgumentException("Unknown chain: $value")
    }
}
