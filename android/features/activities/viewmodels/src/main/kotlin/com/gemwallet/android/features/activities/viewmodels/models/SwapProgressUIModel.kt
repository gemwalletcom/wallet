package com.gemwallet.android.features.activities.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.duration.formatEstimatedConfirmation
import com.gemwallet.android.features.activities.viewmodels.localization.stringRes
import com.gemwallet.android.features.activities.viewmodels.style.markerUIModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemSwapProgress
import uniffi.gemstone.GemSwapProgressMarker
import uniffi.gemstone.GemSwapProgressState
import uniffi.gemstone.GemSwapProgressStep

sealed interface SwapProgressMarkerUIModel {
    data object Spinner : SwapProgressMarkerUIModel
    data object Dots : SwapProgressMarkerUIModel
    data class Icon(val symbol: ListItemSymbol) : SwapProgressMarkerUIModel
}

data class SwapProgressStepUIModel(val title: String, val subtitle: String, val statusLabel: String?, val style: ListItemTextStyle, val marker: SwapProgressMarkerUIModel, val showsEstimatedTime: Boolean)

data class SwapProgressUIModel(val transfer: SwapProgressStepUIModel, val swap: SwapProgressStepUIModel, val estimatedTime: String?, val isConnectorActive: Boolean)

internal fun GemSwapProgress.uiModel(context: Context): SwapProgressUIModel = SwapProgressUIModel(
    transfer = transfer.step(context, title = context.getString(R.string.transfer_title), subtitle = "${amount.text()} ($network)"),
    swap = swap.step(context, title = context.getString(R.string.wallet_swap), subtitle = providerName),
    estimatedTime = etaSeconds?.let(::formatEstimatedConfirmation),
    isConnectorActive = transfer.step == GemSwapProgressStep.COMPLETED,
)

private fun GemSwapProgressState.step(context: Context, title: String, subtitle: String): SwapProgressStepUIModel = SwapProgressStepUIModel(
    title = title,
    subtitle = subtitle,
    statusLabel = step.stringRes()?.let { context.getString(it) },
    style = step.textStyle(),
    marker = marker.markerUIModel(),
    showsEstimatedTime = marker == GemSwapProgressMarker.SPINNER,
)
