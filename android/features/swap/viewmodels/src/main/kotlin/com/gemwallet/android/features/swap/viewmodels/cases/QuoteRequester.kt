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
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import javax.inject.Inject
import java.math.BigInteger

class QuoteRequester @Inject constructor(
    private val getSwapQuotes: GetSwapQuotes
) {

    @OptIn(ExperimentalCoroutinesApi::class)
    internal fun requestQuotes(
        requestParams: Flow<QuoteRequestParams?>,
        refreshState: Flow<Long>,
        onStart: (QuoteRequestParams?) -> Unit,
        onError: (Throwable) -> Unit
    ): Flow<QuotesState?> {
        return combine(requestParams, refreshState) { params, _ -> params }
        .onEach { onStart(it) }
        .mapLatest { params ->
            params?.let {
                delay(500)
                fetchQuotes(it)
            }
        }
        .onEach { data ->
            data?.err?.let { onError(it) }
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
}
