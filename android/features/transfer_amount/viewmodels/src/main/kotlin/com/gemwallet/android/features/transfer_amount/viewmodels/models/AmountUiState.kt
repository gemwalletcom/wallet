package com.gemwallet.android.features.transfer_amount.viewmodels.models

import com.gemwallet.android.ui.components.fields.AmountSymbolPlacement
import com.gemwallet.android.ui.components.fields.AmountSymbolUIModel
import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemInfoTopic

data class AmountUiState(
    val title: String = "",
    val asset: Asset? = null,
    val amountSymbol: AmountSymbolUIModel = AmountSymbolUIModel("", AmountSymbolPlacement.Trailing),
    val canSwitchInputType: Boolean = false,
    val readOnly: Boolean = false,
    val focusesInput: Boolean = false,
    val showsAssetBalance: Boolean = true,
    val usesWholeAmounts: Boolean = false,
    val availableBalance: String = "",
    val reserveForFee: String? = null,
    val equivalent: String = "",
    val error: String = "",
    val errorTopic: GemInfoTopic? = null,
    val buttonState: ButtonState = ButtonState.Disabled,
    val extras: AmountExtrasUIModel = AmountExtrasUIModel.None,
)
