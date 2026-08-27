package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.RequestSwapQuotes
import com.gemwallet.android.application.swap.coordinators.RequestSwapQuotes.Companion.QUOTE_DEBOUNCE_MS
import com.gemwallet.android.application.swap.coordinators.SwapQuoteRequestKey
import com.gemwallet.android.application.swap.coordinators.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.coordinators.SwapQuotesResult
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.serializer.toJson
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
import uniffi.gemstone.GemSwapServiceInterface
import java.math.BigInteger

class RequestSwapQuotesImpl(
    private val sessionRepository: SessionRepository,
    private val swapService: GemSwapServiceInterface,
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
                            delay(QUOTE_DEBOUNCE_MS)
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
        val wallet = checkNotNull(sessionRepository.session().value?.wallet) { "Swap has no active wallet" }
        val amount = Crypto(params.value, params.pay.asset.decimals).atomicValue
        val quotes = swapService.getQuotes(
            wallet = wallet.toJson(),
            fromAsset = params.pay.asset.toJson(),
            toAsset = params.receive.asset.toJson(),
            value = amount.toString(),
            useMaxAmount = BigInteger(params.pay.balance.balance.available) == amount,
            slippageBps = params.slippageBps,
        )
        currentCoroutineContext().ensureActive()
        SwapQuotesResult(quotes, params.key, params.pay, params.receive)
    } catch (err: CancellationException) {
        throw err
    } catch (err: Throwable) {
        SwapQuotesResult(requestKey = params.key, pay = params.pay, receive = params.receive, err = err)
    }
}
