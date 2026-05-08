package com.gemwallet.android.data.service.store.database

import androidx.room.TypeConverter
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.NFTAttribute
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString

class StoreConverters {
    private val assetLinksSerializer = ListSerializer(AssetLink.serializer())
    private val nftAttributesSerializer = ListSerializer(NFTAttribute.serializer())

    @TypeConverter
    fun fromAssetId(value: AssetId): String = value.toIdentifier()

    @TypeConverter
    fun toAssetId(value: String): AssetId = requireNotNull(value.toAssetId()) {
        "Invalid AssetId in database: $value"
    }

    @TypeConverter
    fun fromTransactionId(value: TransactionId): String = value.identifier

    @TypeConverter
    fun toTransactionId(value: String): TransactionId = requireNotNull(TransactionId.from(value)) {
        "Invalid TransactionId in database: $value"
    }

    @TypeConverter
    fun fromWalletId(value: WalletId): String = value.id

    @TypeConverter
    fun toWalletId(value: String): WalletId = WalletId(value)

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
