package com.gemwallet.android.data.coordinators.confirm

import uniffi.gemstone.GemConfirmTransferServiceInterface
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.ext.asset
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.GemTransferData
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class BuildConfirmPropertiesImpl(
    private val confirmService: GemConfirmTransferServiceInterface,
) : BuildConfirmProperties {

    override suspend fun invoke(
        transfer: GemTransferData,
        wallet: Wallet,
        addressName: AddressName?,
    ): List<ConfirmProperty> {
        val chain = transfer.asset.id.chain
        return withContext(Dispatchers.IO) {
            mutableListOf<ConfirmProperty?>().apply {
                add(ConfirmProperty.Source(wallet.name, wallet.type, chain, wallet.imageUrl))
                (transfer.inputType as? TransactionInputType.Generic)?.let { add(ConfirmProperty.Destination.Generic(it.metadata.name)) }
                add(
                    when (val destination = ConfirmProperty.Destination.map(transfer.destination(), chain, addressName)) {
                        is ConfirmProperty.Destination.Transfer -> ConfirmProperty.Destination.Transfer(
                            domain = destination.domain,
                            address = destination.address,
                            chain = destination.chain,
                            addressType = destination.addressType,
                            imageUrl = destination.imageUrl,
                            explorerLink = explorerLink(chain, destination.address),
                        )
                        is ConfirmProperty.Destination.Contract -> ConfirmProperty.Destination.Contract(
                            address = destination.address,
                            chain = destination.chain,
                            explorerLink = explorerLink(chain, destination.address),
                        )
                        is ConfirmProperty.Destination.Stake -> destination.address?.let { address ->
                            ConfirmProperty.Destination.Stake(
                                data = destination.data,
                                address = address,
                                explorerLink = explorerLink(chain, address),
                            )
                        } ?: destination
                        else -> destination
                    }
                )
                add(ConfirmProperty.Network(chain.asset()))
                add(ConfirmProperty.Memo(transfer.recipient.memo.orEmpty()).takeIf { transfer.showsMemo() })
            }.filterNotNull()
        }
    }

    private fun explorerLink(chain: Chain, address: String): BlockExplorerLink =
        confirmService.addressUrl(chain.string, address).let { BlockExplorerLink(it.name, it.link) }
}
