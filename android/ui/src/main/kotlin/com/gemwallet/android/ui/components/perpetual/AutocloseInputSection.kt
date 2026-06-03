package com.gemwallet.android.ui.components.perpetual

import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.sectionHeaderHorizontalPadding
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseError
import com.gemwallet.android.ui.models.perpetual.autoclose.AutocloseUIModel
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.space4
import com.wallet.core.primitives.TpslType

@StringRes
private fun AutocloseError.toStringRes(): Int = when (this) {
    AutocloseError.InvalidAmount -> R.string.errors_invalid_amount
    AutocloseError.TriggerMustBeHigher -> R.string.errors_perpetual_trigger_price_higher
    AutocloseError.TriggerMustBeLower -> R.string.errors_perpetual_trigger_price_lower
}

@Composable
fun AutocloseInputSection(
    field: AutocloseUIModel.Field,
    text: String,
    onTextChanged: (String) -> Unit,
    onFocusChanged: (Boolean) -> Unit,
) {
    SubheaderItem(
        title = stringResource(
            when (field.type) {
                TpslType.TakeProfit -> R.string.perpetual_auto_close_take_profit
                TpslType.StopLoss -> R.string.perpetual_auto_close_stop_loss
            },
        ),
    )
    GemTextField(
        modifier = Modifier.onFocusChanged { onFocusChanged(it.isFocused) },
        value = text,
        onValueChange = onTextChanged,
        label = stringResource(R.string.asset_price),
        error = field.error?.let { stringResource(it.toStringRes()) }.orEmpty(),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
        listPosition = ListPosition.Single,
        errorDivider = true,
        trailing = if (text.isNotEmpty()) {
            {
                Icon(
                    modifier = Modifier
                        .size(compactIconSize)
                        .clickable(
                            interactionSource = null,
                            indication = null,
                            onClick = { onTextChanged("") },
                        ),
                    imageVector = AppIcons.Cancel,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.secondary,
                )
            }
        } else null,
    )
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = sectionHeaderHorizontalPadding, vertical = space4),
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(
            text = stringResource(
                if (field.isProfit) R.string.perpetual_auto_close_expected_profit
                else R.string.perpetual_auto_close_expected_loss,
            ),
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.secondary,
        )
        Text(
            text = field.pnlText,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.SemiBold,
            color = field.pnlDirection.color(),
        )
    }
}
