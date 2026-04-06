package com.gemwallet.android.features.transfer_amount.presents.components

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.PropertyValidatorItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.TransactionType

internal fun LazyListScope.validatorView(
    txType: TransactionType,
    validatorState: DelegationValidator?,
    onValidator: () -> Unit
) {
    validatorState ?: return
    val isValidatorSelectable = txType.canSelectValidatorOnAmountScreen()
    item {
        SubheaderItem(R.string.stake_validator)
    }
    item {
        PropertyValidatorItem(
            validator = validatorState,
            listPosition = ListPosition.Single,
            onClick = if (isValidatorSelectable) {
                { onValidator() }
            } else {
                null
            }
        )
    }
}

internal fun TransactionType.canSelectValidatorOnAmountScreen(): Boolean {
    return this != TransactionType.StakeUndelegate
}
