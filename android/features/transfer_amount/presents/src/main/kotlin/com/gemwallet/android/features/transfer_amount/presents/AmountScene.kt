package com.gemwallet.android.features.transfer_amount.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.features.transfer_amount.presents.components.amountErrorString
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.fields.AmountField
import com.gemwallet.android.ui.components.keyboardAsState
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyAssetInfoItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.AmountInputType
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency

@Composable
fun AmountScene(
    title: String,
    amount: String,
    amountInputType: AmountInputType,
    asset: Asset,
    currency: Currency,
    canSwitchInputType: Boolean,
    readOnly: Boolean,
    showsAssetBalance: Boolean,
    error: AmountError,
    equivalent: String,
    availableBalance: String,
    reserveForFee: String? = null,
    onNext: () -> Unit,
    onInputAmount: (String) -> Unit,
    onInputTypeClick: () -> Unit,
    onMaxAmount: () -> Unit,
    onCancel: () -> Unit,
    additionParams: (@Composable () -> Unit)? = null,
) {
    val focusRequester = remember { FocusRequester() }
    val isKeyBoardOpen by keyboardAsState()
    val density = LocalDensity.current
    val isSmallScreen = with(density) {
        LocalWindowInfo.current.containerSize.height.toDp() < 680.dp
    }

    Scene(
        title = title,
        onClose = onCancel,
        mainAction = {
            if (!isKeyBoardOpen || !isSmallScreen) {
                MainActionButton(
                    title = stringResource(id = R.string.common_continue),
                    onClick = onNext,
                )
            }
        },
        actions = {
            TextButton(
                onClick = onNext,
                colors = ButtonDefaults.textButtonColors().copy(contentColor = MaterialTheme.colorScheme.primary),
            ) { Text(stringResource(R.string.common_continue).uppercase()) }
        },
    ) {
        LazyColumn(horizontalAlignment = Alignment.CenterHorizontally) {
            item {
                Spacer16()
                AmountField(
                    modifier = Modifier.fillMaxWidth().focusRequester(focusRequester),
                    amount = amount,
                    assetSymbol = asset.symbol,
                    currency = currency,
                    inputType = amountInputType,
                    onInputTypeClick = if (canSwitchInputType) onInputTypeClick else null,
                    equivalent = equivalent,
                    readOnly = readOnly,
                    error = amountErrorString(error = error),
                    onValueChange = onInputAmount,
                    onNext = onNext,
                )
            }
            if (showsAssetBalance) {
                item {
                    PropertyAssetInfoItem(
                        asset = asset,
                        availableAmount = availableBalance,
                        onMaxAmount = onMaxAmount,
                    )
                }
            }
            reserveForFee?.let {
                item {
                    ReserveForFeeItem(asset = asset, reserveForFee = it)
                }
            }
            item { additionParams?.invoke() }
        }
    }

    LaunchedEffect(Unit) {
        try { focusRequester.requestFocus() } catch (_: Throwable) {}
    }
}

@Composable
private fun ReserveForFeeItem(asset: Asset, reserveForFee: String) {
    var showInfoSheet by remember { mutableStateOf(false) }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .listItem()
            .clickable { showInfoSheet = true }
            .padding(paddingMiddle),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(paddingMiddle),
    ) {
        Icon(
            modifier = Modifier.size(smallIconSize),
            imageVector = AppIcons.InfoOutlined,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.secondary.copy(alpha = alpha50),
        )
        Text(
            text = stringResource(R.string.transfer_reserved_fees, reserveForFee),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.secondary,
        )
    }
    if (showInfoSheet) {
        InfoBottomSheet(InfoSheetEntity.ReserveForFee(asset.getIconUrl())) {
            showInfoSheet = false
        }
    }
}
