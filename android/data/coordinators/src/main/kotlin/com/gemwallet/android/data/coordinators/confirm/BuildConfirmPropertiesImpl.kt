package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemConfirmRowContent
import uniffi.gemstone.GemConfirmTransferServiceInterface
import uniffi.gemstone.GemTransferData

class BuildConfirmPropertiesImpl(
    private val confirmService: GemConfirmTransferServiceInterface,
) : BuildConfirmProperties {

    override suspend fun invoke(
        transfer: GemTransferData,
        wallet: Wallet,
        addressName: AddressName?,
    ): List<ConfirmProperty> = withContext(Dispatchers.IO) {
        confirmService.rowContents(transfer, wallet.toGem(), addressName?.toGem()).mapNotNull { content ->
            when (content) {
                is GemConfirmRowContent.App -> ConfirmProperty.Destination.Generic(content.name)
                is GemConfirmRowContent.Sender -> ConfirmProperty.Source(content.wallet)
                is GemConfirmRowContent.Recipient -> ConfirmProperty.Destination.map(
                    destination = content.destination,
                    chain = content.chain.toChain(),
                    addressName = content.addressName?.toPrimitives(),
                    explorerLink = content.link.toPrimitives(),
                )
                is GemConfirmRowContent.Network -> ConfirmProperty.Network(content.chain.toChain(), content.name)
                is GemConfirmRowContent.Memo -> ConfirmProperty.Memo(content.memo.orEmpty())
                is GemConfirmRowContent.Details -> null
            }
        }
    }
}
