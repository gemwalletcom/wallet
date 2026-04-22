package com.gemwallet.android.data.service.store.database

import androidx.room.TypeConverter
import com.gemwallet.android.ext.findByString
import com.wallet.core.primitives.Chain

class NftChainConverters {
    @TypeConverter
    fun fromChain(value: Chain): String = value.string

    @TypeConverter
    fun toChain(value: String): Chain {
        return Chain.findByString(value)
            ?: throw IllegalArgumentException("Unknown chain: $value")
    }
}
