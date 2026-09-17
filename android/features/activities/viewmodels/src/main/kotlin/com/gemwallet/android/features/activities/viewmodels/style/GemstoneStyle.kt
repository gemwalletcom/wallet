package com.gemwallet.android.features.activities.viewmodels.style

import com.gemwallet.android.features.activities.viewmodels.models.SwapProgressMarkerUIModel
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import uniffi.gemstone.GemSwapProgressMarker

fun GemSwapProgressMarker.markerUIModel(): SwapProgressMarkerUIModel = when (this) {
    GemSwapProgressMarker.CHECK -> SwapProgressMarkerUIModel.Icon(ListItemSymbol.Check)
    GemSwapProgressMarker.CROSS -> SwapProgressMarkerUIModel.Icon(ListItemSymbol.Close)
    GemSwapProgressMarker.SWAP -> SwapProgressMarkerUIModel.Icon(ListItemSymbol.Swap)
    GemSwapProgressMarker.SPINNER -> SwapProgressMarkerUIModel.Spinner
    GemSwapProgressMarker.DOTS -> SwapProgressMarkerUIModel.Dots
}
