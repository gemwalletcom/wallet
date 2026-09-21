package com.gemwallet.android.features.nft.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.nft.viewmodels.localization.stringRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.ReportReason
import com.wallet.core.primitives.VerificationStatus
import uniffi.gemstone.GemCollectibleAction
import uniffi.gemstone.GemCollectibleAttributeValue
import uniffi.gemstone.GemCollectibleSection
import uniffi.gemstone.GemListRow
import java.text.DateFormat
import java.util.Date

data class NftDetailsUIModel(val asset: NFTAsset, val isVerified: Boolean, val canSend: Boolean, val actions: List<GemCollectibleAction>, val sections: List<NftSectionUIModel>)

sealed interface NftSectionUIModel {
    data class Status(val status: VerificationStatus) : NftSectionUIModel
    data class Info(val rows: List<GemListRow>) : NftSectionUIModel
    data class Attributes(val title: String, val rows: List<ListItemModel>) : NftSectionUIModel
    data class Links(val title: String, val row: GemListRow) : NftSectionUIModel
}

data class ReportReasonUIModel(val reason: ReportReason, val model: ListItemModel)

internal fun NftAssetDetailsData.uiModel(context: Context): NftDetailsUIModel = NftDetailsUIModel(
    asset = asset,
    isVerified = collection.status == VerificationStatus.Verified,
    canSend = details.canSend,
    actions = details.actions,
    sections = details.sections.mapNotNull { it.uiModel(context) },
)

internal fun ReportReason.uiModel(context: Context): ReportReasonUIModel = ReportReasonUIModel(
    reason = this,
    model = ListItemModel(title = context.getString(stringRes())),
)

private fun GemCollectibleSection.uiModel(context: Context): NftSectionUIModel? = when (this) {
    is GemCollectibleSection.Status -> NftSectionUIModel.Status(status.toPrimitives())

    is GemCollectibleSection.Info -> NftSectionUIModel.Info(rows)

    is GemCollectibleSection.Attributes -> NftSectionUIModel.Attributes(
        title = context.getString(R.string.nft_properties),
        rows = attributes.map { ListItemModel(title = it.name, subtitle = it.value.text()) },
    )

    is GemCollectibleSection.Links -> NftSectionUIModel.Links(title = context.getString(R.string.social_links), row = GemListRow.Social(links))
}

private fun GemCollectibleAttributeValue.text(): String = when (this) {
    is GemCollectibleAttributeValue.Text -> value
    is GemCollectibleAttributeValue.Date -> DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(date))
}
