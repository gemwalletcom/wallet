package com.gemwallet.android.features.swap.viewmodels.cases

import com.gemwallet.android.cases.swap.GetSwapQuotes
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.features.swap.viewmodels.models.QuoteRequestParams
import com.gemwallet.android.features.swap.viewmodels.models.QuotesState
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.isActive
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.transformLatest
import javax.inject.Inject
import java.math.BigInteger

class QuoteRequester @Inject constructor(
    private val getSwapQuotes: GetSwapQuotes
) {

    @OptIn(ExperimentalCoroutinesApi::class)
    internal fun requestQuotes(
        requestParams: Flow<QuoteRequestParams?>,
        refreshRequests: Flow<Unit>,
        refreshEnabled: Flow<Boolean>,
        onError: (Throwable) -> Unit,
        refreshIntervalMillis: Long = QUOTE_REFRESH_INTERVAL_MS,
    ): Flow<QuotesState?> {
        return requestParams.flatMapLatest { params ->
            if (params == null) {
                return@flatMapLatest flowOf<QuotesState?>(null)
            }

            refreshEnabled.flatMapLatest { isEnabled ->
                if (!isEnabled) {
                    return@flatMapLatest emptyFlow()
                }

                merge(flowOf(Unit), refreshRequests)
                    .transformLatest {
                        while (currentCoroutineContext().isActive) {
                            delay(500)
                            val data = fetchQuotes(params)
                            emit(data)
                            if (data.err != null) {
                                onError(data.err)
                                break
                            }
                            delay(refreshIntervalMillis)
                        }
                    }
            }
        }
        .flowOn(Dispatchers.IO)
    }

    private suspend fun fetchQuotes(params: QuoteRequestParams): QuotesState = try {
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
        QuotesState(quotes, params.key, params.pay, params.receive)
    } catch (err: CancellationException) {
        throw err
    } catch (err: Throwable) {
        QuotesState(requestKey = params.key, pay = params.pay, receive = params.receive, err = err)
    }

    private companion object {
        const val QUOTE_REFRESH_INTERVAL_MS = 30_000L
    }
}
