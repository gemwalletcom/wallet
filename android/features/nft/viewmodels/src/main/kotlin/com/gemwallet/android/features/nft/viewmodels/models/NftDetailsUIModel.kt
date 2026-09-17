package com.gemwallet.android.features.nft.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.nft.NftAssetDetailsData
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.nft.viewmodels.localization.stringRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.property.LinkRowUIModel
import com.gemwallet.android.ui.components.list_item.property.linkRows
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.ReportReason
import com.wallet.core.primitives.VerificationStatus
import uniffi.gemstone.GemCollectibleAttributeValue
import uniffi.gemstone.GemCollectibleIdentifier
import uniffi.gemstone.GemCollectibleRow
import uniffi.gemstone.GemCollectibleSection
import uniffi.gemstone.socialLinks
import java.text.DateFormat
import java.util.Date

data class NftDetailsUIModel(
    val asset: NFTAsset,
    val isVerified: Boolean,
    val canSend: Boolean,
    val sections: List<NftSectionUIModel>,
)

sealed interface NftSectionUIModel {
    data class Status(val status: VerificationStatus) : NftSectionUIModel
    data class Info(val rows: List<NftInfoRowUIModel>) : NftSectionUIModel
    data class Attributes(val title: String, val rows: List<ListItemModel>) : NftSectionUIModel
    data class Links(val title: String, val links: List<LinkRowUIModel>) : NftSectionUIModel
}

sealed interface NftInfoRowUIModel {
    data class Item(val model: ListItemModel) : NftInfoRowUIModel
    data class Network(val chain: Chain) : NftInfoRowUIModel
    data class Identifier(val title: String, val identifier: GemCollectibleIdentifier) : NftInfoRowUIModel
}

data class ReportReasonUIModel(
    val reason: ReportReason,
    val model: ListItemModel,
)

internal fun NftAssetDetailsData.uiModel(context: Context): NftDetailsUIModel = NftDetailsUIModel(
    asset = asset,
    isVerified = collection.status == VerificationStatus.Verified,
    canSend = details.canSend,
    sections = details.sections.mapNotNull { it.uiModel(context) },
)

internal fun ReportReason.uiModel(context: Context): ReportReasonUIModel = ReportReasonUIModel(
    reason = this,
    model = ListItemModel(title = context.getString(stringRes())),
)

private fun GemCollectibleSection.uiModel(context: Context): NftSectionUIModel? = when (this) {
    is GemCollectibleSection.Status -> NftSectionUIModel.Status(status.toPrimitives())
    is GemCollectibleSection.Info -> NftSectionUIModel.Info(rows.map { it.uiModel(context) })
    is GemCollectibleSection.Attributes -> NftSectionUIModel.Attributes(
        title = context.getString(R.string.nft_properties),
        rows = attributes.map { ListItemModel(title = it.name, subtitle = it.value.text()) },
    )
    is GemCollectibleSection.Links -> socialLinks(links).linkRows(context)
        .takeIf { it.isNotEmpty() }
        ?.let { NftSectionUIModel.Links(title = context.getString(R.string.social_links), links = it) }
}

private fun GemCollectibleRow.uiModel(context: Context): NftInfoRowUIModel = when (this) {
    is GemCollectibleRow.Collection -> NftInfoRowUIModel.Item(ListItemModel(title = context.getString(stringRes()), subtitle = name))
    is GemCollectibleRow.Network -> NftInfoRowUIModel.Network(chain.toChain())
    is GemCollectibleRow.Contract -> NftInfoRowUIModel.Identifier(context.getString(stringRes()), identifier)
    is GemCollectibleRow.TokenId -> NftInfoRowUIModel.Identifier(context.getString(stringRes()), identifier)
}

private fun GemCollectibleAttributeValue.text(): String = when (this) {
    is GemCollectibleAttributeValue.Text -> value
    is GemCollectibleAttributeValue.Date -> DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(date))
}
