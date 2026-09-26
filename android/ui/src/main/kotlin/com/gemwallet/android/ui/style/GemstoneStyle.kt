package com.gemwallet.android.ui.style

import android.content.Context
import androidx.annotation.DrawableRes
import androidx.compose.material3.ButtonColors
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.input.KeyboardType
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.secondaryActionButtonColors
import com.gemwallet.android.ui.components.empty.EmptyStateImage
import com.gemwallet.android.ui.components.fields.AmountSymbolPlacement
import com.gemwallet.android.ui.components.fields.AmountSymbolUIModel
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.image.supportIconModel
import com.gemwallet.android.ui.components.image.walletImageModel
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemImageStyle
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.SwapProgressMarkerUIModel
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.Emoji
import com.gemwallet.android.ui.theme.pendingColor
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.VerificationStatus
import uniffi.gemstone.ChainAddress
import uniffi.gemstone.GemAcquireOption
import uniffi.gemstone.GemAddressFormatStyle
import uniffi.gemstone.GemAddressServiceInterface
import uniffi.gemstone.GemAmountField
import uniffi.gemstone.GemAmountKeyboard
import uniffi.gemstone.GemAmountSymbol
import uniffi.gemstone.GemAmountSymbolPlacement
import uniffi.gemstone.GemBannerButton
import uniffi.gemstone.GemBannerIcon
import uniffi.gemstone.GemEmptyStateImage
import uniffi.gemstone.GemFiatTransactionBadge
import uniffi.gemstone.GemHeaderButtonKind
import uniffi.gemstone.GemInfoImage
import uniffi.gemstone.GemKeystoreAuthentication
import uniffi.gemstone.GemNameIndicator
import uniffi.gemstone.GemNoticeKind
import uniffi.gemstone.GemPerpetualChartLineKind
import uniffi.gemstone.GemPriceAlertToggle
import uniffi.gemstone.GemSecurityReminderItem
import uniffi.gemstone.GemSwapProgressMarker
import uniffi.gemstone.GemSwapProgressStep
import uniffi.gemstone.GemTransactionStateTone
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.GemVerificationLevel
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.SwapPriceImpactType
import uniffi.gemstone.WalletConnectionVerificationStatus
import uniffi.gemstone.verificationLevel

@DrawableRes
fun GemHeaderButtonKind.iconRes(): Int = when (this) {
    GemHeaderButtonKind.SEND -> R.drawable.ic_action_send
    GemHeaderButtonKind.RECEIVE -> R.drawable.ic_action_receive
    GemHeaderButtonKind.BUY -> R.drawable.ic_action_buy
    GemHeaderButtonKind.SWAP -> R.drawable.ic_swap_vert
    GemHeaderButtonKind.DEPOSIT -> R.drawable.ic_action_deposit
    GemHeaderButtonKind.WITHDRAW -> R.drawable.ic_action_withdraw
    GemHeaderButtonKind.MORE -> R.drawable.ic_more_vert
}

@DrawableRes
fun GemTransactionStateTone.badgeIconRes(): Int = when (this) {
    GemTransactionStateTone.PENDING -> R.drawable.transaction_state_pending

    GemTransactionStateTone.SUCCESS -> R.drawable.transaction_state_success

    GemTransactionStateTone.ERROR,
    GemTransactionStateTone.REFUNDED,
    -> R.drawable.transaction_state_error
}

@Composable
fun GemTransactionStateTone.color(): Color = when (this) {
    GemTransactionStateTone.PENDING,
    GemTransactionStateTone.REFUNDED,
    -> pendingColor

    GemTransactionStateTone.SUCCESS -> MaterialTheme.colorScheme.tertiary

    GemTransactionStateTone.ERROR -> MaterialTheme.colorScheme.error
}

fun GemValueTone.textStyle(): ListItemTextStyle = when (this) {
    GemValueTone.PLAIN -> ListItemTextStyle.Body
    GemValueTone.NEUTRAL -> ListItemTextStyle.Secondary
    GemValueTone.POSITIVE -> ListItemTextStyle.Positive
    GemValueTone.WARNING -> ListItemTextStyle.Warning
    GemValueTone.NEGATIVE -> ListItemTextStyle.Negative
}

@Composable
fun GemNoticeKind.color(): Color = when (this) {
    GemNoticeKind.ERROR -> MaterialTheme.colorScheme.error
    GemNoticeKind.WARNING, GemNoticeKind.INFO -> pendingColor
}

fun GemAcquireOption.image(): ListItemImage = ListItemImage.Symbol(
    when (this) {
        GemAcquireOption.BUY -> ListItemSymbol.Buy
        GemAcquireOption.SWAP -> ListItemSymbol.Swap
        GemAcquireOption.RECEIVE -> ListItemSymbol.Receive
    },
    style = ListItemImageStyle.Action,
)

@Composable
fun GemKeystoreAuthentication.icon(): ImageVector? = when (this) {
    GemKeystoreAuthentication.BIOMETRICS -> AppIcons.Fingerprint
    GemKeystoreAuthentication.PASSCODE -> AppIcons.Lock
    GemKeystoreAuthentication.NONE -> null
}

@Composable
fun GemNoticeKind.icon(): ImageVector = when (this) {
    GemNoticeKind.ERROR, GemNoticeKind.WARNING -> AppIcons.Warning
    GemNoticeKind.INFO -> AppIcons.Info
}

@Composable
fun GemValueTone.color(): Color = when (this) {
    GemValueTone.PLAIN -> MaterialTheme.colorScheme.onSurface
    GemValueTone.NEUTRAL -> MaterialTheme.colorScheme.secondary
    GemValueTone.POSITIVE -> MaterialTheme.colorScheme.tertiary
    GemValueTone.WARNING -> pendingColor
    GemValueTone.NEGATIVE -> MaterialTheme.colorScheme.error
}

fun PerpetualDirection.textStyle(): ListItemTextStyle = when (this) {
    PerpetualDirection.Short -> ListItemTextStyle.Negative
    PerpetualDirection.Long -> ListItemTextStyle.Positive
}

fun GemTransactionStateTone.textStyle(): ListItemTextStyle = when (this) {
    GemTransactionStateTone.PENDING,
    GemTransactionStateTone.REFUNDED,
    -> ListItemTextStyle.Warning

    GemTransactionStateTone.SUCCESS -> ListItemTextStyle.Positive

    GemTransactionStateTone.ERROR -> ListItemTextStyle.Negative
}

fun SwapPriceImpactType?.textStyle(): ListItemTextStyle = when (this) {
    SwapPriceImpactType.POSITIVE -> ListItemTextStyle.Positive

    SwapPriceImpactType.MEDIUM -> ListItemTextStyle.Warning

    SwapPriceImpactType.HIGH -> ListItemTextStyle.Negative

    SwapPriceImpactType.LOW,
    null,
    -> ListItemTextStyle.Secondary
}

fun VerificationStatus.textStyle(): ListItemTextStyle = when (this) {
    VerificationStatus.Suspicious -> ListItemTextStyle.Negative

    VerificationStatus.Unverified,
    VerificationStatus.Verified,
    -> ListItemTextStyle.Warning
}

fun GemFiatTransactionBadge.textStyle(): ListItemTextStyle = when (this) {
    GemFiatTransactionBadge.PENDING -> ListItemTextStyle.Warning
    GemFiatTransactionBadge.FAILED -> ListItemTextStyle.Negative
}

fun GemSwapProgressStep.textStyle(): ListItemTextStyle = when (this) {
    GemSwapProgressStep.COMPLETED -> ListItemTextStyle.Positive

    GemSwapProgressStep.PENDING -> ListItemTextStyle.Primary

    GemSwapProgressStep.WAITING -> ListItemTextStyle.Faded

    GemSwapProgressStep.FAILED,
    GemSwapProgressStep.REVERTED,
    -> ListItemTextStyle.Negative

    GemSwapProgressStep.REFUNDED -> ListItemTextStyle.Warning
}

@Composable
fun WalletConnectionVerificationStatus.icon(): ImageVector = when (verificationLevel(this)) {
    GemVerificationLevel.VERIFIED -> AppIcons.Verified
    GemVerificationLevel.UNVERIFIED, GemVerificationLevel.SUSPICIOUS -> AppIcons.Warning
}

fun WalletConnectionVerificationStatus.textStyle(): ListItemTextStyle = when (verificationLevel(this)) {
    GemVerificationLevel.VERIFIED -> ListItemTextStyle.Positive
    GemVerificationLevel.UNVERIFIED -> ListItemTextStyle.Warning
    GemVerificationLevel.SUSPICIOUS -> ListItemTextStyle.Negative
}

@Composable
fun WalletConnectionVerificationStatus.color(): Color = when (verificationLevel(this)) {
    GemVerificationLevel.VERIFIED -> MaterialTheme.colorScheme.tertiary
    GemVerificationLevel.UNVERIFIED -> pendingColor
    GemVerificationLevel.SUSPICIOUS -> MaterialTheme.colorScheme.error
}

fun GemEmptyStateImage.image(): EmptyStateImage = when (this) {
    GemEmptyStateImage.NFTS -> EmptyStateImage.Drawable(R.drawable.empty_nfts)
    GemEmptyStateImage.PRICE_ALERTS -> EmptyStateImage.Drawable(R.drawable.empty_notifications)
    GemEmptyStateImage.CONTACTS -> EmptyStateImage.Drawable(R.drawable.empty_contacts)
    GemEmptyStateImage.ACTIVITY -> EmptyStateImage.Drawable(R.drawable.empty_activity)
    GemEmptyStateImage.STAKE -> EmptyStateImage.Drawable(R.drawable.empty_stake)
    GemEmptyStateImage.WALLET_CONNECT -> EmptyStateImage.Drawable(R.drawable.empty_dapps)
    GemEmptyStateImage.NOTIFICATIONS -> EmptyStateImage.Drawable(R.drawable.empty_notifications)
    GemEmptyStateImage.SEARCH -> EmptyStateImage.Vector(R.drawable.ic_search)
    GemEmptyStateImage.WALLET -> EmptyStateImage.Vector(R.drawable.ic_wallet)
}

fun GemNameIndicator.symbol(): ListItemSymbol = when (this) {
    GemNameIndicator.ERROR -> ListItemSymbol.Error
    GemNameIndicator.LOADING, GemNameIndicator.SUCCESS -> ListItemSymbol.CheckCircle
}

fun GemNameIndicator.style(): ListItemTextStyle = when (this) {
    GemNameIndicator.ERROR -> ListItemTextStyle.Negative
    GemNameIndicator.LOADING, GemNameIndicator.SUCCESS -> ListItemTextStyle.Positive
}

fun GemAddressServiceInterface.formatShort(address: String, chain: String?): String = format(address, chain, GemAddressFormatStyle.Short)

fun GemAddressServiceInterface.formatShort(addresses: List<ChainAddress>): List<String> = formatAll(addresses, GemAddressFormatStyle.Short)

fun GemAmountField.amountSymbol(): AmountSymbolUIModel = AmountSymbolUIModel(
    symbol = when (val symbol = symbol) {
        is GemAmountSymbol.Asset -> symbol.symbol
        is GemAmountSymbol.Currency -> java.util.Currency.getInstance(symbol.currency.toPrimitives().string).symbol
    },
    placement = when (placement) {
        GemAmountSymbolPlacement.LEADING -> AmountSymbolPlacement.Leading
        GemAmountSymbolPlacement.TRAILING -> AmountSymbolPlacement.Trailing
    },
)

fun GemAmountKeyboard.keyboardType(): KeyboardType = when (this) {
    GemAmountKeyboard.DECIMAL -> KeyboardType.Decimal
    GemAmountKeyboard.WHOLE -> KeyboardType.Number
}

@Composable
fun GemBannerButton.colors(): ButtonColors = when (this) {
    GemBannerButton.BUY -> ButtonDefaults.buttonColors()
    GemBannerButton.RECEIVE -> secondaryActionButtonColors()
}

fun GemBannerIcon.image(): ListItemImage = when (this) {
    GemBannerIcon.MoneyBag -> ListItemImage.Emoji(Emoji.moneyBag)
    is GemBannerIcon.Network -> ListItemImage.Asset(chain)
    GemBannerIcon.Warning -> ListItemImage.Symbol(ListItemSymbol.Warning, tint = ListItemTextStyle.Secondary, style = ListItemImageStyle.Banner)
    GemBannerIcon.Suspicious -> ListItemImage.Drawable(R.drawable.suspicious, style = ListItemImageStyle.Banner)
    GemBannerIcon.Bitcoin -> ListItemImage.Symbol(ListItemSymbol.CurrencyBitcoin, tint = ListItemTextStyle.Secondary, style = ListItemImageStyle.Banner)
    GemBannerIcon.Perpetuals -> ListItemImage.Drawable(R.drawable.ic_perpetuals, style = ListItemImageStyle.Banner)
}

fun GemInfoImage.iconModel(): Any? = when (this) {
    GemInfoImage.Logo -> R.drawable.ic_splash

    GemInfoImage.NetworkFee -> R.drawable.ic_network_fee

    GemInfoImage.WatchWallet -> R.drawable.watch_badge

    is GemInfoImage.AssetStatus -> when (status) {
        uniffi.gemstone.VerificationStatus.VERIFIED -> null
        uniffi.gemstone.VerificationStatus.UNVERIFIED -> R.drawable.unverified
        uniffi.gemstone.VerificationStatus.SUSPICIOUS -> R.drawable.suspicious
    }

    is GemInfoImage.SwapProvider -> provider.iconModel()

    is GemInfoImage.Asset -> icon.iconModel()

    is GemInfoImage.TransactionState -> icon.iconModel()
}

fun GemInfoImage.badgeIconModel(): Any? = when (this) {
    is GemInfoImage.Asset -> icon.supportIconModel()
    is GemInfoImage.TransactionState -> tone.badgeIconRes()
    GemInfoImage.Logo, GemInfoImage.NetworkFee, GemInfoImage.WatchWallet, is GemInfoImage.AssetStatus, is GemInfoImage.SwapProvider -> null
}

val GemInfoImage.placeholder: String?
    get() = when (this) {
        is GemInfoImage.Asset -> icon.placeholder
        is GemInfoImage.TransactionState -> icon.placeholder
        GemInfoImage.Logo, GemInfoImage.NetworkFee, GemInfoImage.WatchWallet, is GemInfoImage.AssetStatus, is GemInfoImage.SwapProvider -> null
    }

fun GemSwapProgressMarker.markerUIModel(): SwapProgressMarkerUIModel = when (this) {
    GemSwapProgressMarker.CHECK -> SwapProgressMarkerUIModel.Icon(ListItemSymbol.Check)
    GemSwapProgressMarker.CROSS -> SwapProgressMarkerUIModel.Icon(ListItemSymbol.Close)
    GemSwapProgressMarker.SWAP -> SwapProgressMarkerUIModel.Icon(ListItemSymbol.Swap)
    GemSwapProgressMarker.SPINNER -> SwapProgressMarkerUIModel.Spinner
    GemSwapProgressMarker.DOTS -> SwapProgressMarkerUIModel.Dots
}

fun GemPriceAlertToggle.symbol(): ListItemSymbol = when (this) {
    GemPriceAlertToggle.ENABLED -> ListItemSymbol.Notifications
    GemPriceAlertToggle.DISABLED -> ListItemSymbol.NotificationsOutlined
}

fun GemSecurityReminderItem.emoji(): String = when (this) {
    GemSecurityReminderItem.KEEP_SAFE -> Emoji.lock
    GemSecurityReminderItem.DO_NOT_SHARE -> Emoji.warning
    GemSecurityReminderItem.NO_RECOVERY -> Emoji.gem
}

@Composable
fun GemPerpetualChartLineKind.color(): Color = when (this) {
    GemPerpetualChartLineKind.ENTRY -> MaterialTheme.colorScheme.outline
    GemPerpetualChartLineKind.LIQUIDATION -> MaterialTheme.colorScheme.error
    GemPerpetualChartLineKind.STOP_LOSS -> pendingColor
    GemPerpetualChartLineKind.TAKE_PROFIT -> MaterialTheme.colorScheme.tertiary
}

@Composable
fun GemValueTone.buttonColor(): Color = when (this) {
    GemValueTone.POSITIVE -> MaterialTheme.colorScheme.tertiary
    GemValueTone.NEGATIVE -> MaterialTheme.colorScheme.error
    GemValueTone.PLAIN, GemValueTone.NEUTRAL, GemValueTone.WARNING -> MaterialTheme.colorScheme.primary
}

fun VerificationStatus.badgeIconRes(): Int? = when (this) {
    VerificationStatus.Verified -> null
    VerificationStatus.Unverified -> R.drawable.unverified
    VerificationStatus.Suspicious -> R.drawable.suspicious
}

fun GemWalletRow.iconModel(context: Context): Any? = walletImageModel(context, imageUrl) ?: placeholder.iconModel()

fun GemWalletRow.listItemImage(): ListItemImage = walletListItemImage(imageUrl, placeholder)

fun walletListItemImage(imageUrl: String?, placeholder: GemWalletPlaceholder): ListItemImage = imageUrl?.takeIf { it.isNotEmpty() }?.let { ListItemImage.Stored(it) } ?: when (placeholder) {
    GemWalletPlaceholder.Multicoin -> ListItemImage.Drawable(R.drawable.multicoin_wallet, style = ListItemImageStyle.Avatar)
    is GemWalletPlaceholder.Chain -> ListItemImage.Asset(placeholder.chain)
}

fun GemWalletPlaceholder.iconModel(): Any? = when (this) {
    GemWalletPlaceholder.Multicoin -> R.drawable.multicoin_wallet
    is GemWalletPlaceholder.Chain -> chain.toChain().iconModel()
}

fun GemWalletRow.supportIcon(): String? = if (showsWatchBadge) {
    "android.resource://com.gemwallet.android/drawable/${R.drawable.watch_badge}"
} else {
    null
}
