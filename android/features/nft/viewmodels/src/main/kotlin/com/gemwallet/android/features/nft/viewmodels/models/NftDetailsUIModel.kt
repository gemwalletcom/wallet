package com.gemwallet.android.features.nft.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.titleRes
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.ReportReason
import com.wallet.core.primitives.VerificationStatus
import uniffi.gemstone.GemCollectibleAction
import uniffi.gemstone.GemCollectibleAttributeValue
import uniffi.gemstone.GemCollectibleSection
import uniffi.gemstone.GemCollectibleSectionGroup
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemListRow
import java.text.DateFormat
import java.util.Date

data class NftDetailsUIModel(val asset: NFTAsset, val isVerified: Boolean, val header: GemHeaderActions, val actions: List<NftActionUIModel>, val sections: List<NftSectionUIModel>)

data class NftActionUIModel(val action: GemCollectibleAction, val title: String, val isDestructive: Boolean)

sealed interface NftSectionUIModel {
    data class Status(val status: VerificationStatus) : NftSectionUIModel
    data class Info(val rows: List<GemListRow>) : NftSectionUIModel
    data class Attributes(val title: String, val rows: List<ListItemModel>) : NftSectionUIModel
    data class Links(val title: String, val row: GemListRow) : NftSectionUIModel
}

data class ReportReasonUIModel(val reason: ReportReason, val model: ListItemModel)

internal fun NftAssetDetailsData.uiModel(context: Context): NftDetailsUIModel = NftDetailsUIModel(
    asset = asset,
    isVerified = details.isVerified,
    header = details.header,
    actions = details.actions.map { NftActionUIModel(it, context.getString(it.stringRes()), it == GemCollectibleAction.REPORT) },
    sections = details.sections.map { it.uiModel(context) },
)

internal fun ReportReason.uiModel(context: Context): ReportReasonUIModel = ReportReasonUIModel(
    reason = this,
    model = ListItemModel(title = context.getString(stringRes())),
)

private fun GemCollectibleSectionGroup.uiModel(context: Context): NftSectionUIModel {
    val title = title.titleRes()?.let(context::getString).orEmpty()
    return when (val section = section) {
        is GemCollectibleSection.Status -> NftSectionUIModel.Status(section.status.toPrimitives())

        is GemCollectibleSection.Info -> NftSectionUIModel.Info(section.rows)

        is GemCollectibleSection.Attributes -> NftSectionUIModel.Attributes(
            title = title,
            rows = section.attributes.map { ListItemModel(title = it.name, subtitle = it.value.text()) },
        )

        is GemCollectibleSection.Links -> NftSectionUIModel.Links(title = title, row = GemListRow.Social(section.links))
    }
}

private fun GemCollectibleAttributeValue.text(): String = when (this) {
    is GemCollectibleAttributeValue.Text -> value
    is GemCollectibleAttributeValue.Date -> DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(date))
}
