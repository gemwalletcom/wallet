package com.gemwallet.android.features.settings.networks.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.settings.networks.viewmodels.localization.text
import com.gemwallet.android.ui.components.list_item.ListItemTagType
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemValueTone

data class LatencyUIModel(
    val text: String,
    val tone: GemValueTone,
    val tagType: ListItemTagType,
)

internal fun GemLatencyStatus.uiModel(context: Context) = LatencyUIModel(
    text = text(context),
    tone = tone(),
    tagType = when (this) {
        is GemLatencyStatus.Loading -> ListItemTagType.Progress
        is GemLatencyStatus.Error, is GemLatencyStatus.Result -> ListItemTagType.None
    },
)
