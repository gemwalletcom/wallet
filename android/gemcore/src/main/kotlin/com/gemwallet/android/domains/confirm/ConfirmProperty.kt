package com.gemwallet.android.domains.confirm

import uniffi.gemstone.GemWalletRow
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemConfirmDestination

sealed interface ConfirmProperty {
    class Source(val walletRow: GemWalletRow) : ConfirmProperty
    class Network(val chain: Chain, val name: String) : ConfirmProperty
    class PaymentAsset(val symbol: String, val selectable: Boolean) : ConfirmProperty
    class Memo(memo: String) : ConfirmProperty {
        val data: String = memo.ifEmpty { "-" }
    }

    sealed class Destination(val data: String, val kind: GemConfirmDestination?) : ConfirmProperty {
        class Stake(data: String, val address: String? = null, val explorerLink: BlockExplorerLink? = null, kind: GemConfirmDestination? = null) :
            Destination(data, kind)
        class Provider(data: String, kind: GemConfirmDestination? = null) : Destination(data, kind)
        class Transfer(
            val domain: String?,
            val address: String,
            val chain: Chain,
            val addressType: AddressType? = null,
            val imageUrl: String? = null,
            val explorerLink: BlockExplorerLink? = null,
            kind: GemConfirmDestination? = null,
        ) : Destination(address, kind)
        class Contract(val address: String, val chain: Chain, val explorerLink: BlockExplorerLink? = null, kind: GemConfirmDestination? = null) :
            Destination(address, kind)
        class Resource(val resource: com.wallet.core.primitives.Resource, kind: GemConfirmDestination? = null) : Destination(resource.string, kind)
        class Generic(val appName: String) : Destination(appName, null)

        companion object {
            fun map(destination: GemConfirmDestination, chain: Chain, addressName: AddressName?, explorerLink: BlockExplorerLink): Destination = when (destination) {
                is GemConfirmDestination.Recipient -> Transfer(
                    domain = destination.name,
                    address = destination.address,
                    chain = chain,
                    addressType = addressName?.type,
                    imageUrl = addressName?.imageUrl,
                    explorerLink = explorerLink,
                    kind = destination,
                )
                is GemConfirmDestination.Contract -> Contract(address = destination.address, chain = chain, explorerLink = explorerLink, kind = destination)
                is GemConfirmDestination.Validator -> Stake(data = destination.name, address = destination.address, explorerLink = explorerLink, kind = destination)
                is GemConfirmDestination.Resource -> Resource(destination.resource.toPrimitives(), kind = destination)
                is GemConfirmDestination.Provider -> Provider(destination.name, kind = destination)
            }
        }
    }
}
