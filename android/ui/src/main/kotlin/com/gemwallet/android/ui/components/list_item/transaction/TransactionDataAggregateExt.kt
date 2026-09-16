package com.gemwallet.android.ui.components.list_item.transaction

import com.gemwallet.android.ui.localization.prefixRes
import com.gemwallet.android.ui.localization.infoDescriptionRes
import com.gemwallet.android.ui.localization.statusLabelRes
import com.gemwallet.android.ui.localization.stringRes
import androidx.annotation.StringRes
import com.gemwallet.android.ext.toPrimitives
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.domains.transaction.aggregates.TransactionDetailsAggregate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.model.CurrencyFormatter
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemTransactionRowSubtitle
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionType

private val usdFiatFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)

@Composable
fun TransactionDataAggregate.getTitle(): String = title.string()

@Composable
fun TransactionDetailsAggregate.getTitle(): String = title.string()

@Composable
fun TransactionDataAggregate.getBadgeText(): String =
    if (status.showsBadge) stringResource(id = state.statusLabelRes()) else ""

@Composable
fun TransactionDataAggregate.getBadgeColor(): Color = status.tone.color()

@Composable
fun TransactionDataAggregate.formatAddress(): String? = when (val subtitle = subtitle) {
    is GemTransactionRowSubtitle.ToAddress -> prefixed(subtitle.prefixRes(), subtitle.participant)
    is GemTransactionRowSubtitle.FromAddress -> prefixed(subtitle.prefixRes(), subtitle.participant)
    is GemTransactionRowSubtitle.ToResource -> prefixed(subtitle.prefixRes(), stringResource(subtitle.resource.toPrimitives().stringRes()))
    is GemTransactionRowSubtitle.FromResource -> prefixed(subtitle.prefixRes(), stringResource(subtitle.resource.toPrimitives().stringRes()))
    is GemTransactionRowSubtitle.Price -> subtitle.prefixRes()?.let { "${stringResource(it)}: ${usdFiatFormatter.string(subtitle.value)}" }
    GemTransactionRowSubtitle.None -> null
}

@Composable
private fun prefixed(@StringRes prefix: Int?, value: String): String? =
    prefix?.let { res -> value.takeIf { it.isNotEmpty() }?.let { "${stringResource(res)} $it" } }

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

