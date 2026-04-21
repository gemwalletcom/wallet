package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.TransferDataOutputAction
import uniffi.gemstone.GemApprovalData
import uniffi.gemstone.GemSwapData
import uniffi.gemstone.GemSwapProviderData
import uniffi.gemstone.GemSwapQuote
import uniffi.gemstone.GemSwapQuoteData
import uniffi.gemstone.TransferDataOutputAction as GemTransferDataOutputAction

fun TransferDataOutputAction.toGem(): GemTransferDataOutputAction = when (this) {
    TransferDataOutputAction.Sign -> GemTransferDataOutputAction.SIGN
    TransferDataOutputAction.Send -> GemTransferDataOutputAction.SEND
    TransferDataOutputAction.SignAndSend -> GemTransferDataOutputAction.SIGN_AND_SEND
}

fun ConfirmParams.SwapParams.toGem(): GemSwapData {
    return GemSwapData(
        quote = GemSwapQuote(
            fromAddress = from.address,
            toAddress = toAddress,
            fromValue = fromAmount.toString(),
            toValue = toAmount.toString(),
            providerData = GemSwapProviderData(
                provider = providerId,
                protocolName = protocolId,
                name = providerName,
            ),
            slippageBps = slippageBps,
            etaInSeconds = etaInSeconds,
            useMaxAmount = this@toGem.useMaxAmount
        ),
        data = GemSwapQuoteData(
            to = toAddress,
            data = swapData,
            approval = approval?.let {
                GemApprovalData(
                    token = it.token,
                    spender = it.spender,
                    value = it.value,
                    isUnlimited = it.isUnlimited,
                )
            },
            value = value,
            gasLimit = gasLimit?.toString(),
            dataType = dataType,
            memo = memo()
        )
    )
}
