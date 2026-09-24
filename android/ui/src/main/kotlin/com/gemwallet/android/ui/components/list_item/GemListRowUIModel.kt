package com.gemwallet.android.ui.components.list_item

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.duration.formatDuration
import com.gemwallet.android.domains.duration.formatEstimate
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.property.icon
import com.gemwallet.android.ui.format.rowDateFormatter
import com.gemwallet.android.ui.localization.infoDescriptionRes
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.style.badgeIconRes
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemCopyKind
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowIcon
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemNoticeKind
import uniffi.gemstone.GemSocialLink
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.GemValueTone
import java.time.ZoneId
import java.util.Locale

internal sealed interface GemListRowUIModel {
    data class Notice(val title: String, val message: String?, val kind: GemNoticeKind) : GemListRowUIModel
    data class Item(val model: ListItemModel, val url: String? = null, val opensAnotherScreen: Boolean = false, val trailingImage: ListItemImage? = null, val menu: List<GemListRowMenuItem> = emptyList(), val address: String? = null) :
        GemListRowUIModel
    data class Provider(val model: ListItemModel, val contract: String?) : GemListRowUIModel
    data class Icon(val asset: Asset) : GemListRowUIModel
    data class Network(val chain: Chain, val name: String) : GemListRowUIModel
    data class Address(val address: String, val copy: GemCopy) : GemListRowUIModel
    data class Social(val links: List<GemSocialLink>) : GemListRowUIModel
    data class Toggle(val model: ListItemModel, val title: GemListRowTitle, val isOn: Boolean) : GemListRowUIModel
    data class Picker(val model: ListItemModel, val title: GemListRowTitle) : GemListRowUIModel
    data object Loading : GemListRowUIModel
}

internal sealed interface GemListRowMenuItem {
    val title: String

    data class Copy(override val title: String, val value: String) : GemListRowMenuItem
    data class Open(override val title: String, val url: String) : GemListRowMenuItem
}

internal fun GemListRow.uiModel(context: Context, infoIcon: Any? = null): GemListRowUIModel = when (this) {
    is GemListRow.Latency -> GemListRowUIModel.Item(status.listItemModel(context, title.text(context) + titleSuffix, host))

    is GemListRow.Notice -> GemListRowUIModel.Notice(title = title.text(context), message = message?.string(context), kind = kind)

    is GemListRow.Text -> GemListRowUIModel.Item(ListItemModel(title = title.text(context), subtitle = value))

    is GemListRow.Provider -> GemListRowUIModel.Provider(
        ListItemModel(title = title.text(context), subtitle = name),
        contract = contract,
    )

    is GemListRow.Amount -> GemListRowUIModel.Item(
        ListItemModel(title = title.text(context), subtitle = amount.text(), subtitleStyle = amount.tone.subtitleStyle(), info = info?.infoSheet(context, infoIcon)),
    )

    is GemListRow.Rate -> GemListRowUIModel.Item(ListItemModel(title = title.text(context), subtitle = rate.text(rate.value.text())))

    is GemListRow.Action -> GemListRowUIModel.Item(
        ListItemModel(
            title = title.text(context),
            titleStyle = if (info == null) ListItemTextStyle.Body else ListItemTextStyle.Faded,
            subtitle = value?.text(),
            info = info?.infoSheet(context, infoIcon),
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

    is GemListRow.Ranked -> GemListRowUIModel.Item(ListItemModel(title = title.text(context), subtitle = amount.text(), titleTag = "#$rank"))

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
            info = info?.infoSheet(context, infoIcon),
        ),
    )

    is GemListRow.Label -> GemListRowUIModel.Item(
        ListItemModel(
            title = title.text(context),
            subtitle = text.string(context),
            subtitleStyle = tone.subtitleStyle(),
            subtitleTagType = if (progress) ListItemTagType.Progress else ListItemTagType.None,
            info = info?.infoSheet(context, infoIcon),
        ),
    )

    is GemListRow.Date -> GemListRowUIModel.Item(ListItemModel(title = title.text(context), subtitle = context.rowDateFormatter().row(date, ZoneId.systemDefault(), Locale.getDefault())))

    is GemListRow.Network -> GemListRowUIModel.Network(chain.requireChain(), name)

    is GemListRow.App -> GemListRowUIModel.Item(
        ListItemModel(title = context.getString(R.string.wallet_connect_app), subtitle = name),
        trailingImage = iconUrl?.let { ListItemImage.Url(it) },
        menu = listOfNotNull(websiteUrl?.let { GemListRowMenuItem.Open(context.getString(R.string.settings_website), it) }),
    )

    is GemListRow.Wallet -> GemListRowUIModel.Item(
        ListItemModel(title = context.getString(R.string.common_wallet), subtitle = wallet.name),
        trailingImage = wallet.listItemImage(),
        menu = listOf(
            GemListRowMenuItem.Copy(context.getString(R.string.wallet_copy_address), copy.value),
            GemListRowMenuItem.Open(context.getString(R.string.transaction_view_on, explorer.name), explorer.link),
        ),
    )

    is GemListRow.Memo -> GemListRowUIModel.Item(
        ListItemModel(title = context.getString(R.string.transfer_memo), subtitle = value),
        menu = listOfNotNull(copy?.let { GemListRowMenuItem.Copy(context.getString(R.string.common_copy), it) }),
    )

    is GemListRow.Link -> GemListRowUIModel.Item(listItemModel(context, title, value, icon), opensAnotherScreen = true)

    is GemListRow.Url -> GemListRowUIModel.Item(listItemModel(context, title, value, icon), url = url)

    is GemListRow.Lines -> GemListRowUIModel.Item(
        ListItemModel(
            title = title.text(context),
            subtitle = lines.firstOrNull()?.string(context),
            subtitleExtra = lines.getOrNull(1)?.string(context),
            info = info?.infoSheet(context, infoIcon),
        ),
    )

    is GemListRow.Identifier -> GemListRowUIModel.Item(
        ListItemModel(title = title.text(context), subtitle = copy.display),
        url = explorer?.link,
        menu = listOfNotNull(
            GemListRowMenuItem.Copy(context.getString(copy.kind.copyTitleRes()), copy.value),
            explorer?.let { GemListRowMenuItem.Open(context.getString(R.string.transaction_view_on, it.name), it.link) },
        ),
        address = copy.value.takeIf { title == GemListRowTitle.CONTRACT },
    )

    is GemListRow.Explorer -> GemListRowUIModel.Item(ListItemModel(title = context.getString(R.string.transaction_view_on, name)), url = url)

    is GemListRow.Error -> GemListRowUIModel.Notice(title = GemListRowTitle.ERROR.text(context), message = error.errorText().text(context), kind = GemNoticeKind.ERROR)

    is GemListRow.Icon -> GemListRowUIModel.Icon(asset = chain.requireChain().asset())

    is GemListRow.Address -> GemListRowUIModel.Address(address = address, copy = copy)

    is GemListRow.Toggle -> GemListRowUIModel.Toggle(listItemModel(context, title, null, icon), title, isOn)

    is GemListRow.Picker -> GemListRowUIModel.Picker(listItemModel(context, title, value.string(context), icon), title)

    is GemListRow.Social -> GemListRowUIModel.Social(links)

    GemListRow.Loading -> GemListRowUIModel.Loading
}

internal fun GemSocialLink.uiModel(context: Context): GemListRowUIModel.Item = GemListRowUIModel.Item(
    ListItemModel(title = context.getString(linkType.stringRes()), subtitle = host, image = ListItemImage.Drawable(linkType.icon)),
    url = url,
)

private fun listItemModel(context: Context, title: GemListRowTitle, value: String?, icon: GemListRowIcon): ListItemModel = ListItemModel(
    title = title.text(context),
    subtitle = value,
    image = icon.image(),
)

private fun GemCopyKind.copyTitleRes(): Int = when (this) {
    is GemCopyKind.Address -> R.string.wallet_copy_address
    GemCopyKind.Plain, GemCopyKind.SecretPhrase, GemCopyKind.PrivateKey -> R.string.common_copy
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

fun GemInfoTopic.infoSheet(context: Context, icon: Any?, onBuy: (() -> Unit)? = null): InfoSheetEntity = when (this) {
    is GemInfoTopic.NetworkFee -> asset.toPrimitives().let { InfoSheetEntity.NetworkFeeInfo(it.chain.networkName(), it.symbol) }

    is GemInfoTopic.MinimumAmount -> asset.toPrimitives().let {
        InfoSheetEntity.MinimumAmountInfo(
            networkTitle = it.chain.networkName(),
            value = ValueFormatter(style = GemValueStyle.FULL).string(minimum, it.decimals, it.symbol),
            actionLabel = onBuy?.let { _ -> context.getString(R.string.asset_buy_asset, it.symbol) },
            action = onBuy,
        )
    }

    GemInfoTopic.NoQuote -> InfoSheetEntity.NoQuoteInfo

    GemInfoTopic.PriceImpact -> InfoSheetEntity.PriceImpactInfo

    GemInfoTopic.Slippage -> InfoSheetEntity.Slippage

    GemInfoTopic.OpenInterest -> InfoSheetEntity.OpenInterestInfo

    GemInfoTopic.FundingApr -> InfoSheetEntity.FundingAprInfo

    GemInfoTopic.StakeApr -> InfoSheetEntity.StakeAprInfo(icon)

    GemInfoTopic.StakeLockTime -> InfoSheetEntity.StakeLockTimeInfo(icon)

    GemInfoTopic.StakeFrozenRequired -> InfoSheetEntity.StakeFrozenRequired(icon)

    GemInfoTopic.AutoClose -> InfoSheetEntity.AutoCloseInfo

    GemInfoTopic.LiquidationPrice -> InfoSheetEntity.LiquidationPriceInfo

    GemInfoTopic.FundingPayments -> InfoSheetEntity.FundingPayments

    GemInfoTopic.FullyDilutedValuation -> InfoSheetEntity.FullyDilutedValuation

    GemInfoTopic.CirculatingSupply -> InfoSheetEntity.CirculatingSupply

    GemInfoTopic.TotalSupply -> InfoSheetEntity.TotalSupply

    GemInfoTopic.MaxSupply -> InfoSheetEntity.MaxSupply

    is GemInfoTopic.EstimatedConfirmation -> InfoSheetEntity.EstimatedConfirmationInfo(chain.requireChain())

    is GemInfoTopic.TransactionStatus -> InfoSheetEntity.TransactionInfo(
        icon = icon,
        state = state.toPrimitives(),
        badgeIcon = tone.badgeIconRes(),
        description = tone.infoDescriptionRes(),
    )
}

fun GemListRow.listItemModel(context: Context, infoIcon: Any? = null): ListItemModel? = (uiModel(context, infoIcon) as? GemListRowUIModel.Item)?.model

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
