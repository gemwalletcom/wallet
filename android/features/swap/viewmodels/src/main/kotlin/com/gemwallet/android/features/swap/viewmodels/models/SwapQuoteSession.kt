package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.application.swap.cases.SwapQuoteRequestKey
import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import uniffi.gemstone.SwapperException
import uniffi.gemstone.SwapProvider
import uniffi.gemstone.SwapperQuote

internal sealed interface SwapQuotePhase {
    data object NoInput : SwapQuotePhase
    data class Loading(val requestKey: SwapQuoteRequestKey) : SwapQuotePhase
    data object Ready : SwapQuotePhase
    data class Failed(val requestKey: SwapQuoteRequestKey, val error: Throwable) : SwapQuotePhase
}

internal sealed interface SwapTransferPhase {
    data object Idle : SwapTransferPhase
    data class Loading(
        val requestKey: SwapQuoteRequestKey,
        val providerId: SwapProvider,
    ) : SwapTransferPhase

    data class Failed(
        val requestKey: SwapQuoteRequestKey,
        val providerId: SwapProvider,
        val error: Throwable,
    ) : SwapTransferPhase
}

internal typealias SelectQuote = (List<SwapperQuote>, SwapProvider?) -> SwapperQuote?

internal data class SwapQuoteSession(
    val quotes: SwapQuotesResult? = null,
    val selectedProvider: SwapProvider? = null,
    val selectedQuote: SwapperQuote? = null,
    val quotePhase: SwapQuotePhase = SwapQuotePhase.NoInput,
    val transferPhase: SwapTransferPhase = SwapTransferPhase.Idle,
    val refreshPausedUntilRestart: Boolean = false,
) {
    val quote: QuoteState?
        get() {
            val current = quotes ?: return null
            val selected = selectedQuote ?: return null
            return QuoteState(selected, current.pay, current.receive)
        }

    val acceptsQuotePhase: Boolean
        get() = transferPhase !is SwapTransferPhase.Loading && !refreshPausedUntilRestart

    val acceptsQuotes: Boolean
        get() = transferPhase is SwapTransferPhase.Idle

    val quoteError: SwapperException?
        get() = (quotePhase as? SwapQuotePhase.Failed)?.error as? SwapperException

    val transferError: SwapperException?
        get() = (transferPhase as? SwapTransferPhase.Failed)?.error as? SwapperException
}

internal fun SwapQuoteSession.onRequestParamsChanged(params: SwapQuoteRequestParams?): SwapQuoteSession =
    if (params == null) SwapQuoteSession() else onRefreshRequested(params).copy(quotes = null)

internal fun SwapQuoteSession.onRefreshRequested(params: SwapQuoteRequestParams): SwapQuoteSession =
    onQuoteInvalidated().copy(quotePhase = SwapQuotePhase.Loading(params.key))

internal fun SwapQuoteSession.onFetchStarted(requestKey: SwapQuoteRequestKey): SwapQuoteSession =
    if (acceptsQuotePhase) copy(quotePhase = SwapQuotePhase.Loading(requestKey)) else this

internal fun SwapQuoteSession.onQuoteResults(results: SwapQuotesResult, select: SelectQuote): SwapQuoteSession {
    val error = results.quoteErrorOrNull()
    val quotes = if (acceptsQuotes) results.takeIf { error == null } else quotes
    return copy(
        quotes = quotes,
        selectedQuote = quotes?.let { select(it.items, selectedProvider) },
        quotePhase = if (acceptsQuotePhase) results.phaseFor(error) else quotePhase,
    )
}

private fun SwapQuotesResult.quoteErrorOrNull(): Throwable? = when {
    err != null -> err
    items.isEmpty() -> SwapperException.NoQuoteAvailable()
    else -> null
}

private fun SwapQuotesResult.phaseFor(error: Throwable?): SwapQuotePhase =
    if (error == null) SwapQuotePhase.Ready else SwapQuotePhase.Failed(requestKey, error)

internal fun SwapQuoteSession.onProviderSelected(provider: SwapProvider, select: SelectQuote): SwapQuoteSession = copy(
    selectedProvider = provider,
    selectedQuote = quotes?.let { select(it.items, provider) },
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
    error: Throwable,
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
