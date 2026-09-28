package com.gemwallet.android.serializer

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toNftAssetId
import com.gemwallet.android.ext.toNftCollectionId
import com.gemwallet.android.ext.toPerpetualId
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import java.io.IOException

open class IdentifierSerializer<T : Any>(private val name: String, private val identifier: (T) -> String, private val parse: (String) -> T?) : KSerializer<T> {
    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor(name, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: T) {
        encoder.encodeString(identifier(value))
    }

    override fun deserialize(decoder: Decoder): T {
        val value = decoder.decodeString()
        return parse(value) ?: throw IOException("Invalid $name: $value")
    }
}

object NFTAssetIdSerializer : IdentifierSerializer<NFTAssetId>("NFTAssetId", NFTAssetId::toIdentifier, String::toNftAssetId)

object NFTCollectionIdSerializer : IdentifierSerializer<NFTCollectionId>("NFTCollectionId", NFTCollectionId::toIdentifier, String::toNftCollectionId)

object PerpetualIdSerializer : IdentifierSerializer<PerpetualId>("PerpetualId", PerpetualId::toIdentifier, String::toPerpetualId)

object WalletIdSerializer : IdentifierSerializer<WalletId>("WalletId", WalletId::id, ::WalletId)
