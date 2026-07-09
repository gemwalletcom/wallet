package com.gemwallet.android.features.swap.views

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.isImeVisible
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PercentSuggestionsBar
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
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.ui.theme.space0
import com.gemwallet.android.ui.theme.space4

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
        Column(
            modifier = Modifier.verticalScroll(rememberScrollState()),
        ) {
            Box(
                modifier = Modifier.fillMaxWidth(),
                contentAlignment = Alignment.Center,
            ) {
                Column(verticalArrangement = Arrangement.spacedBy(space4)) {
                    SwapItem(
                        item = pay,
                        equivalent = payEquivalent,
                        state = payValue,
                        interaction = swapState.payItemInteraction,
                        paddingVertical = space0,
                        onBalanceClick = {
                            clearAmountFocus()
                            onAction(SwapSceneAction.SelectPercent(100))
                        },
                        onAssetSelect = {
                            clearAmountFocus()
                            onAction(SwapSceneAction.SelectAsset(SwapItemType.Pay))
                        }
                    )
                    SwapItem(
                        item = receive,
                        equivalent = receiveEquivalent,
                        state = receiveValue,
                        calculating = swapState.isReceiveLoading,
                        interaction = swapState.receiveItemInteraction,
                        paddingVertical = space0,
                        onBalanceClick = {},
                        onAssetSelect = {
                            clearAmountFocus()
                            onAction(SwapSceneAction.SelectAsset(SwapItemType.Receive))
                        }
                    )
                }
                SwapSwitchButton(
                    enabled = swapState.isQuoteInteractionEnabled,
                    onClick = { onAction(SwapSceneAction.SwitchAssets) },
                )
            }
            swapDetails?.let {
                SwapDetailsSummaryItem(model = it, onClick = { onAction(SwapSceneAction.ShowDetails) })
            }
            SwapError(swapState, pay)
        }
    }
}

@Composable
private fun SwapSwitchButton(
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Box(
        modifier = Modifier
            .clip(CircleShape)
            .background(MaterialTheme.colorScheme.surface)
            .clickable(enabled = enabled, onClick = onClick)
            .padding(space4)
            .clip(CircleShape)
            .background(MaterialTheme.colorScheme.background)
            .padding(paddingSmall),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            modifier = Modifier.size(20.dp),
            imageVector = AppIcons.SwapVert,
            contentDescription = stringResource(R.string.wallet_swap),
            tint = MaterialTheme.colorScheme.secondary,
        )
    }
}
