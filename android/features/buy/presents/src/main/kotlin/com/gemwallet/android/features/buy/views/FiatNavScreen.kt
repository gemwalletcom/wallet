package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.buy.viewmodels.FiatViewModel
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.features.buy.viewmodels.models.FiatUiState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space6
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuoteType
import uniffi.gemstone.GemFiatAmountCheck
import uniffi.gemstone.GemFiatQuotePhase

@Composable
fun FiatNavScreen(
    cancelAction: CancelAction,
    onFiatTransactions: () -> Unit,
    viewModel: FiatViewModel = hiltViewModel()
) {
    val type by viewModel.type.collectAsStateWithLifecycle()
    val suggestedAmounts by viewModel.suggestedAmounts.collectAsStateWithLifecycle()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val asset by viewModel.assetInfoUIModel.collectAsStateWithLifecycle()
    val amount by viewModel.amount.collectAsStateWithLifecycle()
    val providers by viewModel.providers.collectAsStateWithLifecycle()
    val selectedProvider by viewModel.selectedProvider.collectAsStateWithLifecycle()
    val showFiatTypePicker by viewModel.showFiatTypePicker.collectAsStateWithLifecycle()

    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val currentAssetInfo = asset
    val currentAsset = currentAssetInfo?.asset ?: return

    BuyScene(
        asset = currentAsset,
        assetInfo = currentAssetInfo,
        uiState = uiState,
        type = type,
        providers = providers,
        selectedProvider = selectedProvider,
        cancelAction = cancelAction,
        fiatAmount = amount,
        suggestedAmounts = suggestedAmounts,
        titleContent = {
            FiatTitle(
                asset = currentAsset,
                type = type,
                showFiatTypePicker = showFiatTypePicker,
                onTypeClick = viewModel::setType,
            )
        },
        onAmount = viewModel::updateAmount,
        onLotSelect = viewModel::updateAmount,
        onProviderSelect = viewModel::setProvider,
        onRetry = viewModel::retry,
        onFiatTransactions = onFiatTransactions,
        onBuy = {
            viewModel.getUrl { url ->
                url?.let { uriHandler.open(context, it) }
            }
        }
    )
}

@Composable
private fun FiatTitle(
    asset: Asset,
    type: FiatQuoteType,
    showFiatTypePicker: Boolean,
    onTypeClick: (FiatQuoteType) -> Unit,
) {
    if (showFiatTypePicker) {
        TabsBar(FiatQuoteType.entries, type, onTypeClick) { item ->
            Text(
                stringResource(
                    when (item) {
                        FiatQuoteType.Buy -> R.string.buy_title
                        FiatQuoteType.Sell -> R.string.sell_title
                    },
                    "",
                ),
            )
        }
    } else {
        Text(
            text = stringResource(
                when (type) {
                    FiatQuoteType.Buy -> R.string.buy_title
                    FiatQuoteType.Sell -> R.string.sell_title
                },
                asset.name,
            ),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
fun LotButton(fiatSuggestion: FiatSuggestion, onLotClick: (FiatSuggestion) -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(paddingSmall))
            .clickable { onLotClick(fiatSuggestion) }
            .background(MaterialTheme.colorScheme.surfaceContainerHighest)
            .heightIn(min = iconSize)
            .padding(horizontal = space6),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = fiatSuggestion.text,
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.W500),
        )
    }
}

@Composable
fun FiatUiState.errorText(type: FiatQuoteType, asset: Asset): String? = when (val phase = phase) {
    is GemFiatQuotePhase.Invalid -> phase.check.errorText(asset)
    GemFiatQuotePhase.InvalidInput -> stringResource(id = R.string.errors_invalid_amount)
    GemFiatQuotePhase.NoInput -> stringResource(
        R.string.input_enter_amount_to, when (type) {
            FiatQuoteType.Buy -> stringResource(R.string.buy_title, "")
            FiatQuoteType.Sell -> stringResource(R.string.sell_title, "")
        }
    )
    GemFiatQuotePhase.NoQuotes -> stringResource(id = R.string.buy_no_results)
    is GemFiatQuotePhase.Failed -> stringResource(R.string.errors_unknown_try_again)
    is GemFiatQuotePhase.Loading -> null
    GemFiatQuotePhase.Ready -> amountCheck.errorText(asset)
}

@Composable
private fun GemFiatAmountCheck.errorText(asset: Asset): String? = when (this) {
    is GemFiatAmountCheck.BelowMinimum -> stringResource(id = R.string.transfer_minimum_amount, "${minimum}$")
    is GemFiatAmountCheck.AboveMaximum -> stringResource(id = R.string.transfer_maximum_amount, "${maximum}$")
    is GemFiatAmountCheck.InsufficientBalance -> stringResource(R.string.transfer_insufficient_balance, "${asset.name} (${asset.symbol})")
    GemFiatAmountCheck.Valid -> null
}
