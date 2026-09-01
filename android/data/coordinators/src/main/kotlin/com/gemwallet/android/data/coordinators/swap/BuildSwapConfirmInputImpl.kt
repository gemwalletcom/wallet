package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.cases.BuildSwapConfirmInput
import com.gemwallet.android.application.swap.cases.SwapNoQuoteException
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.domains.confirm.swap
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.swap.SwapData
import com.wallet.core.primitives.swap.SwapQuote
import com.wallet.core.primitives.swap.SwapQuoteData
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemSwapServiceInterface
import uniffi.gemstone.SwapperQuote

class BuildSwapConfirmInputImpl(
    private val getSession: GetSession,
    private val swapService: GemSwapServiceInterface,
) : BuildSwapConfirmInput {

    override suspend fun invoke(
        quote: SwapperQuote,
        pay: AssetInfo,
        receive: AssetInfo,
    ): GemConfirmInput? {
        val wallet = getSession().firstOrNull()?.wallet ?: return null
        val from = pay.owner ?: throw SwapNoQuoteException()

        val (transfer, swapQuote, swapData) = try {
            val transfer = swapService.getTransfer(wallet.toJson(), quote)
            Triple(transfer, transfer.quote.decodeJson<SwapQuote>(), transfer.data.decodeJson<SwapQuoteData>())
        } catch (error: CancellationException) {
            throw error
        } catch (error: Throwable) {
            throw SwapNoQuoteException(error)
        }

        return GemTransferData(
            inputType = GemTransactionInputType.swap(pay.asset, receive.asset, SwapData(quote = swapQuote, data = swapData)),
            recipient = GemRecipient(address = swapData.to, memo = swapData.memo),
            value = transfer.value,
            useMaxAmount = transfer.useMaxAmount,
            minimumValue = swapQuote.minFromValue,
        ).confirmInput(from)
    }
}
