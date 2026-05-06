package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.GetSwapQuotes
import com.gemwallet.android.application.swap.coordinators.RequestSwapQuotes
import com.gemwallet.android.application.swap.coordinators.SwapQuoteRequestKey
import com.gemwallet.android.application.swap.coordinators.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.coordinators.SwapQuotesResult
import com.gemwallet.android.model.Crypto
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.transformLatest
import kotlinx.coroutines.isActive
import java.math.BigInteger

class RequestSwapQuotesImpl(
    private val getSwapQuotes: GetSwapQuotes,
) : RequestSwapQuotes {

    @OptIn(ExperimentalCoroutinesApi::class)
    override fun invoke(
        requestParams: Flow<SwapQuoteRequestParams?>,
        refreshRequests: Flow<Unit>,
        refreshEnabled: Flow<Boolean>,
        onFetchStarted: (SwapQuoteRequestKey) -> Unit,
        refreshIntervalMillis: Long,
    ): Flow<SwapQuotesResult?> {
        return requestParams.flatMapLatest { params ->
            if (params == null) {
                return@flatMapLatest flowOf<SwapQuotesResult?>(null)
            }

            refreshEnabled.flatMapLatest { isEnabled ->
                if (!isEnabled) {
                    return@flatMapLatest emptyFlow()
                }

                merge(flowOf(Unit), refreshRequests)
                    .transformLatest {
                        while (currentCoroutineContext().isActive) {
                            delay(500)
                            onFetchStarted(params.key)
                            val data = fetchQuotes(params)
                            emit(data)
                            if (data.err != null) {
                                break
                            }
                            delay(refreshIntervalMillis)
                        }
                    }
            }
        }
        .flowOn(Dispatchers.IO)
    }

    private suspend fun fetchQuotes(params: SwapQuoteRequestParams): SwapQuotesResult = try {
        val amount = Crypto(params.value, params.pay.asset.decimals).atomicValue
        val quotes = getSwapQuotes.getQuotes(
            from = params.pay.asset,
            to = params.receive.asset,
            ownerAddress = params.pay.owner!!.address,
            destination = params.receive.owner!!.address,
            amount = amount.toString(),
            useMaxAmount = BigInteger(params.pay.balance.balance.available) == amount,
        )
        currentCoroutineContext().ensureActive()
        SwapQuotesResult(quotes, params.key, params.pay, params.receive)
    } catch (err: CancellationException) {
        throw err
    } catch (err: Throwable) {
        SwapQuotesResult(requestKey = params.key, pay = params.pay, receive = params.receive, err = err)
    }
}
