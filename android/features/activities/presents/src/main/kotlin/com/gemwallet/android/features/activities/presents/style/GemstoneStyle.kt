package com.gemwallet.android.features.activities.presents.style

import androidx.compose.ui.graphics.vector.ImageVector
import com.gemwallet.android.ui.icons.AppIcons
import androidx.compose.runtime.Composable
import uniffi.gemstone.GemSwapProgressMarker

@Composable
internal fun GemSwapProgressMarker.icon(): ImageVector? = when (this) {
    GemSwapProgressMarker.CHECK -> AppIcons.Check
    GemSwapProgressMarker.CROSS -> AppIcons.Close
    GemSwapProgressMarker.SWAP -> AppIcons.SwapVert
    GemSwapProgressMarker.SPINNER, GemSwapProgressMarker.DOTS -> null
}
