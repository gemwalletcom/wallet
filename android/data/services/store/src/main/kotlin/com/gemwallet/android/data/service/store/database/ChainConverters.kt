package com.gemwallet.android.data.service.store.database

import androidx.room.TypeConverter
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.Chain
import kotlinx.serialization.builtins.ListSerializer

class ChainConverters {
    private val chainsSerializer = ListSerializer(Chain.serializer())

    @TypeConverter
    fun fromChain(value: Chain): String = value.string

    @TypeConverter
    fun toChain(value: String): Chain {
        return Chain.entries.firstOrNull { it.string == value }
            ?: throw IllegalArgumentException("Unknown chain: $value")
    }

    @TypeConverter
    fun fromChains(value: List<Chain>): String = jsonEncoder.encodeToString(chainsSerializer, value)

    @TypeConverter
    fun toChains(value: String): List<Chain> =
        runCatching { jsonEncoder.decodeFromString(chainsSerializer, value) }.getOrDefault(emptyList())
}
