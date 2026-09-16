package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.ext.toGem
import uniffi.gemstone.walletRow
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
import uniffi.gemstone.GemConfirmRow
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
            transfer.confirmRows().mapNotNull { row ->
                when (row) {
                    GemConfirmRow.APP -> (transfer.inputType as? TransactionInputType.Generic)?.let { ConfirmProperty.Destination.Generic(it.metadata.name) }
                    GemConfirmRow.SENDER -> ConfirmProperty.Source(walletRow(wallet.toGem()))
                    GemConfirmRow.RECIPIENT -> destination(transfer, chain, addressName)
                    GemConfirmRow.NETWORK -> ConfirmProperty.Network(chain.asset())
                    GemConfirmRow.MEMO -> ConfirmProperty.Memo(transfer.recipient.memo.orEmpty())
                    GemConfirmRow.PAYMENT_ASSET, GemConfirmRow.DETAILS -> null
                }
            }
        }
    }

    private fun destination(transfer: GemTransferData, chain: Chain, addressName: AddressName?): ConfirmProperty? =
        when (val destination = ConfirmProperty.Destination.map(transfer.destination()?.withAddressName(addressName?.toGem()), chain, addressName)) {
            is ConfirmProperty.Destination.Transfer -> ConfirmProperty.Destination.Transfer(
                domain = destination.domain,
                address = destination.address,
                chain = destination.chain,
                addressType = destination.addressType,
                imageUrl = destination.imageUrl,
                explorerLink = explorerLink(chain, destination.address),
                kind = destination.kind,
            )
            is ConfirmProperty.Destination.Contract -> ConfirmProperty.Destination.Contract(
                address = destination.address,
                chain = destination.chain,
                explorerLink = explorerLink(chain, destination.address),
                kind = destination.kind,
            )
            is ConfirmProperty.Destination.Stake -> destination.address?.let { address ->
                ConfirmProperty.Destination.Stake(
                    data = destination.data,
                    address = address,
                    explorerLink = explorerLink(chain, address),
                    kind = destination.kind,
                )
            } ?: destination
            else -> destination
        }

    private fun explorerLink(chain: Chain, address: String): BlockExplorerLink =
        confirmService.addressUrl(chain.string, address).let { BlockExplorerLink(it.name, it.link) }
}
