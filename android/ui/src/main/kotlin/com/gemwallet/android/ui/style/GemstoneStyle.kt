package com.gemwallet.android.ui.style

import androidx.annotation.DrawableRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyStateImage
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.pendingColor
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.VerificationStatus
import uniffi.gemstone.ChainAddress
import uniffi.gemstone.GemAddressFormatStyle
import uniffi.gemstone.GemAddressServiceInterface
import uniffi.gemstone.GemDelegationTone
import uniffi.gemstone.GemEmptyStateImage
import uniffi.gemstone.GemFiatTransactionBadge
import uniffi.gemstone.GemHeaderButtonKind
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemPerpetualChartLineKind
import uniffi.gemstone.GemTransactionStateTone
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.GemVerificationLevel
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
    GemTransactionStateTone.REFUNDED -> R.drawable.transaction_state_error
}

@Composable
fun GemTransactionStateTone.color(): Color = when (this) {
    GemTransactionStateTone.PENDING,
    GemTransactionStateTone.REFUNDED -> pendingColor
    GemTransactionStateTone.SUCCESS -> MaterialTheme.colorScheme.tertiary
    GemTransactionStateTone.ERROR -> MaterialTheme.colorScheme.error
}

fun GemValueTone.textStyle(): ListItemTextStyle = when (this) {
    GemValueTone.PLAIN -> ListItemTextStyle.Body
    GemValueTone.NEUTRAL -> ListItemTextStyle.Secondary
    GemValueTone.POSITIVE -> ListItemTextStyle.Positive
    GemValueTone.NEGATIVE -> ListItemTextStyle.Negative
}

@Composable
fun GemValueTone.color(): Color = when (this) {
    GemValueTone.PLAIN -> MaterialTheme.colorScheme.onSurface
    GemValueTone.NEUTRAL -> MaterialTheme.colorScheme.secondary
    GemValueTone.POSITIVE -> MaterialTheme.colorScheme.tertiary
    GemValueTone.NEGATIVE -> MaterialTheme.colorScheme.error
}

fun PerpetualDirection.textStyle(): ListItemTextStyle = when (this) {
    PerpetualDirection.Short -> ListItemTextStyle.Negative
    PerpetualDirection.Long -> ListItemTextStyle.Positive
}

fun GemTransactionStateTone.textStyle(): ListItemTextStyle = when (this) {
    GemTransactionStateTone.PENDING,
    GemTransactionStateTone.REFUNDED -> ListItemTextStyle.Warning
    GemTransactionStateTone.SUCCESS -> ListItemTextStyle.Positive
    GemTransactionStateTone.ERROR -> ListItemTextStyle.Negative
}

fun SwapPriceImpactType?.textStyle(): ListItemTextStyle = when (this) {
    SwapPriceImpactType.POSITIVE -> ListItemTextStyle.Positive
    SwapPriceImpactType.MEDIUM -> ListItemTextStyle.Warning
    SwapPriceImpactType.HIGH -> ListItemTextStyle.Negative
    SwapPriceImpactType.LOW,
    null -> ListItemTextStyle.Secondary
}

fun VerificationStatus.textStyle(): ListItemTextStyle = when (this) {
    VerificationStatus.Suspicious -> ListItemTextStyle.Negative
    VerificationStatus.Unverified,
    VerificationStatus.Verified -> ListItemTextStyle.Warning
}

fun GemFiatTransactionBadge.textStyle(): ListItemTextStyle = when (this) {
    GemFiatTransactionBadge.PENDING -> ListItemTextStyle.Warning
    GemFiatTransactionBadge.FAILED -> ListItemTextStyle.Negative
}

fun GemDelegationTone.textStyle(): ListItemTextStyle = when (this) {
    GemDelegationTone.POSITIVE -> ListItemTextStyle.Positive
    GemDelegationTone.PENDING -> ListItemTextStyle.Warning
    GemDelegationTone.NEGATIVE -> ListItemTextStyle.Negative
}

@Composable
fun GemDelegationTone.color(): Color = when (this) {
    GemDelegationTone.POSITIVE -> MaterialTheme.colorScheme.tertiary
    GemDelegationTone.PENDING -> pendingColor
    GemDelegationTone.NEGATIVE -> MaterialTheme.colorScheme.error
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

@Composable
fun GemPerpetualChartLineKind.color(): Color = when (this) {
    GemPerpetualChartLineKind.ENTRY -> MaterialTheme.colorScheme.outline
    GemPerpetualChartLineKind.LIQUIDATION -> MaterialTheme.colorScheme.error
    GemPerpetualChartLineKind.STOP_LOSS -> pendingColor
    GemPerpetualChartLineKind.TAKE_PROFIT -> MaterialTheme.colorScheme.tertiary
}

sealed interface NameResolveIndicatorStyle {
    data object Loading : NameResolveIndicatorStyle
    data class Icon(val vector: ImageVector, val tint: Color, val contentDescription: String?) : NameResolveIndicatorStyle
}

@Composable
fun GemNameRecordState.indicator(): NameResolveIndicatorStyle? = when (this) {
    is GemNameRecordState.Loading -> NameResolveIndicatorStyle.Loading
    GemNameRecordState.Error -> NameResolveIndicatorStyle.Icon(AppIcons.Error, MaterialTheme.colorScheme.error, stringResource(R.string.errors_error_occurred))
    is GemNameRecordState.Complete -> NameResolveIndicatorStyle.Icon(AppIcons.CheckCircle, MaterialTheme.colorScheme.tertiary, null)
    GemNameRecordState.None -> null
}

fun GemAddressServiceInterface.formatShort(address: String, chain: String?): String = format(address, chain, GemAddressFormatStyle.Short)

fun GemAddressServiceInterface.formatShort(addresses: List<ChainAddress>): List<String> = formatAll(addresses, GemAddressFormatStyle.Short)
