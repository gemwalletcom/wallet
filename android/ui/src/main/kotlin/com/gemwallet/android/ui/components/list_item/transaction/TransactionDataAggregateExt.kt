package com.gemwallet.android.ui.components.list_item.transaction

import androidx.annotation.StringRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.domains.transaction.aggregates.TransactionDetailsAggregate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.showsStatusBadge
import com.gemwallet.android.ui.components.statusColor
import com.gemwallet.android.ui.components.statusLabelRes
import com.gemwallet.android.ui.components.titleRes
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.decodeJsonOrNull
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemTransactionSubtitle
import uniffi.gemstone.GemTransactionTitle
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionType

private val usdFiatFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)

@Composable
fun TransactionDataAggregate.getTitle(): String = title.string()

@Composable
fun TransactionDetailsAggregate.getTitle(): String = title.string()

@Composable
fun GemTransactionTitle.string(): String = when (this) {
    GemTransactionTitle.Received -> stringResource(R.string.transaction_title_received)
    GemTransactionTitle.Sent -> stringResource(R.string.transaction_title_sent)
    GemTransactionTitle.Transfer -> stringResource(R.string.transfer_title)
    GemTransactionTitle.SmartContract -> stringResource(R.string.transfer_smart_contract_title)
    GemTransactionTitle.Swap -> stringResource(R.string.wallet_swap)
    GemTransactionTitle.Approve -> stringResource(R.string.transfer_approve_title)
    GemTransactionTitle.Stake -> stringResource(R.string.transfer_stake_title)
    GemTransactionTitle.Unstake -> stringResource(R.string.transfer_unstake_title)
    GemTransactionTitle.Redelegate -> stringResource(R.string.transfer_redelegate_title)
    GemTransactionTitle.Rewards -> stringResource(R.string.transfer_rewards_title)
    GemTransactionTitle.Withdraw -> stringResource(R.string.transfer_withdraw_title)
    GemTransactionTitle.ActivateAsset -> stringResource(R.string.transfer_activate_asset_title)
    GemTransactionTitle.Freeze -> stringResource(R.string.transfer_freeze_title)
    GemTransactionTitle.Unfreeze -> stringResource(R.string.transfer_unfreeze_title)
    GemTransactionTitle.Earn -> stringResource(R.string.common_earn)
    is GemTransactionTitle.PerpetualOpen -> perpetualTitle(direction, R.string.perpetual_open_direction, R.string.perpetual_position)
    is GemTransactionTitle.PerpetualClose -> perpetualTitle(direction, R.string.perpetual_close_direction, R.string.perpetual_close_position)
    GemTransactionTitle.PerpetualModify -> stringResource(R.string.perpetual_modify)
}

@Composable
private fun perpetualTitle(direction: String?, @StringRes directionTitle: Int, @StringRes fallback: Int): String {
    val side = when (direction?.decodeJsonOrNull<PerpetualDirection>()) {
        PerpetualDirection.Long -> stringResource(R.string.perpetual_long)
        PerpetualDirection.Short -> stringResource(R.string.perpetual_short)
        null -> return stringResource(fallback)
    }
    return stringResource(directionTitle, side)
}

@Composable
fun TransactionDataAggregate.getBadgeText(): String =
    if (state.showsStatusBadge()) stringResource(id = state.statusLabelRes()) else ""

@Composable
fun TransactionDataAggregate.getBadgeColor(): Color = state.statusColor()

@Composable
fun TransactionDataAggregate.formatAddress(): String? = when (val subtitle = subtitle) {
    is GemTransactionSubtitle.ToAddress -> prefixed(R.string.transfer_to, addressName ?: address)
    is GemTransactionSubtitle.FromAddress -> prefixed(R.string.transfer_from, addressName ?: address)
    is GemTransactionSubtitle.ToResource -> prefixed(R.string.transfer_to, stringResource(subtitle.resource.decodeJson<Resource>().titleRes()))
    is GemTransactionSubtitle.FromResource -> prefixed(R.string.transfer_from, stringResource(subtitle.resource.decodeJson<Resource>().titleRes()))
    is GemTransactionSubtitle.Price -> "${stringResource(R.string.asset_price)}: ${usdFiatFormatter.string(subtitle.value)}"
    GemTransactionSubtitle.None -> null
}

@Composable
private fun prefixed(@StringRes prefix: Int, value: String): String? =
    value.takeIf { it.isNotEmpty() }?.let { "${stringResource(prefix)} $it" }

@Composable
fun TransactionDataAggregate.getValueColor(): Color = when {
    type == TransactionType.PerpetualClosePosition -> when {
        (pnl ?: 0.0) > 0 -> MaterialTheme.colorScheme.tertiary
        (pnl ?: 0.0) < 0 -> MaterialTheme.colorScheme.error
        else -> MaterialTheme.colorScheme.onSurface
    }
    valueSign == GemAmountSign.INCOMING -> MaterialTheme.colorScheme.tertiary
    else -> MaterialTheme.colorScheme.onSurface
}

