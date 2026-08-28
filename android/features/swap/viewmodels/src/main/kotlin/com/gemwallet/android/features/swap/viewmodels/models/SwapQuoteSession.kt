package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.application.swap.cases.SwapQuoteRequestKey
import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import com.gemwallet.android.application.swap.cases.getQuote
import uniffi.gemstone.SwapperProvider

internal sealed interface SwapQuotePhase {
    data object NoInput : SwapQuotePhase
    data class Loading(val requestKey: SwapQuoteRequestKey) : SwapQuotePhase
    data object Ready : SwapQuotePhase
    data class Failed(val requestKey: SwapQuoteRequestKey, val error: SwapError) : SwapQuotePhase
}

internal sealed interface SwapTransferPhase {
    data object Idle : SwapTransferPhase
    data class Loading(
        val requestKey: SwapQuoteRequestKey,
        val providerId: SwapperProvider,
    ) : SwapTransferPhase

    data class Failed(
        val requestKey: SwapQuoteRequestKey,
        val providerId: SwapperProvider,
        val error: SwapError,
    ) : SwapTransferPhase
}

internal data class SwapQuoteSession(
    val quotes: SwapQuotesResult? = null,
    val selectedProvider: SwapperProvider? = null,
    val quotePhase: SwapQuotePhase = SwapQuotePhase.NoInput,
    val transferPhase: SwapTransferPhase = SwapTransferPhase.Idle,
    val refreshPausedUntilRestart: Boolean = false,
) {
    val quote: QuoteState?
        get() {
            val current = quotes ?: return null
            val selected = current.getQuote(selectedProvider) ?: return null
            return QuoteState(selected, current.pay, current.receive)
        }

    val acceptsQuotePhase: Boolean
        get() = transferPhase !is SwapTransferPhase.Loading && !refreshPausedUntilRestart

    val acceptsQuotes: Boolean
        get() = transferPhase is SwapTransferPhase.Idle
}

internal fun SwapQuoteSession.onRequestParamsChanged(params: SwapQuoteRequestParams?): SwapQuoteSession =
    if (params == null) SwapQuoteSession() else onRefreshRequested(params).copy(quotes = null)

internal fun SwapQuoteSession.onRefreshRequested(params: SwapQuoteRequestParams): SwapQuoteSession =
    onQuoteInvalidated().copy(quotePhase = SwapQuotePhase.Loading(params.key))

internal fun SwapQuoteSession.onFetchStarted(requestKey: SwapQuoteRequestKey): SwapQuoteSession =
    if (acceptsQuotePhase) copy(quotePhase = SwapQuotePhase.Loading(requestKey)) else this

internal fun SwapQuoteSession.onQuoteResults(results: SwapQuotesResult): SwapQuoteSession {
    val error = results.quoteErrorOrNull()
    return copy(
        quotes = if (acceptsQuotes) results.takeIf { error == null } else quotes,
        quotePhase = if (acceptsQuotePhase) results.phaseFor(error) else quotePhase,
    )
}

private fun SwapQuotesResult.quoteErrorOrNull(): SwapError? {
    val failure = err
    return when {
        failure != null -> SwapError.toError(failure)
        items.isEmpty() -> SwapError.NoQuote
        else -> null
    }
}

private fun SwapQuotesResult.phaseFor(error: SwapError?): SwapQuotePhase =
    if (error == null) SwapQuotePhase.Ready else SwapQuotePhase.Failed(requestKey, error)

internal fun SwapQuoteSession.onProviderSelected(provider: SwapperProvider): SwapQuoteSession = copy(
    selectedProvider = provider,
    transferPhase = SwapTransferPhase.Idle,
    refreshPausedUntilRestart = false,
)

internal fun SwapQuoteSession.onQuoteInvalidated(): SwapQuoteSession = copy(
    transferPhase = SwapTransferPhase.Idle,
    refreshPausedUntilRestart = false,
)

internal fun SwapQuoteSession.startTransfer(): Pair<SwapQuoteSession, SwapTransferPhase.Loading?> {
    if (transferPhase is SwapTransferPhase.Loading) return this to null
    val current = quotes ?: return this to null
    val pending = quote ?: return this to null
    val transfer = SwapTransferPhase.Loading(
        requestKey = current.requestKey,
        providerId = pending.quote.data.provider.id,
    )
    return copy(transferPhase = transfer) to transfer
}

internal fun SwapQuoteSession.onTransferFailed(
    transfer: SwapTransferPhase.Loading,
    error: SwapError,
): SwapQuoteSession = if (transferPhase == transfer) {
    copy(
        transferPhase = SwapTransferPhase.Failed(
            requestKey = transfer.requestKey,
            providerId = transfer.providerId,
            error = error,
        )
    )
} else {
    this
}

internal fun SwapQuoteSession.onTransferHandedOff(transfer: SwapTransferPhase.Loading): SwapQuoteSession =
    if (transferPhase == transfer) {
        copy(transferPhase = SwapTransferPhase.Idle, refreshPausedUntilRestart = true)
    } else {
        this
    }

internal fun SwapQuoteSession.onTransferAbandoned(transfer: SwapTransferPhase.Loading): SwapQuoteSession =
    if (transferPhase == transfer) onQuoteInvalidated() else this
