package com.gemwallet.android.ui.components.list_item

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.duration.formatDuration
import com.gemwallet.android.domains.duration.formatEstimate
import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.domains.swap.AssetRatePair
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.property.icon
import com.gemwallet.android.ui.format.rowDateFormatter
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.style.listItemImage
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.style.walletListItemImage
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemAvatar
import uniffi.gemstone.GemConnectionRow
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowIcon
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemNoticeKind
import uniffi.gemstone.GemRowAction
import uniffi.gemstone.GemRowMenuItem
import uniffi.gemstone.GemSocialLink
import uniffi.gemstone.GemValueTone
import java.time.ZoneId
import java.util.Locale

internal sealed interface GemListRowUIModel {
    data class Notice(val title: String, val message: String?, val kind: GemNoticeKind) : GemListRowUIModel
    data class Item(val model: ListItemModel, val url: String? = null, val opensAnotherScreen: Boolean = false, val trailingImage: ListItemImage? = null, val menu: List<GemListRowMenuItem> = emptyList(), val address: String? = null) :
        GemListRowUIModel
    data class Provider(val model: ListItemModel, val contract: String?) : GemListRowUIModel
    data class Rate(val title: String, val rate: AssetRatePair) : GemListRowUIModel
    data class Icon(val icon: GemAssetIcon, val imageUrl: String?) : GemListRowUIModel
    data class Avatar(val image: ListItemImage) : GemListRowUIModel
    data class Address(val address: String, val copy: GemCopy, val menu: List<GemListRowMenuItem>) : GemListRowUIModel
    data class Network(val chain: Chain, val name: String) : GemListRowUIModel
    data class Social(val links: List<GemSocialLink>) : GemListRowUIModel
    data class Toggle(val model: ListItemModel, val action: GemRowAction, val isOn: Boolean) : GemListRowUIModel
    data class Picker(val model: ListItemModel, val action: GemRowAction) : GemListRowUIModel
    data object Loading : GemListRowUIModel
}

internal sealed interface GemListRowMenuItem {
    val title: String

    data class Copy(override val title: String, val value: String) : GemListRowMenuItem
    data class Open(override val title: String, val url: String) : GemListRowMenuItem
}

internal fun GemListRow.uiModel(context: Context): GemListRowUIModel = when (this) {
    is GemListRow.Latency -> GemListRowUIModel.Item(status.listItemModel(context, title.text(context) + titleSuffix, host))

    is GemListRow.Notice -> GemListRowUIModel.Notice(title = title.text(context), message = message?.string(context), kind = kind)

    is GemListRow.Text -> GemListRowUIModel.Item(ListItemModel(title = title.text(context), subtitle = value))

    is GemListRow.Provider -> GemListRowUIModel.Provider(
        ListItemModel(title = title.text(context), subtitle = name),
        contract = contract,
    )

    is GemListRow.Amount -> GemListRowUIModel.Item(
        ListItemModel(title = title.text(context), subtitle = amount.text(), subtitleStyle = amount.tone.subtitleStyle(), info = info?.infoSheet()),
    )

    is GemListRow.Rate -> inverse?.let { inverse ->
        GemListRowUIModel.Rate(title.text(context), AssetRatePair(forward = AssetRateFormatter().format(rate), reverse = AssetRateFormatter().format(inverse)))
    } ?: GemListRowUIModel.Item(ListItemModel(title = title.text(context), subtitle = AssetRateFormatter().format(rate)))

    is GemListRow.Action -> GemListRowUIModel.Item(
        ListItemModel(
            title = title.text(context),
            titleStyle = if (info == null) ListItemTextStyle.Body else ListItemTextStyle.Faded,
            subtitle = value?.text(),
            info = info?.infoSheet(),
        ),
    )

    is GemListRow.Quote -> GemListRowUIModel.Item(
        ListItemModel(
            title = title.text(context),
            subtitle = value?.text(),
            subtitleSuffix = change?.text(),
            subtitleSuffixStyle = change?.tone?.textStyle() ?: ListItemTextStyle.Secondary,
        ),
    )

    is GemListRow.Ranked -> GemListRowUIModel.Item(ListItemModel(title = title.text(context), subtitle = amount.text(), titleTag = tag))

    is GemListRow.AllTime -> GemListRowUIModel.Item(
        ListItemModel(
            title = title.text(context),
            titleExtra = context.rowDateFormatter().section(date, ZoneId.systemDefault(), Locale.getDefault()),
            subtitle = value.text(),
            subtitleExtra = change.text(),
            subtitleExtraStyle = change.tone.textStyle(),
        ),
    )

    is GemListRow.Duration -> GemListRowUIModel.Item(
        ListItemModel(
            title = title.text(context),
            subtitle = if (estimate) parts.formatEstimate() else parts.formatDuration(),
            info = info?.infoSheet(),
        ),
    )

    is GemListRow.Label -> GemListRowUIModel.Item(
        ListItemModel(
            title = title.text(context),
            subtitle = text.string(context),
            subtitleStyle = tone.subtitleStyle(),
            subtitleTagType = if (progress) ListItemTagType.Progress else ListItemTagType.None,
            info = info?.infoSheet(),
        ),
    )

    is GemListRow.Date -> GemListRowUIModel.Item(ListItemModel(title = title.text(context), subtitle = context.rowDateFormatter().row(date, ZoneId.systemDefault(), Locale.getDefault())))

    is GemListRow.Network -> GemListRowUIModel.Network(chain.requireChain(), name)

    is GemListRow.App -> GemListRowUIModel.Item(
        ListItemModel(title = title.text(context), subtitle = name),
        trailingImage = iconUrl?.let { ListItemImage.Url(it) },
        menu = menu.map { it.uiModel(context) },
    )

    is GemListRow.Wallet -> GemListRowUIModel.Item(
        ListItemModel(title = title.text(context), subtitle = wallet.name),
        trailingImage = wallet.listItemImage(),
        menu = menu.map { it.uiModel(context) },
    )

    is GemListRow.Memo -> GemListRowUIModel.Item(
        ListItemModel(title = title.text(context), subtitle = value),
        menu = menu.map { it.uiModel(context) },
    )

    is GemListRow.Link -> GemListRowUIModel.Item(listItemModel(context, title, value, icon), opensAnotherScreen = true)

    is GemListRow.Url -> GemListRowUIModel.Item(listItemModel(context, title, value, icon), url = url)

    is GemListRow.Lines -> GemListRowUIModel.Item(
        ListItemModel(
            title = title.text(context),
            subtitle = lines.firstOrNull()?.string(context),
            subtitleExtra = lines.getOrNull(1)?.string(context),
            info = info?.infoSheet(),
        ),
    )

    is GemListRow.Identifier -> GemListRowUIModel.Item(
        ListItemModel(title = title.text(context), subtitle = copy.display),
        url = explorer?.link,
        menu = menu.map { it.uiModel(context) },
        address = address,
    )

    is GemListRow.Explorer -> GemListRowUIModel.Item(ListItemModel(title = title.string(context)), url = url)

    is GemListRow.Error -> GemListRowUIModel.Notice(title = GemListRowTitle.ERROR.text(context), message = error.errorText().text(context), kind = GemNoticeKind.ERROR)

    is GemListRow.Icon -> GemListRowUIModel.Icon(icon = icon, imageUrl = imageUrl)

    is GemListRow.AssetChange -> GemListRowUIModel.Item(
        ListItemModel(title = name, subtitle = amount.text(), subtitleStyle = amount.tone.textStyle(), image = ListItemImage.Asset(icon)),
    )

    is GemListRow.Avatar -> GemListRowUIModel.Avatar(image = avatar.listItemImage())

    is GemListRow.WalletAvatar -> GemListRowUIModel.Avatar(image = walletListItemImage(imageUrl, placeholder))

    is GemListRow.Address -> GemListRowUIModel.Address(
        address = address,
        copy = copy,
        menu = listOf(GemRowMenuItem.Copy(copy).uiModel(context)),
    )

    is GemListRow.Toggle -> GemListRowUIModel.Toggle(ListItemModel(title = label.string(context), image = icon.image()), action, isOn)

    is GemListRow.Picker -> GemListRowUIModel.Picker(listItemModel(context, title, value.string(context), icon), action)

    is GemListRow.Social -> GemListRowUIModel.Social(links)

    GemListRow.Loading -> GemListRowUIModel.Loading
}

fun GemConnectionRow.listItem(): ListItemModel = ListItemModel(
    title = title,
    titleExtra = host,
    image = iconUrl?.let { ListItemImage.Url(it, placeholder = initial) } ?: ListItemImage.Initials(initial),
)

fun GemAvatar.listItemImage(): ListItemImage = imageUrl?.let { ListItemImage.Stored(it, initials) } ?: ListItemImage.Initials(initials)

internal fun GemSocialLink.uiModel(context: Context): GemListRowUIModel.Item = GemListRowUIModel.Item(
    ListItemModel(title = context.getString(linkType.stringRes()), subtitle = host, image = ListItemImage.Drawable(linkType.icon)),
    url = url,
)

private fun listItemModel(context: Context, title: GemListRowTitle, value: String?, icon: GemListRowIcon): ListItemModel = ListItemModel(
    title = title.text(context),
    subtitle = value,
    image = icon.image(),
)

private fun GemRowMenuItem.uiModel(context: Context): GemListRowMenuItem = when (this) {
    is GemRowMenuItem.Copy -> GemListRowMenuItem.Copy(context.getString(R.string.common_copy), copy.value)
    is GemRowMenuItem.Open -> GemListRowMenuItem.Open(title.string(context), url)
}

private fun GemValueTone.subtitleStyle(): ListItemTextStyle = when (this) {
    GemValueTone.PLAIN -> ListItemTextStyle.Secondary
    GemValueTone.NEUTRAL, GemValueTone.POSITIVE, GemValueTone.WARNING, GemValueTone.NEGATIVE -> textStyle()
}

private fun GemListRowIcon.image(): ListItemImage? = when (this) {
    GemListRowIcon.NONE -> null
    GemListRowIcon.APP_LOGO -> ListItemImage.Drawable(R.drawable.ic_gem_foreground)
    GemListRowIcon.WALLETS -> ListItemImage.Drawable(R.drawable.settings_wallets)
    GemListRowIcon.SECURITY -> ListItemImage.Drawable(R.drawable.settings_security)
    GemListRowIcon.NOTIFICATIONS -> ListItemImage.Drawable(R.drawable.settings_notifications)
    GemListRowIcon.PRICE_ALERTS -> ListItemImage.Drawable(R.drawable.settings_pricealert)
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
    GemListRowIcon.PIN -> ListItemImage.Symbol(ListItemSymbol.Pin)
    GemListRowIcon.UNPIN -> ListItemImage.Symbol(ListItemSymbol.Unpin)
    GemListRowIcon.ADD_TO_WALLET -> ListItemImage.Symbol(ListItemSymbol.AddCircle)
}

fun GemListRow.listItemModel(context: Context): ListItemModel? = (uiModel(context) as? GemListRowUIModel.Item)?.model

fun GemLatencyStatus.listItemModel(context: Context, title: String, titleExtra: String?): ListItemModel = ListItemModel(
    title = title,
    titleTag = text(context),
    titleTagStyle = tone().textStyle(),
    titleTagType = when (this) {
        GemLatencyStatus.Loading -> ListItemTagType.Progress
        GemLatencyStatus.Error, is GemLatencyStatus.Result -> ListItemTagType.None
    },
    titleExtra = titleExtra,
    titleExtraStyle = ListItemTextStyle.Body,
)
