package com.gemwallet.android.features.swap.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.isImeVisible
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PercentSuggestionsBar
import com.gemwallet.android.ui.components.buttons.IndicatorButton
import com.gemwallet.android.ui.components.list_item.sectionHeaderItem
import com.gemwallet.android.ui.components.screen.MainActionWidth
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.swap.SwapDetailsSummaryItem
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.features.swap.viewmodels.SwapViewModel
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState
import com.gemwallet.android.features.swap.views.components.SwapAction
import com.gemwallet.android.features.swap.views.components.SwapError
import com.gemwallet.android.features.swap.views.components.SwapItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.ui.theme.space0

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun SwapScene(
    swapState: SwapUiState,
    pay: AssetInfo?,
    receive: AssetInfo?,
    payEquivalent: String,
    receiveEquivalent: String,
    swapDetails: SwapDetailsUIModel?,
    payValue: TextFieldState,
    receiveValue: TextFieldState,
    showsSlippageIndicator: Boolean,
    onAction: (SwapSceneAction) -> Unit,
) {
    val focusManager = LocalFocusManager.current
    fun clearAmountFocus() {
        focusManager.clearFocus(force = true)
    }
    val isKeyboardVisible = WindowInsets.isImeVisible
    val isPercentBarVisible = isKeyboardVisible && pay != null && swapState.isInputEmpty

    Scene(
        title = stringResource(id = R.string.wallet_swap),
        actions = {
            IndicatorButton(
                imageVector = AppIcons.Tune,
                showsIndicator = showsSlippageIndicator,
                onClick = { onAction(SwapSceneAction.Slippage) },
            )
        },
        mainActionWidth = if (isPercentBarVisible) MainActionWidth.FillWidth else MainActionWidth.Constrained,
        mainActionPadding = PaddingValues(
            horizontal = sceneContentPadding(),
            vertical = if (isPercentBarVisible) paddingSmall else paddingDefault,
        ),
        mainAction = {
            if (isPercentBarVisible) {
                PercentSuggestionsBar(
                    suggestions = SwapViewModel.percentSuggestions,
                    onPercentSelected = {
                        clearAmountFocus()
                        onAction(SwapSceneAction.SelectPercent(it))
                    },
                )
            } else {
                SwapAction(
                    swapState = swapState,
                    onSwap = {
                        clearAmountFocus()
                        onAction(SwapSceneAction.Swap)
                    },
                )
            }
        },
        onClose = { onAction(SwapSceneAction.Cancel) },
    ) {
        LaunchedEffect(pay) {
            if (pay == null) {
                clearAmountFocus()
            }
        }
        LazyColumn {
            item {
                SwapSectionHeader(R.string.swap_you_pay)
            }
            item {
                SwapItem(
                    item = pay,
                    equivalent = payEquivalent,
                    state = payValue,
                    interaction = swapState.payItemInteraction,
                    onBalanceClick = {
                        clearAmountFocus()
                        onAction(SwapSceneAction.SelectPercent(100))
                    },
                    onAssetSelect = {
                        clearAmountFocus()
                        onAction(SwapSceneAction.SelectAsset(SwapItemType.Pay))
                    }
                )
            }
            item {
                SwapReceiveHeader(
                    enabled = swapState.isQuoteInteractionEnabled,
                    onSwitch = { onAction(SwapSceneAction.SwitchAssets) },
                )
            }
            item {
                SwapItem(
                    item = receive,
                    equivalent = receiveEquivalent,
                    state = receiveValue,
                    calculating = swapState.isReceiveLoading,
                    interaction = swapState.receiveItemInteraction,
                    onBalanceClick = {},
                    onAssetSelect = {
                        clearAmountFocus()
                        onAction(SwapSceneAction.SelectAsset(SwapItemType.Receive))
                    }

                )
            }
            item {
                swapDetails?.let {
                    SwapDetailsSummaryItem(model = it, onClick = { onAction(SwapSceneAction.ShowDetails) })
                }
            }

            item {
                SwapError(swapState, pay)
            }
        }
    }
}

@Composable
private fun SwapReceiveHeader(enabled: Boolean, onSwitch: () -> Unit) {
    Box(
        modifier = Modifier.fillMaxWidth(),
        contentAlignment = Alignment.Center,
    ) {
        SwapSectionHeader(
            resId = R.string.swap_you_receive,
            modifier = Modifier.fillMaxWidth(),
            topPadding = space0,
        )
        SwitchButton(enabled = enabled, onClick = onSwitch)
    }
}

@Composable
private fun SwitchButton(enabled: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(iconSize)
            .clip(MaterialTheme.shapes.medium)
            .clickable(enabled = enabled, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = AppIcons.SwapVert,
            contentDescription = stringResource(R.string.wallet_swap),
        )
    }
}

@Composable
private fun SwapSectionHeader(resId: Int, modifier: Modifier = Modifier, topPadding: Dp? = null) {
    Text(
        modifier = modifier
            .sectionHeaderItem(paddingVertical = topPadding),
        text = stringResource(resId),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.secondary,
    )
}
