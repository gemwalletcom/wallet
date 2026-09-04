package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemConfirmDestination

sealed interface ConfirmProperty {
    class Source(val data: String, val walletType: WalletType, val walletChain: Chain?, val walletImageUrl: String?) : ConfirmProperty
    class Network(val data: Asset) : ConfirmProperty
    class Memo(memo: String) : ConfirmProperty {
        val data: String = memo.ifEmpty { "-" }
    }

    sealed class Destination(val data: String) : ConfirmProperty {
        class Stake(data: String, val address: String? = null, val explorerLink: BlockExplorerLink? = null) : Destination(data)
        class Provider(data: String) : Destination(data)
        class Transfer(
            val domain: String?,
            val address: String,
            val chain: Chain,
            val addressType: AddressType? = null,
            val imageUrl: String? = null,
            val explorerLink: BlockExplorerLink? = null,
        ) : Destination(address)
        class Contract(val address: String, val chain: Chain, val explorerLink: BlockExplorerLink? = null) : Destination(address)
        class Resource(val resource: com.wallet.core.primitives.Resource) : Destination(resource.string)
        class Generic(val appName: String) : Destination(appName)
        class PerpetualOper(val providerName: String) : Destination(providerName)

        companion object {
            fun map(destination: GemConfirmDestination?, chain: Chain, addressName: AddressName? = null): Destination? = when (destination) {
                null -> null
                is GemConfirmDestination.Recipient -> Transfer(
                    domain = destination.name ?: addressName?.name,
                    address = destination.address,
                    chain = chain,
                    addressType = addressName?.type,
                    imageUrl = addressName?.imageUrl,
                )
                is GemConfirmDestination.Contract -> Contract(address = destination.address, chain = chain)
                is GemConfirmDestination.Validator -> Stake(data = destination.name, address = destination.address)
                is GemConfirmDestination.Resource -> Resource(destination.resource.toPrimitives())
                is GemConfirmDestination.Provider -> Provider(destination.name)
            }
        }
    }
}
