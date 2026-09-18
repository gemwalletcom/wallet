package com.gemwallet.android.ui.components.list_item

import android.content.Context
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.icon
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.titleRes
import com.wallet.core.primitives.Asset
import com.gemwallet.android.ui.components.InfoSheetEntity
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowIcon
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemSocialLink

internal sealed interface GemListRowUIModel {
    data class Item(val model: ListItemModel, val url: String? = null, val opensAnotherScreen: Boolean = false) : GemListRowUIModel
    data class Icon(val asset: Asset) : GemListRowUIModel
    data class Address(val address: String, val copy: GemCopy) : GemListRowUIModel
    data class Social(val links: List<GemSocialLink>) : GemListRowUIModel
    data class Toggle(val model: ListItemModel, val title: GemListRowTitle, val isOn: Boolean) : GemListRowUIModel
    data class Picker(val model: ListItemModel, val title: GemListRowTitle) : GemListRowUIModel
    data object Loading : GemListRowUIModel
}

internal fun GemListRow.uiModel(context: Context): GemListRowUIModel = when (this) {
    is GemListRow.Text -> GemListRowUIModel.Item(ListItemModel(title = context.getString(title.titleRes()), subtitle = value))
    is GemListRow.Amount -> GemListRowUIModel.Item(ListItemModel(title = context.getString(title.titleRes()), subtitle = amount.text(), info = info?.infoSheet()))
    is GemListRow.Link -> GemListRowUIModel.Item(listItemModel(context, title, value, icon), opensAnotherScreen = true)
    is GemListRow.Url -> GemListRowUIModel.Item(listItemModel(context, title, value, icon), url = url)
    is GemListRow.Explorer -> GemListRowUIModel.Item(ListItemModel(title = context.getString(R.string.transaction_view_on, name)), url = url)
    is GemListRow.Error -> GemListRowUIModel.Item(
        ListItemModel(title = context.getString(GemListRowTitle.ERROR.titleRes()), subtitle = error.message, titleStyle = ListItemTextStyle.Negative),
    )
    is GemListRow.Icon -> GemListRowUIModel.Icon(asset = chain.requireChain().asset())
    is GemListRow.Address -> GemListRowUIModel.Address(address = address, copy = copy)
    is GemListRow.Toggle -> GemListRowUIModel.Toggle(listItemModel(context, title, null, icon), title, isOn)
    is GemListRow.Picker -> GemListRowUIModel.Picker(listItemModel(context, title, value, icon), title)
    is GemListRow.Social -> GemListRowUIModel.Social(links)
    GemListRow.Loading -> GemListRowUIModel.Loading
}

internal fun GemSocialLink.uiModel(context: Context): GemListRowUIModel.Item = GemListRowUIModel.Item(
    ListItemModel(title = context.getString(linkType.stringRes()), image = ListItemImage.Drawable(linkType.icon)),
    url = url,
)

private fun listItemModel(context: Context, title: GemListRowTitle, value: String?, icon: GemListRowIcon): ListItemModel = ListItemModel(
    title = context.getString(title.titleRes()),
    subtitle = value,
    image = icon.image(),
)

private fun GemListRowIcon.image(): ListItemImage? = when (this) {
    GemListRowIcon.NONE -> null
    GemListRowIcon.APP_LOGO -> ListItemImage.Drawable(R.drawable.ic_gem_foreground)
    GemListRowIcon.WALLETS -> ListItemImage.Drawable(R.drawable.settings_wallets)
    GemListRowIcon.SECURITY -> ListItemImage.Drawable(R.drawable.settings_security)
    GemListRowIcon.NOTIFICATIONS -> ListItemImage.Drawable(R.drawable.settings_notifications)
    GemListRowIcon.PREFERENCES -> ListItemImage.Drawable(R.drawable.settings_preferences)
    GemListRowIcon.WALLET_CONNECT -> ListItemImage.Drawable(R.drawable.settings_wc)
    GemListRowIcon.SUPPORT -> ListItemImage.Drawable(R.drawable.settings_support)
    GemListRowIcon.REWARDS -> ListItemImage.Drawable(R.drawable.settings_wallets)
    GemListRowIcon.ABOUT_US -> ListItemImage.Drawable(R.drawable.settings_about_us)
    GemListRowIcon.DEVELOPER -> ListItemImage.Drawable(R.drawable.settings_developer)
    GemListRowIcon.CURRENCY -> ListItemImage.Drawable(R.drawable.settings_currency)
    GemListRowIcon.LANGUAGE -> ListItemImage.Drawable(R.drawable.settings_language)
    GemListRowIcon.APPEARANCE -> ListItemImage.Drawable(R.drawable.settings_appearance)
    GemListRowIcon.NETWORKS -> ListItemImage.Drawable(R.drawable.settings_networks)
    GemListRowIcon.CONTACTS -> ListItemImage.Drawable(R.drawable.settings_contacts)
    GemListRowIcon.PERPETUALS -> ListItemImage.Drawable(R.drawable.settings_pricealert)
}

private fun GemInfoTopic.infoSheet(): InfoSheetEntity = when (this) {
    GemInfoTopic.OPEN_INTEREST -> InfoSheetEntity.OpenInterestInfo
    GemInfoTopic.FUNDING_APR -> InfoSheetEntity.FundingAprInfo
}
