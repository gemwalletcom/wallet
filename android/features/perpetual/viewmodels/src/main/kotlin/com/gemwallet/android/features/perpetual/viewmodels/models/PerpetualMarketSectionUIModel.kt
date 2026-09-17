package com.gemwallet.android.features.perpetual.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.perpetual.viewmodels.localization.stringRes
import uniffi.gemstone.GemPerpetualMarketSection

sealed interface PerpetualMarketSectionUIModel {
    data object Recents : PerpetualMarketSectionUIModel
    data class Positions(val title: String?) : PerpetualMarketSectionUIModel
    data object Pinned : PerpetualMarketSectionUIModel
    data class Markets(val title: String?) : PerpetualMarketSectionUIModel
    data object Empty : PerpetualMarketSectionUIModel
}

internal fun GemPerpetualMarketSection.uiModel(context: Context): PerpetualMarketSectionUIModel {
    val title = stringRes()?.let { context.getString(it) }
    return when (this) {
        GemPerpetualMarketSection.RECENTS -> PerpetualMarketSectionUIModel.Recents
        GemPerpetualMarketSection.POSITIONS -> PerpetualMarketSectionUIModel.Positions(title)
        GemPerpetualMarketSection.PINNED -> PerpetualMarketSectionUIModel.Pinned
        GemPerpetualMarketSection.MARKETS -> PerpetualMarketSectionUIModel.Markets(title)
        GemPerpetualMarketSection.EMPTY -> PerpetualMarketSectionUIModel.Empty
    }
}
