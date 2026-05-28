package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.coordinators.GetSwapQuoteData
import com.gemwallet.android.application.swap.coordinators.SwapNoQuoteException
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.toModel
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.SwapperQuote
import java.math.BigInteger

class BuildSwapConfirmParamsImpl(
    private val sessionRepository: SessionRepository,
    private val getSwapQuoteData: GetSwapQuoteData,
) : BuildSwapConfirmParams {

    override suspend fun invoke(
        quote: SwapperQuote,
        pay: AssetInfo,
        receive: AssetInfo,
    ): ConfirmParams.SwapParams? {
        val wallet = sessionRepository.session().firstOrNull()?.wallet ?: return null

        val swapData = try {
            getSwapQuoteData(quote, wallet)
        } catch (_: Throwable) {
            throw SwapNoQuoteException()
        }

        val from = pay.owner ?: throw SwapNoQuoteException()
        return ConfirmParams.SwapParams(
            from = from,
            fromAsset = pay.asset,
            toAsset = receive.asset,
            fromAmount = BigInteger(quote.fromValue),
            minFromAmount = quote.minFromValue?.toBigIntegerOrNull(),
            toAmount = BigInteger(quote.toValue),
            swapData = swapData.data,
            providerId = quote.data.provider.id,
            protocol = quote.data.provider.protocol,
            providerName = quote.data.provider.name,
            protocolId = quote.data.provider.protocolId,
            toAddress = swapData.to,
            value = swapData.value,
            approval = swapData.approval?.toModel(),
            gasLimit = swapData.gasLimit?.toBigIntegerOrNull(),
            useMaxAmount = quote.request.options.useMaxAmount,
            etaInSeconds = quote.etaInSeconds,
            slippageBps = quote.data.slippageBps,
            memo = swapData.memo,
            dataType = swapData.dataType,
        )
    }
}
