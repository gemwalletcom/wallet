package com.gemwallet.android.features.fiat_connect.presents

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
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.features.fiat_connect.viewmodels.FiatViewModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.ObserveStartedState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space6
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuoteType
import kotlinx.coroutines.launch
import uniffi.gemstone.GemFiatSuggestedAmount

@Composable
fun FiatScreen(cancelAction: CancelAction, onFiatTransactions: () -> Unit, viewModel: FiatViewModel = hiltViewModel()) {
    val type by viewModel.type.collectAsStateWithLifecycle()
    val viewState by viewModel.viewState.collectAsStateWithLifecycle()
    val asset by viewModel.assetInfoUIModel.collectAsStateWithLifecycle()
    val amount by viewModel.amount.collectAsStateWithLifecycle()
    val providers by viewModel.providers.collectAsStateWithLifecycle()
    val selectedProvider by viewModel.selectedProvider.collectAsStateWithLifecycle()
    val providerListItem by viewModel.providerListItem.collectAsStateWithLifecycle()
    val rateRow by viewModel.rateRow.collectAsStateWithLifecycle()
    val showsTypePicker by viewModel.showsTypePicker.collectAsStateWithLifecycle()

    ObserveStartedState(viewModel::setRefreshEnabled)

    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val scope = rememberCoroutineScope()
    val snackbar = remember { SnackbarHostState() }
    val title = stringResource(type.titleRes(), "")
    val currentAssetInfo = asset ?: return LoadingScene(title = title, onCancel = { cancelAction() })
    val currentAsset = currentAssetInfo.asset

    FiatScene(
        asset = currentAsset,
        assetInfo = currentAssetInfo,
        snackbar = snackbar,
        viewState = viewState,
        type = type,
        providers = providers,
        selectedProvider = selectedProvider,
        providerListItem = providerListItem,
        rateRow = rateRow,
        cancelAction = cancelAction,
        fiatAmount = amount,
        suggestedAmounts = viewModel.suggestedAmounts,
        titleContent = {
            FiatTitle(
                asset = currentAsset,
                type = type,
                showsTypePicker = showsTypePicker,
                onTypeClick = viewModel::setType,
            )
        },
        onAmount = viewModel::updateAmount,
        onLotSelect = viewModel::selectAmount,
        onRandomAmount = viewModel::selectRandomAmount,
        onProviderSelect = viewModel::setProvider,
        onRetry = viewModel::retry,
        onFiatTransactions = onFiatTransactions,
        onBuy = {
            scope.launch {
                viewModel.quoteUrl()
                    .onSuccess { uriHandler.open(context, it) }
                    .onFailure { snackbar.showSnackbar(it.errorText().text(context), R.drawable.ic_error) }
            }
        },
    )
}

@Composable
private fun FiatTitle(asset: Asset, type: FiatQuoteType, showsTypePicker: Boolean, onTypeClick: (FiatQuoteType) -> Unit) {
    if (showsTypePicker) {
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
fun LotButton(fiatSuggestion: GemFiatSuggestedAmount, onLotClick: (GemFiatSuggestedAmount) -> Unit) {
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
            text = fiatSuggestion.value.text(),
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.W500),
        )
    }
}
