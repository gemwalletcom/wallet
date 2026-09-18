package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.features.buy.viewmodels.models.BuyFiatProviderUIModel
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.features.buy.viewmodels.models.FiatUiState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.buttons.RandomGradientButton
import com.gemwallet.android.ui.components.fields.AmountField
import com.gemwallet.android.ui.components.fields.AmountSymbolPlacement
import com.gemwallet.android.ui.components.fields.AmountSymbolUIModel
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WindowDimension
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.isCompactDimension
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space1
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.FiatQuoteType

private val loadingIndicatorSize = 30.dp
private val quotesMessagePadding = 20.dp

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun BuyScene(
    asset: Asset,
    assetInfo: AssetInfoDataAggregate?,
    uiState: FiatUiState,
    type: FiatQuoteType,
    providers: List<BuyFiatProviderUIModel>,
    selectedProvider: BuyFiatProviderUIModel?,
    providerListItem: ListItemModel?,
    rateListItem: ListItemModel?,
    fiatAmount: String,
    suggestedAmounts: List<FiatSuggestion>,
    cancelAction: CancelAction,
    snackbar: SnackbarHostState,
    titleContent: @Composable () -> Unit,
    onLotSelect: (FiatSuggestion) -> Unit,
    onAmount: (String) -> Unit,
    onProviderSelect: (FiatProviderName) -> Unit,
    onRetry: () -> Unit,
    onFiatTransactions: () -> Unit,
    onBuy: () -> Unit
) {
    val isShowProviders = remember { mutableStateOf(false) }
    val isCompactWidth = isCompactDimension(WindowDimension.Width)
    val assetRowSuggestions = visibleSuggestedAmountsInAssetRow(
        suggestedAmounts = suggestedAmounts,
        isCompactWidth = isCompactWidth,
    )
    Scene(
        titleContent = titleContent,
        onClose = { cancelAction() },
        snackbar = snackbar,
        actions = {
            IconButton(onClick = onFiatTransactions) {
                Icon(
                    imageVector = AppIcons.History,
                    contentDescription = stringResource(R.string.activity_title),
                )
            }
        },
        mainAction = {
            MainActionButton(
                title = stringResource(uiState.actionTitle),
                state = uiState.buttonState,
                onClick = if (uiState.retries) onRetry else onBuy,
            )
        }
    ) {
        Spacer16()
        AmountField(
            amount = fiatAmount,
            symbol = AmountSymbolUIModel(symbol = "$", placement = AmountSymbolPlacement.Trailing),
            equivalent = selectedProvider?.cryptoFormatted ?: " ",
            error = uiState.amountError ?: "",
            onValueChange = onAmount,
            keyboardType = KeyboardType.Number,
            maximumFractionDigits = 0u,
            textStyle = MaterialTheme.typography.displayMedium,
            onNext = { },
        )
        Spacer16()
        AssetListItem(
            asset = asset,
            listPosition = ListPosition.Single,
            support = { ListItemSupportText(assetInfo?.balance ?: " ") },
            trailing = assetRowSuggestions.takeIf { it.isNotEmpty() }?.let { suggestions ->
                {
                    FiatSuggestionRow(
                        suggestedAmounts = suggestions,
                        onLotSelect = onLotSelect,
                    )
                }
            },
        )

        val quotesMessage = uiState.quotesMessage
        when {
            uiState.isLoading -> {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(paddingDefault)
                ) {
                    CircularProgressIndicator(
                        modifier = Modifier
                            .size(loadingIndicatorSize)
                            .align(Alignment.Center),
                        strokeWidth = space1,
                    )
                }
            }

            quotesMessage != null -> {
                Text(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(quotesMessagePadding),
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.error,
                    text = quotesMessage,
                    style = MaterialTheme.typography.bodyLarge
                )
            }

            selectedProvider != null -> {
                providerListItem?.let {
                    ListItem(
                        model = it,
                        listPosition = ListPosition.First,
                        modifier = Modifier.clickable(enabled = uiState.canSelectProvider) { isShowProviders.value = true },
                        accessory = {
                            DataBadgeChevron(
                                icon = selectedProvider.provider.iconModel(),
                                isShowChevron = uiState.canSelectProvider,
                            )
                        },
                    )
                }
                rateListItem?.let { ListItem(model = it, listPosition = ListPosition.Last) }
            }
        }
    }

    ProviderList(
        isShow = isShowProviders,
        providers = providers,
        selectedProvider = selectedProvider,
        onProviderSelect = onProviderSelect,
    )
}

@Composable
private fun FiatSuggestionRow(
    suggestedAmounts: List<FiatSuggestion>,
    onLotSelect: (FiatSuggestion) -> Unit,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(paddingSmall)
    ) {
        suggestedAmounts.forEach { suggestion ->
            when (suggestion) {
                FiatSuggestion.RandomAmount -> RandomGradientButton(
                    size = iconSize,
                    borderWidth = 2f,
                    onClick = { onLotSelect(FiatSuggestion.RandomAmount) }
                )
                is FiatSuggestion.SuggestionAmount -> LotButton(suggestion, onLotSelect)
            }
        }
    }
}

internal fun visibleSuggestedAmountsInAssetRow(
    suggestedAmounts: List<FiatSuggestion>,
    isCompactWidth: Boolean,
): List<FiatSuggestion> {
    if (!isCompactWidth) {
        return suggestedAmounts
    }

    return listOfNotNull(
        suggestedAmounts.firstOrNull { it is FiatSuggestion.SuggestionAmount }
            ?: suggestedAmounts.firstOrNull()
    )
}
