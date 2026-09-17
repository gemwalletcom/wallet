package com.gemwallet.android.features.buy.views

import com.gemwallet.android.features.buy.localization.titleRes
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
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
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space6
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuoteType
import kotlinx.coroutines.launch

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
    val providerListItem by viewModel.providerListItem.collectAsStateWithLifecycle()
    val rateListItem by viewModel.rateListItem.collectAsStateWithLifecycle()
    val showFiatTypePicker by viewModel.showFiatTypePicker.collectAsStateWithLifecycle()

    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val scope = rememberCoroutineScope()
    val snackbar = remember { SnackbarHostState() }
    val errorOccurred = stringResource(R.string.errors_error_occurred)
    val title = stringResource(type.titleRes(), "")
    val currentAssetInfo = asset ?: return LoadingScene(title = title, onCancel = { cancelAction() })
    val currentAsset = currentAssetInfo.asset

    BuyScene(
        asset = currentAsset,
        assetInfo = currentAssetInfo,
        snackbar = snackbar,
        uiState = uiState,
        type = type,
        providers = providers,
        selectedProvider = selectedProvider,
        providerListItem = providerListItem,
        rateListItem = rateListItem,
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
            scope.launch {
                viewModel.quoteUrl()
                    .onSuccess { uriHandler.open(context, it) }
                    .onFailure { snackbar.showSnackbar(errorOccurred, R.drawable.ic_error) }
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
            Text(stringResource(item.titleRes(), ""))
        }
    } else {
        Text(
            text = stringResource(type.titleRes(), asset.name),
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

