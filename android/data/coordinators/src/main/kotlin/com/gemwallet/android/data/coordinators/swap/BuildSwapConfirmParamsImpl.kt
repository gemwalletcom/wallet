package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.coordinators.SwapNoQuoteException
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.swap.SwapData
import com.wallet.core.primitives.swap.SwapQuote
import com.wallet.core.primitives.swap.SwapQuoteData
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemSwapServiceInterface
import uniffi.gemstone.SwapperQuote

class BuildSwapConfirmParamsImpl(
    private val sessionRepository: SessionRepository,
    private val swapService: GemSwapServiceInterface,
) : BuildSwapConfirmParams {

    override suspend fun invoke(
        quote: SwapperQuote,
        pay: AssetInfo,
        receive: AssetInfo,
    ): ConfirmParams.SwapParams? {
        val wallet = sessionRepository.session().firstOrNull()?.wallet ?: return null

        val transfer = try {
            swapService.getTransfer(wallet.toJson(), quote)
        } catch (_: Throwable) {
            throw SwapNoQuoteException()
        }
        val swapQuote = transfer.quote.decodeJson<SwapQuote>()
        val swapData = transfer.data.decodeJson<SwapQuoteData>()

        val from = pay.owner ?: throw SwapNoQuoteException()
        return ConfirmParams.SwapParams(
            from = from,
            fromAsset = pay.asset,
            toAsset = receive.asset,
            swapData = SwapData(quote = swapQuote, data = swapData),
            useMaxAmount = transfer.useMaxAmount,
        )
    }
}
