package com.gemwallet.android.features.transfer.viewmodels.amount.models

import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAmountExtras
import uniffi.gemstone.GemAmountField
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemInfoTopic

data class AmountUIState(
    val title: String = "",
    val asset: Asset? = null,
    val icon: GemAssetIcon? = null,
    val field: GemAmountField? = null,
    val canSwitchInputType: Boolean = false,
    val readOnly: Boolean = false,
    val focusesInput: Boolean = false,
    val showsAssetBalance: Boolean = true,
    val availableBalance: String = "",
    val reserveForFee: String? = null,
    val equivalent: String = "",
    val error: String = "",
    val errorTopic: GemInfoTopic? = null,
    val buttonState: ButtonState = ButtonState.Disabled,
    val extras: GemAmountExtras = GemAmountExtras.None,
)
