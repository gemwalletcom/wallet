package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.asset.getFiatProviderIcon
import com.gemwallet.android.features.buy.viewmodels.models.BuyFiatProviderUIModel
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.features.buy.viewmodels.models.FiatUiState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.buttons.RandomGradientButton
import com.gemwallet.android.ui.components.fields.AmountField
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WindowDimension
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.isCompactDimension
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatQuoteType
import uniffi.gemstone.GemFiatButtonAction
import uniffi.gemstone.GemFiatQuotePhase

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun BuyScene(
    asset: Asset,
    assetInfo: AssetInfoDataAggregate?,
    uiState: FiatUiState,
    type: FiatQuoteType,
    providers: List<BuyFiatProviderUIModel>,
    selectedProvider: BuyFiatProviderUIModel?,
    fiatAmount: String,
    suggestedAmounts: List<FiatSuggestion>,
    cancelAction: CancelAction,
    titleContent: @Composable () -> Unit,
    onLotSelect: (FiatSuggestion) -> Unit,
    onAmount: (String) -> Unit,
    onProviderSelect: (FiatProvider) -> Unit,
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
        actions = {
            IconButton(onClick = onFiatTransactions) {
                Icon(
                    imageVector = AppIcons.History,
                    contentDescription = stringResource(R.string.activity_title),
                )
            }
        },
        mainAction = {
            when (uiState.buttonAction) {
                GemFiatButtonAction.CONTINUE -> MainActionButton(
                    title = stringResource(R.string.common_continue),
                    state = uiState.buttonState,
                    onClick = onBuy,
                )
                GemFiatButtonAction.RETRY_QUOTE -> MainActionButton(
                    title = stringResource(R.string.common_try_again),
                    state = uiState.buttonState,
                    onClick = onRetry,
                )
            }
        }
    ) {
        Spacer16()
        AmountField(
            amount = fiatAmount,
            assetSymbol = "$",
            currency = Currency.USD,
            equivalent = selectedProvider?.cryptoFormatted ?: " ",
            error = "",
            onValueChange = onAmount,
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

        val errorText = uiState.errorText(type, asset)
        when {
            uiState.phase is GemFiatQuotePhase.Loading -> {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp)
                ) {
                    CircularProgressIndicator(
                        modifier = Modifier
                            .size(30.dp)
                            .align(Alignment.Center),
                        strokeWidth = 1.dp,
                    )
                }
            }

            errorText != null -> {
                Text(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(20.dp),
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.error,
                    text = errorText,
                    style = MaterialTheme.typography.bodyLarge
                )
            }

            selectedProvider != null -> {
                PropertyItem(
                    modifier = Modifier.clickable(enabled = uiState.canSelectProvider) { isShowProviders.value = true },
                    title = { PropertyTitleText(R.string.common_provider) },
                    data = {
                        PropertyDataText(
                            selectedProvider.provider.name,
                            badge = {
                                DataBadgeChevron(isShowChevron = uiState.canSelectProvider) {
                                    AsyncImage(
                                        model = selectedProvider.provider.getFiatProviderIcon(),
                                        size = smallIconSize,
                                    )
                                }
                            }
                        )
                    },
                    listPosition = ListPosition.First,
                )
                PropertyItem(
                    title = R.string.buy_rate,
                    data = selectedProvider.rate,
                    listPosition = ListPosition.Last,
                )
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
