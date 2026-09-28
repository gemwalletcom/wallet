package com.gemwallet.android.features.swap.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
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
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.features.swap.presents.components.SwapButton
import com.gemwallet.android.features.swap.presents.components.SwapError
import com.gemwallet.android.features.swap.presents.components.SwapToken
import com.gemwallet.android.features.swap.viewmodels.SwapViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PercentSuggestionsBar
import com.gemwallet.android.ui.components.buttons.IndicatorButton
import com.gemwallet.android.ui.components.isKeyboardVisible
import com.gemwallet.android.ui.components.list_item.sectionHeaderItem
import com.gemwallet.android.ui.components.screen.MainActionWidth
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.swap.SwapDetailsSummaryItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.AssetData
import uniffi.gemstone.GemSwapViewState

@Composable
internal fun SwapScene(viewState: GemSwapViewState, pay: AssetData?, receive: AssetData?, payValue: TextFieldState, receiveValue: TextFieldState, showsSlippageIndicator: Boolean, onAction: (SwapAction) -> Unit) {
    val focusManager = LocalFocusManager.current
    fun clearAmountFocus() {
        focusManager.clearFocus(force = true)
    }
    val isKeyboardVisible = WindowInsets.isKeyboardVisible
    val isPercentBarVisible = isKeyboardVisible && pay != null && viewState.isInputEmpty

    Scene(
        title = stringResource(id = R.string.wallet_swap),
        actions = {
            IndicatorButton(
                imageVector = AppIcons.Tune,
                showsIndicator = showsSlippageIndicator,
                onClick = { onAction(SwapAction.Slippage) },
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
                        onAction(SwapAction.SelectPercent(it))
                    },
                )
            } else if (viewState.showsButton) {
                SwapButton(
                    viewState = viewState,
                    pay = pay,
                    onSwap = {
                        clearAmountFocus()
                        onAction(SwapAction.Swap)
                    },
                )
            }
        },
        onClose = { onAction(SwapAction.Cancel) },
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
                SwapToken(
                    item = pay,
                    side = viewState.pay,
                    state = payValue,
                    onBalanceClick = {
                        clearAmountFocus()
                        onAction(SwapAction.SelectPercent(100))
                    },
                    onAssetSelect = {
                        clearAmountFocus()
                        onAction(SwapAction.SelectAsset(SwapItemType.Pay))
                    },
                )
            }
            item {
                SwapReceiveHeader(
                    enabled = !viewState.isTransferLoading,
                    onSwitch = { onAction(SwapAction.SwitchAssets) },
                )
            }
            item {
                SwapToken(
                    item = receive,
                    side = viewState.receive,
                    state = receiveValue,
                    calculating = viewState.isReceiveLoading,
                    onBalanceClick = {},
                    onAssetSelect = {
                        clearAmountFocus()
                        onAction(SwapAction.SelectAsset(SwapItemType.Receive))
                    },

                )
            }
            item {
                viewState.details?.let {
                    SwapDetailsSummaryItem(details = it, onClick = { onAction(SwapAction.ShowDetails) })
                }
            }

            item {
                SwapError(viewState)
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
