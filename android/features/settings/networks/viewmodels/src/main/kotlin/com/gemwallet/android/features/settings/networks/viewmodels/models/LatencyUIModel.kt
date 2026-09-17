package com.gemwallet.android.features.settings.networks.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.settings.networks.viewmodels.localization.text
import com.gemwallet.android.ui.components.list_item.ListItemTagType
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.LatencyType

enum class LatencyTone { Loading, Fast, Normal, Slow, Error }

data class LatencyUIModel(
    val text: String,
    val tone: LatencyTone,
)

internal fun GemLatencyStatus.uiModel(context: Context) = LatencyUIModel(
    text = text(context),
    tone = when (this) {
        is GemLatencyStatus.Loading -> LatencyTone.Loading
        is GemLatencyStatus.Error -> LatencyTone.Error
        is GemLatencyStatus.Result -> when (latency.latencyType) {
            LatencyType.FAST -> LatencyTone.Fast
            LatencyType.NORMAL -> LatencyTone.Normal
            LatencyType.SLOW -> LatencyTone.Slow
        }
    },
)

fun LatencyUIModel.tagStyle(): ListItemTextStyle = when (tone) {
    LatencyTone.Loading -> ListItemTextStyle.Secondary
    LatencyTone.Fast -> ListItemTextStyle.Positive
    LatencyTone.Normal -> ListItemTextStyle.Warning
    LatencyTone.Slow, LatencyTone.Error -> ListItemTextStyle.Negative
}

fun LatencyUIModel.tagType(): ListItemTagType = when (tone) {
    LatencyTone.Loading -> ListItemTagType.Progress
    LatencyTone.Fast, LatencyTone.Normal, LatencyTone.Slow, LatencyTone.Error -> ListItemTagType.None
}
