package com.gemwallet.android.data.service.store.database

import androidx.room.TypeConverter
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.NFTAttribute
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString

class StoreConverters {
    private val assetLinksSerializer = ListSerializer(AssetLink.serializer())
    private val nftAttributesSerializer = ListSerializer(NFTAttribute.serializer())

    @TypeConverter
    fun fromAssetLinks(value: List<AssetLink>?): String? {
        return value?.let { jsonEncoder.encodeToString(assetLinksSerializer, it) }
    }

    @TypeConverter
    fun toAssetLinks(value: String?): List<AssetLink>? {
        return value?.let { runCatching { jsonEncoder.decodeFromString(assetLinksSerializer, it) }.getOrDefault(emptyList()) }
    }

    @TypeConverter
    fun fromNftAttributes(value: List<NFTAttribute>?): String? {
        return value?.let { jsonEncoder.encodeToString(nftAttributesSerializer, it) }
    }

    @TypeConverter
    fun toNftAttributes(value: String?): List<NFTAttribute>? {
        return value?.let { runCatching { jsonEncoder.decodeFromString(nftAttributesSerializer, it) }.getOrDefault(emptyList()) }
    }
}
