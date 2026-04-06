package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.domains.asset.getFiatProviderIcon
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer2
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.space2
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.FiatTransactionAssetData
import com.wallet.core.primitives.FiatTransactionStatus
import java.math.BigInteger

@Composable
fun FiatTransactionItem(
    info: FiatTransactionAssetData,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    val transaction = info.transaction
    val asset = info.asset

    val typeTitle = when (transaction.transactionType) {
        FiatQuoteType.Buy -> stringResource(R.string.wallet_buy)
        FiatQuoteType.Sell -> stringResource(R.string.wallet_sell)
    }

    val cryptoAmount = asset.format(BigInteger(transaction.value))

    val fiatCurrency = Currency.entries.first { it.string == transaction.fiatCurrency }
    val fiatFormatted = fiatCurrency.format(transaction.fiatAmount)

    val isDimmed = transaction.status == FiatTransactionStatus.Failed ||
            transaction.status == FiatTransactionStatus.Unknown

    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        leading = {
            AsyncImage(
                model = transaction.provider.getFiatProviderIcon(),
                size = listItemIconSize,
            )
        },
        title = {
            ListItemTitleText(
                text = typeTitle,
                titleBadge = { FiatTransactionStatusBadge(transaction.status) }
            )
        },
        subtitle = { ListItemSupportText("${asset.name} (${transaction.provider.name})") },
        listPosition = listPosition,
        trailing = {
            Column(horizontalAlignment = Alignment.End) {
                ListItemTitleText(
                    text = cryptoAmount,
                    color = if (isDimmed) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.onSurface,
                )
                Spacer2()
                ListItemSupportText(fiatFormatted)
            }
        }
    )
}

@Composable
private fun FiatTransactionStatusBadge(status: FiatTransactionStatus) {
    val text: String
    val color: Color
    when (status) {
        FiatTransactionStatus.Complete,
        FiatTransactionStatus.Unknown -> return
        FiatTransactionStatus.Pending -> {
            text = stringResource(R.string.transaction_status_pending)
            color = pendingColor
        }
        FiatTransactionStatus.Failed -> {
            text = stringResource(R.string.transaction_status_failed)
            color = MaterialTheme.colorScheme.error
        }
    }

    Text(
        modifier = Modifier
            .padding(start = paddingHalfSmall)
            .background(color = color.copy(alpha = alpha10), shape = RoundedCornerShape(paddingHalfSmall))
            .padding(horizontal = paddingHalfSmall, vertical = space2),
        text = text,
        color = color,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        style = MaterialTheme.typography.labelMedium,
    )
}
