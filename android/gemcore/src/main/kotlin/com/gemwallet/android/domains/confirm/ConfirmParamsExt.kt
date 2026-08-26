package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.SwapProvider
import com.wallet.core.primitives.swap.SwapData
import com.wallet.core.primitives.swap.SwapProviderData
import com.wallet.core.primitives.swap.SwapQuote
import com.wallet.core.primitives.swap.SwapQuoteData
import uniffi.gemstone.SwapData as GemSwapData

fun ConfirmParams.SwapParams.toGem(): GemSwapData = SwapData(
    quote = SwapQuote(
        fromAddress = from.address,
        toAddress = toAddress,
        fromValue = fromAmount.toString(),
        minFromValue = minFromAmount?.toString(),
        toValue = toAmount.toString(),
        providerData = SwapProviderData(
            provider = SwapProvider.entries.first { it.string == providerId.name.lowercase() },
            protocolName = protocolId,
            name = providerName,
        ),
        slippageBps = slippageBps,
        etaInSeconds = etaInSeconds,
        useMaxAmount = useMaxAmount,
    ),
    data = SwapQuoteData(
        to = toAddress,
        data = swapData,
        approval = approval,
        value = value,
        gasLimit = gasLimit?.toString(),
        dataType = dataType,
        memo = memo(),
    ),
).toJson()
