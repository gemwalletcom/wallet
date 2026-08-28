package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.cases.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.cases.SwapNoQuoteException
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.swap.SwapData
import com.wallet.core.primitives.swap.SwapQuote
import com.wallet.core.primitives.swap.SwapQuoteData
import kotlinx.coroutines.CancellationException
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
        val from = pay.owner ?: throw SwapNoQuoteException()

        val (transfer, swapQuote, swapData) = try {
            val transfer = swapService.getTransfer(wallet.toJson(), quote)
            Triple(transfer, transfer.quote.decodeJson<SwapQuote>(), transfer.data.decodeJson<SwapQuoteData>())
        } catch (error: CancellationException) {
            throw error
        } catch (error: Throwable) {
            throw SwapNoQuoteException(error)
        }

        return ConfirmParams.SwapParams(
            from = from,
            fromAsset = pay.asset,
            toAsset = receive.asset,
            swapData = SwapData(quote = swapQuote, data = swapData),
            amount = transfer.value.toBigInteger(),
            useMaxAmount = transfer.useMaxAmount,
        )
    }
}
