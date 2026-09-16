package com.gemwallet.android.features.perpetual.views.components

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.features.perpetual.localization.stringRes
import uniffi.gemstone.GemPerpetualInfoRow

fun LazyListScope.perpetualInfo(data: PerpetualDetailsDataAggregate, rows: List<GemPerpetualInfoRow>) {
    itemsPositioned(rows) { position, row ->
        when (row) {
            GemPerpetualInfoRow.DAILY_VOLUME -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = data.dayVolume,
                listPosition = position,
            )
            GemPerpetualInfoRow.OPEN_INTEREST -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = data.openInterest,
                info = InfoSheetEntity.OpenInterestInfo,
                listPosition = position,
            )
            GemPerpetualInfoRow.FUNDING_RATE -> PropertyItem(
                title = stringResource(row.stringRes()),
                data = data.funding,
                info = InfoSheetEntity.FundingAprInfo,
                listPosition = position,
            )
        }
    }
}