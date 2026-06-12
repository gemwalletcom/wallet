package com.gemwallet.android.data.service.store.database

import androidx.room.TypeConverter
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toNftAssetId
import com.gemwallet.android.ext.toNftCollectionId
import com.gemwallet.android.ext.toPerpetualId
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.CoreListItem
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTAttribute
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.SupportMessageImage
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString

class StoreConverters {
    private val assetLinksSerializer = ListSerializer(AssetLink.serializer())
    private val nftAttributesSerializer = ListSerializer(NFTAttribute.serializer())
    private val supportImagesSerializer = ListSerializer(SupportMessageImage.serializer())

    @TypeConverter
    fun fromSupportSender(value: SupportMessageSender): String = jsonEncoder.encodeToString(value)

    @TypeConverter
    fun toSupportSender(value: String): SupportMessageSender = jsonEncoder.decodeFromString(value)

    @TypeConverter
    fun fromSupportImages(value: List<SupportMessageImage>): String = jsonEncoder.encodeToString(supportImagesSerializer, value)

    @TypeConverter
    fun toSupportImages(value: String): List<SupportMessageImage> = jsonEncoder.decodeFromString(supportImagesSerializer, value)

    @TypeConverter
    fun fromSupportStatus(value: SupportMessageStatus): String = value.string

    @TypeConverter
    fun toSupportStatus(value: String): SupportMessageStatus = SupportMessageStatus.entries.first { it.string == value }

    @TypeConverter
    fun fromAssetId(value: AssetId): String = value.toIdentifier()

    @TypeConverter
    fun toAssetId(value: String): AssetId = requireNotNull(value.toAssetId()) {
        "Invalid AssetId in database: $value"
    }

    @TypeConverter
    fun fromNftAssetId(value: NFTAssetId): String = value.toIdentifier()

    @TypeConverter
    fun toNftAssetId(value: String): NFTAssetId = requireNotNull(value.toNftAssetId()) {
        "Invalid NFTAssetId in database: $value"
    }

    @TypeConverter
    fun fromNftCollectionId(value: NFTCollectionId): String = value.toIdentifier()

    @TypeConverter
    fun toNftCollectionId(value: String): NFTCollectionId = requireNotNull(value.toNftCollectionId()) {
        "Invalid NFTCollectionId in database: $value"
    }

    @TypeConverter
    fun fromPerpetualId(value: PerpetualId): String = value.toIdentifier()

    @TypeConverter
    fun toPerpetualId(value: String): PerpetualId = requireNotNull(value.toPerpetualId()) {
        "Invalid PerpetualId in database: $value"
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

    @TypeConverter
    fun fromCoreListItem(value: CoreListItem): String = jsonEncoder.encodeToString(CoreListItem.serializer(), value)

    @TypeConverter
    fun toCoreListItem(value: String): CoreListItem = jsonEncoder.decodeFromString(CoreListItem.serializer(), value)
}
